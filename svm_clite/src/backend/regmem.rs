//! Direct CLIR -> RegMem lowering.
//! The lowering is target-owned; only target-local instruction legalization follows.

use std::collections::{BTreeMap, HashMap};

use super::layout::{Fun, Layout, Var, lines, parse_call, parse_fn_header, width_of};

type Vars = HashMap<String, Var>;
type Funcs = HashMap<String, Fun>;
type Globals = BTreeMap<String, (String, Option<u16>)>;

#[derive(Default)]
struct Resident {
    temp: Option<String>,
}

impl Resident {
    fn is(&self, temp: &str) -> bool {
        self.temp.as_deref() == Some(temp)
    }
    fn set(&mut self, temp: &str) {
        self.temp = Some(temp.to_string());
    }
    fn clear(&mut self) {
        self.temp = None;
    }
    fn take(&mut self) -> Option<String> {
        self.temp.take()
    }
}

fn resolve(vars: &Vars, function: &str, name: &str) -> Result<Var, String> {
    vars.get(&format!("{function}::{name}"))
        .or_else(|| vars.get(name))
        .cloned()
        .ok_or_else(|| format!("unknown variable {name} in {function}"))
}

fn spill_resident(
    out: &mut String,
    vars: &Vars,
    function: &str,
    resident: &mut Resident,
) -> Result<(), String> {
    let Some(temp) = resident.take() else {
        return Ok(());
    };
    let var = resolve(vars, function, &temp)?;
    out.push_str(&format!("    STORE16 [0x{:04X}], R0\n", var.addr));
    Ok(())
}

fn load_temp(
    out: &mut String,
    vars: &Vars,
    function: &str,
    temp: &str,
    reg: &str,
    resident: &mut Resident,
) -> Result<(), String> {
    if resident.is(temp) {
        if reg != "R0" {
            out.push_str(&format!("    MOV {reg}, R0\n"));
        }
        resident.clear();
        return Ok(());
    }
    if reg == "R0" {
        spill_resident(out, vars, function, resident)?;
    }
    let var = resolve(vars, function, temp)?;
    out.push_str(&format!("    LOAD16 {reg}, [0x{:04X}]\n", var.addr));
    Ok(())
}

fn store_temp(
    _out: &mut String,
    _vars: &Vars,
    _function: &str,
    temp: &str,
    _reg: &str,
    resident: &mut Resident,
) -> Result<(), String> {
    resident.set(temp);
    Ok(())
}

fn emit_startup(out: &mut String, vars: &Vars, globals: &Globals) {
    out.push_str(".load 0x0100\n.entry __start\n\n.proc __start\n");

    for (name, (_, init)) in globals {
        let Some(value) = init else { continue };
        let var = &vars[name];
        let store = if var.width == 1 { "STORE8" } else { "STORE16" };
        out.push_str(&format!(
            "    MOVI R0, {value}\n    {store} [0x{:04X}], R0\n",
            var.addr
        ));
    }

    out.push_str("    CALL main\n    HALT\n.endproc\n\n");
}

fn lower_raw(clir: &str) -> Result<String, String> {
    let lines = lines(clir);

    let layout = Layout::scan(&lines, true)?;
    let vars = &layout.vars;
    let funcs = &layout.funcs;
    let globals = &layout.globals;
    let mut out = String::new();
    emit_startup(&mut out, vars, globals);

    let mut current = String::new();
    let mut label_id = 0usize;
    let mut resident = Resident::default();

    for line in lines {
        if line.starts_with("global ") {
            continue;
        }
        if line.starts_with("fn ") {
            let (name, _) = parse_fn_header(line)?;
            current = name.clone();
            out.push_str(&format!(".proc {name}\n"));
            continue;
        }
        if line == "end" {
            spill_resident(&mut out, &vars, &current, &mut resident)?;
            out.push_str(".endproc\n\n");
            current.clear();
            continue;
        }
        if line.starts_with("local ") {
            continue;
        }
        if line.ends_with(':') {
            spill_resident(&mut out, &vars, &current, &mut resident)?;
            out.push_str(&format!("{}__{}\n", current, line));
            continue;
        }

        emit_instruction(
            &mut out,
            line,
            &current,
            &vars,
            &funcs,
            &mut label_id,
            &mut resident,
        )?;
    }

    Ok(out)
}

