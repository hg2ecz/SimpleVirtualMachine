use super::{
    error::{AsmError, syntax},
    instruction::op,
    program::Program,
};
use std::collections::HashMap;

#[derive(Clone, Debug)]
enum Value {
    Num(u16),
    Sym(String),
}
#[derive(Clone, Debug)]
struct Inst {
    mnemonic: String,
    operand: Option<Value>,
    line: usize,
    short: bool,
}
#[derive(Clone, Debug)]
enum Stmt {
    Inst(Inst),
    Load(Value, usize),
    Entry(Value, usize),
}

pub fn assemble(source: &str) -> Result<Program, AsmError> {
    let mut stmts = Vec::new();
    let mut label_pos: HashMap<String, usize> = HashMap::new();

    for (i, raw) in source.lines().enumerate() {
        let line = i + 1;
        let text = raw.split_once(';').map_or(raw, |x| x.0).trim();
        if text.is_empty() {
            continue;
        }
        let mut rest = text;
        if let Some(pos) = rest.find(':') {
            let (lab, rem) = rest.split_at(pos);
            let lab = lab.trim();
            if lab.is_empty() {
                return Err(syntax(line, "empty label"));
            }
            if label_pos
                .insert(lab.to_ascii_lowercase(), stmts.len())
                .is_some()
            {
                return Err(syntax(line, format!("duplicate label '{lab}'")));
            }
            rest = rem[1..].trim();
            if rest.is_empty() {
                continue;
            }
        }
        if rest.starts_with('.') {
            let mut p = rest.split_whitespace();
            let d = p.next().unwrap().to_ascii_lowercase();
            let v = p
                .next()
                .ok_or_else(|| syntax(line, "directive requires a value"))?;
            let value = parse_value(v);
            match d.as_str() {
                ".load" => stmts.push(Stmt::Load(value, line)),
                ".entry" => stmts.push(Stmt::Entry(value, line)),
                _ => return Err(syntax(line, format!("unknown directive {d}"))),
            }
            continue;
        }
        let mut p = rest.split_whitespace();
        let raw_mnemonic = p.next().unwrap().to_ascii_uppercase();
        let tail = rest[raw_mnemonic.len()..].trim();
        let mnemonic = normalize_memory_mnemonic(&raw_mnemonic, tail);
        let normalized_tail = if mnemonic != raw_mnemonic { "" } else { tail };
        let operand = parse_operand(&mnemonic, normalized_tail, line)?;
        // Validate before layout.
        let _ = encoded_len(&mnemonic, operand.is_some(), false, line)?;
        stmts.push(Stmt::Inst(Inst {
            mnemonic,
            operand,
            line,
            short: false,
        }));
    }

    let load = stmts
        .iter()
        .find_map(|s| {
            if let Stmt::Load(v, l) = s {
                Some(resolve(v, &HashMap::new(), *l))
            } else {
                None
            }
        })
        .transpose()?
        .unwrap_or(0);

    // Monotonic relaxation: start with long branches/calls and repeatedly shrink any
    // target that fits signed 8-bit PC-relative displacement. Shrinking never makes
    // an already-short instruction longer, so convergence is fast and deterministic.
    loop {
        let offsets = stmt_offsets(&stmts)?;
        let labels = absolute_labels(load, &label_pos, &offsets, &stmts)?;
        let mut changed = false;
        for (idx, stmt) in stmts.iter_mut().enumerate() {
            let Stmt::Inst(inst) = stmt else { continue };
            if inst.short || !is_relaxable(&inst.mnemonic) {
                continue;
            }
            let Some(v) = inst.operand.as_ref() else {
                continue;
            };
            let target = resolve(v, &labels, inst.line)?;
            let here = (load as usize)
                .checked_add(offsets[idx])
                .ok_or_else(|| syntax(inst.line, "address overflow"))?;
            let next_pc = here + 2;
            let disp = i32::from(target) - next_pc as i32;
            if (-128..=127).contains(&disp) {
                inst.short = true;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    let offsets = stmt_offsets(&stmts)?;
    let absolute = absolute_labels(load, &label_pos, &offsets, &stmts)?;
    let entry = stmts
        .iter()
        .find_map(|s| {
            if let Stmt::Entry(v, l) = s {
                Some(resolve(v, &absolute, *l))
            } else {
                None
            }
        })
        .transpose()?
        .unwrap_or(load);
    let mut payload = Vec::new();
    for (idx, s) in stmts.iter().enumerate() {
        if let Stmt::Inst(i) = s {
            encode(i, &absolute, load, offsets[idx], &mut payload)?;
        }
    }
    if load as usize + payload.len() > 65_536 {
        return Err(syntax(0, "program does not fit in memory"));
    }
    Ok(Program {
        load_address: load,
        entry_address: entry,
        payload,
    })
}

fn stmt_offsets(stmts: &[Stmt]) -> Result<Vec<usize>, AsmError> {
    let mut out = Vec::with_capacity(stmts.len() + 1);
    let mut off = 0usize;
    for s in stmts {
        out.push(off);
        if let Stmt::Inst(i) = s {
            off += encoded_len(&i.mnemonic, i.operand.is_some(), i.short, i.line)?;
        }
    }
    out.push(off);
    Ok(out)
}

fn absolute_labels(
    load: u16,
    defs: &HashMap<String, usize>,
    offsets: &[usize],
    stmts: &[Stmt],
) -> Result<HashMap<String, u16>, AsmError> {
    defs.iter()
        .map(|(name, &pos)| {
            let off = if pos < stmts.len() {
                offsets[pos]
            } else {
                *offsets.last().unwrap_or(&0)
            };
            let a = load as usize + off;
            if a > 0xffff {
                Err(syntax(0, "label outside 16-bit address space"))
            } else {
                Ok((name.clone(), a as u16))
            }
        })
        .collect()
}

fn normalize_memory_mnemonic(m: &str, tail: &str) -> String {
    let t = tail.trim().to_ascii_uppercase();
    match (m, t.as_str()) {
        ("LDA8", "[X]") => "LDA8X".into(),
        ("LDA16", "[X]") => "LDA16X".into(),
        ("STA8", "[X]") => "STA8X".into(),
        ("STA16", "[X]") => "STA16X".into(),
        ("LDA8", "[X+]") => "LDA8XP".into(),
        ("LDA16", "[X+]") => "LDA16XP".into(),
        ("STA8", "[X+]") => "STA8XP".into(),
        ("STA16", "[X+]") => "STA16XP".into(),
        ("STA8", "[Y]") => "STA8Y".into(),
        ("STA16", "[Y]") => "STA16Y".into(),
        ("STA8", "[Y+]") => "STA8YP".into(),
        ("STA16", "[Y+]") => "STA16YP".into(),
        ("LDA8", "[-X]") => "LDA8XM".into(),
        ("LDA16", "[-X]") => "LDA16XM".into(),
        ("STA8", "[-Y]") => "STA8YM".into(),
        ("STA16", "[-Y]") => "STA16YM".into(),
        ("VLDA8", "[X]") => "VLDA8X".into(),
        ("VLDA16", "[X]") => "VLDA16X".into(),
        ("VSTA8", "[X]") => "VSTA8X".into(),
        ("VSTA16", "[X]") => "VSTA16X".into(),
        ("VLDA8", "[X+]") => "VLDA8XP".into(),
        ("VLDA16", "[X+]") => "VLDA16XP".into(),
        ("VSTA8", "[X+]") => "VSTA8XP".into(),
        ("VSTA16", "[X+]") => "VSTA16XP".into(),
        ("VSTA8", "[Y]") => "VSTA8Y".into(),
        ("VSTA16", "[Y]") => "VSTA16Y".into(),
        ("VSTA8", "[Y+]") => "VSTA8YP".into(),
        ("VSTA16", "[Y+]") => "VSTA16YP".into(),
        ("VLDA8", "[-X]") => "VLDA8XM".into(),
        ("VLDA16", "[-X]") => "VLDA16XM".into(),
        ("VSTA8", "[-Y]") => "VSTA8YM".into(),
        ("VSTA16", "[-Y]") => "VSTA16YM".into(),
        _ => m.to_string(),
    }
}

fn parse_operand(m: &str, tail: &str, line: usize) -> Result<Option<Value>, AsmError> {
    let noarg = [
        "NOP", "HALT", "RET", "EI", "DI", "IRET", "ASR1", "MULQ15X", "ADCX", "SBCX", "MULHUX",
        "RCR1", "TAX", "TXA", "PUSHA", "POPA", "PUSHX", "POPX", "INC", "DEC", "NEG", "NOT", "SHL1",
        "SHR1", "INX", "DEX", "TAY", "TYA", "INY", "DEY", "ADDX", "SUBX", "MULX", "DIVX", "MODX",
        "ANDX", "ORX", "XORX", "SHLX", "SHRX", "CMPX",
    ];
    let mem_noarg = [
        "LDA8X", "LDA16X", "STA8X", "STA16X", "LDA8XP", "LDA16XP", "STA8XP", "STA16XP", "STA8Y",
        "STA16Y", "STA8YP", "STA16YP", "LDA8XM", "LDA16XM", "STA8YM", "STA16YM", "VLDA8X",
        "VLDA16X", "VSTA8X", "VSTA16X", "VLDA8XP", "VLDA16XP", "VSTA8XP", "VSTA16XP", "VSTA8Y",
        "VSTA16Y", "VSTA8YP", "VSTA16YP", "VLDA8XM", "VLDA16XM", "VSTA8YM", "VSTA16YM",
    ];
    if noarg.contains(&m) || mem_noarg.contains(&m) {
        if !tail.is_empty() {
            return Err(syntax(line, format!("{m} takes no operand")));
        }
        return Ok(None);
    }
    if tail.is_empty() {
        return Err(syntax(line, format!("{m} requires an operand")));
    }
    if tail.split(',').count() != 1 {
        return Err(syntax(
            line,
            "accumulator ISA instructions take at most one explicit operand",
        ));
    }
    Ok(Some(parse_value(tail)))
}

fn is_relaxable(m: &str) -> bool {
    matches!(
        m,
        "JMP" | "CALL" | "JZ" | "JNZ" | "JC" | "JNC" | "JN" | "JNN"
    )
}

fn encoded_len(m: &str, has: bool, short: bool, line: usize) -> Result<usize, AsmError> {
    let one = [
        "NOP", "HALT", "RET", "EI", "DI", "IRET", "ASR1", "MULQ15X", "ADCX", "SBCX", "MULHUX",
        "RCR1", "TAX", "TXA", "PUSHA", "POPA", "PUSHX", "POPX", "INC", "DEC", "NEG", "NOT", "SHL1",
        "SHR1", "INX", "DEX", "TAY", "TYA", "INY", "DEY", "ADDX", "SUBX", "MULX", "DIVX", "MODX",
        "ANDX", "ORX", "XORX", "SHLX", "SHRX", "CMPX", "LDA8X", "LDA16X", "STA8X", "STA16X",
        "LDA8XP", "LDA16XP", "STA8XP", "STA16XP", "STA8Y", "STA16Y", "STA8YP", "STA16YP", "LDA8XM",
        "LDA16XM", "STA8YM", "STA16YM",
    ];
    let two = [
        "LDA8Z", "LDA16Z", "STA8Z", "STA16Z", "VLDA8X", "VLDA16X", "VSTA8X", "VSTA16X", "VLDA8XP",
        "VLDA16XP", "VSTA8XP", "VSTA16XP", "VSTA8Y", "VSTA16Y", "VSTA8YP", "VSTA16YP", "VLDA8XM",
        "VLDA16XM", "VSTA8YM", "VSTA16YM",
    ];
    let three = [
        "LDAI", "LDXI", "LDYI", "ADDI", "SUBI", "CMPI", "ANDI", "ORI", "XORI", "LDA8", "LDA16",
        "STA8", "STA16", "JMP", "CALL", "JZ", "JNZ", "JC", "JNC", "JN", "JNN",
    ];
    if one.contains(&m) {
        return Ok(1);
    }
    if two.contains(&m) {
        return Ok(2);
    }
    if three.contains(&m) {
        return Ok(if short && is_relaxable(m) {
            2
        } else if has {
            3
        } else {
            1
        });
    }
    Err(syntax(line, format!("unknown instruction '{m}'")))
}

fn encode(
    i: &Inst,
    labels: &HashMap<String, u16>,
    load: u16,
    offset: usize,
    out: &mut Vec<u8>,
) -> Result<(), AsmError> {
    use op::*;
    let m = i.mnemonic.as_str();
    let video_sub = match m {
        "VLDA8X" => Some(0x00),
        "VLDA16X" => Some(0x01),
        "VSTA8X" => Some(0x02),
        "VSTA16X" => Some(0x03),
        "VLDA8XP" => Some(0x04),
        "VLDA16XP" => Some(0x05),
        "VSTA8XP" => Some(0x06),
        "VSTA16XP" => Some(0x07),
        "VSTA8Y" => Some(0x08),
        "VSTA16Y" => Some(0x09),
        "VSTA8YP" => Some(0x0A),
        "VSTA16YP" => Some(0x0B),
        "VLDA8XM" => Some(0x0C),
        "VLDA16XM" => Some(0x0D),
        "VSTA8YM" => Some(0x0E),
        "VSTA16YM" => Some(0x0F),
        _ => None,
    };
    if let Some(sub) = video_sub {
        out.extend_from_slice(&[VEXT, sub]);
        return Ok(());
    }
    let fixed = match m {
        "NOP" => Some(NOP),
        "HALT" => Some(HALT),
        "RET" => Some(RET),
        "EI" => Some(EI),
        "DI" => Some(DI),
        "IRET" => Some(IRET),
        "ASR1" => Some(ASR1),
        "MULQ15X" => Some(MULQ15X),
        "ADCX" => Some(ADCX),
        "SBCX" => Some(SBCX),
        "MULHUX" => Some(MULHUX),
        "RCR1" => Some(RCR1),
        "TAX" => Some(TAX),
        "TXA" => Some(TXA),
        "PUSHA" => Some(PUSHA),
        "POPA" => Some(POPA),
        "PUSHX" => Some(PUSHX),
        "POPX" => Some(POPX),
        "INC" => Some(INC),
        "DEC" => Some(DEC),
        "NEG" => Some(NEG),
        "NOT" => Some(NOT),
        "SHL1" => Some(SHL1),
        "SHR1" => Some(SHR1),
        "INX" => Some(INX),
        "DEX" => Some(DEX),
        "TAY" => Some(TAY),
        "TYA" => Some(TYA),
        "INY" => Some(INY),
        "DEY" => Some(DEY),
        "ADDX" => Some(ADDX),
        "SUBX" => Some(SUBX),
        "MULX" => Some(MULX),
        "DIVX" => Some(DIVX),
        "MODX" => Some(MODX),
        "ANDX" => Some(ANDX),
        "ORX" => Some(ORX),
        "XORX" => Some(XORX),
        "SHLX" => Some(SHLX),
        "SHRX" => Some(SHRX),
        "CMPX" => Some(CMPX),
        "LDA8X" => Some(LDA8X),
        "LDA16X" => Some(LDA16X),
        "STA8X" => Some(STA8X),
        "STA16X" => Some(STA16X),
        "LDA8XP" => Some(LDA8XP),
        "LDA16XP" => Some(LDA16XP),
        "STA8XP" => Some(STA8XP),
        "STA16XP" => Some(STA16XP),
        "STA8Y" => Some(STA8Y),
        "STA16Y" => Some(STA16Y),
        "STA8YP" => Some(STA8YP),
        "STA16YP" => Some(STA16YP),
        "LDA8XM" => Some(LDA8XM),
        "LDA16XM" => Some(LDA16XM),
        "STA8YM" => Some(STA8YM),
        "STA16YM" => Some(STA16YM),
        _ => None,
    };
    if let Some(opcode) = fixed {
        out.push(opcode);
        return Ok(());
    }
    let v = resolve(
        i.operand
            .as_ref()
            .ok_or_else(|| syntax(i.line, "missing operand"))?,
        labels,
        i.line,
    )?;
    if matches!(m, "LDA8Z" | "LDA16Z" | "STA8Z" | "STA16Z") {
        if v > 0x00FF {
            return Err(syntax(i.line, "zero-page address exceeds 0xFF"));
        }
        let opcode = match m {
            "LDA8Z" => LDA8Z,
            "LDA16Z" => LDA16Z,
            "STA8Z" => STA8Z,
            "STA16Z" => STA16Z,
            _ => unreachable!(),
        };
        out.push(opcode);
        out.push(v as u8);
        return Ok(());
    }
    if i.short && is_relaxable(m) {
        let opcode = match m {
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
        let next_pc = load as usize + offset + 2;
        let disp = i32::from(v) - next_pc as i32;
        if !(-128..=127).contains(&disp) {
            return Err(syntax(i.line, "internal short-branch range error"));
        }
        out.push(opcode);
        out.push((disp as i8) as u8);
        return Ok(());
    }
    let opcode = match m {
        "LDAI" => LDAI,
        "LDXI" => LDXI,
        "LDYI" => LDYI,
        "ADDI" => ADDI,
        "SUBI" => SUBI,
        "CMPI" => CMPI,
        "ANDI" => ANDI,
        "ORI" => ORI,
        "XORI" => XORI,
        "LDA8" => LDA8A,
        "LDA16" => LDA16A,
        "STA8" => STA8A,
        "STA16" => STA16A,
        "JMP" => JMP,
        "CALL" => CALL,
        "JZ" => JZ,
        "JNZ" => JNZ,
        "JC" => JC,
        "JNC" => JNC,
        "JN" => JN,
        "JNN" => JNN,
        _ => return Err(syntax(i.line, format!("unknown instruction {m}"))),
    };
    out.push(opcode);
    out.extend_from_slice(&v.to_le_bytes());
    Ok(())
}

fn parse_value(s: &str) -> Value {
    let t = s.trim();
    let n = if let Some(h) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        u16::from_str_radix(h, 16).ok()
    } else {
        t.parse::<u16>().ok()
    };
    n.map(Value::Num)
        .unwrap_or_else(|| Value::Sym(t.to_ascii_lowercase()))
}
fn resolve(v: &Value, labels: &HashMap<String, u16>, line: usize) -> Result<u16, AsmError> {
    match v {
        Value::Num(n) => Ok(*n),
        Value::Sym(s) => labels
            .get(s)
            .copied()
            .ok_or_else(|| syntax(line, format!("unknown symbol '{s}'"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accumulator::instruction::op;

    #[test]
    fn relaxes_local_conditional_branch() {
        let p = assemble(
            ".load 0\n.entry start\nstart:\nLDAI 0\nCMPI 0\nJZ yes\nLDAI 99\nyes:\nINC\nHALT\n",
        )
        .unwrap();
        assert_eq!(p.payload[6], op::RJZ);
        assert_eq!(p.payload[7] as i8, 3);
    }

    #[test]
    fn keeps_distant_branch_absolute() {
        let mut src = String::from(".load 0\nJMP far\n");
        for _ in 0..130 {
            src.push_str("NOP\n");
        }
        src.push_str("far:\nHALT\n");
        let p = assemble(&src).unwrap();
        assert_eq!(p.payload[0], op::JMP);
    }

    #[test]
    fn relaxes_local_call() {
        let p = assemble(".load 0\nCALL f\nHALT\nf:\nRET\n").unwrap();
        assert_eq!(p.payload[0], op::RCALL);
    }
    #[test]
    fn zero_page_forms_are_two_bytes() {
        assert_eq!(
            assemble("LDA16Z 0x20\nSTA8Z 0x21\n").unwrap().payload,
            vec![op::LDA16Z, 0x20, op::STA8Z, 0x21]
        );
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
mod dsp_encoding_tests {
    use super::*;
    #[test]
    fn encodes_dsp_ops() {
        assert_eq!(
            assemble("ASR1\nMULQ15X\n").unwrap().payload,
            vec![op::ASR1, op::MULQ15X]
        );
    }
}

#[cfg(test)]
mod video_space_encoding_tests {
    use super::*;
    #[test]
    fn encodes_video_prefix_forms() {
        assert_eq!(
            assemble("VLDA8 [X+]\nVSTA8 [Y+]\n").unwrap().payload,
            vec![op::VEXT, 0x04, op::VEXT, 0x0A]
        );
    }
}
