use super::{AsmError, Program};
use std::collections::HashMap;

fn err(line: usize, msg: impl Into<String>) -> AsmError {
    AsmError::Assembler(format!("line {line}: {}", msg.into()))
}
fn reg(s: &str, line: usize) -> Result<u8, AsmError> {
    let u = s.trim().to_ascii_uppercase();
    let n = u
        .strip_prefix('R')
        .ok_or_else(|| err(line, format!("expected register, got '{s}'")))?
        .parse::<u8>()
        .map_err(|_| err(line, "invalid register"))?;
    if n > 7 {
        return Err(err(line, "register must be R0..R7"));
    }
    Ok(n)
}
fn num(s: &str) -> Option<i32> {
    let t = s.trim().replace('_', "");
    if let Some(h) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        i32::from_str_radix(h, 16).ok()
    } else if let Some(h) = t.strip_prefix("-0x").or_else(|| t.strip_prefix("-0X")) {
        i32::from_str_radix(h, 16).ok().map(|v| -v)
    } else {
        t.parse::<i32>().ok()
    }
}
fn val(
    s: &str,
    labels: &HashMap<String, u16>,
    line: usize,
    resolve: bool,
) -> Result<u16, AsmError> {
    if let Some(v) = num(s) {
        return Ok(v as u16);
    }
    if let Some(v) = labels.get(&s.trim().to_ascii_lowercase()) {
        return Ok(*v);
    }
    if resolve {
        Err(err(line, format!("unknown symbol '{s}'")))
    } else {
        Ok(0)
    }
}
fn operands(s: &str) -> Vec<String> {
    s.split(',')
        .map(|x| x.trim().to_string())
        .filter(|x| !x.is_empty())
        .collect()
}
fn mem(s: &str, line: usize) -> Result<(u8, i8), AsmError> {
    let t = s.trim();
    if !(t.starts_with('[') && t.ends_with(']')) {
        return Err(err(line, format!("expected memory operand, got '{s}'")));
    }
    let inner = t[1..t.len() - 1].trim();
    let mut split = None;
    for (i, c) in inner.char_indices().skip(1) {
        if c == '+' || c == '-' {
            split = Some((i, c));
            break;
        }
    }
    match split {
        None => Ok((reg(inner, line)?, 0)),
        Some((i, c)) => {
            let r = reg(inner[..i].trim(), line)?;
            let mut o =
                num(inner[i + 1..].trim()).ok_or_else(|| err(line, "offset must be numeric"))?;
            if c == '-' {
                o = -o;
            }
            if !(-32..=31).contains(&o) {
                return Err(err(line, "load/store offset must fit -32..31"));
            }
            Ok((r, o as i8))
        }
    }
}
fn emit_word(out: &mut Vec<u8>, w: u16) {
    out.extend_from_slice(&w.to_le_bytes());
}
fn alu3(fn3: u16, d: u8, a: u8, b: u8) -> u16 {
    0x1000 | (fn3 << 9) | ((d as u16) << 6) | ((a as u16) << 3) | b as u16
}
fn unary(fn3: u16, d: u8, a: u8) -> u16 {
    0x2000 | (fn3 << 9) | ((d as u16) << 6) | ((a as u16) << 3)
}
fn simm6(v: i32) -> u16 {
    (v as i16 as u16) & 0x3f
}
fn encode(
    text: &str,
    line: usize,
    labels: &HashMap<String, u16>,
    resolve: bool,
) -> Result<Vec<u8>, AsmError> {
    let mut p = text.trim().splitn(2, char::is_whitespace);
    let m = p.next().unwrap_or("").to_ascii_uppercase();
    let ops = operands(p.next().unwrap_or(""));
    let mut o = Vec::new();
    macro_rules! need {
        ($n:expr) => {
            if ops.len() != $n {
                return Err(err(line, format!("{m} expects {} operands", $n)));
            }
        };
    }
    match m.as_str() {
        "NOP" => emit_word(&mut o, 0x0000),
        "HALT" => emit_word(&mut o, 0x0001),
        "RET" => emit_word(&mut o, 0x0002),
        "EI" => emit_word(&mut o, 0x0003),
        "DI" => emit_word(&mut o, 0x0004),
        "IRET" => emit_word(&mut o, 0x0005),
        "MOV" => {
            need!(2);
            let d = reg(&ops[0], line)?;
            let a = reg(&ops[1], line)?;
            emit_word(&mut o, unary(0, d, a));
        }
        "CMP" => {
            need!(2);
            let a = reg(&ops[0], line)?;
            let b = reg(&ops[1], line)?;
            emit_word(&mut o, unary(1, b, a));
        }
        "NOT" => {
            need!(1);
            let d = reg(&ops[0], line)?;
            emit_word(&mut o, unary(2, d, d));
        }
        "NEG" => {
            need!(1);
            let d = reg(&ops[0], line)?;
            emit_word(&mut o, unary(3, d, d));
        }
        "ASR1" => {
            need!(1);
            let d = reg(&ops[0], line)?;
            emit_word(&mut o, unary(4, d, d));
        }
        "SHL1" => {
            need!(1);
            let d = reg(&ops[0], line)?;
            emit_word(&mut o, 0x3000 | ((6u16) << 9) | ((d as u16) << 6) | 1);
        }
        "SHR1" => {
            need!(1);
            let d = reg(&ops[0], line)?;
            emit_word(&mut o, 0x3000 | ((7u16) << 9) | ((d as u16) << 6) | 1);
        }
        "INC" => {
            need!(1);
            let d = reg(&ops[0], line)?;
            emit_word(&mut o, 0x3000 | ((d as u16) << 6) | 1);
        }
        "DEC" => {
            need!(1);
            let d = reg(&ops[0], line)?;
            emit_word(&mut o, 0x3000 | ((d as u16) << 6) | simm6(-1));
        }
        "ADD" | "SUB" | "AND" | "OR" | "XOR" | "MUL" | "SHL" | "SHR" => {
            if ops.len() == 2 {
                let d = reg(&ops[0], line)?;
                let b = reg(&ops[1], line)?;
                let f = match m.as_str() {
                    "ADD" => 0,
                    "SUB" => 1,
                    "AND" => 2,
                    "OR" => 3,
                    "XOR" => 4,
                    "MUL" => 5,
                    "SHL" => 6,
                    _ => 7,
                };
                emit_word(&mut o, alu3(f, d, d, b));
            } else if ops.len() == 3 {
                let d = reg(&ops[0], line)?;
                let a = reg(&ops[1], line)?;
                let b = reg(&ops[2], line)?;
                let f = match m.as_str() {
                    "ADD" => 0,
                    "SUB" => 1,
                    "AND" => 2,
                    "OR" => 3,
                    "XOR" => 4,
                    "MUL" => 5,
                    "SHL" => 6,
                    _ => 7,
                };
                emit_word(&mut o, alu3(f, d, a, b));
            } else {
                return Err(err(line, "ALU op expects 2 or 3 operands"));
            }
        }
        "DIV" | "DIVU" | "MOD" | "MODU" | "MULQ15" | "ADC" | "SBC" | "MULHU" => {
            if ops.len() != 2 && ops.len() != 3 {
                return Err(err(line, "arithmetic extension expects 2 or 3 operands"));
            }
            let d = reg(&ops[0], line)?;
            let (a, b) = if ops.len() == 2 {
                (d, reg(&ops[1], line)?)
            } else {
                (reg(&ops[1], line)?, reg(&ops[2], line)?)
            };
            let f = if m.starts_with("DIV") {
                0
            } else if m.starts_with("MOD") {
                1
            } else if m == "MULQ15" {
                2
            } else if m == "ADC" {
                3
            } else if m == "SBC" {
                4
            } else {
                5
            };
            emit_word(
                &mut o,
                0xC000 | ((f as u16) << 9) | ((d as u16) << 6) | ((a as u16) << 3) | b as u16,
            );
        }
        "RCR1" => {
            need!(1);
            let d = reg(&ops[0], line)?;
            emit_word(
                &mut o,
                0xC000 | ((6u16) << 9) | ((d as u16) << 6) | ((d as u16) << 3),
            );
        }
        "MOVI" | "LDI" => {
            need!(2);
            let d = reg(&ops[0], line)?;
            let v = val(&ops[1], labels, line, resolve)?;
            emit_word(&mut o, 0x9000 | ((d as u16) << 6));
            emit_word(&mut o, v);
        }
        "ADDI" | "SUBI" | "CMPI" => {
            need!(2);
            let d = reg(&ops[0], line)?;
            if m == "SUBI" {
                let v = val(&ops[1], labels, line, resolve)?;
                emit_word(&mut o, 0x9000 | ((3u16) << 9) | ((d as u16) << 6));
                emit_word(&mut o, v);
            } else {
                let numeric = num(&ops[1]);
                let raw = if let Some(v) = numeric {
                    v
                } else {
                    val(&ops[1], labels, line, resolve)? as i32
                };
                let f = if m == "CMPI" { 2 } else { 0 };
                if numeric.is_some() && (-32..=31).contains(&raw) {
                    emit_word(
                        &mut o,
                        0x3000 | ((f as u16) << 9) | ((d as u16) << 6) | simm6(raw),
                    );
                } else {
                    let lf = if m == "CMPI" { 2 } else { 1 };
                    emit_word(&mut o, 0x9000 | ((lf as u16) << 9) | ((d as u16) << 6));
                    emit_word(&mut o, raw as u16);
                }
            }
        }
        "ANDI" | "ORI" | "XORI" => {
            need!(2);
            let d = reg(&ops[0], line)?;
            let v = val(&ops[1], labels, line, resolve)?;
            if v > 63 {
                return Err(err(line, "logical immediate must fit 0..63"));
            }
            let f = if m == "ANDI" {
                3
            } else if m == "ORI" {
                4
            } else {
                5
            };
            emit_word(&mut o, 0x3000 | ((f as u16) << 9) | ((d as u16) << 6) | v);
        }
        "LOAD8" | "LOAD16" => {
            need!(2);
            let d = reg(&ops[0], line)?;
            let (a, off) = mem(&ops[1], line)?;
            let major = if m == "LOAD8" { 4 } else { 5 };
            emit_word(
                &mut o,
                (major << 12)
                    | ((d as u16) << 9)
                    | ((a as u16) << 6)
                    | ((off as i16 as u16) & 0x3f),
            );
        }
        "STORE8" | "STORE16" => {
            need!(2);
            let (a, off) = mem(&ops[0], line)?;
            let s = reg(&ops[1], line)?;
            let major = if m == "STORE8" { 6 } else { 7 };
            emit_word(
                &mut o,
                (major << 12)
                    | ((s as u16) << 9)
                    | ((a as u16) << 6)
                    | ((off as i16 as u16) & 0x3f),
            );
        }
        "VLOAD8" | "VLOAD16" | "VLD8" | "VLD16" => {
            need!(2);
            let d = reg(&ops[0], line)?;
            let (a, off) = mem(&ops[1], line)?;
            if !(-8..=7).contains(&(off as i32)) {
                return Err(err(line, "video offset must fit -8..7"));
            }
            let f = if m.ends_with("16") { 1 } else { 0 };
            emit_word(
                &mut o,
                0xB000
                    | ((f as u16) << 10)
                    | ((d as u16) << 7)
                    | ((a as u16) << 4)
                    | ((off as i16 as u16) & 0xf),
            );
        }
        "VSTORE8" | "VSTORE16" | "VST8" | "VST16" => {
            need!(2);
            let (a, off) = mem(&ops[0], line)?;
            let s = reg(&ops[1], line)?;
            if !(-8..=7).contains(&(off as i32)) {
                return Err(err(line, "video offset must fit -8..7"));
            }
            let f = if m.ends_with("16") { 3 } else { 2 };
            emit_word(
                &mut o,
                0xB000
                    | ((f as u16) << 10)
                    | ((s as u16) << 7)
                    | ((a as u16) << 4)
                    | ((off as i16 as u16) & 0xf),
            );
        }
        "BRA" | "BZ" | "BNZ" | "BC" | "BNC" | "BN" | "BNN" => {
            need!(1);
            let r = num(&ops[0]).ok_or_else(|| {
                err(
                    line,
                    "relative branch operand is a signed instruction-word displacement",
                )
            })?;
            if !(-512..=511).contains(&r) {
                return Err(err(
                    line,
                    "relative branch displacement must fit -512..511 words",
                ));
            }
            let c = match m.as_str() {
                "BRA" => 0,
                "BZ" => 1,
                "BNZ" => 2,
                "BC" => 3,
                "BNC" => 4,
                "BN" => 5,
                _ => 6,
            };
            emit_word(
                &mut o,
                0x8000 | ((c as u16) << 9) | ((r as i16 as u16) & 0x01ff),
            );
        }
        "JMP" | "JZ" | "JNZ" | "JC" | "JNC" | "JN" | "JNN" | "CALL" => {
            need!(1);
            let f = match m.as_str() {
                "JMP" => 0,
                "CALL" => 1,
                "JZ" => 2,
                "JNZ" => 3,
                "JC" => 4,
                "JNC" => 5,
                "JN" => 6,
                _ => 7,
            };
            emit_word(&mut o, 0xA000 | ((f as u16) << 9));
            emit_word(&mut o, val(&ops[0], labels, line, resolve)?);
        }
        "PUSH" => {
            need!(1);
            let r = reg(&ops[0], line)?;
            emit_word(&mut o, 0x3000 | ((6u16) << 6) | simm6(-2));
            emit_word(&mut o, 0x7000 | ((r as u16) << 9) | ((6u16) << 6));
        }
        "POP" => {
            need!(1);
            let r = reg(&ops[0], line)?;
            emit_word(&mut o, 0x5000 | ((r as u16) << 9) | ((6u16) << 6));
            emit_word(&mut o, 0x3000 | ((6u16) << 6) | 2);
        }
        "ZLOAD8" | "ZLOAD16" => {
            need!(1);
            emit_word(&mut o, 0x9000 | ((7u16) << 6));
            emit_word(&mut o, val(&ops[0], labels, line, resolve)?);
            let major = if m == "ZLOAD8" { 4 } else { 5 };
            emit_word(&mut o, (major << 12) | ((0u16) << 9) | ((7u16) << 6));
        }
        "ZSTORE8" | "ZSTORE16" => {
            need!(1);
            emit_word(&mut o, 0x9000 | ((7u16) << 6));
            emit_word(&mut o, val(&ops[0], labels, line, resolve)?);
            let major = if m == "ZSTORE8" { 6 } else { 7 };
            emit_word(&mut o, (major << 12) | ((0u16) << 9) | ((7u16) << 6));
        }
        _ => return Err(err(line, format!("unknown instruction '{m}'"))),
    }
    Ok(o)
}

