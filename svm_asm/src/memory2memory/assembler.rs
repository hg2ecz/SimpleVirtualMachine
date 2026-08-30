use super::{AsmError, Program};
use std::collections::HashMap;
fn err(l: usize, m: impl Into<String>) -> AsmError {
    AsmError::Assembler(format!("line {l}: {}", m.into()))
}
fn num(s: &str) -> Option<i32> {
    let t = s.trim().replace('_', "");
    if let Some(h) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        i32::from_str_radix(h, 16).ok()
    } else if let Some(h) = t.strip_prefix("-0x").or_else(|| t.strip_prefix("-0X")) {
        i32::from_str_radix(h, 16).ok().map(|v| -v)
    } else {
        t.parse().ok()
    }
}
fn val(s: &str, labs: &HashMap<String, u16>, l: usize, res: bool) -> Result<u16, AsmError> {
    if let Some(v) = num(s) {
        return Ok(v as u16);
    }
    if let Some(v) = labs.get(&s.trim().to_ascii_lowercase()) {
        return Ok(*v);
    }
    if res {
        Err(err(l, format!("unknown symbol '{s}'")))
    } else {
        Ok(0)
    }
}
fn areg(s: &str, l: usize) -> Result<u8, AsmError> {
    let u = s.trim().to_ascii_uppercase();
    let n = u
        .strip_prefix('A')
        .ok_or_else(|| err(l, "expected A0..A3"))?
        .parse::<u8>()
        .map_err(|_| err(l, "bad address register"))?;
    if n > 3 {
        return Err(err(l, "address register must be A0..A3"));
    }
    Ok(n)
}
fn ops(s: &str) -> Vec<String> {
    s.split(',')
        .map(|x| x.trim().to_string())
        .filter(|x| !x.is_empty())
        .collect()
}
fn e16(o: &mut Vec<u8>, v: u16) {
    o.extend_from_slice(&v.to_le_bytes())
}
fn mdesc(
    s: &str,
    labs: &HashMap<String, u16>,
    l: usize,
    res: bool,
    source: bool,
) -> Result<Vec<u8>, AsmError> {
    let t = s.trim();
    let mut o = Vec::new();
    if t.starts_with('[') && t.ends_with(']') {
        let inner = t[1..t.len() - 1].trim();
        if let Some(x) = inner.strip_suffix('+') {
            o.push(0x84 | areg(x.trim(), l)?);
            return Ok(o);
        }
        if let Some(x) = inner.strip_prefix('-') {
            o.push(0x88 | areg(x.trim(), l)?);
            return Ok(o);
        }
        let mut pos = None;
        for (i, c) in inner.char_indices().skip(1) {
            if c == '+' || c == '-' {
                pos = Some((i, c));
                break;
            }
        }
        if let Some((i, c)) = pos {
            if inner[..i].trim().to_ascii_uppercase().starts_with('A') {
                let r = areg(inner[..i].trim(), l)?;
                let mut n =
                    num(inner[i + 1..].trim()).ok_or_else(|| err(l, "offset must be numeric"))?;
                if c == '-' {
                    n = -n
                }
                if !(-128..=127).contains(&n) {
                    return Err(err(l, "offset must fit i8"));
                }
                o.push(0x8c | r);
                o.push(n as i8 as u8);
                return Ok(o);
            }
        }
        if inner.to_ascii_uppercase().starts_with('A') {
            o.push(0x80 | areg(inner, l)?);
            return Ok(o);
        }
        let numeric = num(inner).is_some();
        let a = val(inner, labs, l, res)?;
        if numeric && a <= 0x7f {
            o.push(a as u8)
        } else {
            o.push(0xf0);
            e16(&mut o, a)
        }
        return Ok(o);
    }
    if !source {
        return Err(err(l, "destination must be a memory operand"));
    }
    let numeric = num(t).is_some();
    let v = val(t, labs, l, res)?;
    if numeric && v <= 255 {
        o.push(0xf2);
        o.push(v as u8)
    } else {
        o.push(0xf1);
        e16(&mut o, v)
    }
    Ok(o)
}
fn enc(t: &str, l: usize, labs: &HashMap<String, u16>, res: bool) -> Result<Vec<u8>, AsmError> {
    let mut q = t.trim().splitn(2, char::is_whitespace);
    let m = q.next().unwrap_or("").to_ascii_uppercase();
    let a = ops(q.next().unwrap_or(""));
    let mut o = Vec::new();
    macro_rules! n {
        ($x:expr) => {
            if a.len() != $x {
                return Err(err(l, format!("{m} expects {} operands", $x)));
            }
        };
    }
    match m.as_str() {
        "NOP" => o.push(0),
        "HALT" => o.push(1),
        "RET" => o.push(2),
        "EI" => o.push(3),
        "DI" => o.push(4),
        "IRET" => o.push(5),
        "MOV8" | "ADD8" | "SUB8" | "AND8" | "OR8" | "XOR8" | "CMP8" => {
            n!(2);
            o.push(match m.as_str() {
                "MOV8" => 0x10,
                "ADD8" => 0x11,
                "SUB8" => 0x12,
                "AND8" => 0x13,
                "OR8" => 0x14,
                "XOR8" => 0x15,
                _ => 0x16,
            });
            o.extend(mdesc(&a[0], labs, l, res, false)?);
            o.extend(mdesc(&a[1], labs, l, res, true)?);
        }
        "MOV16" | "ADD16" | "SUB16" | "AND16" | "OR16" | "XOR16" | "CMP16" | "MUL16" | "DIV16"
        | "MOD16" | "SHL16" | "SHR16" | "MULQ15" | "ADC16" | "SBC16" | "MULHU16" => {
            n!(2);
            o.push(match m.as_str() {
                "MOV16" => 0x20,
                "ADD16" => 0x21,
                "SUB16" => 0x22,
                "AND16" => 0x23,
                "OR16" => 0x24,
                "XOR16" => 0x25,
                "CMP16" => 0x26,
                "MUL16" => 0x27,
                "DIV16" => 0x28,
                "MOD16" => 0x29,
                "SHL16" => 0x2a,
                "SHR16" => 0x2b,
                "MULQ15" => 0x2c,
                "ADC16" => 0x2d,
                "SBC16" => 0x2e,
                _ => 0x2f,
            });
            o.extend(mdesc(&a[0], labs, l, res, false)?);
            o.extend(mdesc(&a[1], labs, l, res, true)?);
        }
        "INC8" | "DEC8" | "NOT8" | "NEG8" | "INC16" | "DEC16" | "NOT16" | "NEG16" | "ASR1"
        | "RCR1" | "SHL1" | "SHR1" => {
            n!(1);
            o.push(match m.as_str() {
                "INC8" => 0x30,
                "DEC8" => 0x31,
                "NOT8" => 0x32,
                "NEG8" => 0x33,
                "INC16" => 0x38,
                "DEC16" => 0x39,
                "NOT16" => 0x3a,
                "NEG16" => 0x3b,
                "ASR1" => 0x3c,
                "RCR1" => 0x3d,
                "SHL1" => 0x3e,
                _ => 0x3f,
            });
            o.extend(mdesc(&a[0], labs, l, res, false)?);
        }
        "LEA" => {
            n!(2);
            o.push(0x40 | areg(&a[0], l)?);
            e16(&mut o, val(&a[1], labs, l, res)?);
        }
        "ADDA" => {
            n!(2);
            let v = num(&a[1]).ok_or_else(|| err(l, "ADDA immediate must be numeric"))?;
            if !(-128..=127).contains(&v) {
                return Err(err(l, "ADDA immediate must fit i8"));
            }
            o.push(0x44 | areg(&a[0], l)?);
            o.push(v as i8 as u8);
        }
        "MOVA" => {
            n!(2);
            o.push(0x48 | areg(&a[0], l)?);
            o.extend(mdesc(&a[1], labs, l, res, false)?);
        }
        "STORA" => {
            n!(2);
            o.push(0x4c | areg(&a[1], l)?);
            o.extend(mdesc(&a[0], labs, l, res, false)?);
        }
        "BRA" | "BZ" | "BNZ" | "BC" | "BNC" | "BN" | "BNN" | "CALLR" => {
            n!(1);
            let r = num(&a[0])
                .ok_or_else(|| err(l, "short branch operand is a signed byte displacement"))?;
            if !(-128..=127).contains(&r) {
                return Err(err(l, "short branch displacement must fit i8"));
            }
            o.push(match m.as_str() {
                "BRA" => 0x50,
                "BZ" => 0x51,
                "BNZ" => 0x52,
                "BC" => 0x53,
                "BNC" => 0x54,
                "BN" => 0x55,
                "BNN" => 0x56,
                _ => 0x57,
            });
            o.push(r as i8 as u8);
        }
        "JMP" | "JZ" | "JNZ" | "JC" | "JNC" | "JN" | "JNN" | "CALL" => {
            n!(1);
            o.push(match m.as_str() {
                "JMP" => 0x58,
                "JZ" => 0x59,
                "JNZ" => 0x5a,
                "JC" => 0x5b,
                "JNC" => 0x5c,
                "JN" => 0x5d,
                "JNN" => 0x5e,
                _ => 0x5f,
            });
            e16(&mut o, val(&a[0], labs, l, res)?);
        }
        "VLD8" | "VLD16" => {
            n!(2);
            o.push(if m == "VLD8" { 0x60 } else { 0x61 });
            o.extend(mdesc(&a[0], labs, l, res, false)?);
            o.extend(mdesc(&a[1], labs, l, res, false)?);
        }
        "VST8" | "VST16" => {
            n!(2);
            o.push(if m == "VST8" { 0x62 } else { 0x63 });
            o.extend(mdesc(&a[0], labs, l, res, false)?);
            o.extend(mdesc(&a[1], labs, l, res, true)?);
        }
        _ => return Err(err(l, format!("unknown instruction '{m}'"))),
    }
    Ok(o)
}
pub fn assemble(src: &str) -> Result<Program, AsmError> {
    let mut load = 0x0100u16;
    let mut entry: Option<String> = None;
    let mut labs = HashMap::new();
    let mut pc = load as usize;
    for (i, raw) in src.lines().enumerate() {
        let l = i + 1;
        let mut t = raw.split(';').next().unwrap_or("").trim();
        if t.is_empty() {
            continue;
        }
        if let Some(p) = t.find(':') {
            labs.insert(t[..p].trim().to_ascii_lowercase(), pc as u16);
            t = t[p + 1..].trim();
            if t.is_empty() {
                continue;
            }
        }
        if t.starts_with('.') {
            let mut z = t.split_whitespace();
            let d = z.next().unwrap();
            let v = z.next().ok_or_else(|| err(l, "directive value missing"))?;
            match d.to_ascii_lowercase().as_str() {
                ".load" => {
                    load = val(v, &labs, l, false)?;
                    pc = load as usize
                }
                ".entry" => entry = Some(v.to_string()),
                _ => return Err(err(l, "unknown directive")),
            }
            continue;
        }
        pc += enc(t, l, &labs, false)?.len();
        if (load as usize) < 0xFF00 && pc > 0xFF00 {
            return Err(err(
                l,
                "program image overlaps MMIO at 0xFF00; enable optimization/reduce code size",
            ));
        }
    }
    let ep = match entry {
        Some(ref e) => val(e, &labs, 0, true)?,
        None => load,
    };
    let mut payload = Vec::new();
    for (i, raw) in src.lines().enumerate() {
        let l = i + 1;
        let mut t = raw.split(';').next().unwrap_or("").trim();
        if t.is_empty() {
            continue;
        }
        if let Some(p) = t.find(':') {
            t = t[p + 1..].trim();
            if t.is_empty() {
                continue;
            }
        }
        if t.starts_with('.') {
            continue;
        }
        payload.extend(enc(t, l, &labs, true)?)
    }
    Ok(Program {
        load_address: load,
        entry_address: ep,
        payload,
    })
}
