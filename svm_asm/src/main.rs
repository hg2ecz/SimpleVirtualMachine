use std::{env, path::PathBuf};

use svm_asm::procedure_gc::{ProcedureSyntax, eliminate_unused_procedures};
use svm_asm::source_include::{IncludeStyle, expand_source_file};
use svm_asm::source_preprocess::expand_equ;

fn usage() -> &'static str {
    "usage: svm-asm [-I dir|-Idir] <register|stack|accumulator|memreg|loadstore|regmem|memory2memory|belt|tta> input [output]"
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1).peekable();
    let mut include_dirs = Vec::new();
    let mut positional = Vec::new();

    while let Some(arg) = args.next() {
        if arg == "-I" {
            include_dirs.push(PathBuf::from(args.next().ok_or("-I requires a directory")?));
        } else if let Some(dir) = arg.strip_prefix("-I") {
            if dir.is_empty() {
                return Err("-I requires a directory".into());
            }
            include_dirs.push(PathBuf::from(dir));
        } else if arg.starts_with('-') {
            return Err(format!("unknown option: {arg}\n{}", usage()).into());
        } else {
            positional.push(arg);
        }
    }

    if !(2..=3).contains(&positional.len()) {
        return Err(usage().into());
    }

    let target = positional[0].clone();
    let canonical_target = match target.as_str() {
        "register" | "reg" => "register",
        "stack" => "stack",
        "accumulator" | "acc" => "accumulator",
        "memreg" | "file" | "pic" => "memreg",
        "loadstore" | "risc" | "ls" => "loadstore",
        "regmem" | "register-memory" | "rm" => "regmem",
        "memory2memory" | "memory-to-memory" | "m2m" | "cisc" => "memory2memory",
        "belt" | "belt16" => "belt",
        "tta" | "tta16" | "transport" => "tta",
        _ => return Err(format!("unknown target '{target}'").into()),
    };
    let builtin_lib = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("lib")
        .join(canonical_target);
    if builtin_lib.is_dir() {
        include_dirs.push(builtin_lib);
    }
    let input = PathBuf::from(&positional[1]);
    let output = positional.get(2).map(PathBuf::from);
    let source = expand_source_file(&input, &include_dirs, IncludeStyle::Assembly)?;
    let source = expand_equ(&source)?;
    let proc_syntax = if canonical_target == "stack" {
        ProcedureSyntax::StackLabels
    } else {
        ProcedureSyntax::Labels
    };
    let source = eliminate_unused_procedures(&source, proc_syntax)?;

    match target.as_str() {
        "register" | "reg" => {
            let p = svm_asm::register::assembler::assemble(&source)?;
            let o = output.unwrap_or_else(|| input.with_extension("svm"));
            p.write_file(&o)?;
            println!(
                "assembled {} -> {} ({} code bytes)",
                input.display(),
                o.display(),
                p.payload.len()
            );
        }
        "stack" => {
            let p = svm_asm::stack::assembler::assemble(&source)?;
            let o = output.unwrap_or_else(|| input.with_extension("svs"));
            std::fs::write(&o, p.to_bytes()?)?;
            println!(
                "assembled {} -> {} ({} code bytes)",
                input.display(),
                o.display(),
                p.payload.len()
            );
        }
        "accumulator" | "acc" => {
            let p = svm_asm::accumulator::assembler::assemble(&source)?;
            let o = output.unwrap_or_else(|| input.with_extension("sva"));
            p.write_file(&o)?;
            println!(
                "assembled {} -> {} ({} code bytes)",
                input.display(),
                o.display(),
                p.payload.len()
            );
        }
        "memreg" | "file" | "pic" => {
            let p = svm_asm::memreg::assembler::assemble(&source)?;
            let o = output.unwrap_or_else(|| input.with_extension("svf"));
            p.write_file(&o)?;
            println!(
                "assembled {} -> {} ({} code bytes)",
                input.display(),
                o.display(),
                p.payload.len()
            );
        }

        "loadstore" | "risc" | "ls" => {
            let p = svm_asm::loadstore::assembler::assemble(&source)?;
            let o = output.unwrap_or_else(|| input.with_extension("svl"));
            p.write_file(&o)?;
            println!(
                "assembled {} -> {} ({} code bytes)",
                input.display(),
                o.display(),
                p.payload.len()
            );
        }
        "regmem" | "register-memory" | "rm" => {
            let p = svm_asm::regmem::assembler::assemble(&source)?;
            let o = output.unwrap_or_else(|| input.with_extension("svr"));
            p.write_file(&o)?;
            println!(
                "assembled {} -> {} ({} code bytes)",
                input.display(),
                o.display(),
                p.payload.len()
            );
        }
        "belt" | "belt16" => {
            let p = svm_asm::belt::assembler::assemble(&source)?;
            let o = output.unwrap_or_else(|| input.with_extension("svb"));
            p.write_file(&o)?;
            println!(
                "assembled {} -> {} ({} code bytes)",
                input.display(),
                o.display(),
                p.payload.len()
            );
        }
        "tta" | "tta16" | "transport" => {
            let p = svm_asm::tta::assembler::assemble(&source)?;
            let o = output.unwrap_or_else(|| input.with_extension("svt"));
            p.write_file(&o)?;
            println!(
                "assembled {} -> {} ({} code bytes)",
                input.display(),
                o.display(),
                p.payload.len()
            );
        }
        "memory2memory" | "memory-to-memory" | "m2m" | "cisc" => {
            let p = svm_asm::memory2memory::assembler::assemble(&source)?;
            let o = output.unwrap_or_else(|| input.with_extension("svc"));
            p.write_file(&o)?;
            println!(
                "assembled {} -> {} ({} code bytes)",
                input.display(),
                o.display(),
                p.payload.len()
            );
        }
        _ => return Err(format!("unknown target '{target}'").into()),
    }
    Ok(())
}
