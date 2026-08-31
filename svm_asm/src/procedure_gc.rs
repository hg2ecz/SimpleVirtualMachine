use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcedureSyntax {
    Labels,
    StackLabels,
    StackWords,
}

#[derive(Debug)]
struct Procedure {
    name: String,
    body: Vec<String>,
}

#[derive(Debug)]
enum SourceItem {
    Global(String),
    Procedure(usize),
}

/// Remove unreachable procedures from expanded assembly source.
///
/// Procedures are explicitly delimited by:
///     .proc name
///       ... body ...
///     .endproc
///
/// `.entry name` and `.keep name` are roots. Any symbolic reference from
/// global source or a live procedure to another procedure makes that target
/// live as well. References are recognized as identifier tokens, so CALL/JMP,
/// address operands and future data directives all participate automatically.
///
/// `.keep` is consumed by this pass and is not passed to target assemblers.
pub fn eliminate_unused_procedures(
    source: &str,
    syntax: ProcedureSyntax,
) -> Result<String, String> {
    let mut globals = Vec::<String>::new();
    let mut items = Vec::<SourceItem>::new();
    let mut procedures = Vec::<Procedure>::new();
    let mut proc_index = HashMap::<String, usize>::new();
    let mut keep_roots = Vec::<String>::new();
    let mut entry_roots = Vec::<String>::new();
    let mut current: Option<Procedure> = None;

    for (idx, raw) in source.lines().enumerate() {
        let line_no = idx + 1;
        let trimmed = raw.trim();
        if let Some(rest) = directive_arg(trimmed, ".proc") {
            if current.is_some() {
                return Err(format!("line {line_no}: nested .proc is not allowed"));
            }
            let name = parse_ident_arg(rest, line_no, ".proc")?;
            let key = name.to_ascii_lowercase();
            if proc_index.contains_key(&key) {
                return Err(format!("line {line_no}: duplicate procedure '{name}'"));
            }
            proc_index.insert(key, procedures.len());
            current = Some(Procedure {
                name,
                body: Vec::new(),
            });
            continue;
        }
        if trimmed.eq_ignore_ascii_case(".endproc") {
            let proc = current
                .take()
                .ok_or_else(|| format!("line {line_no}: .endproc without matching .proc"))?;
            let index = procedures.len();
            procedures.push(proc);
            items.push(SourceItem::Procedure(index));
            continue;
        }
        if let Some(rest) = directive_arg(trimmed, ".keep") {
            if current.is_some() {
                return Err(format!(
                    "line {line_no}: .keep is only allowed outside procedures"
                ));
            }
            keep_roots.push(parse_ident_arg(rest, line_no, ".keep")?);
            continue;
        }
        if let Some(rest) = directive_arg(trimmed, ".entry") {
            if current.is_some() {
                return Err(format!(
                    "line {line_no}: .entry is only allowed outside procedures"
                ));
            }
            if let Some(token) = first_identifier(rest) {
                entry_roots.push(token.to_string());
            }
            globals.push(raw.to_string());
            items.push(SourceItem::Global(raw.to_string()));
            continue;
        }

        if let Some(proc) = current.as_mut() {
            proc.body.push(raw.to_string());
        } else {
            globals.push(raw.to_string());
            items.push(SourceItem::Global(raw.to_string()));
        }
    }

    if let Some(proc) = current {
        return Err(format!("unterminated .proc '{}'", proc.name));
    }
    if procedures.is_empty() {
        return Ok(globals.join("\n") + "\n");
    }

    // proc_index was filled before pushes. Since nested procedures are forbidden,
    // declaration order and final vector order are identical.
    let names: HashSet<String> = procedures
        .iter()
        .map(|p| p.name.to_ascii_lowercase())
        .collect();
    let mut live = HashSet::<String>::new();
    let mut queue = VecDeque::<String>::new();

    let mut add_root = |name: &str| {
        let key = name.to_ascii_lowercase();
        if names.contains(&key) && live.insert(key.clone()) {
            queue.push_back(key);
        }
    };

    for root in &keep_roots {
        if !names.contains(&root.to_ascii_lowercase()) {
            return Err(format!(".keep names unknown procedure '{root}'"));
        }
    }
    for root in &entry_roots {
        add_root(root);
    }
    for root in &keep_roots {
        add_root(root);
    }
    for line in &globals {
        for token in identifier_tokens(strip_comment(line, syntax)) {
            add_root(token);
        }
    }
    drop(add_root);

    let by_name: HashMap<String, &Procedure> = procedures
        .iter()
        .map(|p| (p.name.to_ascii_lowercase(), p))
        .collect();

    while let Some(name) = queue.pop_front() {
        let proc = by_name[&name];
        for line in &proc.body {
            for token in identifier_tokens(strip_comment(line, syntax)) {
                let key = token.to_ascii_lowercase();
                if names.contains(&key) && live.insert(key.clone()) {
                    queue.push_back(key);
                }
            }
        }
    }

    let mut out = String::new();
    for item in &items {
        match item {
            SourceItem::Global(line) => {
                out.push_str(line);
                out.push('\n');
            }
            SourceItem::Procedure(index) => {
                let proc = &procedures[*index];
                if !live.contains(&proc.name.to_ascii_lowercase()) {
                    continue;
                }
                match syntax {
                    ProcedureSyntax::Labels | ProcedureSyntax::StackLabels => {
                        out.push_str(&proc.name);
                        out.push_str(":\n");
                        for line in &proc.body {
                            out.push_str(line);
                            out.push('\n');
                        }
                    }
                    ProcedureSyntax::StackWords => {
                        out.push_str(": ");
                        out.push_str(&proc.name);
                        out.push('\n');
                        for line in &proc.body {
                            out.push_str(line);
                            out.push('\n');
                        }
                        out.push_str(";\n");
                    }
                }
            }
        }
    }
    Ok(out)
}

