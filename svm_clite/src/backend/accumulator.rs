use super::layout::{Layout, Var, lines, parse_call, parse_fn_header, width_of};

#[derive(Default)]
struct Resident {
    temp: Option<String>,
}

impl Resident {
    fn is(&self, temp: &str) -> bool {
        self.temp.as_deref() == Some(temp)
    }

    fn take(&mut self) -> Option<String> {
        self.temp.take()
    }

    fn set(&mut self, temp: &str) {
        self.temp = Some(temp.to_string());
    }

    fn clear(&mut self) {
        self.temp = None;
    }
}

fn spill_resident(
    out: &mut String,
    layout: &Layout,
    function: &str,
    resident: &mut Resident,
) -> Result<(), String> {
    let Some(temp) = resident.take() else {
        return Ok(());
    };
    let var = layout.resolve(function, &temp)?;
    out.push_str(&format!("    STA16 0x{:04X}\n", var.addr));
    Ok(())
}

fn load_temp(
    out: &mut String,
    layout: &Layout,
    function: &str,
    temp: &str,
    resident: &mut Resident,
) -> Result<(), String> {
    if resident.is(temp) {
        resident.clear();
        return Ok(());
    }
    spill_resident(out, layout, function, resident)?;
    let var = layout.resolve(function, temp)?;
    out.push_str(&format!("    LDA16 0x{:04X}\n", var.addr));
    Ok(())
}

fn store_temp(
    _out: &mut String,
    _layout: &Layout,
    _function: &str,
    temp: &str,
    resident: &mut Resident,
) -> Result<(), String> {
    resident.set(temp);
    Ok(())
}

fn load_var(out: &mut String, var: &Var, ty: &str, label_id: &mut usize) {
    let op = if var.width == 1 { "LDA8" } else { "LDA16" };
    out.push_str(&format!("    {op} 0x{:04X}\n", var.addr));
    if ty == "i8" {
        let n = *label_id;
        *label_id += 1;
        out.push_str(&format!(
            "    ANDI 0x00FF\n    TAX\n    ANDI 0x0080\n    JZ __cl_acc_sext_pos_{n}\n    TXA\n    ORI 0xFF00\n    JMP __cl_acc_sext_end_{n}\n__cl_acc_sext_pos_{n}:\n    TXA\n__cl_acc_sext_end_{n}:\n"
        ));
    }
}

fn store_var(out: &mut String, var: &Var) {
    let op = if var.width == 1 { "STA8" } else { "STA16" };
    out.push_str(&format!("    {op} 0x{:04X}\n", var.addr));
}

fn normalize(out: &mut String, ty: &str, label_id: &mut usize) {
    match ty {
        "bool" => out.push_str("    ANDI 1\n"),
        "u8" => out.push_str("    ANDI 0x00FF\n"),
        "i8" => {
            let n = *label_id;
            *label_id += 1;
            out.push_str(&format!(
                "    ANDI 0x00FF\n    TAX\n    ANDI 0x0080\n    JZ __cl_acc_norm_pos_{n}\n    TXA\n    ORI 0xFF00\n    JMP __cl_acc_norm_end_{n}\n__cl_acc_norm_pos_{n}:\n    TXA\n__cl_acc_norm_end_{n}:\n"
            ));
        }
        _ => {}
    }
}

fn emit_startup(out: &mut String, layout: &Layout) {
    out.push_str(".load 0x0100\n.entry __start\n\n.proc __start\n");
    for (name, (_, init)) in &layout.globals {
        let Some(value) = init else { continue };
        let var = &layout.vars[name];
        out.push_str(&format!("    LDAI {value}\n"));
        store_var(out, var);
    }
    out.push_str("    CALL main\n    HALT\n.endproc\n\n");
}

