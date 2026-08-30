use super::{AsmError, Program};
use std::collections::HashMap;
fn err(l: usize, m: impl Into<String>) -> AsmError {
    AsmError::Assembler(format!("line {l}: {}", m.into()))
}
fn num(s: &str) -> Option<i32> {
    let t = s.trim().replace('_', "");
    if let Some(h) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        i32::from_str_radix(h, 16).ok()
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
fn reg(s: &str, l: usize) -> Result<u8, AsmError> {
    let u = s.trim().to_ascii_uppercase();
    let n = u
        .strip_prefix('R')
        .ok_or_else(|| err(l, "expected register"))?
        .parse::<u8>()
        .map_err(|_| err(l, "bad register"))?;
    if n > 7 {
        return Err(err(l, "register must be R0..R7"));
    }
    Ok(n)
}
fn ops(s: &str) -> Vec<String> {
    s.split(',')
        .map(|x| x.trim().to_string())
        .filter(|x| !x.is_empty())
        .collect()
}
fn emit16(o: &mut Vec<u8>, v: u16) {
    o.extend_from_slice(&v.to_le_bytes())
}
fn desc(
    s: &str,
    labs: &HashMap<String, u16>,
    l: usize,
    res: bool,
    allow_imm: bool,
) -> Result<Vec<u8>, AsmError> {
    let t = s.trim();
    let mut o = Vec::new();
    if t.starts_with('[') && t.ends_with(']') {
        let inner = t[1..t.len() - 1].trim();
        if let Some(x) = inner.strip_suffix('+') {
            o.push(0x18 | reg(x.trim(), l)?);
            return Ok(o);
        }
        if let Some(x) = inner.strip_prefix('-') {
            o.push(0x28 | reg(x.trim(), l)?);
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
            let r = reg(inner[..i].trim(), l)?;
            let mut n =
                num(inner[i + 1..].trim()).ok_or_else(|| err(l, "offset must be numeric"))?;
            if c == '-' {
                n = -n
            }
            if !(-128..=127).contains(&n) {
                return Err(err(l, "offset must fit i8"));
            }
            o.push(0x20 | r);
            o.push(n as i8 as u8);
            return Ok(o);
        }
        if let Ok(r) = reg(inner, l) {
            o.push(0x10 | r);
            return Ok(o);
        }
        let numeric = num(inner).is_some();
        let a = val(inner, labs, l, res)?;
        if numeric && a <= 255 {
            o.push(0xE0);
            o.push(a as u8)
        } else {
            o.push(0xE1);
            emit16(&mut o, a)
        }
        return Ok(o);
    }
    if let Ok(r) = reg(t, l) {
        o.push(r);
        return Ok(o);
    }
    if !allow_imm {
        return Err(err(l, "memory operand required"));
    }
    let numeric = num(t).is_some();
    let v = val(t, labs, l, res)?;
    if numeric && v <= 255 {
        o.push(0xF0);
        o.push(v as u8)
    } else {
        o.push(0xF1);
        emit16(&mut o, v)
    }
    Ok(o)
}
fn enc(text: &str, l: usize, labs: &HashMap<String, u16>, res: bool) -> Result<Vec<u8>, AsmError> {
    let mut q = text.trim().splitn(2, char::is_whitespace);
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
        "MOV" | "ADD" | "SUB" | "AND" | "OR" | "XOR" | "CMP" | "MUL" | "DIV" | "MOD" | "SHL"
        | "SHR" | "MULQ15" | "ADC" | "SBC" | "MULHU" => {
            n!(2);
            if a[1].contains("+]") || a[1].starts_with("[-") {
                return Err(err(
                    l,
                    "ALU source may not auto-update its address register",
                ));
            }
            let op = match m.as_str() {
                "MOV" => 0x20,
                "ADD" => 0x21,
                "SUB" => 0x22,
                "AND" => 0x23,
                "OR" => 0x24,
                "XOR" => 0x25,
                "CMP" => 0x26,
                "MUL" => 0x27,
                "DIV" => 0x28,
                "MOD" => 0x29,
                "SHL" => 0x2A,
                "SHR" => 0x2B,
                "MULQ15" => 0x2C,
                "ADC" => 0x2D,
                "SBC" => 0x2E,
                _ => 0x2F,
            };
            o.push(op);
            o.push(reg(&a[0], l)?);
            o.extend(desc(&a[1], labs, l, res, true)?);
        }
        "MOVI" => {
            n!(2);
            o.push(0x20);
            o.push(reg(&a[0], l)?);
            o.extend(desc(&a[1], labs, l, res, true)?);
        }
        "ADDI" | "SUBI" | "ANDI" | "ORI" | "XORI" | "CMPI" => {
            n!(2);
            let base = match m.as_str() {
                "ADDI" => "ADD",
                "SUBI" => "SUB",
                "ANDI" => "AND",
                "ORI" => "OR",
                "XORI" => "XOR",
                _ => "CMP",
            };
            return enc(&format!("{base} {},{}", a[0], a[1]), l, labs, res);
        }
        "NOT" | "NEG" | "INC" | "DEC" | "ASR1" | "SHL1" | "SHR1" | "RCR1" => {
            n!(1);
            let op = match m.as_str() {
                "NOT" => 0x30,
                "NEG" => 0x31,
                "INC" => 0x32,
                "DEC" => 0x33,
                "ASR1" => 0x34,
                "SHL1" => 0x35,
                "SHR1" => 0x36,
                _ => 0x37,
            };
            o.push(op);
            o.push(reg(&a[0], l)?);
        }
        "LOAD8" | "LOAD16" => {
            n!(2);
            o.push(if m == "LOAD8" { 0x40 } else { 0x41 });
            o.push(reg(&a[0], l)?);
            o.extend(desc(&a[1], labs, l, res, false)?);
        }
        "STORE8" | "STORE16" => {
            n!(2);
            o.push(if m == "STORE8" { 0x42 } else { 0x43 });
            o.extend(desc(&a[0], labs, l, res, false)?);
            o.push(reg(&a[1], l)?);
        }
        "ZLOAD8" | "ZLOAD16" => {
            n!(1);
            return enc(
                &format!(
                    "{} R0,[{}]",
                    if m == "ZLOAD8" { "LOAD8" } else { "LOAD16" },
                    a[0]
                ),
                l,
                labs,
                res,
            );
        }
        "ZSTORE8" | "ZSTORE16" => {
            n!(1);
            return enc(
                &format!(
                    "{} [{}],R0",
                    if m == "ZSTORE8" { "STORE8" } else { "STORE16" },
                    a[0]
                ),
                l,
                labs,
                res,
            );
        }
        "VLOAD8" | "VLOAD16" => {
            n!(2);
            o.push(if m == "VLOAD8" { 0x70 } else { 0x71 });
            o.push(reg(&a[0], l)?);
            o.extend(desc(&a[1], labs, l, res, false)?);
        }
        "VSTORE8" | "VSTORE16" => {
            n!(2);
            o.push(if m == "VSTORE8" { 0x72 } else { 0x73 });
            o.extend(desc(&a[0], labs, l, res, false)?);
            o.push(reg(&a[1], l)?);
        }
        "JMP" | "JZ" | "JNZ" | "JC" | "JNC" | "JN" | "JNN" | "CALL" => {
            n!(1);
            o.push(match m.as_str() {
                "JMP" => 0x60,
                "JZ" => 0x61,
                "JNZ" => 0x62,
                "JC" => 0x63,
                "JNC" => 0x64,
                "JN" => 0x65,
                "JNN" => 0x66,
                _ => 0x67,
            });
            emit16(&mut o, val(&a[0], labs, l, res)?);
        }
        "PUSH" => {
            n!(1);
            let r = reg(&a[0], l)?;
            let mut x = enc("SUB R6,2", l, labs, res)?;
            o.append(&mut x);
            let mut x = enc(&format!("STORE16 [R6],R{r}"), l, labs, res)?;
            o.append(&mut x);
        }
        "POP" => {
            n!(1);
            let r = reg(&a[0], l)?;
            let mut x = enc(&format!("LOAD16 R{r},[R6]"), l, labs, res)?;
            o.append(&mut x);
            let mut x = enc("ADD R6,2", l, labs, res)?;
            o.append(&mut x);
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
