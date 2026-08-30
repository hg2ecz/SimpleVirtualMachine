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
fn reg(s: &str) -> Option<u8> {
    let u = s.trim().to_ascii_uppercase();
    let n = u.strip_prefix('R')?.parse::<u8>().ok()?;
    if n < 8 { Some(n) } else { None }
}
fn src_code(s: &str) -> Option<u8> {
    if let Some(r) = reg(s) {
        return Some(r);
    }
    match s.trim().to_ascii_uppercase().as_str() {
        "ALU.OUT" => Some(8),
        "MEM.R8" => Some(9),
        "MEM.R16" => Some(10),
        "VMEM.R8" => Some(11),
        "VMEM.R16" => Some(12),
        "STACK.POP" => Some(13),
        "CTRL.RETADDR" => Some(14),
        "CTRL.IRETADDR" => Some(15),
        "FLAGS" => Some(16),
        "ZERO" => Some(17),
        _ => None,
    }
}
fn dst_code(s: &str) -> Option<u8> {
    if let Some(r) = reg(s) {
        return Some(r);
    }
    match s.trim().to_ascii_uppercase().as_str() {
        "ALU.X" => Some(8),
        "ALU.ADD" => Some(9),
        "ALU.ADC" => Some(10),
        "ALU.SUB" => Some(11),
        "ALU.SBC" => Some(12),
        "ALU.AND" => Some(13),
        "ALU.OR" => Some(14),
        "ALU.XOR" => Some(15),
        "ALU.MUL" => Some(16),
        "ALU.MULHU" => Some(17),
        "ALU.MULQ15" => Some(18),
        "ALU.DIV" => Some(19),
        "ALU.MOD" => Some(20),
        "ALU.SHL" => Some(21),
        "ALU.SHR" => Some(22),
        "ALU.CMP" => Some(23),
        "ALU.NOT" => Some(24),
        "ALU.NEG" => Some(25),
        "ALU.ASR1" => Some(26),
        "ALU.SHL1" => Some(27),
        "ALU.SHR1" => Some(28),
        "ALU.RCR1" => Some(29),
        "MEM.ADDR" => Some(30),
        "MEM.W8" => Some(31),
        "MEM.W16" => Some(32),
        "VMEM.ADDR" => Some(33),
        "VMEM.W8" => Some(34),
        "VMEM.W16" => Some(35),
        "CTRL.JMP" => Some(36),
        "CTRL.JZ" => Some(37),
        "CTRL.JNZ" => Some(38),
        "CTRL.JC" => Some(39),
        "CTRL.JNC" => Some(40),
        "CTRL.JN" => Some(41),
        "CTRL.JNN" => Some(42),
        "CTRL.CALL" => Some(43),
        "CTRL.HALT" => Some(44),
        "CTRL.EI" => Some(45),
        "CTRL.DI" => Some(46),
        "STACK.PUSH" => Some(47),
        _ => None,
    }
}
fn ops(s: &str) -> Vec<String> {
    s.split(',')
        .map(|x| x.trim().to_string())
        .filter(|x| !x.is_empty())
        .collect()
}
fn word(o: &mut Vec<u8>, w: u16) {
    o.extend_from_slice(&w.to_le_bytes())
}
fn mov(
    o: &mut Vec<u8>,
    src: &str,
    dst: &str,
    labels: &HashMap<String, u16>,
    line: usize,
    resolve: bool,
) -> Result<(), AsmError> {
    let d =
        dst_code(dst).ok_or_else(|| err(line, format!("unknown TTA destination port '{dst}'")))?;
    if let Some(s) = src_code(src) {
        word(o, ((s as u16) << 6) | (d as u16));
    } else {
        word(o, (63u16 << 6) | (d as u16));
        word(o, val(src, labels, line, resolve)?);
    }
    Ok(())
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
        "MOV" => {
            need!(2);
            mov(&mut o, &a[0], &a[1], labels, line, resolve)?
        }
        "NOP" => {
            need!(0);
            mov(&mut o, "R0", "R0", labels, line, resolve)?
        }
        "HALT" => {
            need!(0);
            mov(&mut o, "ZERO", "CTRL.HALT", labels, line, resolve)?
        }
        "EI" => {
            need!(0);
            mov(&mut o, "ZERO", "CTRL.EI", labels, line, resolve)?
        }
        "DI" => {
            need!(0);
            mov(&mut o, "ZERO", "CTRL.DI", labels, line, resolve)?
        }
        "RET" => {
            need!(0);
            mov(&mut o, "CTRL.RETADDR", "CTRL.JMP", labels, line, resolve)?
        }
        "IRET" => {
            need!(0);
            mov(&mut o, "CTRL.IRETADDR", "CTRL.JMP", labels, line, resolve)?
        }
        "PUSH" => {
            need!(1);
            mov(&mut o, &a[0], "STACK.PUSH", labels, line, resolve)?
        }
        "POP" => {
            need!(1);
            mov(&mut o, "STACK.POP", &a[0], labels, line, resolve)?
        }
        "JMP" | "JZ" | "JNZ" | "JC" | "JNC" | "JN" | "JNN" | "CALL" => {
            need!(1);
            let d = match m.as_str() {
                "JMP" => "CTRL.JMP",
                "JZ" => "CTRL.JZ",
                "JNZ" => "CTRL.JNZ",
                "JC" => "CTRL.JC",
                "JNC" => "CTRL.JNC",
                "JN" => "CTRL.JN",
                "JNN" => "CTRL.JNN",
                _ => "CTRL.CALL",
            };
            mov(&mut o, &a[0], d, labels, line, resolve)?
        }
        _ => {
            return Err(err(
                line,
                format!("unknown instruction '{m}'; TTA16 core syntax uses MOV source,destination"),
            ));
        }
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
                "program image overlaps MMIO at 0xFF00; enable optimization/reduce code size",
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
    fn tta_add_sequence() {
        let p = assemble("MOV 10,R0\nMOV R0,ALU.X\nMOV 20,ALU.ADD\nMOV ALU.OUT,R1\nHALT").unwrap();
        assert!(!p.payload.is_empty());
    }
}
