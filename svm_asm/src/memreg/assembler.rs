use super::{
    error::{AsmError, syntax},
    instruction::op,
    program::Program,
};
use std::collections::HashMap;
#[derive(Clone, Debug)]
enum V {
    N(u16),
    S(String),
}
#[derive(Clone, Debug)]
struct I {
    m: String,
    args: Vec<String>,
    line: usize,
    short: bool,
}
#[derive(Clone, Debug)]
enum S {
    I(I),
    Load(V, usize),
    Entry(V, usize),
}
pub fn assemble(src: &str) -> Result<Program, AsmError> {
    let mut ss = Vec::new();
    let mut defs = HashMap::new();
    for (n, raw) in src.lines().enumerate() {
        let line = n + 1;
        let mut t = raw.split_once(';').map_or(raw, |x| x.0).trim();
        if t.is_empty() {
            continue;
        }
        if let Some(k) = t.find(':') {
            let lab = t[..k].trim();
            if defs.insert(lab.to_ascii_lowercase(), ss.len()).is_some() {
                return Err(syntax(line, "duplicate label"));
            }
            t = t[k + 1..].trim();
            if t.is_empty() {
                continue;
            }
        }
        if t.starts_with('.') {
            let mut p = t.split_whitespace();
            let d = p.next().unwrap();
            let v = p
                .next()
                .ok_or_else(|| syntax(line, "directive requires value"))?;
            let x = pv(v);
            match d.to_ascii_lowercase().as_str() {
                ".load" => ss.push(S::Load(x, line)),
                ".entry" => ss.push(S::Entry(x, line)),
                _ => return Err(syntax(line, "unknown directive")),
            }
            continue;
        }
        let mut p = t.splitn(2, char::is_whitespace);
        let m = p.next().unwrap().to_ascii_uppercase();
        let tail = p.next().unwrap_or("").trim();
        let args = if tail.is_empty() {
            vec![]
        } else {
            tail.split(',').map(|x| x.trim().to_string()).collect()
        };
        let i = I {
            m,
            args,
            line,
            short: false,
        };
        len(&i)?;
        ss.push(S::I(i))
    }
    let load = ss
        .iter()
        .find_map(|s| {
            if let S::Load(v, l) = s {
                Some(res(v, &HashMap::new(), *l))
            } else {
                None
            }
        })
        .transpose()?
        .unwrap_or(0x0100);
    loop {
        let offs = offsets(&ss)?;
        let labs = labs(load, &defs, &offs, &ss)?;
        let mut ch = false;
        for (idx, s) in ss.iter_mut().enumerate() {
            let S::I(i) = s else { continue };
            if i.short || !rel(&i.m) {
                continue;
            }
            let Some(a) = i.args.get(0) else { continue };
            let target = res(&pv(a), &labs, i.line)?;
            let next = load as i32 + offs[idx] as i32 + 2;
            let d = target as i32 - next;
            if (-128..=127).contains(&d) {
                i.short = true;
                ch = true
            }
        }
        if !ch {
            break;
        }
    }
    let offs = offsets(&ss)?;
    let lm = labs(load, &defs, &offs, &ss)?;
    let entry = ss
        .iter()
        .find_map(|s| {
            if let S::Entry(v, l) = s {
                Some(res(v, &lm, *l))
            } else {
                None
            }
        })
        .transpose()?
        .unwrap_or(load);
    let mut out = Vec::new();
    for (idx, s) in ss.iter().enumerate() {
        if let S::I(i) = s {
            enc(i, &lm, load, offs[idx], &mut out)?
        }
    }
    Ok(Program {
        load_address: load,
        entry_address: entry,
        payload: out,
    })
}
fn offsets(s: &[S]) -> Result<Vec<usize>, AsmError> {
    let mut o = Vec::new();
    let mut n = 0;
    for x in s {
        o.push(n);
        if let S::I(i) = x {
            n += len(i)?
        }
    }
    o.push(n);
    Ok(o)
}
fn labs(
    load: u16,
    d: &HashMap<String, usize>,
    o: &[usize],
    s: &[S],
) -> Result<HashMap<String, u16>, AsmError> {
    d.iter()
        .map(|(n, p)| {
            let a = load as usize
                + if *p < s.len() {
                    o[*p]
                } else {
                    *o.last().unwrap()
                };
            if a > 0xffff {
                Err(syntax(0, "label overflow"))
            } else {
                Ok((n.clone(), a as u16))
            }
        })
        .collect()
}
fn rel(m: &str) -> bool {
    matches!(
        m,
        "JMP" | "CALL" | "JZ" | "JNZ" | "JC" | "JNC" | "JN" | "JNN"
    )
}
fn noarg(m: &str) -> bool {
    matches!(
        m,
        "NOP"
            | "HALT"
            | "RET"
            | "EI"
            | "DI"
            | "IRET"
            | "PUSHW"
            | "POPW"
            | "INCW"
            | "DECW"
            | "NEGW"
            | "NOTW"
            | "SHL1W"
            | "SHR1W"
            | "RCR1W"
            | "W2F0"
            | "W2F1"
            | "F02W"
            | "F12W"
            | "ASR1W"
            | "LDB0"
            | "LDW0"
            | "STB0"
            | "STW0"
            | "LDB0+"
            | "LDW0+"
            | "STB0+"
            | "STW0+"
            | "LDB0-"
            | "LDW0-"
            | "STB0-"
            | "STW0-"
            | "LDB1"
            | "LDW1"
            | "STB1"
            | "STW1"
            | "LDB1+"
            | "LDW1+"
            | "STB1+"
            | "STW1+"
            | "LDB1-"
            | "LDW1-"
            | "STB1-"
            | "STW1-"
            | "VLDB0"
            | "VLDW0"
            | "VSTB0"
            | "VSTW0"
            | "VLDB0+"
            | "VLDW0+"
            | "VSTB0+"
            | "VSTW0+"
            | "VLDB0-"
            | "VLDW0-"
            | "VSTB0-"
            | "VSTW0-"
            | "VLDB1"
            | "VLDW1"
            | "VSTB1"
            | "VSTW1"
            | "VLDB1+"
            | "VLDW1+"
            | "VSTB1+"
            | "VSTW1+"
            | "VLDB1-"
            | "VLDW1-"
            | "VSTB1-"
            | "VSTW1-"
    )
}
fn imm16(m: &str) -> bool {
    matches!(
        m,
        "LDI" | "FSR0I" | "FSR1I" | "ADDI" | "SUBI" | "CMPI" | "ANDI" | "ORI" | "XORI"
    ) || rel(m)
}
fn direct(m: &str) -> bool {
    matches!(
        m,
        "MOV8"
            | "MOV16"
            | "ADD"
            | "SUB"
            | "AND"
            | "OR"
            | "XOR"
            | "SHL"
            | "SHR"
            | "MUL"
            | "MULQ15"
            | "ADC"
            | "SBC"
            | "MULHU"
            | "DIV"
            | "MOD"
            | "CMP"
            | "INC"
            | "DEC"
    )
}
fn len(i: &I) -> Result<usize, AsmError> {
    if i.m.starts_with("VLD") || i.m.starts_with("VST") {
        if !i.args.is_empty() {
            return Err(syntax(i.line, "instruction takes no operand"));
        }
        return Ok(2);
    }
    if noarg(&i.m) {
        if !i.args.is_empty() {
            return Err(syntax(i.line, "instruction takes no operand"));
        }
        return Ok(1);
    }
    if imm16(&i.m) {
        if i.args.len() != 1 {
            return Err(syntax(i.line, "instruction requires one operand"));
        }
        return Ok(if i.short && rel(&i.m) { 2 } else { 3 });
    }
    if direct(&i.m) {
        return if matches!(i.m.as_str(), "INC" | "DEC" | "CMP") {
            if i.args.len() != 1 {
                return Err(syntax(i.line, "instruction requires file operand"));
            }
            Ok(
                if hot_addr(&i.args[0]).is_some() && matches!(i.m.as_str(), "INC" | "DEC") {
                    2
                } else {
                    2
                },
            )
        } else {
            if i.args.len() != 2 {
                return Err(syntax(i.line, "file operation requires two operands"));
            }
            let h = hot_addr(&i.args[0]).or_else(|| hot_addr(&i.args[1]));
            let can = matches!(i.m.as_str(), "MOV8" | "MOV16" | "ADD" | "AND") && h.is_some();
            Ok(if can { 1 } else { 2 })
        };
    }
    Err(syntax(i.line, format!("unknown instruction {}", i.m)))
}
fn hot_addr(s: &str) -> Option<u8> {
    num(s).ok().filter(|v| *v < 16).map(|v| v as u8)
}
fn enc(
    i: &I,
    l: &HashMap<String, u16>,
    load: u16,
    off: usize,
    o: &mut Vec<u8>,
) -> Result<(), AsmError> {
    use op::*;
    let m = i.m.as_str();
    let vsub = match m {
        "VLDB0" => Some(0x00),
        "VLDW0" => Some(0x01),
        "VSTB0" => Some(0x02),
        "VSTW0" => Some(0x03),
        "VLDB0+" => Some(0x04),
        "VLDW0+" => Some(0x05),
        "VSTB0+" => Some(0x06),
        "VSTW0+" => Some(0x07),
        "VLDB0-" => Some(0x08),
        "VLDW0-" => Some(0x09),
        "VSTB0-" => Some(0x0A),
        "VSTW0-" => Some(0x0B),
        "VLDB1" => Some(0x0C),
        "VLDW1" => Some(0x0D),
        "VSTB1" => Some(0x0E),
        "VSTW1" => Some(0x0F),
        "VLDB1+" => Some(0x10),
        "VLDW1+" => Some(0x11),
        "VSTB1+" => Some(0x12),
        "VSTW1+" => Some(0x13),
        "VLDB1-" => Some(0x14),
        "VLDW1-" => Some(0x15),
        "VSTB1-" => Some(0x16),
        "VSTW1-" => Some(0x17),
        _ => None,
    };
    if let Some(sub) = vsub {
        o.extend_from_slice(&[VEXT, sub]);
        return Ok(());
    }
    let fixed = match m {
        "NOP" => Some(NOP),
        "HALT" => Some(HALT),
        "RET" => Some(RET),
        "EI" => Some(EI),
        "DI" => Some(DI),
        "IRET" => Some(IRET),
        "PUSHW" => Some(PUSHW),
        "POPW" => Some(POPW),
        "INCW" => Some(INCW),
        "DECW" => Some(DECW),
        "NEGW" => Some(NEGW),
        "NOTW" => Some(NOTW),
        "SHL1W" => Some(SHL1W),
        "SHR1W" => Some(SHR1W),
        "RCR1W" => Some(RCR1W),
        "W2F0" => Some(W2F0),
        "W2F1" => Some(W2F1),
        "F02W" => Some(F02W),
        "F12W" => Some(F12W),
        "ASR1W" => Some(ASR1W),
        "LDB0" => Some(LDB0),
        "LDW0" => Some(LDW0),
        "STB0" => Some(STB0),
        "STW0" => Some(STW0),
        "LDB0+" => Some(LDB0P),
        "LDW0+" => Some(LDW0P),
        "STB0+" => Some(STB0P),
        "STW0+" => Some(STW0P),
        "LDB0-" => Some(LDB0M),
        "LDW0-" => Some(LDW0M),
        "STB0-" => Some(STB0M),
        "STW0-" => Some(STW0M),
        "LDB1" => Some(LDB1),
        "LDW1" => Some(LDW1),
        "STB1" => Some(STB1),
        "STW1" => Some(STW1),
        "LDB1+" => Some(LDB1P),
        "LDW1+" => Some(LDW1P),
        "STB1+" => Some(STB1P),
        "STW1+" => Some(STW1P),
        "LDB1-" => Some(LDB1M),
        "LDW1-" => Some(LDW1M),
        "STB1-" => Some(STB1M),
        "STW1-" => Some(STW1M),
        _ => None,
    };
    if let Some(x) = fixed {
        o.push(x);
        return Ok(());
    }
    if imm16(m) {
        let v = res(&pv(&i.args[0]), l, i.line)?;
        if i.short && rel(m) {
            let op = match m {
                "JMP" => RJMP,
                "CALL" => RCALL,
                "JZ" => RJZ,
                "JNZ" => RJNZ,
                "JC" => RJC,
                "JNC" => RJNC,
                "JN" => RJN,
                "JNN" => RJNN,
                _ => unreachable!(),
            };
            let d = v as i32 - (load as i32 + off as i32 + 2);
            o.extend_from_slice(&[op, (d as i8) as u8]);
            return Ok(());
        }
        let op = match m {
            "LDI" => LDI,
            "FSR0I" => FSR0I,
            "FSR1I" => FSR1I,
            "ADDI" => ADDI,
            "SUBI" => SUBI,
            "CMPI" => CMPI,
            "ANDI" => ANDI,
            "ORI" => ORI,
            "XORI" => XORI,
            "JMP" => JMP,
            "CALL" => CALL,
            "JZ" => JZ,
            "JNZ" => JNZ,
            "JC" => JC,
            "JNC" => JNC,
            "JN" => JN,
            "JNN" => JNN,
            _ => unreachable!(),
        };
        o.push(op);
        o.extend_from_slice(&v.to_le_bytes());
        return Ok(());
    }
    let parsef = |s: &str| -> Result<u8, AsmError> {
        let v = res(&pv(s), l, i.line)?;
        u8::try_from(v).map_err(|_| syntax(i.line, "direct file address exceeds 0xFF"))
    };
    if matches!(m, "INC" | "DEC" | "CMP") {
        let f = parsef(&i.args[0])?;
        o.extend_from_slice(&[
            if m == "INC" {
                INC_F
            } else if m == "DEC" {
                DEC_F
            } else {
                CMP_F
            },
            f,
        ]);
        return Ok(());
    }
    let (file, dest) = if i.args[0].eq_ignore_ascii_case("W") {
        if !matches!(m, "MOV8" | "MOV16") {
            return Err(syntax(
                i.line,
                "W may be the first operand only for MOV8/MOV16 stores",
            ));
        }
        (parsef(&i.args[1])?, "F")
    } else {
        (parsef(&i.args[0])?, i.args[1].as_str())
    };
    if !dest.eq_ignore_ascii_case("W") && !dest.eq_ignore_ascii_case("F") {
        return Err(syntax(i.line, "destination must be W or F"));
    }
    let dfile = dest.eq_ignore_ascii_case("F");
    if file < 16 {
        let base = match (m, dfile) {
            ("MOV8", false) => HOT_LD8,
            ("MOV8", true) => HOT_ST8,
            ("MOV16", false) => HOT_LD16,
            ("MOV16", true) => HOT_ST16,
            ("ADD", false) => HOT_ADDW,
            ("ADD", true) => HOT_ADDF,
            ("AND", false) => HOT_ANDW,
            ("AND", true) => HOT_ANDF,
            _ => 0,
        };
        if base != 0 {
            o.push(base | file);
            return Ok(());
        }
    }
    let op = match (m, dfile) {
        ("SHL", false) => SHL_FW,
        ("SHR", false) => SHR_FW,
        ("MUL", false) => MUL_FW,
        ("MULQ15", false) => MULQ15_FW,
        ("ADC", false) => ADC_FW,
        ("ADC", true) => ADC_FF,
        ("SBC", false) => SBC_FW,
        ("SBC", true) => SBC_FF,
        ("MULHU", false) => MULHU_FW,
        ("DIV", false) => DIV_FW,
        ("MOD", false) => MOD_FW,
        ("MOV8", false) => MOV8_FW,
        ("MOV8", true) => MOV8_WF,
        ("MOV16", false) => MOV16_FW,
        ("MOV16", true) => MOV16_WF,
        ("ADD", false) => ADD_FW,
        ("ADD", true) => ADD_FF,
        ("SUB", false) => SUB_FW,
        ("SUB", true) => SUB_FF,
        ("AND", false) => AND_FW,
        ("AND", true) => AND_FF,
        ("OR", false) => OR_FW,
        ("OR", true) => OR_FF,
        ("XOR", false) => XOR_FW,
        ("XOR", true) => XOR_FF,
        _ => return Err(syntax(i.line, "invalid direct file operation")),
    };
    o.extend_from_slice(&[op, file]);
    Ok(())
}
fn pv(s: &str) -> V {
    num(s)
        .map(V::N)
        .unwrap_or_else(|_| V::S(s.to_ascii_lowercase()))
}
fn num(s: &str) -> Result<u16, ()> {
    let t = s.trim();
    if let Some(h) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        u16::from_str_radix(h, 16).map_err(|_| ())
    } else {
        t.parse().map_err(|_| ())
    }
}
fn res(v: &V, l: &HashMap<String, u16>, line: usize) -> Result<u16, AsmError> {
    match v {
        V::N(n) => Ok(*n),
        V::S(s) => l
            .get(s)
            .copied()
            .ok_or_else(|| syntax(line, format!("unknown symbol {s}"))),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hot_file_is_one_byte() {
        assert_eq!(
            assemble("MOV16 3,W\nADD 3,F\nHALT").unwrap().payload.len(),
            3
        )
    }
    #[test]
    fn hot_and_uses_one_byte_and_xor_remains_general() {
        assert_eq!(assemble("AND 3,W").unwrap().payload, vec![op::HOT_ANDW | 3]);
        assert_eq!(assemble("AND 3,F").unwrap().payload, vec![op::HOT_ANDF | 3]);
        assert!(assemble("XOR 3,W").unwrap().payload.len() > 1);
    }

    #[test]
    fn copy_walkers_are_one_byte() {
        assert_eq!(
            assemble("LDB0+\nSTB1+\nLDB0-\nSTB1-\nHALT")
                .unwrap()
                .payload
                .len(),
            5
        )
    }
}

#[cfg(test)]
mod irq_encoding_tests {
    use super::*;
    #[test]
    fn irq_control_is_one_byte_each() {
        assert_eq!(
            assemble("EI\nDI\nIRET\n").unwrap().payload,
            vec![op::EI, op::DI, op::IRET]
        );
    }
}

#[cfg(test)]
mod operand_review_tests {
    use super::*;
    #[test]
    fn rejects_invalid_destination_name() {
        assert!(assemble("ADD 3,Q\n").is_err());
    }
    #[test]
    fn rejects_w_first_for_arithmetic() {
        assert!(assemble("ADD W,3\n").is_err());
    }
}

#[cfg(test)]
mod dsp_encoding_tests {
    use super::*;
    #[test]
    fn encodes_dsp_ops() {
        assert_eq!(
            assemble("ASR1W\nMULQ15 3,W\n").unwrap().payload,
            vec![op::ASR1W, op::MULQ15_FW, 3]
        );
    }
}

#[cfg(test)]
mod video_space_encoding_tests {
    use super::*;
    #[test]
    fn encodes_video_fsr_forms() {
        assert_eq!(
            assemble("VLDB0+\nVSTB1+\nVLDB0-\nVSTB1-\n")
                .unwrap()
                .payload,
            vec![
                op::VEXT,
                0x04,
                op::VEXT,
                0x12,
                op::VEXT,
                0x08,
                op::VEXT,
                0x16
            ]
        );
    }
}