fn emit_compare(
    out: &mut String,
    layout: &Layout,
    function: &str,
    op: &str,
    ty: &str,
    left: &str,
    right: &str,
    label_id: &mut usize,
    resident: &mut Resident,
) -> Result<(), String> {
    load_temp(out, layout, function, right, resident)?;
    if ty == "i8" {
        out.push_str("    XORI 0x0080\n");
    } else if ty == "i16" {
        out.push_str("    XORI 0x8000\n");
    }
    out.push_str("    TAX\n");
    load_temp(out, layout, function, left, resident)?;
    if ty == "i8" {
        out.push_str("    XORI 0x0080\n");
    } else if ty == "i16" {
        out.push_str("    XORI 0x8000\n");
    }
    out.push_str("    CMPX\n");

    let n = *label_id;
    *label_id += 1;
    let t = format!("__cl_acc_cmp_t_{n}");
    let f = format!("__cl_acc_cmp_f_{n}");
    let e = format!("__cl_acc_cmp_e_{n}");
    match op {
        "eq" => out.push_str(&format!("    JZ {t}\n")),
        "ne" => out.push_str(&format!("    JNZ {t}\n")),
        "lt" => out.push_str(&format!("    JNC {t}\n")),
        "ge" => out.push_str(&format!("    JC {t}\n")),
        "le" => out.push_str(&format!("    JNC {t}\n    JZ {t}\n")),
        "gt" => out.push_str(&format!("    JNC {f}\n    JZ {f}\n    JMP {t}\n")),
        _ => return Err(format!("bad compare {op}")),
    }
    out.push_str(&format!(
        "{f}:\n    LDAI 0\n    JMP {e}\n{t}:\n    LDAI 1\n{e}:\n"
    ));
    Ok(())
}

fn emit_signed_divmod(
    out: &mut String,
    layout: &Layout,
    function: &str,
    op: &str,
    left: &str,
    right: &str,
    scratch: [u16; 3],
    label_id: &mut usize,
    resident: &mut Resident,
) -> Result<(), String> {
    let [a, b, sign] = scratch;
    spill_resident(out, layout, function, resident)?;
    let n = *label_id;
    *label_id += 1;

    out.push_str("    LDAI 0\n");
    out.push_str(&format!("    STA16 0x{sign:04X}\n"));

    load_temp(out, layout, function, left, resident)?;
    out.push_str(&format!(
        "    CMPI 0\n    JNN __cl_acc_sd_a_{n}\n    NEG\n    STA16 0x{a:04X}\n    LDAI 1\n    STA16 0x{sign:04X}\n    JMP __cl_acc_sd_beg_{n}\n__cl_acc_sd_a_{n}:\n    STA16 0x{a:04X}\n__cl_acc_sd_beg_{n}:\n"
    ));

    load_temp(out, layout, function, right, resident)?;
    out.push_str(&format!(
        "    CMPI 0\n    JNN __cl_acc_sd_b_{n}\n    NEG\n    STA16 0x{b:04X}\n"
    ));
    if op == "div" {
        out.push_str(&format!(
            "    LDA16 0x{sign:04X}\n    XORI 1\n    STA16 0x{sign:04X}\n"
        ));
    }
    out.push_str(&format!(
        "    JMP __cl_acc_sd_calc_{n}\n__cl_acc_sd_b_{n}:\n    STA16 0x{b:04X}\n__cl_acc_sd_calc_{n}:\n    LDA16 0x{b:04X}\n    TAX\n    LDA16 0x{a:04X}\n    {}X\n    STA16 0x{a:04X}\n    LDA16 0x{sign:04X}\n    CMPI 0\n    JZ __cl_acc_sd_done_{n}\n    LDA16 0x{a:04X}\n    NEG\n    JMP __cl_acc_sd_end_{n}\n__cl_acc_sd_done_{n}:\n    LDA16 0x{a:04X}\n__cl_acc_sd_end_{n}:\n",
        if op == "div" { "DIV" } else { "MOD" }
    ));
    Ok(())
}