fn emit_instruction(
    out: &mut String,
    line: &str,
    function: &str,
    vars: &Vars,
    funcs: &Funcs,
    label_id: &mut usize,
    resident: &mut Resident,
) -> Result<(), String> {
    if let Some(rest) = line.strip_prefix("const.") {
        let (_, args) = rest.split_once(' ').ok_or("bad const")?;
        let (dst, value) = args.split_once(',').ok_or("bad const")?;
        spill_resident(out, vars, function, resident)?;
        out.push_str(&format!("    MOVI R0, {}\n", value.trim()));
        return store_temp(out, vars, function, dst.trim(), "R0", resident);
    }

    if let Some(rest) = line.strip_prefix("load.") {
        let (ty, args) = rest.split_once(' ').ok_or("bad load")?;
        let (dst, name) = args.split_once(',').ok_or("bad load")?;
        let var = resolve(vars, function, name.trim())?;
        spill_resident(out, vars, function, resident)?;
        let load = if width_of(ty) == 1 { "LOAD8" } else { "LOAD16" };
        out.push_str(&format!("    {load} R0, [0x{:04X}]\n", var.addr));
        if ty == "i8" {
            let n = *label_id;
            *label_id += 1;
            out.push_str(&format!(
                "    MOV R1, R0\n    ANDI R1, 0x80\n    JZ __cl_sext_done_{n}\n    ORI R0, 0xFF00\n__cl_sext_done_{n}:\n"
            ));
        }
        return store_temp(out, vars, function, dst.trim(), "R0", resident);
    }

    if let Some(rest) = line.strip_prefix("store.") {
        let (ty, args) = rest.split_once(' ').ok_or("bad store")?;
        let (name, temp) = args.split_once(',').ok_or("bad store")?;
        load_temp(out, vars, function, temp.trim(), "R0", resident)?;
        let var = resolve(vars, function, name.trim())?;
        let store = if width_of(ty) == 1 {
            "STORE8"
        } else {
            "STORE16"
        };
        out.push_str(&format!("    {store} [0x{:04X}], R0\n", var.addr));
        return Ok(());
    }

    if let Some(rest) = line.strip_prefix("addr ") {
        let (dst, name) = rest.split_once(',').ok_or("bad addr")?;
        let var = resolve(vars, function, name.trim())?;
        spill_resident(out, vars, function, resident)?;
        out.push_str(&format!("    MOVI R0, 0x{:04X}\n", var.addr));
        return store_temp(out, vars, function, dst.trim(), "R0", resident);
    }

    if let Some(rest) = line.strip_prefix("index ") {
        let args = rest.split(',').map(str::trim).collect::<Vec<_>>();
        if args.len() != 4 {
            return Err("bad index".into());
        }
        if resident.is(args[2]) {
            out.push_str("    MOV R1, R0\n");
            resident.clear();
            load_temp(out, vars, function, args[1], "R0", resident)?;
        } else {
            load_temp(out, vars, function, args[1], "R0", resident)?;
            load_temp(out, vars, function, args[2], "R1", resident)?;
        }
        if args[3] == "2" {
            out.push_str("    SHL1 R1\n");
        }
        out.push_str("    ADD R0, R1\n");
        return store_temp(out, vars, function, args[0], "R0", resident);
    }

    if line.starts_with("loadmem") {
        return emit_loadmem(out, line, function, vars, resident);
    }
    if line.starts_with("storemem") {
        return emit_storemem(out, line, function, vars, resident);
    }

    if line.strip_prefix("drop ").is_some() {
        if let Some(temp) = line.strip_prefix("drop ") {
            if resident.is(temp.trim()) {
                resident.clear();
            }
        }
        return Ok(());
    }

    if let Some(label) = line.strip_prefix("jmp ") {
        spill_resident(out, vars, function, resident)?;
        out.push_str(&format!("    JMP {function}__{}\n", label.trim()));
        return Ok(());
    }

    if let Some(rest) = line.strip_prefix("jz ") {
        let (temp, label) = rest.split_once(',').ok_or("bad jz")?;
        load_temp(out, vars, function, temp.trim(), "R0", resident)?;
        out.push_str(&format!(
            "    CMPI R0, 0\n    JZ {function}__{}\n",
            label.trim()
        ));
        return Ok(());
    }

    if line == "ret" {
        out.push_str("    RET\n");
        return Ok(());
    }

    if let Some(temp) = line.strip_prefix("ret ") {
        load_temp(out, vars, function, temp.trim(), "R0", resident)?;
        out.push_str("    RET\n");
        return Ok(());
    }

    if line.starts_with("call") {
        return emit_call(out, line, function, funcs, vars, resident);
    }

    if let Some((op_ty, args)) = line.split_once(' ') {
        if let Some((op, ty)) = op_ty.split_once('.') {
            return emit_operation(out, op, ty, args, function, vars, label_id, resident);
        }
    }

    Err(format!("unsupported CLIR line: {line}"))
}

