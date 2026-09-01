use std::{
    collections::HashSet,
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use svm_clite::Target;

/// Very small textual C-Lite include expander.
/// Includes are handled before lexing, exactly once per canonical file.
fn load_source(
    path: &Path,
    include_dirs: &[PathBuf],
    stack: &mut Vec<PathBuf>,
    seen: &mut HashSet<PathBuf>,
) -> Result<String, String> {
    let canonical = path
        .canonicalize()
        .map_err(|e| format!("{}: {e}", path.display()))?;

    if let Some(pos) = stack.iter().position(|p| p == &canonical) {
        let mut cycle = stack[pos..]
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>();
        cycle.push(canonical.display().to_string());
        return Err(format!("cyclic include: {}", cycle.join(" -> ")));
    }
    if !seen.insert(canonical.clone()) {
        return Ok(String::new());
    }

    stack.push(canonical.clone());
    let source =
        fs::read_to_string(&canonical).map_err(|e| format!("{}: {e}", canonical.display()))?;
    let mut output = String::new();
    for (line_no, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("include ") {
            let name = rest
                .strip_prefix('"')
                .and_then(|s| s.strip_suffix("\";"))
                .ok_or_else(|| {
                    format!("{}:{}: malformed include", canonical.display(), line_no + 1)
                })?;
            let local = canonical
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join(name);
            let include = if local.exists() {
                Some(local)
            } else {
                include_dirs
                    .iter()
                    .map(|d| d.join(name))
                    .find(|p| p.exists())
            }
            .ok_or_else(|| {
                format!(
                    "{}:{}: include not found: {name}",
                    canonical.display(),
                    line_no + 1
                )
            })?;
            output.push_str(&load_source(&include, include_dirs, stack, seen)?);
            if !output.ends_with('\n') {
                output.push('\n');
            }
            continue;
        }
        output.push_str(line);
        output.push('\n');
    }
    stack.pop();
    Ok(output)
}

fn usage() -> &'static str {
    "Usage:\n  svm-clite [options] source.cl [output]\n\nOptions:\n  -h, --help            Show this help\n  --target TARGET       Select target architecture (default: register)\n  --emit asm|ir         Emit target assembly or CLIR (default: asm)\n  --check               Parse and validate only\n  --assemble            Run external svm-asm after compilation\n  --assembler PATH      External assembler executable (default: svm-asm)\n  -I DIR                Add include search directory\n\nTargets:\n  register, stack, accumulator, memreg, loadstore, regmem,\n  memory2memory, belt, tta\n\nExamples:\n  svm-clite --target stack program.cl\n  svm-clite --emit ir program.cl program.clir\n  svm-clite --target tta --assemble program.cl program.svm"
}

