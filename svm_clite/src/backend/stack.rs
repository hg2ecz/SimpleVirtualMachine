use std::collections::{BTreeMap, HashMap};

const DATA_BASE: u16 = 0x8000;
const DATA_LIMIT: u32 = 0xFF00;

#[derive(Clone, Debug)]
struct Var {
    addr: u16,
    width: u16,
}

#[derive(Clone, Debug, Default)]
struct Fun {
    params: Vec<(String, String)>,
}

type Vars = HashMap<String, Var>;
type Funcs = HashMap<String, Fun>;
type Globals = BTreeMap<String, (String, Option<u16>)>;

fn width_of(ty: &str) -> u16 {
    if matches!(ty, "bool" | "u8" | "i8") {
        1
    } else {
        2
    }
}

fn qualified(function: &str, name: &str) -> String {
    format!("{function}::{name}")
}

fn parse_fn_header(line: &str) -> Result<(String, Vec<(String, String)>), String> {
    let rest = line.strip_prefix("fn ").ok_or("bad function header")?;
    let left = rest.find('(').ok_or("bad function header")?;
    let right = rest.rfind(')').ok_or("bad function header")?;
    let name = rest[..left].trim().to_owned();
    rest[right + 1..]
        .trim()
        .strip_prefix("->")
        .ok_or("bad function return")?;

    let mut params = Vec::new();
    let inside = &rest[left + 1..right];
    if !inside.trim().is_empty() {
        for param in inside.split(',') {
            let mut parts = param.split_whitespace();
            let ty = parts.next().ok_or("bad parameter")?.to_owned();
            let name = parts.next().ok_or("bad parameter")?.to_owned();
            params.push((name, ty));
        }
    }
    Ok((name, params))
}

fn parse_decl(rest: &str, kind: &str) -> Result<(String, String, u16), String> {
    let before_init = rest.split_once(" = ").map_or(rest, |(left, _)| left);
    let mut parts = before_init.split_whitespace();
    let ty = parts
        .next()
        .ok_or_else(|| format!("bad {kind}"))?
        .to_owned();
    let mut name = parts
        .next()
        .ok_or_else(|| format!("bad {kind}"))?
        .to_owned();
    let mut count = 1u16;
    if let Some(left) = name.find('[') {
        let right = name.find(']').ok_or_else(|| format!("bad {kind} array"))?;
        count = name[left + 1..right]
            .parse()
            .map_err(|_| format!("bad {kind} array length"))?;
        name.truncate(left);
    }
    Ok((ty, name, count))
}

fn allocate(next: &mut u32, width: u16) -> Result<u16, String> {
    let addr = *next;
    if addr + u32::from(width) > DATA_LIMIT {
        return Err("C-Lite static data exceeds RAM below MMIO".into());
    }
    *next += u32::from(width);
    Ok(addr as u16)
}

fn scan_storage(lines: &[&str]) -> Result<(Vars, Funcs, Globals), String> {
    let mut vars = Vars::new();
    let mut funcs = Funcs::new();
    let mut globals = Globals::new();
    let mut next = u32::from(DATA_BASE);
    let mut current = String::new();

    for line in lines {
        if let Some(rest) = line.strip_prefix("global ") {
            let (ty, name, count) = parse_decl(rest, "global")?;
            let init = rest
                .split_once(" = ")
                .map(|(_, value)| {
                    value
                        .trim()
                        .parse::<u16>()
                        .map_err(|_| "bad global initializer")
                })
                .transpose()?;
            let width = width_of(&ty);
            let addr = allocate(&mut next, width.saturating_mul(count))?;
            vars.insert(name.clone(), Var { addr, width });
            globals.insert(name, (ty, init));
            continue;
        }

        if line.starts_with("fn ") {
            let (name, params) = parse_fn_header(line)?;
            current = name.clone();
            let mut function = Fun::default();
            for (param, ty) in params {
                let width = width_of(&ty);
                let addr = allocate(&mut next, width)?;
                vars.insert(qualified(&current, &param), Var { addr, width });
                function.params.push((param, ty));
            }
            funcs.insert(current.clone(), function);
            continue;
        }

        if *line == "end" {
            current.clear();
            continue;
        }

        if let Some(rest) = line.strip_prefix("local ") {
            let (ty, name, count) = parse_decl(rest, "local")?;
            let width = width_of(&ty);
            let addr = allocate(&mut next, width.saturating_mul(count))?;
            vars.insert(qualified(&current, &name), Var { addr, width });
        }
    }
    Ok((vars, funcs, globals))
}