fn emit_loadmem(
    out: &mut String,
    line: &str,
    function: &str,
    vars: &Vars,
    resident: &mut Resident,
) -> Result<(), String> {
    let volatile = line.starts_with("loadmemv");
    let dot = line.find('.').ok_or("bad loadmem")?;
    let (ty, args) = line[dot + 1..].split_once(' ').ok_or("bad loadmem")?;
    let (dst, address) = args.split_once(',').ok_or("bad loadmem")?;

    if !resident.is(address.trim()) {
        spill_resident(out, vars, function, resident)?;
    }
    load_temp(out, vars, function, address.trim(), "R7", resident)?;
    let op = match (volatile, width_of(ty)) {
        (false, 1) => "LOAD8",
        (false, _) => "LOAD16",
        (true, 1) => "VLOAD8",
        (true, _) => "VLOAD16",
    };
    out.push_str(&format!("    {op} R0, [R7]\n"));
    store_temp(out, vars, function, dst.trim(), "R0", resident)
}

fn emit_storemem(
    out: &mut String,
    line: &str,
    function: &str,
    vars: &Vars,
    resident: &mut Resident,
) -> Result<(), String> {
    let volatile = line.starts_with("storememv");
    let dot = line.find('.').ok_or("bad storemem")?;
    let (ty, args) = line[dot + 1..].split_once(' ').ok_or("bad storemem")?;
    let (address, value) = args.split_once(',').ok_or("bad storemem")?;

    if !resident.is(value.trim()) && !resident.is(address.trim()) {
        spill_resident(out, vars, function, resident)?;
    }
    if resident.is(value.trim()) {
        resident.clear();
        load_temp(out, vars, function, address.trim(), "R7", resident)?;
    } else if resident.is(address.trim()) {
        out.push_str("    MOV R7, R0\n");
        resident.clear();
        load_temp(out, vars, function, value.trim(), "R0", resident)?;
    } else {
        load_temp(out, vars, function, address.trim(), "R7", resident)?;
        load_temp(out, vars, function, value.trim(), "R0", resident)?;
    }
    let op = match (volatile, width_of(ty)) {
        (false, 1) => "STORE8",
        (false, _) => "STORE16",
        (true, 1) => "VSTORE8",
        (true, _) => "VSTORE16",
    };
    out.push_str(&format!("    {op} [R7], R0\n"));
    Ok(())
}