fn directive_arg<'a>(line: &'a str, directive: &str) -> Option<&'a str> {
    let prefix = line.get(..directive.len())?;
    if !prefix.eq_ignore_ascii_case(directive) {
        return None;
    }
    let rest = &line[directive.len()..];
    if rest.is_empty() || rest.chars().next().is_some_and(char::is_whitespace) {
        Some(rest.trim())
    } else {
        None
    }
}

fn parse_ident_arg(rest: &str, line: usize, directive: &str) -> Result<String, String> {
    let mut p = rest.split_whitespace();
    let name = p.next().unwrap_or("");
    if !is_ident(name) || p.next().is_some() {
        return Err(format!(
            "line {line}: {directive} expects exactly one identifier"
        ));
    }
    Ok(name.to_string())
}

fn first_identifier(s: &str) -> Option<&str> {
    identifier_tokens(s).next()
}

fn is_ident(s: &str) -> bool {
    let mut chars = s.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn strip_comment(line: &str, syntax: ProcedureSyntax) -> &str {
    match syntax {
        ProcedureSyntax::Labels => line.split(';').next().unwrap_or(line),
        ProcedureSyntax::StackLabels | ProcedureSyntax::StackWords => {
            line.split('\\').next().unwrap_or(line)
        }
    }
}

fn identifier_tokens(s: &str) -> impl Iterator<Item = &str> {
    struct Tokens<'a> {
        s: &'a str,
        pos: usize,
    }
    impl<'a> Iterator for Tokens<'a> {
        type Item = &'a str;
        fn next(&mut self) -> Option<Self::Item> {
            let b = self.s.as_bytes();
            while self.pos < b.len() {
                let c = b[self.pos] as char;
                if c.is_ascii_alphabetic() || c == '_' {
                    break;
                }
                self.pos += 1;
            }
            if self.pos >= b.len() {
                return None;
            }
            let start = self.pos;
            self.pos += 1;
            while self.pos < b.len() {
                let c = b[self.pos] as char;
                if c.is_ascii_alphanumeric() || c == '_' {
                    self.pos += 1;
                } else {
                    break;
                }
            }
            Some(&self.s[start..self.pos])
        }
    }
    Tokens { s, pos: 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_unreachable_and_keeps_transitive_calls() {
        let src = ".entry start\n.proc start\n CALL used\n HALT\n.endproc\n.proc used\n RET\n.endproc\n.proc dead\n RET\n.endproc\n";
        let out = eliminate_unused_procedures(src, ProcedureSyntax::Labels).unwrap();
        assert!(out.contains("start:"));
        assert!(out.contains("used:"));
        assert!(!out.contains("dead:"));
    }

    #[test]
    fn address_reference_keeps_procedure() {
        let src = ".entry start\n.proc start\n MOVI R0, callback\n RET\n.endproc\n.proc callback\n RET\n.endproc\n";
        let out = eliminate_unused_procedures(src, ProcedureSyntax::Labels).unwrap();
        assert!(out.contains("callback:"));
    }

    #[test]
    fn comments_do_not_create_false_references() {
        let src =
            ".entry start\n.proc start\n ; dead\n RET\n.endproc\n.proc dead\n RET\n.endproc\n";
        let out = eliminate_unused_procedures(src, ProcedureSyntax::Labels).unwrap();
        assert!(!out.contains("dead:"));
    }

    #[test]
    fn keep_is_an_explicit_root() {
        let src =
            ".entry start\n.keep irq\n.proc start\n RET\n.endproc\n.proc irq\n RET\n.endproc\n";
        let out = eliminate_unused_procedures(src, ProcedureSyntax::Labels).unwrap();
        assert!(out.contains("irq:"));
        assert!(!out.contains(".keep"));
    }

    #[test]
    fn emits_stack_words() {
        let src = ".entry main\n.proc main\n helper\n.endproc\n.proc helper\n EXIT\n.endproc\n";
        let out = eliminate_unused_procedures(src, ProcedureSyntax::StackWords).unwrap();
        assert!(out.contains(": main\n"));
        assert!(out.contains(": helper\n"));
        assert!(out.contains(";\n"));
    }
}
