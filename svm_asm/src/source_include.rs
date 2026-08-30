use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IncludeStyle {
    Assembly,
    C,
}

pub fn expand_source_file(
    input: &Path,
    include_dirs: &[PathBuf],
    style: IncludeStyle,
) -> Result<String, String> {
    let root = fs::canonicalize(input)
        .map_err(|e| format!("cannot open source '{}': {e}", input.display()))?;
    let mut seen = HashSet::new();
    let mut stack = Vec::new();
    expand_file(&root, include_dirs, style, &mut seen, &mut stack, 0)
}

fn expand_file(
    path: &Path,
    include_dirs: &[PathBuf],
    style: IncludeStyle,
    seen: &mut HashSet<PathBuf>,
    stack: &mut Vec<PathBuf>,
    depth: usize,
) -> Result<String, String> {
    const MAX_INCLUDE_DEPTH: usize = 64;
    if depth > MAX_INCLUDE_DEPTH {
        return Err(format!("include nesting exceeds {MAX_INCLUDE_DEPTH} files"));
    }

    let canonical = fs::canonicalize(path)
        .map_err(|e| format!("cannot open included file '{}': {e}", path.display()))?;

    if let Some(pos) = stack.iter().position(|p| p == &canonical) {
        let mut cycle = stack[pos..]
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>();
        cycle.push(canonical.display().to_string());
        return Err(format!("cyclic include: {}", cycle.join(" -> ")));
    }
    if seen.contains(&canonical) {
        return Ok(String::new());
    }
    seen.insert(canonical.clone());
    stack.push(canonical.clone());

    let source = fs::read_to_string(&canonical)
        .map_err(|e| format!("cannot read source '{}': {e}", canonical.display()))?;
    let base_dir = canonical.parent().unwrap_or_else(|| Path::new("."));
    let mut output = String::new();

    for (line_index, raw_line) in source.lines().enumerate() {
        match parse_include_line(raw_line, style) {
            Ok(Some(name)) => {
                let included = resolve_include(&name, base_dir, include_dirs).ok_or_else(|| {
                    let chain = stack
                        .iter()
                        .map(|p| p.display().to_string())
                        .collect::<Vec<_>>()
                        .join(" -> ");
                    format!(
                        "{}:{}: include file '{}' not found (include chain: {})",
                        canonical.display(),
                        line_index + 1,
                        name,
                        chain
                    )
                })?;
                output.push_str(&expand_file(
                    &included,
                    include_dirs,
                    style,
                    seen,
                    stack,
                    depth + 1,
                )?);
                if !output.ends_with('\n') {
                    output.push('\n');
                }
            }
            Ok(None) => {
                output.push_str(raw_line);
                output.push('\n');
            }
            Err(message) => {
                return Err(format!(
                    "{}:{}: {message}",
                    canonical.display(),
                    line_index + 1
                ));
            }
        }
    }

    stack.pop();
    Ok(output)
}

fn resolve_include(name: &str, base_dir: &Path, include_dirs: &[PathBuf]) -> Option<PathBuf> {
    let requested = Path::new(name);
    if requested.is_absolute() {
        return requested.is_file().then(|| requested.to_path_buf());
    }

    let local = base_dir.join(requested);
    if local.is_file() {
        return Some(local);
    }

    include_dirs
        .iter()
        .map(|dir| dir.join(requested))
        .find(|candidate| candidate.is_file())
}

fn parse_include_line(line: &str, style: IncludeStyle) -> Result<Option<String>, String> {
    let trimmed = line.trim_start();
    let rest = match style {
        IncludeStyle::Assembly => trimmed
            .strip_prefix(".include")
            .filter(|rest| rest.chars().next().is_some_and(char::is_whitespace)),
        IncludeStyle::C => trimmed
            .strip_prefix("include")
            .filter(|rest| rest.chars().next().is_some_and(char::is_whitespace)),
    };
    let Some(mut rest) = rest else {
        return Ok(None);
    };

    rest = rest.trim_start();
    if !rest.starts_with('"') {
        return Err("include expects a quoted file name".into());
    }
    let after_open = &rest[1..];
    let Some(end_quote) = after_open.find('"') else {
        return Err("unterminated include file name".into());
    };
    let name = &after_open[..end_quote];
    if name.is_empty() {
        return Err("include file name must not be empty".into());
    }

    let trailing = after_open[end_quote + 1..].trim();
    let valid_trailing = match style {
        IncludeStyle::Assembly => trailing.is_empty() || trailing.starts_with(';'),
        IncludeStyle::C => {
            if trailing.is_empty() || trailing.starts_with("//") {
                true
            } else if let Some(after_semicolon) = trailing.strip_prefix(';') {
                let after_semicolon = after_semicolon.trim_start();
                after_semicolon.is_empty() || after_semicolon.starts_with("//")
            } else {
                false
            }
        }
    };
    if !valid_trailing {
        return Err("unexpected text after include file name".into());
    }

    Ok(Some(name.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_assembly_include() {
        assert_eq!(
            parse_include_line(
                "  .include \"lib/io.asm\" ; comment",
                IncludeStyle::Assembly
            )
            .unwrap(),
            Some("lib/io.asm".into())
        );
    }

    #[test]
    fn parses_c_include() {
        assert_eq!(
            parse_include_line("include \"lib/io.sc\";", IncludeStyle::C).unwrap(),
            Some("lib/io.sc".into())
        );
    }
}