fn emit_operation(
    out: &mut String,
    op: &str,
    ty: &str,
    args: &str,
    function: &str,
    vars: &Vars,
    label_id: &mut usize,
    resident: &mut Resident,
) -> Result<(), String> {
    if matches!(op, "neg" | "not") {
        let (dst, src) = args.split_once(',').ok_or("bad unary")?;
        load_temp(out, vars, function, src.trim(), "R0", resident)?;
        out.push_str(if op == "neg" {
            "    NEG R0\n"
        } else {
            "    NOT R0\n"
        });
        normalize(out, ty, "R0", label_id);
        return store_temp(out, vars, function, dst.trim(), "R0", resident);
    }

    let parts = args.split(',').map(str::trim).collect::<Vec<_>>();
    if parts.len() != 3 {
        return Err(format!("bad binary CLIR: {op}.{ty} {args}"));
    }

    if matches!(op, "eq" | "ne" | "lt" | "le" | "gt" | "ge")
        || (matches!(op, "div" | "mod") && matches!(ty, "i8" | "i16"))
    {
        if resident.is(parts[2]) {
            out.push_str("    MOV R1, R0\n");
            resident.clear();
            load_temp(out, vars, function, parts[1], "R0", resident)?;
        } else {
            load_temp(out, vars, function, parts[1], "R0", resident)?;
            load_temp(out, vars, function, parts[2], "R1", resident)?;
        }
        if matches!(op, "eq" | "ne" | "lt" | "le" | "gt" | "ge") {
            emit_compare(out, op, ty, label_id);
        } else {
            emit_signed_divmod(out, op, ty, label_id);
        }
    } else {
        let mnemonic = match op {
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
            _ => return Err(format!("unsupported CLIR op {op}")),
        };

        if resident.is(parts[1]) {
            let right = resolve(vars, function, parts[2])?;
            resident.clear();
            out.push_str(&format!("    {mnemonic} R0, [0x{:04X}]\n", right.addr));
        } else if resident.is(parts[2]) {
            // Preserve operand order for non-commutative operations.
            out.push_str("    MOV R1, R0\n");
            resident.clear();
            load_temp(out, vars, function, parts[1], "R0", resident)?;
            out.push_str(&format!("    {mnemonic} R0, R1\n"));
        } else {
            load_temp(out, vars, function, parts[1], "R0", resident)?;
            let right = resolve(vars, function, parts[2])?;
            out.push_str(&format!("    {mnemonic} R0, [0x{:04X}]\n", right.addr));
        }
        normalize(out, ty, "R0", label_id);
    }

    store_temp(out, vars, function, parts[0], "R0", resident)
}

fn normalize(out: &mut String, ty: &str, reg: &str, label_id: &mut usize) {
    if ty == "bool" {
        out.push_str(&format!("    ANDI {reg}, 1\n"));
        return;
    }

    if !matches!(ty, "u8" | "i8") {
        return;
    }

    out.push_str(&format!("    ANDI {reg}, 0xFF\n"));
    if ty == "i8" {
        let n = *label_id;
        *label_id += 1;
        out.push_str(&format!(
            "    MOV R6, {reg}\n    ANDI R6, 0x80\n    JZ __cl_norm_{n}\n    ORI {reg}, 0xFF00\n__cl_norm_{n}:\n"
        ));
    }
}

fn emit_compare(out: &mut String, op: &str, ty: &str, label_id: &mut usize) {
    if ty == "i8" {
        out.push_str("    XORI R0, 0x80\n    XORI R1, 0x80\n");
    } else if ty == "i16" {
        out.push_str("    XORI R0, 0x8000\n    XORI R1, 0x8000\n");
    }

    out.push_str("    CMP R0, R1\n");
    let n = *label_id;
    *label_id += 1;
    let true_label = format!("__cl_cmp_t_{n}");
    let false_label = format!("__cl_cmp_f_{n}");
    let end_label = format!("__cl_cmp_e_{n}");

    match op {
        "eq" => out.push_str(&format!("    JZ {true_label}\n")),
        "ne" => out.push_str(&format!("    JNZ {true_label}\n")),
        "lt" => out.push_str(&format!("    JNC {true_label}\n")),
        "ge" => out.push_str(&format!("    JC {true_label}\n")),
        "le" => out.push_str(&format!("    JNC {true_label}\n    JZ {true_label}\n")),
        "gt" => out.push_str(&format!(
            "    JNC {false_label}\n    JZ {false_label}\n    JMP {true_label}\n"
        )),
        _ => {}
    }

    out.push_str(&format!(
        "{false_label}:\n    MOVI R0, 0\n    JMP {end_label}\n{true_label}:\n    MOVI R0, 1\n{end_label}:\n"
    ));
}