fn resolve(vars: &Vars, function: &str, name: &str) -> Result<Var, String> {
    vars.get(&qualified(function, name))
        .or_else(|| vars.get(name))
        .cloned()
        .ok_or_else(|| format!("unknown variable {name} in {function}"))
}

fn push_addr(out: &mut String, addr: u16) {
    out.push_str(&format!("    0x{addr:04X}\n"));
}

fn emit_load_var(out: &mut String, var: &Var, ty: &str) {
    push_addr(out, var.addr);
    out.push_str(if var.width == 1 {
        "    C@\n"
    } else {
        "    @\n"
    });
    if ty == "i8" {
        out.push_str("    DUP\n    0x80\n    AND\n    IF\n    0xFF00\n    OR\n    THEN\n");
    }
}

fn emit_store_var(out: &mut String, var: &Var) {
    push_addr(out, var.addr);
    out.push_str(if var.width == 1 {
        "    C!\n"
    } else {
        "    !\n"
    });
}

fn normalize(out: &mut String, ty: &str) {
    match ty {
        "bool" => out.push_str("    1\n    AND\n"),
        "u8" => out.push_str("    0xFF\n    AND\n"),
        "i8" => out.push_str(
            "    0xFF\n    AND\n    DUP\n    0x80\n    AND\n    IF\n    0xFF00\n    OR\n    THEN\n",
        ),
        _ => {}
    }
}

fn pop_expected(stack: &mut Vec<String>, temp: &str) -> Result<(), String> {
    match stack.pop() {
        Some(top) if top == temp => Ok(()),
        Some(top) => Err(format!(
            "stack CLIR order mismatch: expected {temp}, found {top}"
        )),
        None => Err(format!("stack CLIR underflow while consuming {temp}")),
    }
}

fn push_temp(stack: &mut Vec<String>, temp: &str) {
    stack.push(temp.trim().to_owned());
}

fn consume_store_operands(
    out: &mut String,
    stack: &mut Vec<String>,
    address: &str,
    value: &str,
) -> Result<(), String> {
    let address = address.trim();
    let value = value.trim();
    if stack.len() < 2 {
        return Err(format!(
            "stack CLIR underflow while storing {value} through {address}"
        ));
    }

    let top = stack[stack.len() - 1].as_str();
    let below = stack[stack.len() - 2].as_str();

    if top == address && below == value {
        stack.pop();
        stack.pop();
        return Ok(());
    }

    if top == value && below == address {
        stack.pop();
        stack.pop();
        out.push_str("    SWAP\n");
        return Ok(());
    }

    Err(format!(
        "stack CLIR store order mismatch: need adjacent {value} and {address}, found {below}, {top}"
    ))
}

fn split_binary(args: &str) -> Result<(&str, &str, &str), String> {
    let parts = args.split(',').map(str::trim).collect::<Vec<_>>();
    if parts.len() != 3 {
        return Err(format!("bad binary CLIR: {args}"));
    }
    Ok((parts[0], parts[1], parts[2]))
}

