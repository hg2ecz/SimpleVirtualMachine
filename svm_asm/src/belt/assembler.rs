use super::{AsmError, Program};
use std::collections::HashMap;

fn err(line: usize, msg: impl Into<String>) -> AsmError {
    AsmError::Assembler(format!("line {line}: {}", msg.into()))
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
fn val(s: &str, l: &HashMap<String, u16>, line: usize, resolve: bool) -> Result<u16, AsmError> {
    if let Some(v) = num(s) {
        return Ok(v as u16);
    }
    if let Some(v) = l.get(&s.trim().to_ascii_lowercase()) {
        return Ok(*v);
    }
    if resolve {
        Err(err(line, format!("unknown symbol '{s}'")))
    } else {
        Ok(0)
    }
}
fn belt(s: &str, line: usize) -> Result<u8, AsmError> {
    let u = s.trim().to_ascii_lowercase();
    let n = u
        .strip_prefix('b')
        .ok_or_else(|| err(line, format!("expected belt operand b0..b7, got '{s}'")))?
        .parse::<u8>()
        .map_err(|_| err(line, "invalid belt operand"))?;
    if n > 7 {
        return Err(err(line, "belt operand must be b0..b7"));
    }
    Ok(n)
}
fn ops(s: &str) -> Vec<String> {
    s.split(',')
        .map(|x| x.trim().to_string())
        .filter(|x| !x.is_empty())
        .collect()
}
fn mem_belt(s: &str, line: usize) -> Result<u8, AsmError> {
    let t = s.trim();
    if !(t.starts_with('[') && t.ends_with(']')) {
        return Err(err(line, "expected [bN]"));
    }
    belt(t[1..t.len() - 1].trim(), line)
}
fn word(o: &mut Vec<u8>, w: u16) {
    o.extend_from_slice(&w.to_le_bytes())
}
fn encode(
    text: &str,
    line: usize,
    labels: &HashMap<String, u16>,
    resolve: bool,
) -> Result<Vec<u8>, AsmError> {
    let mut p = text.trim().splitn(2, char::is_whitespace);
    let m = p.next().unwrap_or("").to_ascii_uppercase();
    let a = ops(p.next().unwrap_or(""));
    let mut o = Vec::new();
    macro_rules! need {
        ($n:expr) => {
            if a.len() != $n {
                return Err(err(line, format!("{m} expects {} operands", $n)));
            }
        };
    }
    match m.as_str() {
        "NOP" => word(&mut o, 0x0000),
        "HALT" => word(&mut o, 0x0001),
        "RET" => word(&mut o, 0x0002),
        "EI" => word(&mut o, 0x0003),
        "DI" => word(&mut o, 0x0004),
        "IRET" => word(&mut o, 0x0005),
        "LDI" => {
            need!(1);
            word(&mut o, 0x1000);
            word(&mut o, val(&a[0], labels, line, resolve)?);
        }
        "LD8A" | "LD16A" => {
            need!(1);
            word(&mut o, 0x2000 | (if m == "LD16A" { 0x0800 } else { 0 }));
            word(&mut o, val(&a[0], labels, line, resolve)?);
        }
        "ST8A" | "ST16A" => {
            need!(2);
            let b = belt(&a[1], line)?;
            word(
                &mut o,
                0x3000 | (if m == "ST16A" { 0x0800 } else { 0 }) | ((b as u16) << 8),
            );
            word(&mut o, val(&a[0], labels, line, resolve)?);
        }
        "ADD" | "SUB" | "AND" | "OR" | "XOR" | "MUL" | "DIV" | "MOD" | "SHL" | "SHR" | "CMP"
        | "ADC" | "SBC" | "MULHU" | "MULQ15" => {
            need!(2);
            let x = belt(&a[0], line)?;
            let y = belt(&a[1], line)?;
            let f = match m.as_str() {
                "ADD" => 0,
                "SUB" => 1,
                "AND" => 2,
                "OR" => 3,
                "XOR" => 4,
                "MUL" => 5,
                "DIV" => 6,
                "MOD" => 7,
                "SHL" => 8,
                "SHR" => 9,
                "CMP" => 10,
                "ADC" => 11,
                "SBC" => 12,
                "MULHU" => 13,
                _ => 14,
            };
            word(
                &mut o,
                0x4000 | ((f as u16) << 8) | ((x as u16) << 5) | ((y as u16) << 2),
            );
        }
        "PASS" | "NOT" | "NEG" | "ASR1" | "SHL1" | "SHR1" | "RCR1" => {
            need!(1);
            let x = belt(&a[0], line)?;
            let f = match m.as_str() {
                "PASS" => 0,
                "NOT" => 1,
                "NEG" => 2,
                "ASR1" => 3,
                "SHL1" => 4,
                "SHR1" => 5,
                _ => 6,
            };
            word(&mut o, 0x5000 | ((f as u16) << 8) | ((x as u16) << 5));
        }
        "LD8" | "LD16" => {
            need!(1);
            let x = mem_belt(&a[0], line)?;
            word(
                &mut o,
                0x6000 | (if m == "LD16" { 0x0800 } else { 0 }) | ((x as u16) << 8),
            );
        }
        "ST8" | "ST16" => {
            need!(2);
            let x = mem_belt(&a[0], line)?;
            let v = belt(&a[1], line)?;
            word(
                &mut o,
                0x7000
                    | (if m == "ST16" { 0x0800 } else { 0 })
                    | ((x as u16) << 8)
                    | ((v as u16) << 5),
            );
        }
        "VLD8" | "VLD16" => {
            need!(1);
            let x = mem_belt(&a[0], line)?;
            word(
                &mut o,
                0x8000 | (if m == "VLD16" { 0x0800 } else { 0 }) | ((x as u16) << 8),
            );
        }
        "VST8" | "VST16" => {
            need!(2);
            let x = mem_belt(&a[0], line)?;
            let v = belt(&a[1], line)?;
            word(
                &mut o,
                0x9000
                    | (if m == "VST16" { 0x0800 } else { 0 })
                    | ((x as u16) << 8)
                    | ((v as u16) << 5),
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
            word(&mut o, 0xA000 | ((f as u16) << 8));
            word(&mut o, val(&a[0], labels, line, resolve)?);
        }
        "PUSH" => {
            need!(1);
            word(&mut o, 0xB000 | ((belt(&a[0], line)? as u16) << 8));
        }
        "POP" => {
            need!(0);
            word(&mut o, 0xB800);
        }
        "ZLD8" | "ZLD16" => {
            need!(1);
            let a8 = val(&a[0], labels, line, resolve)?;
            if a8 > 0x00FF {
                return Err(err(line, "zero-page address must be <= 0x00FF"));
            }
            word(
                &mut o,
                0xC000 | (if m == "ZLD16" { 0x0800 } else { 0 }) | a8,
            );
        }
        "ZST8" | "ZST16" => {
            need!(2);
            let a8 = val(&a[0], labels, line, resolve)?;
            if a8 > 0x00FF {
                return Err(err(line, "zero-page address must be <= 0x00FF"));
            }
            let b = belt(&a[1], line)?;
            word(
                &mut o,
                0xD000 | (if m == "ZST16" { 0x0800 } else { 0 }) | ((b as u16) << 8) | a8,
            );
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
            labels.insert(t[..pos].trim().to_ascii_lowercase(), pc as u16);
            t = t[pos + 1..].trim();
            if t.is_empty() {
                continue;
            }
        }
        if t.starts_with('.') {
            let mut q = t.split_whitespace();
            let d = q.next().unwrap();
            let x = q.next().ok_or_else(|| err(line, "directive needs value"))?;
            match d.to_ascii_lowercase().as_str() {
                ".load" => {
                    load = val(x, &labels, line, false)?;
                    pc = load as usize
                }
                ".entry" => entry_expr = Some(x.to_string()),
                _ => return Err(err(line, "unknown directive")),
            }
            continue;
        }
        pc += encode(t, line, &labels, false)?.len();
        if pc > 65536 {
            return Err(err(line, "program too large"));
        }
        if (load as usize) < 0xFF00 && pc > 0xFF00 {
            return Err(err(
                line,
                "program image overlaps MMIO at 0xFF00; reduce code size or use a non-overlapping .load region",
            ));
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
        payload.extend_from_slice(&encode(t, line, &labels, true)?)
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
    fn basic_belt_sequence() {
        let p = assemble("LDI 10\nLDI 20\nADD b1,b0\nHALT").unwrap();
        assert_eq!(p.payload.len(), 12);
    }
    #[test]
    fn zero_page_forms_are_compact() {
        let p = assemble("ZLD16 0x0E\nZST16 0x0E,b0\nHALT").unwrap();
        assert_eq!(p.payload.len(), 6);
    }
    #[test]
    fn rejects_mmio_overlap() {
        let src = format!(".load 0x6FF0\n{}", "NOP\n".repeat(16));
        assert!(assemble(&src).is_err());
    }
}
