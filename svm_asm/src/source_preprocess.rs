use std::collections::HashMap;

/// Expand simple symbolic constants before the target-specific assembler runs.
///
/// Syntax: `.equ NAME, VALUE` or `.equ NAME VALUE`.
/// VALUE is deliberately a single token (numeric literal, label, register alias,
/// etc.). This keeps the feature architecture-neutral and predictable.
pub fn expand_equ(source: &str) -> Result<String, String> {
    let mut defs = HashMap::<String, String>::new();
    let mut body = Vec::<(usize, &str)>::new();

    for (idx, line) in source.lines().enumerate() {
        let n = idx + 1;
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed
            .strip_prefix(".equ")
            .filter(|r| r.chars().next().is_some_and(char::is_whitespace))
        {
            let code = strip_comment(rest).trim();
            let (name, value) = if let Some((a, b)) = code.split_once(',') {
                (a.trim(), b.trim())
            } else {
                let mut p = code.split_whitespace();
                let a = p.next().unwrap_or("");
                let b = p.next().unwrap_or("");
                if p.next().is_some() {
                    return Err(format!("line {n}: .equ value must be one token"));
                }
                (a, b)
            };
            if !is_ident(name) {
                return Err(format!("line {n}: invalid .equ name '{name}'"));
            }
            if value.is_empty() || value.chars().any(char::is_whitespace) {
                return Err(format!("line {n}: .equ value must be one token"));
            }
            let key = name.to_ascii_lowercase();
            if defs.insert(key, value.to_string()).is_some() {
                return Err(format!("line {n}: duplicate .equ '{name}'"));
            }
        } else {
            body.push((n, line));
        }
    }

    // Resolve constants through other constants (forward references included).
    for _ in 0..64 {
        let snapshot = defs.clone();
        let mut changed = false;
        for value in defs.values_mut() {
            if let Some(next) = snapshot.get(&value.to_ascii_lowercase()) {
                if next != value {
                    *value = next.clone();
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
    for (name, value) in &defs {
        if defs.contains_key(&value.to_ascii_lowercase()) {
            return Err(format!("cyclic .equ definition involving '{name}'"));
        }
    }

    let mut out = String::new();
    for (_n, line) in body {
        out.push_str(&replace_tokens(line, &defs));
        out.push('\n');
    }
    Ok(out)
}

fn strip_comment(s: &str) -> &str {
    let semi = s.find(';');
    let slash = s.find("//");
    let backslash = s.find('\\');
    let cut = [semi, slash, backslash]
        .into_iter()
        .flatten()
        .min()
        .unwrap_or(s.len());
    &s[..cut]
}

fn is_ident(s: &str) -> bool {
    let mut c = s.chars();
    c.next()
        .is_some_and(|x| x.is_ascii_alphabetic() || x == '_')
        && c.all(|x| x.is_ascii_alphanumeric() || x == '_')
}

fn replace_tokens(line: &str, defs: &HashMap<String, String>) -> String {
    let mut out = String::with_capacity(line.len());
    let bytes = line.as_bytes();
    let mut i = 0;
    let mut quoted = false;
    while i < bytes.len() {
        let ch = bytes[i] as char;
        if ch == '"' {
            quoted = !quoted;
            out.push(ch);
            i += 1;
            continue;
        }
        if !quoted && (ch == ';' || ch == '\\') {
            out.push_str(&line[i..]);
            break;
        }
        if !quoted && (ch.is_ascii_alphabetic() || ch == '_') {
            let start = i;
            i += 1;
            while i < bytes.len() {
                let c = bytes[i] as char;
                if c.is_ascii_alphanumeric() || c == '_' {
                    i += 1;
                } else {
                    break;
                }
            }
            let token = &line[start..i];
            if let Some(v) = defs.get(&token.to_ascii_lowercase()) {
                out.push_str(v);
            } else {
                out.push_str(token);
            }
        } else {
            out.push(ch);
            i += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_equ_case_insensitively() {
        let s = ".equ CONSOLE, 0xFF20\nMOVI R1, console ; CONSOLE\n";
        assert_eq!(expand_equ(s).unwrap(), "MOVI R1, 0xFF20 ; CONSOLE\n");
    }

    #[test]
    fn supports_forward_aliases() {
        let s = ".equ A, B\n.equ B, 42\nMOVI R0, A\n";
        assert_eq!(expand_equ(s).unwrap(), "MOVI R0, 42\n");
    }

    #[test]
    fn rejects_cycle() {
        assert!(expand_equ(".equ A,B\n.equ B,A\nNOP\n").is_err());
    }
}