fn emit_compare(out: &mut String, op: &str, ty: &str) -> Result<(), String> {
    let lt = if matches!(ty, "i8" | "i16") {
        "SLT"
    } else {
        "ULT"
    };
    match op {
        "eq" => out.push_str("    EQ\n"),
        "ne" => out.push_str("    EQ\n    0=\n"),
        "lt" => out.push_str(&format!("    {lt}\n")),
        "ge" => out.push_str(&format!("    {lt}\n    0=\n")),
        "gt" => out.push_str(&format!("    SWAP\n    {lt}\n")),
        "le" => out.push_str(&format!("    SWAP\n    {lt}\n    0=\n")),
        _ => return Err(format!("bad compare {op}")),
    }
    Ok(())
}

fn emit_signed_divmod(
    out: &mut String,
    op: &str,
    ty: &str,
    scratch: [u16; 3],
) -> Result<(), String> {
    let [a_slot, b_slot, sign_slot] = scratch;

    // The Stack ISA has unsigned DIV/MOD. Signed division is uncommon enough
    // that three compiler-private static words are clearer than clever stack
    // gymnastics. This is still direct Stack code, not a register emulation.
    push_addr(out, b_slot);
    out.push_str("    !\n");
    push_addr(out, a_slot);
    out.push_str("    !\n");

    push_addr(out, a_slot);
    out.push_str("    @\n    0<\n");
    if op == "div" {
        push_addr(out, b_slot);
        out.push_str("    @\n    0<\n    XOR\n");
    }
    push_addr(out, sign_slot);
    out.push_str("    !\n");

    push_addr(out, a_slot);
    out.push_str("    @\n    DUP\n    0<\n    IF\n    NEG\n    THEN\n");
    push_addr(out, b_slot);
    out.push_str("    @\n    DUP\n    0<\n    IF\n    NEG\n    THEN\n");
    out.push_str(if op == "div" {
        "    DIV\n"
    } else {
        "    MOD\n"
    });

    push_addr(out, sign_slot);
    out.push_str("    @\n    IF\n    NEG\n    THEN\n");
    normalize(out, ty);
    Ok(())
}

fn scratch_slots(vars: &Vars) -> Result<[u16; 3], String> {
    let mut next = vars
        .values()
        .map(|var| u32::from(var.addr) + u32::from(var.width))
        .max()
        .unwrap_or(u32::from(DATA_BASE));
    let a = allocate(&mut next, 2)?;
    let b = allocate(&mut next, 2)?;
    let sign = allocate(&mut next, 2)?;
    Ok([a, b, sign])
}

fn emit_startup(out: &mut String, vars: &Vars, globals: &Globals) {
    out.push_str(".load 0x0100\n.entry __start\n\n.proc __start\n");
    for (name, (_, init)) in globals {
        let Some(value) = init else { continue };
        out.push_str(&format!("    {value}\n"));
        emit_store_var(out, &vars[name]);
    }
    out.push_str("    CALL main\n    HALT\n.endproc\n\n");
}

fn emit_function_entry(
    out: &mut String,
    name: &str,
    funcs: &Funcs,
    vars: &Vars,
) -> Result<(), String> {
    let function = funcs
        .get(name)
        .ok_or_else(|| format!("unknown function {name}"))?;
    // Arguments are pushed left-to-right, therefore the last parameter is on top.
    for (param, _) in function.params.iter().rev() {
        let slot = resolve(vars, name, param)?;
        emit_store_var(out, &slot);
    }
    Ok(())
}

fn parse_call(line: &str) -> Result<(Option<&str>, &str, Vec<&str>), String> {
    let (dst, call) = if let Some(eq) = line.find(" = ") {
        let left = &line[5..eq];
        let dst = left
            .split_whitespace()
            .last()
            .ok_or("bad call destination")?;
        (Some(dst), line[eq + 3..].trim())
    } else {
        (None, line.strip_prefix("call ").unwrap_or(line).trim())
    };
    let left = call.find('(').ok_or("bad call")?;
    let right = call.rfind(')').ok_or("bad call")?;
    let name = call[..left].trim();
    let inside = call[left + 1..right].trim();
    let args = if inside.is_empty() {
        Vec::new()
    } else {
        inside.split(',').map(str::trim).collect()
    };
    Ok((dst, name, args))
}

