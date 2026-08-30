use std::{env, fs, path::PathBuf};

use svm_asm::source_include::{IncludeStyle, expand_source_file};

use svm_c::common::model::Target;
use svm_c::unopt::pipeline::compile_source_unoptimized;

fn ensure_c_program_does_not_overlap_upper_data(
    load: u16,
    code_len: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    const UPPER_DATA_BASE: usize = 0xE000;
    let end = (load as usize)
        .checked_add(code_len)
        .ok_or("C program size overflow")?;
    if code_len != 0 && end > UPPER_DATA_BASE {
        return Err(format!(
            "C program image reaches 0xE000 reserved by the static-data convention; code end is 0x{end:04X}"
        ).into());
    }
    Ok(())
}

fn usage() -> &'static str {
    "usage: svm-c-unopt-only --target register|stack|accumulator|memreg|loadstore|regmem|memory2memory|belt|tta [-I dir|-Idir] [--emit asm|bin] source.sc [output]"
}

fn parse_target(s: &str) -> Result<Target, Box<dyn std::error::Error>> {
    Ok(match s {
        "register" | "reg" => Target::Register,
        "stack" => Target::Stack,
        "accumulator" | "acc" => Target::Accumulator,
        "memreg" | "file" | "wfile" | "pic" => Target::MemReg,
        "loadstore" | "risc" | "ls" => Target::LoadStore,
        "regmem" | "register-memory" | "rm" => Target::RegMem,
        "memory2memory" | "memory-to-memory" | "m2m" | "cisc" => Target::Memory2Memory,
        "belt" | "belt16" => Target::Belt,
        "tta" | "tta16" | "transport" => Target::Tta,
        _ => return Err("--target must be one of register, stack, accumulator, memreg, loadstore, regmem, memory2memory, belt, tta".into()),
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let mut target = None;
    let mut emit = String::from("bin");
    let mut input = None;
    let mut output = None;
    let mut include_dirs = Vec::new();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--target" => target = Some(parse_target(&args.next().ok_or(usage())?)?),
            "-I" => include_dirs.push(PathBuf::from(args.next().ok_or("-I requires a directory")?)),
            _ if arg.starts_with("-I") => {
                let dir = &arg[2..];
                if dir.is_empty() {
                    return Err("-I requires a directory".into());
                }
                include_dirs.push(PathBuf::from(dir));
            }
            "--emit" => {
                emit = args.next().ok_or(usage())?;
                if emit != "asm" && emit != "bin" {
                    return Err("--emit must be asm or bin".into());
                }
            }
            _ if arg.starts_with("-O") => {
                return Err(
                    "svm-c-unopt-only has no optimization levels; use svm-c for -O0/-O1/-O2/-Os"
                        .into(),
                );
            }
            _ if arg.starts_with('-') => return Err(format!("unknown option: {arg}").into()),
            _ if input.is_none() => input = Some(PathBuf::from(arg)),
            _ if output.is_none() => output = Some(PathBuf::from(arg)),
            _ => return Err(usage().into()),
        }
    }

    let target = target.ok_or(usage())?;
    let input = input.ok_or(usage())?;
    let source = expand_source_file(&input, &include_dirs, IncludeStyle::C)?;
    let assembly =
        compile_source_unoptimized(&source, target).map_err(|e| format!("compile error: {e}"))?;

    if emit == "asm" {
        let ext = match target {
            Target::Register => "asm",
            Target::Stack => "fsasm",
            Target::Accumulator => "aasm",
            Target::MemReg => "masm",
            Target::LoadStore => "lsasm",
            Target::RegMem => "rmasm",
            Target::Memory2Memory => "ciscasm",
            Target::Belt => "beltasm",
            Target::Tta => "ttaasm",
        };
        let out = output.unwrap_or_else(|| input.with_extension(ext));
        fs::write(&out, assembly)?;
        println!(
            "compiled {} -> {} (unoptimized)",
            input.display(),
            out.display()
        );
        return Ok(());
    }

    match target {
        Target::Register => {
            let program = svm_asm::register::assembler::assemble(&assembly)?;
            ensure_c_program_does_not_overlap_upper_data(
                program.load_address,
                program.payload.len(),
            )?;
            let out = output.unwrap_or_else(|| input.with_extension("svm"));
            program.write_file(&out)?;
            println!(
                "compiled {} -> {} ({} code bytes, unoptimized)",
                input.display(),
                out.display(),
                program.payload.len()
            );
        }
        Target::Stack => {
            let program = svm_asm::stack::assembler::assemble(&assembly)?;
            ensure_c_program_does_not_overlap_upper_data(
                program.load_address,
                program.payload.len(),
            )?;
            let out = output.unwrap_or_else(|| input.with_extension("svs"));
            fs::write(&out, program.to_bytes()?)?;
            println!(
                "compiled {} -> {} ({} code bytes, unoptimized)",
                input.display(),
                out.display(),
                program.payload.len()
            );
        }
        Target::Accumulator => {
            let program = svm_asm::accumulator::assembler::assemble(&assembly)?;
            ensure_c_program_does_not_overlap_upper_data(
                program.load_address,
                program.payload.len(),
            )?;
            let out = output.unwrap_or_else(|| input.with_extension("sva"));
            program.write_file(&out)?;
            println!(
                "compiled {} -> {} ({} code bytes, unoptimized)",
                input.display(),
                out.display(),
                program.payload.len()
            );
        }
        Target::MemReg => {
            let program = svm_asm::memreg::assembler::assemble(&assembly)?;
            ensure_c_program_does_not_overlap_upper_data(
                program.load_address,
                program.payload.len(),
            )?;
            let out = output.unwrap_or_else(|| input.with_extension("svf"));
            program.write_file(&out)?;
            println!(
                "compiled {} -> {} ({} code bytes, unoptimized)",
                input.display(),
                out.display(),
                program.payload.len()
            );
        }
        Target::LoadStore => {
            let program = svm_asm::loadstore::assembler::assemble(&assembly)?;
            ensure_c_program_does_not_overlap_upper_data(
                program.load_address,
                program.payload.len(),
            )?;
            let out = output.unwrap_or_else(|| input.with_extension("svl"));
            program.write_file(&out)?;
            println!(
                "compiled {} -> {} ({} code bytes, unoptimized)",
                input.display(),
                out.display(),
                program.payload.len()
            );
        }
        Target::RegMem => {
            let program = svm_asm::regmem::assembler::assemble(&assembly)?;
            ensure_c_program_does_not_overlap_upper_data(
                program.load_address,
                program.payload.len(),
            )?;
            let out = output.unwrap_or_else(|| input.with_extension("svr"));
            program.write_file(&out)?;
            println!(
                "compiled {} -> {} ({} code bytes, unoptimized)",
                input.display(),
                out.display(),
                program.payload.len()
            );
        }
        Target::Belt => {
            let program = svm_asm::belt::assembler::assemble(&assembly)?;
            ensure_c_program_does_not_overlap_upper_data(
                program.load_address,
                program.payload.len(),
            )?;
            let out = output.unwrap_or_else(|| input.with_extension("svb"));
            program.write_file(&out)?;
            println!(
                "compiled {} -> {} ({} code bytes, unoptimized)",
                input.display(),
                out.display(),
                program.payload.len()
            );
        }
        Target::Tta => {
            let program = svm_asm::tta::assembler::assemble(&assembly)?;
            ensure_c_program_does_not_overlap_upper_data(
                program.load_address,
                program.payload.len(),
            )?;
            let out = output.unwrap_or_else(|| input.with_extension("svt"));
            program.write_file(&out)?;
            println!(
                "compiled {} -> {} ({} code bytes, unoptimized)",
                input.display(),
                out.display(),
                program.payload.len()
            );
        }
        Target::Memory2Memory => {
            let program = svm_asm::memory2memory::assembler::assemble(&assembly)?;
            ensure_c_program_does_not_overlap_upper_data(
                program.load_address,
                program.payload.len(),
            )?;
            let out = output.unwrap_or_else(|| input.with_extension("svc"));
            program.write_file(&out)?;
            println!(
                "compiled {} -> {} ({} code bytes, unoptimized)",
                input.display(),
                out.display(),
                program.payload.len()
            );
        }
    }
    Ok(())
}