fn emit_signed_divmod(out: &mut String, op: &str, ty: &str, label_id: &mut usize) {
    normalize(out, ty, "R0", label_id);
    normalize(out, ty, "R1", label_id);

    let n = *label_id;
    *label_id += 1;
    out.push_str(&format!(
        "    MOVI R2, 0\n    CMPI R0, 0\n    JNN __cl_sd_a_{n}\n    NEG R0\n    MOVI R2, 1\n__cl_sd_a_{n}:\n    CMPI R1, 0\n    JNN __cl_sd_b_{n}\n    NEG R1\n"
    ));

    if op == "div" {
        out.push_str(&format!(
            "    XORI R2, 1\n__cl_sd_b_{n}:\n    DIV R0, R1\n    CMPI R2, 0\n    JZ __cl_sd_e_{n}\n    NEG R0\n__cl_sd_e_{n}:\n"
        ));
    } else {
        out.push_str(&format!(
            "__cl_sd_b_{n}:\n    MOD R0, R1\n    CMPI R2, 0\n    JZ __cl_sd_e_{n}\n    NEG R0\n__cl_sd_e_{n}:\n"
        ));
    }

    normalize(out, ty, "R0", label_id);
}

fn emit_call(
    out: &mut String,
    line: &str,
    current: &str,
    funcs: &Funcs,
    vars: &Vars,
    resident: &mut Resident,
) -> Result<(), String> {
    let (dst, name, args) = parse_call(line)?;

    let function = funcs
        .get(name)
        .ok_or_else(|| format!("unknown call target {name}"))?;
    if args.len() != function.params.len() {
        return Err(format!("call arity mismatch for {name}"));
    }

    for (arg, (param, ty)) in args.iter().zip(&function.params) {
        load_temp(out, vars, current, arg, "R0", resident)?;
        let slot = resolve(vars, name, param)?;
        let store = if width_of(ty) == 1 {
            "STORE8"
        } else {
            "STORE16"
        };
        out.push_str(&format!("    {store} [0x{:04X}], R0\n", slot.addr));
    }

    spill_resident(out, vars, current, resident)?;
    out.push_str(&format!("    CALL {name}\n"));
    if let Some(dst) = dst {
        store_temp(out, vars, current, dst, "R0", resident)?;
    }
    Ok(())
}

pub fn lower(clir: &str) -> Result<String, String> {
    lower_raw(clir)
}

#[cfg(test)]
mod direct_tests {
    use super::*;

    #[test]
    fn lowers_clir_directly() {
        let c = "fn main() -> u16\n  const.u16 %0, 7\n  ret %0\nend\n";
        let a = lower(c).unwrap();
        assert!(a.contains("MOVI R0, 7"));
    }

    #[test]
    fn uses_native_memory_source_for_binary_op() {
        // Force both ADD operands out of the resident R0 value so the right
        // operand must be consumed directly from its native memory slot.
        let c = "fn main() -> u16\n  const.u16 %0, 7\n  const.u16 %1, 5\n  const.u16 %2, 99\n  add.u16 %3, %0, %1\n  ret %3\nend\n";
        let a = lower(c).unwrap();
        assert!(a.contains("ADD R0, [0x"));
    }

    #[test]
    fn static_memory_uses_native_absolute_descriptors() {
        let clir = "fn main() -> u16\n  local u16 x\n  const.u16 %0, 7\n  store.u16 x, %0\n  load.u16 %1, x\n  ret %1\nend\n";
        let asm = lower(clir).unwrap();
        assert!(asm.contains("STORE16 [0x"));
        assert!(asm.contains("LOAD16 R0, [0x"));
        assert!(!asm.contains("MOVI R7"));
    }

    #[test]
    fn keeps_latest_expression_temp_in_r0() {
        let clir = "fn main() -> u16\n  const.u16 %0, 7\n  const.u16 %1, 5\n  mul.u16 %2, %0, %1\n  ret %2\nend\n";
        let asm = lower(clir).unwrap();
        let spills = asm
            .lines()
            .filter(|line| line.trim_start().starts_with("STORE16 "))
            .count();
        assert_eq!(spills, 1, "only the displaced first temp should spill");
    }
}