fn run_assembler(
    assembler: &str,
    target: Target,
    assembly: &str,
    output: &Path,
    include_dirs: &[PathBuf],
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = env::temp_dir().join(format!(
        "svm-clite-{}-{}.asm",
        std::process::id(),
        target.as_str()
    ));
    fs::write(&tmp, assembly)?;
    let mut cmd = Command::new(assembler);
    for d in include_dirs {
        cmd.arg("-I").arg(d);
    }
    let status = cmd
        .arg(target.as_str())
        .arg(&tmp)
        .arg(output)
        .status()
        .map_err(|e| format!("cannot execute assembler '{assembler}': {e}"))?;
    let _ = fs::remove_file(&tmp);
    if !status.success() {
        return Err(format!("assembler failed with status {status}").into());
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let mut target = Target::Register;
    let mut emit = String::from("asm");
    let mut check_only = false;
    let mut assemble = false;
    let mut assembler = String::from("svm-asm");
    let mut include_dirs = Vec::new();
    let mut input = None;
    let mut output = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                println!("{}", usage());
                return Ok(());
            }
            "--target" => target = Target::parse(&args.next().ok_or("--target requires value")?)?,
            "--emit" => emit = args.next().ok_or("--emit requires asm|ir")?,
            "--check" => check_only = true,
            "--assemble" => assemble = true,
            "--assembler" => assembler = args.next().ok_or("--assembler requires path")?,
            "-I" => include_dirs.push(PathBuf::from(args.next().ok_or("-I requires dir")?)),
            _ if arg.starts_with("-I") => include_dirs.push(PathBuf::from(&arg[2..])),
            _ if input.is_none() => input = Some(PathBuf::from(arg)),
            _ if output.is_none() => output = Some(PathBuf::from(arg)),
            _ => return Err(usage().into()),
        }
    }

    if !matches!(emit.as_str(), "asm" | "ir") {
        return Err("--emit must be asm or ir".into());
    }
    if assemble && emit == "ir" {
        return Err("--assemble cannot be combined with --emit ir".into());
    }

    let input = input.ok_or("missing source")?;
    let source = load_source(&input, &include_dirs, &mut Vec::new(), &mut HashSet::new())?;

    if check_only {
        svm_clite::check(&source).map_err(|e| format!("C-Lite check failed: {e}"))?;
        println!("ok: {}", input.display());
        return Ok(());
    }

    if emit == "ir" {
        let text = svm_clite::compile_to_ir(&source)?;
        let output = output.unwrap_or_else(|| input.with_extension("clir"));
        fs::write(&output, text)?;
        println!("compiled {} -> {}", input.display(), output.display());
        return Ok(());
    }

    let assembly = svm_clite::compile_to_asm(&source, target)?;
    if assemble {
        let output = output.unwrap_or_else(|| input.with_extension("svm"));
        run_assembler(&assembler, target, &assembly, &output, &include_dirs)?;
        println!(
            "compiled+assembled {} -> {}",
            input.display(),
            output.display()
        );
    } else {
        let output = output.unwrap_or_else(|| input.with_extension("asm"));
        fs::write(&output, assembly)?;
        println!("compiled {} -> {}", input.display(), output.display());
    }
    Ok(())
}

#[cfg(test)]
mod include_tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(tag: &str) -> PathBuf {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let p = env::temp_dir().join(format!("svm-clite-{tag}-{}-{n}", std::process::id()));
        fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn help_lists_all_targets() {
        let text = usage();
        for target in [
            "register",
            "stack",
            "accumulator",
            "memreg",
            "loadstore",
            "regmem",
            "memory2memory",
            "belt",
            "tta",
        ] {
            assert!(text.contains(target), "help is missing target {target}");
        }
        assert!(text.contains("-h, --help"));
        assert!(text.contains("default: register"));
    }

    #[test]
    fn include_is_expanded_once() {
        let d = temp_dir("include-once");
        fs::write(d.join("lib.cl"), "fn helper()->u16{return 1;}\n").unwrap();
        fs::write(
            d.join("main.cl"),
            "include \"lib.cl\";\ninclude \"lib.cl\";\nfn main()->u16{return helper();}\n",
        )
        .unwrap();
        let src = load_source(
            &d.join("main.cl"),
            &[],
            &mut Vec::new(),
            &mut HashSet::new(),
        )
        .unwrap();
        assert_eq!(src.matches("fn helper").count(), 1);
        let _ = fs::remove_dir_all(d);
    }

    #[test]
    fn include_cycle_is_reported() {
        let d = temp_dir("include-cycle");
        fs::write(d.join("a.cl"), "include \"b.cl\";\n").unwrap();
        fs::write(d.join("b.cl"), "include \"a.cl\";\n").unwrap();
        let err =
            load_source(&d.join("a.cl"), &[], &mut Vec::new(), &mut HashSet::new()).unwrap_err();
        assert!(err.contains("cyclic include"));
        let _ = fs::remove_dir_all(d);
    }
    #[test]
    fn missing_include_reports_source_line() {
        let d = temp_dir("include-missing");
        fs::write(
            d.join("main.cl"),
            "include \"missing.cl\";\nfn main()->u16{return 0;}\n",
        )
        .unwrap();
        let err = load_source(
            &d.join("main.cl"),
            &[],
            &mut Vec::new(),
            &mut HashSet::new(),
        )
        .unwrap_err();
        assert!(err.contains("main.cl:1"));
        assert!(err.contains("include not found: missing.cl"));
        let _ = fs::remove_dir_all(d);
    }
}