pub fn assemble(source: &str) -> Result<Program, AsmError> {
    let mut load = 0x0100u16;
    let mut entry_expr: Option<String> = None;
    let mut labels = HashMap::new();
    let mut pc = load as usize;
    for (i, raw) in source.lines().enumerate() {
        let line = i + 1;
        let mut t = raw.split(';').next().unwrap_or("").trim();
        if t.is_empty() {
            continue;
        }
        if let Some(pos) = t.find(':') {
            let name = t[..pos].trim().to_ascii_lowercase();
            labels.insert(name, pc as u16);
            t = t[pos + 1..].trim();
            if t.is_empty() {
                continue;
            }
        }
        if t.starts_with('.') {
            let mut q = t.split_whitespace();
            let d = q.next().unwrap();
            let a = q.next().ok_or_else(|| err(line, "directive needs value"))?;
            match d.to_ascii_lowercase().as_str() {
                ".load" => {
                    load = val(a, &labels, line, false)?;
                    pc = load as usize;
                }
                ".entry" => entry_expr = Some(a.to_string()),
                _ => return Err(err(line, "unknown directive")),
            }
            continue;
        }
        pc += encode(t, line, &labels, false)?.len();
        if pc > 65536 {
            return Err(err(line, "program too large"));
        }
    }
    let entry = match entry_expr {
        Some(ref e) => val(e, &labels, 0, true)?,
        None => load,
    };
    let mut payload = Vec::new();
    for (i, raw) in source.lines().enumerate() {
        let line = i + 1;
        let mut t = raw.split(';').next().unwrap_or("").trim();
        if t.is_empty() {
            continue;
        }
        if let Some(pos) = t.find(':') {
            t = t[pos + 1..].trim();
            if t.is_empty() {
                continue;
            }
        }
        if t.starts_with('.') {
            continue;
        }
        payload.extend_from_slice(&encode(t, line, &labels, true)?);
    }
    Ok(Program {
        load_address: load,
        entry_address: entry,
        payload,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subi_uses_long_native_subtract_form() {
        let p = assemble("SUBI R2, 1\nHALT").unwrap();
        // major 9, fn 3, Rd=R2, followed by imm16=1.
        assert_eq!(&p.payload[..4], &[0x80, 0x96, 0x01, 0x00]);
    }
}
