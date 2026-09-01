#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tok {
    Id(String),
    Num(u16),
    Sym(String),
    Eof,
}

fn pos(src: &str, byte: usize) -> (usize, usize) {
    let prefix = &src[..byte.min(src.len())];
    let line = prefix.bytes().filter(|&b| b == b'\n').count() + 1;
    let col = prefix
        .rsplit('\n')
        .next()
        .map(|s| s.chars().count() + 1)
        .unwrap_or(1);
    (line, col)
}

fn at(src: &str, byte: usize, msg: impl AsRef<str>) -> String {
    let (line, col) = pos(src, byte);
    format!("{line}:{col}: {}", msg.as_ref())
}

pub fn lex(src: &str) -> Result<Vec<Tok>, String> {
    let b = src.as_bytes();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < b.len() {
        let c = b[i] as char;
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        if c == '/' && i + 1 < b.len() && b[i + 1] == b'/' {
            i += 2;
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if c == '/' && i + 1 < b.len() && b[i + 1] == b'*' {
            i += 2;
            let mut closed = false;
            while i + 1 < b.len() {
                if b[i] == b'*' && b[i + 1] == b'/' {
                    i += 2;
                    closed = true;
                    break;
                }
                i += 1;
            }
            if !closed {
                return Err(at(src, i.saturating_sub(2), "unterminated block comment"));
            }
            continue;
        }
        if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            i += 1;
            while i < b.len() {
                let d = b[i] as char;
                if d.is_ascii_alphanumeric() || d == '_' {
                    i += 1;
                } else {
                    break;
                }
            }
            out.push(Tok::Id(src[start..i].to_string()));
            continue;
        }
        if c.is_ascii_digit() {
            let start = i;
            if c == '0' && i + 1 < b.len() && (b[i + 1] == b'x' || b[i + 1] == b'X') {
                i += 2;
                let digits = i;
                while i < b.len() && (b[i] as char).is_ascii_hexdigit() {
                    i += 1;
                }
                if i == digits {
                    return Err(at(src, start, "hex literal requires digits"));
                }
            } else if c == '0' && i + 1 < b.len() && (b[i + 1] == b'b' || b[i + 1] == b'B') {
                i += 2;
                let digits = i;
                while i < b.len() && matches!(b[i], b'0' | b'1') {
                    i += 1;
                }
                if i == digits {
                    return Err(at(src, start, "binary literal requires digits"));
                }
            } else {
                i += 1;
                while i < b.len() && (b[i] as char).is_ascii_digit() {
                    i += 1;
                }
            }
            let raw = &src[start..i];
            let value = if raw.starts_with("0x") || raw.starts_with("0X") {
                u16::from_str_radix(&raw[2..], 16)
                    .map_err(|_| at(src, start, format!("invalid number {raw}")))?
            } else if raw.starts_with("0b") || raw.starts_with("0B") {
                u16::from_str_radix(&raw[2..], 2)
                    .map_err(|_| at(src, start, format!("invalid number {raw}")))?
            } else {
                raw.parse::<u16>()
                    .map_err(|_| at(src, start, format!("invalid number {raw}")))?
            };
            out.push(Tok::Num(value));
            continue;
        }
        let two = if i + 1 < b.len() { &src[i..i + 2] } else { "" };
        if ["->", "==", "!=", "<=", ">=", "<<", ">>"].contains(&two) {
            out.push(Tok::Sym(two.to_string()));
            i += 2;
            continue;
        }
        if "{}();,:=+-*/%&|^~<>[]".contains(c) {
            out.push(Tok::Sym(c.to_string()));
            i += 1;
            continue;
        }
        return Err(at(src, i, format!("unexpected character '{c}'")));
    }
    out.push(Tok::Eof);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lex_hex_and_rust_like_syntax() {
        let t = lex("fn f(u16 x) -> u16 { return x + 0x10; }").unwrap();
        assert!(t.contains(&Tok::Num(0x10)));
        assert!(t.contains(&Tok::Sym("->".into())));
    }
    #[test]
    fn lex_binary_literal() {
        let t = lex("fn main()->u16{return 0b10101010;}").unwrap();
        assert!(t.contains(&Tok::Num(0xaa)));
    }

    #[test]
    fn lex_line_and_block_comments() {
        let t = lex("// one\nfn main()->u16{/* two */ return 1;} ").unwrap();
        assert!(t.contains(&Tok::Id("main".into())));
        assert!(t.contains(&Tok::Num(1)));
    }
}