/// Lower CLIR directly to the native Stack ISA.
///
/// CLIR temporaries live on the VM data stack. They are never assigned fake
/// R0..R7 registers and never receive static RAM slots. C-Lite locals and
/// globals still have static addresses because recursion is intentionally not
/// part of the language.
pub fn lower(clir: &str) -> Result<String, String> {
    let lines = clir
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with(';'))
        .collect::<Vec<_>>();
    let (vars, funcs, globals) = scan_storage(&lines)?;
    let scratch = scratch_slots(&vars)?;
    let mut out =
        String::from("\\ Direct CLIR -> Stack16 output; CLIR temporaries use the data stack.\n");
    emit_startup(&mut out, &vars, &globals);

    let mut current = String::new();
    let mut stack: Vec<String> = Vec::new();

    for line in lines {
        if line.starts_with("global ") || line.starts_with("local ") {
            continue;
        }
        if line.starts_with("fn ") {
            let (name, _) = parse_fn_header(line)?;
            current = name.clone();
            stack.clear();
            out.push_str(&format!(".proc {name}\n"));
            emit_function_entry(&mut out, &name, &funcs, &vars)?;
            continue;
        }
        if line == "end" {
            if !stack.is_empty() {
                return Err(format!(
                    "stack backend leaves CLIR temporaries live at end of {current}: {stack:?}"
                ));
            }
            out.push_str(".endproc\n\n");
            current.clear();
            continue;
        }
        if line.ends_with(':') {
            if !stack.is_empty() {
                return Err(format!(
                    "stack backend requires empty temp stack at label {line}: {stack:?}"
                ));
            }
            out.push_str(&format!("{}__{}\n", current, line));
            continue;
        }

        if let Some(rest) = line.strip_prefix("const.") {
            let (_, args) = rest.split_once(' ').ok_or("bad const")?;
            let (dst, value) = args.split_once(',').ok_or("bad const")?;
            out.push_str(&format!("    {}\n", value.trim()));
            push_temp(&mut stack, dst);
            continue;
        }
        if let Some(rest) = line.strip_prefix("load.") {
            let (ty, args) = rest.split_once(' ').ok_or("bad load")?;
            let (dst, name) = args.split_once(',').ok_or("bad load")?;
            let var = resolve(&vars, &current, name.trim())?;
            emit_load_var(&mut out, &var, ty);
            push_temp(&mut stack, dst);
            continue;
        }
        if let Some(rest) = line.strip_prefix("store.") {
            let (_, args) = rest.split_once(' ').ok_or("bad store")?;
            let (name, temp) = args.split_once(',').ok_or("bad store")?;
            pop_expected(&mut stack, temp.trim())?;
            let var = resolve(&vars, &current, name.trim())?;
            emit_store_var(&mut out, &var);
            continue;
        }
        if let Some(rest) = line.strip_prefix("addr ") {
            let (dst, name) = rest.split_once(',').ok_or("bad addr")?;
            let var = resolve(&vars, &current, name.trim())?;
            push_addr(&mut out, var.addr);
            push_temp(&mut stack, dst);
            continue;
        }
        if let Some(rest) = line.strip_prefix("index ") {
            let parts = rest.split(',').map(str::trim).collect::<Vec<_>>();
            if parts.len() != 4 {
                return Err("bad index".into());
            }
            pop_expected(&mut stack, parts[2])?;
            pop_expected(&mut stack, parts[1])?;
            if parts[3] == "2" {
                out.push_str("    2*\n");
            }
            out.push_str("    ADD\n");
            push_temp(&mut stack, parts[0]);
            continue;
        }
        if line.starts_with("loadmem") {
            let volatile = line.starts_with("loadmemv");
            let dot = line.find('.').ok_or("bad loadmem")?;
            let (ty, args) = line[dot + 1..].split_once(' ').ok_or("bad loadmem")?;
            let (dst, address) = args.split_once(',').ok_or("bad loadmem")?;
            pop_expected(&mut stack, address.trim())?;
            let op = match (volatile, width_of(ty)) {
                (false, 1) => "C@",
                (false, _) => "@",
                (true, 1) => "VC@",
                (true, _) => "V@",
            };
            out.push_str(&format!("    {op}\n"));
            if ty == "i8" {
                normalize(&mut out, "i8");
            }
            push_temp(&mut stack, dst);
            continue;
        }
        if line.starts_with("storemem") {
            let volatile = line.starts_with("storememv");
            let dot = line.find('.').ok_or("bad storemem")?;
            let (ty, args) = line[dot + 1..].split_once(' ').ok_or("bad storemem")?;
            let (address, value) = args.split_once(',').ok_or("bad storemem")?;
            // Assignment lowering and store builtins can evaluate these two
            // operands in opposite orders. Both are naturally representable
            // on a stack machine; at most one SWAP is needed.
            consume_store_operands(&mut out, &mut stack, address, value)?;
            let op = match (volatile, width_of(ty)) {
                (false, 1) => "C!",
                (false, _) => "!",
                (true, 1) => "VC!",
                (true, _) => "V!",
            };
            out.push_str(&format!("    {op}\n"));
            continue;
        }
        if let Some(temp) = line.strip_prefix("drop ") {
            pop_expected(&mut stack, temp.trim())?;
            out.push_str("    DROP\n");
            continue;
        }
        if let Some(label) = line.strip_prefix("jmp ") {
            if !stack.is_empty() {
                return Err(format!("live CLIR temp across jmp: {stack:?}"));
            }
            out.push_str(&format!("    JMP {current}__{}\n", label.trim()));
            continue;
        }
        if let Some(rest) = line.strip_prefix("jz ") {
            let (temp, label) = rest.split_once(',').ok_or("bad jz")?;
            pop_expected(&mut stack, temp.trim())?;
            if !stack.is_empty() {
                return Err(format!("live CLIR temp across jz: {stack:?}"));
            }
            out.push_str(&format!("    JZ {current}__{}\n", label.trim()));
            continue;
        }
        if line == "ret" {
            if !stack.is_empty() {
                return Err(format!("void return with live CLIR temp: {stack:?}"));
            }
            out.push_str("    RET\n");
            continue;
        }
        if let Some(temp) = line.strip_prefix("ret ") {
            pop_expected(&mut stack, temp.trim())?;
            if !stack.is_empty() {
                return Err(format!("return with extra live CLIR temps: {stack:?}"));
            }
            out.push_str("    RET\n");
            continue;
        }
        if line.starts_with("call") {
            let (dst, name, args) = parse_call(line)?;
            let function = funcs
                .get(name)
                .ok_or_else(|| format!("unknown call target {name}"))?;
            if args.len() != function.params.len() {
                return Err(format!("call arity mismatch for {name}"));
            }
            for arg in args.iter().rev() {
                pop_expected(&mut stack, arg)?;
            }
            out.push_str(&format!("    CALL {name}\n"));
            if let Some(dst) = dst {
                push_temp(&mut stack, dst);
            }
            continue;
        }

        if let Some((op_ty, args)) = line.split_once(' ') {
            if let Some((op, ty)) = op_ty.split_once('.') {
                if matches!(op, "neg" | "not") {
                    let (dst, src) = args.split_once(',').ok_or("bad unary")?;
                    pop_expected(&mut stack, src.trim())?;
                    out.push_str(if op == "neg" {
                        "    NEG\n"
                    } else {
                        "    NOT\n"
                    });
                    normalize(&mut out, ty);
                    push_temp(&mut stack, dst);
                    continue;
                }

                let (dst, left, right) = split_binary(args)?;
                pop_expected(&mut stack, right)?;
                pop_expected(&mut stack, left)?;
                if matches!(op, "eq" | "ne" | "lt" | "le" | "gt" | "ge") {
                    emit_compare(&mut out, op, ty)?;
                } else if matches!(op, "div" | "mod") && matches!(ty, "i8" | "i16") {
                    emit_signed_divmod(&mut out, op, ty, scratch)?;
                } else {
                    let asm = match op {
                        "add" => "ADD",
                        "sub" => "SUB",
                        "mul" => "MUL",
                        "div" => "DIV",
                        "mod" => "MOD",
                        "and" => "AND",
                        "or" => "OR",
                        "xor" => "XOR",
                        "shl" => "SHL",
                        "shr" => "SHR",
                        _ => return Err(format!("unsupported stack CLIR op {op}")),
                    };
                    out.push_str(&format!("    {asm}\n"));
                    normalize(&mut out, ty);
                }
                push_temp(&mut stack, dst);
                continue;
            }
        }

        return Err(format!("unsupported Stack CLIR line: {line}"));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arithmetic_temporaries_stay_on_data_stack() {
        let clir = "; test\nfn main() -> u16\n  local u16 a\n  local u16 b\n  const.u16 %0, 7\n  store.u16 a, %0\n  const.u16 %1, 5\n  store.u16 b, %1\n  load.u16 %2, a\n  load.u16 %3, b\n  mul.u16 %4, %2, %3\n  const.u16 %5, 3\n  add.u16 %6, %4, %5\n  ret %6\nend\n";
        let asm = lower(clir).unwrap();
        assert!(asm.contains("    MUL\n    3\n    ADD\n    RET\n"));
        assert!(!asm.contains("R0"));
        assert!(!asm.contains("0x00D0"));
    }

    #[test]
    fn indexed_store_uses_native_value_address_order() {
        let clir = "; test\nfn main() -> u16\n  local u16 data[4]\n  const.u16 %0, 10\n  addr %1, data\n  const.u16 %2, 0\n  index %3, %1, %2, 2\n  storemem.u16 %3, %0\n  addr %4, data\n  const.u16 %5, 2\n  index %6, %4, %5, 2\n  loadmem.u16 %7, %6\n  ret %7\nend\n";
        let asm = lower(clir).unwrap();
        assert!(asm.contains("    !\n"));
        assert!(!asm.contains("R0"));
        assert!(!asm.contains("0x00D0"));
    }

    #[test]
    fn store_builtin_can_use_address_value_order() {
        let clir = "; test\nfn main() -> u16\n  const.u16 %0, 4096\n  const.u16 %1, 7\n  storemem.u16 %0, %1\n  const.u16 %2, 0\n  ret %2\nend\n";
        let asm = lower(clir).unwrap();
        assert!(asm.contains("    SWAP\n    !\n"));
    }

    #[test]
    fn pointer_array_example_lowers_without_temp_slots() {
        let clir = "; test\nfn main() -> u16\n  local u16 data[4]\n  local u16* p\n  const.u16 %0, 10\n  addr %1, data\n  const.u16 %2, 0\n  index %3, %1, %2, 2\n  storemem.u16 %3, %0\n  const.u16 %4, 20\n  addr %5, data\n  const.u16 %6, 1\n  index %7, %5, %6, 2\n  storemem.u16 %7, %4\n  const.u16 %8, 30\n  addr %9, data\n  const.u16 %10, 2\n  index %11, %9, %10, 2\n  storemem.u16 %11, %8\n  const.u16 %12, 40\n  addr %13, data\n  const.u16 %14, 3\n  index %15, %13, %14, 2\n  storemem.u16 %15, %12\n  addr %16, data\n  const.u16 %17, 0\n  index %18, %16, %17, 2\n  store.u16 p, %18\n  load.u16 %19, p\n  const.u16 %20, 2\n  index %21, %19, %20, 2\n  loadmem.u16 %22, %21\n  ret %22\nend\n";
        let asm = lower(clir).unwrap();
        assert!(asm.contains("    2*\n    ADD\n"));
        assert!(!asm.contains("R0"));
        assert!(!asm.contains("0x00D0"));
    }
}
