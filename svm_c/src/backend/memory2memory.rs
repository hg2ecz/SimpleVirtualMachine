use crate::{backend_register, model::*};
fn vr(s: &str) -> Result<String, String> {
    let u = s.trim().to_ascii_uppercase();
    let n = u
        .strip_prefix('R')
        .ok_or_else(|| format!("expected virtual register, got {s}"))?
        .parse::<u8>()
        .map_err(|_| format!("bad virtual register {s}"))?;
    if n > 7 {
        return Err(format!("virtual register out of range: {s}"));
    }
    Ok(format!("[0x{:04X}]", u16::from(n) * 2))
}
fn split_ops(s: &str) -> Vec<String> {
    s.split(',')
        .map(|x| x.trim().to_string())
        .filter(|x| !x.is_empty())
        .collect()
}
fn ptr_reg(mem: &str) -> Result<String, String> {
    let t = mem.trim();
    if !(t.starts_with('[') && t.ends_with(']')) {
        return Err(format!("expected [Rn], got {mem}"));
    }
    let inner = t[1..t.len() - 1]
        .trim()
        .trim_end_matches('+')
        .trim_start_matches('-')
        .trim();
    vr(inner)
}
fn translate(register_asm: &str) -> Result<String, String> {
    let mut out = String::new();
    for raw in register_asm.lines() {
        let t = raw.trim();
        if t.is_empty() {
            out.push('\n');
            continue;
        }
        if t.starts_with(';') || t.starts_with('.') || t.ends_with(':') {
            out.push_str(t);
            out.push('\n');
            continue;
        }
        let mut p = t.splitn(2, char::is_whitespace);
        let m = p.next().unwrap_or("").to_ascii_uppercase();
        let a = split_ops(p.next().unwrap_or(""));
        let mut emit = |s: &str| {
            out.push_str("    ");
            out.push_str(s);
            out.push('\n');
        };
        match m.as_str() {
            "NOP" | "HALT" | "RET" | "EI" | "DI" | "IRET" => emit(&m),
            "JMP" | "JZ" | "JNZ" | "JC" | "JNC" | "JN" | "JNN" | "CALL" => {
                if a.len() != 1 {
                    return Err(format!("bad {m}"));
                }
                emit(&format!("{m} {}", a[0]));
            }
            "MOVI" => {
                if a.len() != 2 {
                    return Err("bad MOVI".into());
                }
                emit(&format!("MOV16 {}, {}", vr(&a[0])?, a[1]));
            }
            "MOV" => {
                if a.len() != 2 {
                    return Err("bad MOV".into());
                }
                emit(&format!("MOV16 {}, {}", vr(&a[0])?, vr(&a[1])?));
            }
            "ADD" | "SUB" | "AND" | "OR" | "XOR" | "MUL" | "DIV" | "MOD" | "SHL" | "SHR"
            | "MULQ15" | "CMP" => {
                if a.len() != 2 {
                    return Err(format!("bad {m}"));
                }
                let op = match m.as_str() {
                    "ADD" => "ADD16",
                    "SUB" => "SUB16",
                    "AND" => "AND16",
                    "OR" => "OR16",
                    "XOR" => "XOR16",
                    "MUL" => "MUL16",
                    "DIV" => "DIV16",
                    "MOD" => "MOD16",
                    "SHL" => "SHL16",
                    "SHR" => "SHR16",
                    "MULQ15" => "MULQ15",
                    _ => "CMP16",
                };
                emit(&format!("{op} {}, {}", vr(&a[0])?, vr(&a[1])?));
            }
            "ADDI" | "SUBI" | "ANDI" | "ORI" | "XORI" | "CMPI" => {
                if a.len() != 2 {
                    return Err(format!("bad {m}"));
                }
                let op = match m.as_str() {
                    "ADDI" => "ADD16",
                    "SUBI" => "SUB16",
                    "ANDI" => "AND16",
                    "ORI" => "OR16",
                    "XORI" => "XOR16",
                    _ => "CMP16",
                };
                emit(&format!("{op} {}, {}", vr(&a[0])?, a[1]));
            }
            "NOT" | "NEG" | "INC" | "DEC" | "ASR1" => {
                if a.len() != 1 {
                    return Err(format!("bad {m}"));
                }
                let op = match m.as_str() {
                    "NOT" => "NOT16",
                    "NEG" => "NEG16",
                    "INC" => "INC16",
                    "DEC" => "DEC16",
                    _ => "ASR1",
                };
                emit(&format!("{op} {}", vr(&a[0])?));
            }
            "SHL1" | "SHR1" => {
                if a.len() != 1 {
                    return Err(format!("bad {m}"));
                }
                emit(&format!("{m} {}", vr(&a[0])?));
            }
            "PUSH" => {
                if a.len() != 1 {
                    return Err("bad PUSH".into());
                }
                emit("ADDA A3, -2");
                emit(&format!("MOV16 [A3], {}", vr(&a[0])?));
            }
            "POP" => {
                if a.len() != 1 {
                    return Err("bad POP".into());
                }
                emit(&format!("MOV16 {}, [A3]", vr(&a[0])?));
                emit("ADDA A3, 2");
            }
            "ZLOAD8" | "ZLOAD16" => {
                if a.len() != 1 {
                    return Err(format!("bad {m}"));
                }
                if m == "ZLOAD8" {
                    emit("MOV16 [0x0000], 0");
                    emit(&format!("MOV8 [0x0000], [{}]", a[0]));
                } else {
                    emit(&format!("MOV16 [0x0000], [{}]", a[0]));
                }
            }
            "ZSTORE8" | "ZSTORE16" => {
                if a.len() != 1 {
                    return Err(format!("bad {m}"));
                }
                emit(&format!(
                    "{} [{}], [0x0000]",
                    if m == "ZSTORE8" { "MOV8" } else { "MOV16" },
                    a[0]
                ));
            }
            "LOAD8" | "LOAD16" => {
                if a.len() != 2 {
                    return Err(format!("bad {m}"));
                }
                let p = ptr_reg(&a[1])?;
                emit(&format!("MOVA A0, {p}"));
                if m == "LOAD8" {
                    emit(&format!("MOV16 {}, 0", vr(&a[0])?));
                    emit(&format!("MOV8 {}, [A0]", vr(&a[0])?));
                } else {
                    emit(&format!("MOV16 {}, [A0]", vr(&a[0])?));
                }
            }
            "STORE8" | "STORE16" => {
                if a.len() != 2 {
                    return Err(format!("bad {m}"));
                }
                let p = ptr_reg(&a[0])?;
                emit(&format!("MOVA A0, {p}"));
                emit(&format!(
                    "{} [A0], {}",
                    if m == "STORE8" { "MOV8" } else { "MOV16" },
                    vr(&a[1])?
                ));
            }
            "VLOAD8" | "VLOAD16" => {
                if a.len() != 2 {
                    return Err(format!("bad {m}"));
                }
                let p = ptr_reg(&a[1])?;
                emit(&format!("MOVA A0, {p}"));
                if m == "VLOAD8" {
                    emit(&format!("MOV16 {}, 0", vr(&a[0])?));
                }
                emit(&format!(
                    "{} {}, [A0]",
                    if m == "VLOAD8" { "VLD8" } else { "VLD16" },
                    vr(&a[0])?
                ));
            }
            "VSTORE8" | "VSTORE16" => {
                if a.len() != 2 {
                    return Err(format!("bad {m}"));
                }
                let p = ptr_reg(&a[0])?;
                emit(&format!("MOVA A0, {p}"));
                emit(&format!(
                    "{} [A0], {}",
                    if m == "VSTORE8" { "VST8" } else { "VST16" },
                    vr(&a[1])?
                ));
            }
            _ => {
                return Err(format!(
                    "memory-to-memory lowering does not know instruction: {t}"
                ));
            }
        }
    }
    Ok(out.replace(
        "Generated by SVM-C for the register-machine target.",
        "Generated by SVM-C for the memory-to-memory target.",
    ))
}
pub fn emit(p: &Program, l: &Layout, opt: OptLevel) -> Result<String, String> {
    translate(&backend_register::emit(p, l, opt)?)
}