pub fn lower(clir: &str) -> Result<String, String> {
    let lines = lines(clir);
    let mut layout = Layout::scan(&lines, true)?;
    let scratch = [layout.scratch(2)?, layout.scratch(2)?, layout.scratch(2)?];
    let mut out = String::new();
    emit_startup(&mut out, &layout);

    let mut current = String::new();
    let mut label_id = 0usize;
    let mut resident = Resident::default();

    for line in lines {
        if line.starts_with("global ") || line.starts_with("local ") {
            continue;
        }
        if line.starts_with("fn ") {
            let (name, _) = parse_fn_header(line)?;
            current = name.clone();
            out.push_str(&format!(".proc {name}\n"));
            continue;
        }
        if line == "end" {
            spill_resident(&mut out, &layout, &current, &mut resident)?;
            out.push_str(".endproc\n\n");
            current.clear();
            continue;
        }
        if line.ends_with(':') {
            spill_resident(&mut out, &layout, &current, &mut resident)?;
            out.push_str(&format!("{}__{}\n", current, line));
            continue;
        }

        if let Some(rest) = line.strip_prefix("const.") {
            let (_, args) = rest.split_once(' ').ok_or("bad const")?;
            let (dst, value) = args.split_once(',').ok_or("bad const")?;
            spill_resident(&mut out, &layout, &current, &mut resident)?;
            out.push_str(&format!("    LDAI {}\n", value.trim()));
            store_temp(&mut out, &layout, &current, dst.trim(), &mut resident)?;
            continue;
        }
        if let Some(rest) = line.strip_prefix("load.") {
            let (ty, args) = rest.split_once(' ').ok_or("bad load")?;
            let (dst, name) = args.split_once(',').ok_or("bad load")?;
            let var = layout.resolve(&current, name.trim())?;
            spill_resident(&mut out, &layout, &current, &mut resident)?;
            load_var(&mut out, &var, ty, &mut label_id);
            store_temp(&mut out, &layout, &current, dst.trim(), &mut resident)?;
            continue;
        }
        if let Some(rest) = line.strip_prefix("store.") {
            let (_, args) = rest.split_once(' ').ok_or("bad store")?;
            let (name, temp) = args.split_once(',').ok_or("bad store")?;
            load_temp(&mut out, &layout, &current, temp.trim(), &mut resident)?;
            let var = layout.resolve(&current, name.trim())?;
            store_var(&mut out, &var);
            continue;
        }
        if let Some(rest) = line.strip_prefix("addr ") {
            let (dst, name) = rest.split_once(',').ok_or("bad addr")?;
            let var = layout.resolve(&current, name.trim())?;
            spill_resident(&mut out, &layout, &current, &mut resident)?;
            out.push_str(&format!("    LDAI 0x{:04X}\n", var.addr));
            store_temp(&mut out, &layout, &current, dst.trim(), &mut resident)?;
            continue;
        }
        if let Some(rest) = line.strip_prefix("index ") {
            let a = rest.split(',').map(str::trim).collect::<Vec<_>>();
            if a.len() != 4 {
                return Err("bad index".into());
            }
            load_temp(&mut out, &layout, &current, a[2], &mut resident)?;
            if a[3] == "2" {
                out.push_str("    SHL1\n");
            }
            out.push_str("    TAX\n");
            load_temp(&mut out, &layout, &current, a[1], &mut resident)?;
            out.push_str("    ADDX\n");
            store_temp(&mut out, &layout, &current, a[0], &mut resident)?;
            continue;
        }
        if line.starts_with("loadmem") {
            let volatile = line.starts_with("loadmemv");
            let dot = line.find('.').ok_or("bad loadmem")?;
            let (ty, args) = line[dot + 1..].split_once(' ').ok_or("bad loadmem")?;
            let (dst, address) = args.split_once(',').ok_or("bad loadmem")?;
            load_temp(&mut out, &layout, &current, address.trim(), &mut resident)?;
            out.push_str("    TAX\n");
            let op = match (volatile, width_of(ty)) {
                (false, 1) => "LDA8",
                (false, _) => "LDA16",
                (true, 1) => "VLDA8",
                (true, _) => "VLDA16",
            };
            out.push_str(&format!("    {op} [X]\n"));
            if ty == "i8" {
                normalize(&mut out, "i8", &mut label_id);
            }
            store_temp(&mut out, &layout, &current, dst.trim(), &mut resident)?;
            continue;
        }
        if line.starts_with("storemem") {
            let volatile = line.starts_with("storememv");
            let dot = line.find('.').ok_or("bad storemem")?;
            let (ty, args) = line[dot + 1..].split_once(' ').ok_or("bad storemem")?;
            let (address, value) = args.split_once(',').ok_or("bad storemem")?;
            load_temp(&mut out, &layout, &current, address.trim(), &mut resident)?;
            out.push_str("    TAX\n");
            load_temp(&mut out, &layout, &current, value.trim(), &mut resident)?;
            let op = match (volatile, width_of(ty)) {
                (false, 1) => "STA8",
                (false, _) => "STA16",
                (true, 1) => "VSTA8",
                (true, _) => "VSTA16",
            };
            out.push_str(&format!("    {op} [X]\n"));
            continue;
        }
        if let Some(temp) = line.strip_prefix("drop ") {
            if resident.is(temp.trim()) {
                resident.clear();
            }
            continue;
        }
        if let Some(label) = line.strip_prefix("jmp ") {
            spill_resident(&mut out, &layout, &current, &mut resident)?;
            out.push_str(&format!("    JMP {current}__{}\n", label.trim()));
            continue;
        }
        if let Some(rest) = line.strip_prefix("jz ") {
            let (temp, label) = rest.split_once(',').ok_or("bad jz")?;
            load_temp(&mut out, &layout, &current, temp.trim(), &mut resident)?;
            out.push_str(&format!("    CMPI 0\n    JZ {current}__{}\n", label.trim()));
            continue;
        }
        if line == "ret" {
            out.push_str("    RET\n");
            continue;
        }
        if let Some(temp) = line.strip_prefix("ret ") {
            load_temp(&mut out, &layout, &current, temp.trim(), &mut resident)?;
            out.push_str("    RET\n");
            continue;
        }
        if line.starts_with("call") {
            let (dst, name, args) = parse_call(line)?;
            let function = layout
                .funcs
                .get(name)
                .ok_or_else(|| format!("unknown call target {name}"))?;
            if args.len() != function.params.len() {
                return Err(format!("call arity mismatch for {name}"));
            }
            for (arg, (param, _)) in args.iter().zip(&function.params) {
                load_temp(&mut out, &layout, &current, arg, &mut resident)?;
                let slot = layout.resolve(name, param)?;
                store_var(&mut out, &slot);
            }
            spill_resident(&mut out, &layout, &current, &mut resident)?;
            out.push_str(&format!("    CALL {name}\n"));
            if let Some(dst) = dst {
                store_temp(&mut out, &layout, &current, dst, &mut resident)?;
            }
            continue;
        }

        if let Some((op_ty, args)) = line.split_once(' ') {
            if let Some((op, ty)) = op_ty.split_once('.') {
                if matches!(op, "neg" | "not") {
                    let (dst, src) = args.split_once(',').ok_or("bad unary")?;
                    load_temp(&mut out, &layout, &current, src.trim(), &mut resident)?;
                    out.push_str(if op == "neg" {
                        "    NEG\n"
                    } else {
                        "    NOT\n"
                    });
                    normalize(&mut out, ty, &mut label_id);
                    store_temp(&mut out, &layout, &current, dst.trim(), &mut resident)?;
                    continue;
                }
                let p = args.split(',').map(str::trim).collect::<Vec<_>>();
                if p.len() != 3 {
                    return Err(format!("bad binary CLIR: {line}"));
                }
                if matches!(op, "eq" | "ne" | "lt" | "le" | "gt" | "ge") {
                    emit_compare(
                        &mut out,
                        &layout,
                        &current,
                        op,
                        ty,
                        p[1],
                        p[2],
                        &mut label_id,
                        &mut resident,
                    )?;
                } else if matches!(op, "div" | "mod") && matches!(ty, "i8" | "i16") {
                    emit_signed_divmod(
                        &mut out,
                        &layout,
                        &current,
                        op,
                        p[1],
                        p[2],
                        scratch,
                        &mut label_id,
                        &mut resident,
                    )?;
                } else {
                    load_temp(&mut out, &layout, &current, p[2], &mut resident)?;
                    out.push_str("    TAX\n");
                    load_temp(&mut out, &layout, &current, p[1], &mut resident)?;
                    let asm = match op {
                        "add" => "ADDX",
                        "sub" => "SUBX",
                        "mul" => "MULX",
                        "div" => "DIVX",
                        "mod" => "MODX",
                        "and" => "ANDX",
                        "or" => "ORX",
                        "xor" => "XORX",
                        "shl" => "SHLX",
                        "shr" => "SHRX",
                        _ => return Err(format!("unsupported accumulator CLIR op {op}")),
                    };
                    out.push_str(&format!("    {asm}\n"));
                    normalize(&mut out, ty, &mut label_id);
                }
                store_temp(&mut out, &layout, &current, p[0], &mut resident)?;
                continue;
            }
        }

        return Err(format!("unsupported Accumulator CLIR line: {line}"));
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowers_clir_without_virtual_register_file() {
        let clir = "fn main() -> u16\n  const.u16 %0, 7\n  const.u16 %1, 5\n  mul.u16 %2, %0, %1\n  ret %2\nend\n";
        let asm = lower(clir).unwrap();
        assert!(asm.contains("LDAI 7"));
        assert!(asm.contains("MULX"));
        assert!(!asm.contains("0x00D0"));
        assert!(!asm.contains(" R0"));
        assert_eq!(
            asm.matches("STA16").count(),
            1,
            "only the displaced first temp should spill"
        );
    }
}
