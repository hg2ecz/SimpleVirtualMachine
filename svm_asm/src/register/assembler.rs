use std::collections::HashMap;

use super::{
    error::AsmError,
    instruction::{
        COMPACT_REGISTER_COUNT, Encoding, OperandForm, REGISTER_COUNT, encode_compact_pair,
        encode_register_pair, instruction_spec,
    },
    program::Program,
};

#[derive(Debug, Clone)]
enum Value {
    Number(u32),
    Symbol(String),
}

#[derive(Debug, Clone)]
enum Operand {
    Register(u8),
    MemoryRegister(u8),
    MemoryRegisterPostInc(u8),
    MemoryRegisterPreDec(u8),
    Value(Value),
}

#[derive(Debug, Clone)]
struct Instruction {
    mnemonic: String,
    operands: Vec<Operand>,
    line: usize,
}

#[derive(Debug, Clone)]
enum Statement {
    Instruction(Instruction),
    Load(Value, usize),
    Entry(Value, usize),
}

pub fn assemble(source: &str) -> Result<Program, AsmError> {
    let (statements, label_offsets) = parse_source(source)?;

    let load_address = resolve_load_address(&statements)?;
    let labels = label_offsets
        .into_iter()
        .map(|(name, offset)| {
            let address = (load_address as usize)
                .checked_add(offset)
                .filter(|value| *value <= u16::MAX as usize)
                .ok_or_else(|| {
                    asm_error(
                        0,
                        format!("label '{name}' is outside the 16-bit address space"),
                    )
                })?;
            Ok((name, address as u32))
        })
        .collect::<Result<HashMap<_, _>, AsmError>>()?;

    let entry_address = resolve_entry_address(&statements, &labels, load_address)?;
    let mut payload = Vec::new();

    for statement in &statements {
        if let Statement::Instruction(instruction) = statement {
            encode_instruction(instruction, &labels, &mut payload)?;
        }
    }

    let end = (load_address as usize)
        .checked_add(payload.len())
        .ok_or_else(|| asm_error(0, "program address range overflow"))?;
    if end > 65_536 {
        return Err(asm_error(
            0,
            "assembled program does not fit into 64 KiB memory",
        ));
    }

    Ok(Program {
        load_address,
        entry_address,
        payload,
    })
}

fn parse_source(source: &str) -> Result<(Vec<Statement>, HashMap<String, usize>), AsmError> {
    let mut statements = Vec::new();
    let mut labels = HashMap::new();
    let mut offset = 0usize;

    for (index, raw_line) in source.lines().enumerate() {
        let line_number = index + 1;
        let without_comment = raw_line
            .split_once(';')
            .map_or(raw_line, |(text, _)| text)
            .trim();
        if without_comment.is_empty() {
            continue;
        }

        let mut rest = without_comment;
        if let Some((label, remainder)) = split_label(rest) {
            validate_symbol(label, line_number)?;
            if labels.insert(label.to_ascii_lowercase(), offset).is_some() {
                return Err(asm_error(line_number, format!("duplicate label '{label}'")));
            }
            rest = remainder.trim();
            if rest.is_empty() {
                continue;
            }
        }

        if rest.starts_with('.') {
            let statement = parse_directive(rest, line_number)?;
            statements.push(statement);
            continue;
        }

        let instruction = parse_instruction(rest, line_number)?;
        offset = offset
            .checked_add(instruction_len(&instruction)?)
            .ok_or_else(|| asm_error(line_number, "program size overflow"))?;
        statements.push(Statement::Instruction(instruction));
    }

    Ok((statements, labels))
}

fn split_label(line: &str) -> Option<(&str, &str)> {
    let colon = line.find(':')?;
    let candidate = line[..colon].trim();
    if candidate.chars().any(char::is_whitespace) {
        return None;
    }
    Some((candidate, &line[colon + 1..]))
}

fn parse_directive(text: &str, line: usize) -> Result<Statement, AsmError> {
    let mut parts = text.split_whitespace();
    let directive = parts.next().unwrap_or_default().to_ascii_lowercase();
    let value_text = parts
        .next()
        .ok_or_else(|| asm_error(line, "directive requires one value"))?;
    if parts.next().is_some() {
        return Err(asm_error(line, "directive accepts exactly one value"));
    }
    let value = parse_value(value_text, line)?;
    match directive.as_str() {
        ".load" => Ok(Statement::Load(value, line)),
        ".entry" => Ok(Statement::Entry(value, line)),
        _ => Err(asm_error(line, format!("unknown directive '{directive}'"))),
    }
}

fn parse_instruction(text: &str, line: usize) -> Result<Instruction, AsmError> {
    let mut parts = text.splitn(2, |ch: char| ch.is_whitespace());
    let mnemonic = parts.next().unwrap_or_default().to_ascii_uppercase();
    if mnemonic.is_empty() {
        return Err(asm_error(line, "missing instruction mnemonic"));
    }
    let operands_text = parts.next().unwrap_or_default().trim();
    let operands = if operands_text.is_empty() {
        Vec::new()
    } else {
        operands_text
            .split(',')
            .map(|operand| parse_operand(operand.trim(), line))
            .collect::<Result<Vec<_>, _>>()?
    };

    // Natural addressing syntax selects the dedicated post-increment opcode.
    // LOAD8 R0,[R1+] etc. remain ordinary CPU instructions; memory itself
    // has no special virtual-address semantics.
    let mnemonic = match mnemonic.as_str() {
        "LOAD8" if matches!(operands.get(1), Some(Operand::MemoryRegisterPostInc(_))) => {
            "LOAD8P".to_string()
        }
        "LOAD16" if matches!(operands.get(1), Some(Operand::MemoryRegisterPostInc(_))) => {
            "LOAD16P".to_string()
        }
        "STORE8" if matches!(operands.get(0), Some(Operand::MemoryRegisterPostInc(_))) => {
            "STORE8P".to_string()
        }
        "STORE16" if matches!(operands.get(0), Some(Operand::MemoryRegisterPostInc(_))) => {
            "STORE16P".to_string()
        }
        "LOAD8" if matches!(operands.get(1), Some(Operand::MemoryRegisterPreDec(_))) => {
            "LOAD8M".to_string()
        }
        "LOAD16" if matches!(operands.get(1), Some(Operand::MemoryRegisterPreDec(_))) => {
            "LOAD16M".to_string()
        }
        "STORE8" if matches!(operands.get(0), Some(Operand::MemoryRegisterPreDec(_))) => {
            "STORE8M".to_string()
        }
        "STORE16" if matches!(operands.get(0), Some(Operand::MemoryRegisterPreDec(_))) => {
            "STORE16M".to_string()
        }
        // Video-space operations accept the same natural [Rn+]/[-Rn]
        // syntax as normal memory operations. Select the dedicated video
        // post-increment/pre-decrement encoding before instruction matching.
        "VLOAD8" if matches!(operands.get(1), Some(Operand::MemoryRegisterPostInc(_))) => {
            "VLOAD8P".to_string()
        }
        "VLOAD16" if matches!(operands.get(1), Some(Operand::MemoryRegisterPostInc(_))) => {
            "VLOAD16P".to_string()
        }
        "VSTORE8" if matches!(operands.get(0), Some(Operand::MemoryRegisterPostInc(_))) => {
            "VSTORE8P".to_string()
        }
        "VSTORE16" if matches!(operands.get(0), Some(Operand::MemoryRegisterPostInc(_))) => {
            "VSTORE16P".to_string()
        }
        "VLOAD8" if matches!(operands.get(1), Some(Operand::MemoryRegisterPreDec(_))) => {
            "VLOAD8M".to_string()
        }
        "VLOAD16" if matches!(operands.get(1), Some(Operand::MemoryRegisterPreDec(_))) => {
            "VLOAD16M".to_string()
        }
        "VSTORE8" if matches!(operands.get(0), Some(Operand::MemoryRegisterPreDec(_))) => {
            "VSTORE8M".to_string()
        }
        "VSTORE16" if matches!(operands.get(0), Some(Operand::MemoryRegisterPreDec(_))) => {
            "VSTORE16M".to_string()
        }
        _ => mnemonic,
    };

    // Zero-cost assembler conveniences and exact peephole shortening.
    // These preserve architectural state/flags while using existing shorter opcodes.
    // ADDI Rn,1 -> INC Rn; SUBI Rn,1 -> DEC Rn; MOV Rn,Rn -> NOP.
    if matches!(mnemonic.as_str(), "ADDI" | "SUBI") && operands.len() == 2 {
        if let (Operand::Register(register), Operand::Value(Value::Number(1))) =
            (&operands[0], &operands[1])
        {
            return Ok(Instruction {
                mnemonic: if mnemonic == "ADDI" { "INC" } else { "DEC" }.to_string(),
                operands: vec![Operand::Register(*register)],
                line,
            });
        }
    }
    if mnemonic == "MOV" && operands.len() == 2 {
        if let (Operand::Register(a), Operand::Register(b)) = (&operands[0], &operands[1]) {
            if a == b {
                return Ok(Instruction {
                    mnemonic: "NOP".to_string(),
                    operands: vec![],
                    line,
                });
            }
        }
    }

    // CLR Rn  -> XOR Rn,Rn
    // TEST Rn -> OR  Rn,Rn   (updates Z/N, leaves C unchanged)
    if matches!(mnemonic.as_str(), "CLR" | "TEST") {
        if operands.len() != 1 {
            return Err(asm_error(
                line,
                format!("{mnemonic} expects exactly one register operand"),
            ));
        }
        let Operand::Register(register) = &operands[0] else {
            return Err(asm_error(
                line,
                format!("{mnemonic} expects a register operand"),
            ));
        };
        let operation = if mnemonic == "CLR" { "XOR" } else { "OR" };
        return Ok(Instruction {
            mnemonic: operation.to_string(),
            operands: vec![Operand::Register(*register), Operand::Register(*register)],
            line,
        });
    }

    Ok(Instruction {
        mnemonic,
        operands,
        line,
    })
}

fn parse_operand(text: &str, line: usize) -> Result<Operand, AsmError> {
    if text.is_empty() {
        return Err(asm_error(line, "empty operand"));
    }
    if text.starts_with('[') || text.ends_with(']') {
        if !(text.starts_with('[') && text.ends_with(']')) {
            return Err(asm_error(
                line,
                format!("malformed memory operand '{text}'"),
            ));
        }
        let inner = text[1..text.len() - 1].trim();
        if let Some(register) = inner.strip_prefix('-') {
            return Ok(Operand::MemoryRegisterPreDec(parse_register(
                register.trim(),
                line,
            )?));
        }
        if let Some(register) = inner.strip_suffix('+') {
            return Ok(Operand::MemoryRegisterPostInc(parse_register(
                register.trim(),
                line,
            )?));
        }
        return Ok(Operand::MemoryRegister(parse_register(inner, line)?));
    }
    if looks_like_register(text) {
        return Ok(Operand::Register(parse_register(text, line)?));
    }
    Ok(Operand::Value(parse_value(text, line)?))
}

fn looks_like_register(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() >= 2 && matches!(bytes[0], b'R' | b'r') && bytes[1..].iter().all(u8::is_ascii_digit)
}

fn parse_register(text: &str, line: usize) -> Result<u8, AsmError> {
    let upper = text.to_ascii_uppercase();
    let Some(number) = upper.strip_prefix('R') else {
        return Err(asm_error(
            line,
            format!("expected register, found '{text}'"),
        ));
    };
    let register = number
        .parse::<u8>()
        .map_err(|_| asm_error(line, format!("invalid register '{text}'")))?;
    if register >= REGISTER_COUNT {
        return Err(asm_error(
            line,
            format!("register '{text}' is outside R0..R{}", REGISTER_COUNT - 1),
        ));
    }
    Ok(register)
}

fn parse_value(text: &str, line: usize) -> Result<Value, AsmError> {
    let clean = text.replace('_', "");
    let number = if let Some(hex) = clean
        .strip_prefix("0x")
        .or_else(|| clean.strip_prefix("0X"))
    {
        u32::from_str_radix(hex, 16).ok()
    } else {
        clean.parse::<u32>().ok()
    };
    if let Some(number) = number {
        return Ok(Value::Number(number));
    }
    validate_symbol(text, line)?;
    Ok(Value::Symbol(text.to_ascii_lowercase()))
}

fn validate_symbol(symbol: &str, line: usize) -> Result<(), AsmError> {
    let mut chars = symbol.chars();
    let Some(first) = chars.next() else {
        return Err(asm_error(line, "empty symbol"));
    };
    if !(first.is_ascii_alphabetic() || first == '_')
        || !chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    {
        return Err(asm_error(line, format!("invalid symbol '{symbol}'")));
    }
    Ok(())
}

fn instruction_len(instruction: &Instruction) -> Result<usize, AsmError> {
    let spec = instruction_spec(&instruction.mnemonic).ok_or_else(|| {
        asm_error(
            instruction.line,
            format!("unknown instruction '{}'", instruction.mnemonic),
        )
    })?;

    match spec.encoding {
        Encoding::Fixed(_) | Encoding::EmbeddedRegister { .. } => Ok(1),
        Encoding::EmbeddedRegisterImmediate16 { .. }
        | Encoding::Address16 { .. }
        | Encoding::VideoRegisterPair { .. } => Ok(3),
        Encoding::IntegerExtensionPair { .. } | Encoding::IntegerExtensionRegister { .. } => Ok(3),
        Encoding::ZeroPage8 { .. }
        | Encoding::ExtendedRegister { .. }
        | Encoding::ExtendedRegisterPair { .. } => Ok(2),
        Encoding::CompactOrGeneral { compact_base, .. } => {
            let (first, second) = register_pair_for_spec(instruction, spec.form)?;
            Ok(
                if compact_base.is_some()
                    && first < COMPACT_REGISTER_COUNT
                    && second < COMPACT_REGISTER_COUNT
                {
                    1
                } else {
                    2
                },
            )
        }
    }
}

fn register_pair_for_spec(
    instruction: &Instruction,
    form: OperandForm,
) -> Result<(u8, u8), AsmError> {
    expect_operands(instruction, 2)?;
    match form {
        OperandForm::RegisterPair => Ok((
            expect_register(&instruction.operands[0], instruction.line)?,
            expect_register(&instruction.operands[1], instruction.line)?,
        )),
        OperandForm::RegisterMemory => Ok((
            expect_register(&instruction.operands[0], instruction.line)?,
            expect_memory_register(&instruction.operands[1], instruction.line)?,
        )),
        OperandForm::MemoryRegister => Ok((
            expect_memory_register(&instruction.operands[0], instruction.line)?,
            expect_register(&instruction.operands[1], instruction.line)?,
        )),
        OperandForm::RegisterMemoryPostInc => {
            let destination = expect_register(&instruction.operands[0], instruction.line)?;
            let address =
                expect_memory_register_postinc(&instruction.operands[1], instruction.line)?;
            if destination == address {
                return Err(asm_error(
                    instruction.line,
                    "post-increment load requires distinct data and address registers",
                ));
            }
            Ok((destination, address))
        }
        OperandForm::MemoryPostIncRegister => Ok((
            expect_memory_register_postinc(&instruction.operands[0], instruction.line)?,
            expect_register(&instruction.operands[1], instruction.line)?,
        )),
        OperandForm::RegisterMemoryPreDec => {
            let destination = expect_register(&instruction.operands[0], instruction.line)?;
            let address =
                expect_memory_register_predec(&instruction.operands[1], instruction.line)?;
            if destination == address {
                return Err(asm_error(
                    instruction.line,
                    "pre-decrement load requires distinct data and address registers",
                ));
            }
            Ok((destination, address))
        }
        OperandForm::MemoryPreDecRegister => Ok((
            expect_memory_register_predec(&instruction.operands[0], instruction.line)?,
            expect_register(&instruction.operands[1], instruction.line)?,
        )),
        _ => Err(asm_error(
            instruction.line,
            "internal assembler operand-form mismatch",
        )),
    }
}

fn resolve_load_address(statements: &[Statement]) -> Result<u16, AsmError> {
    let mut result = None;
    for statement in statements {
        if let Statement::Load(value, line) = statement {
            if result.is_some() {
                return Err(asm_error(*line, ".load may only be specified once"));
            }
            let Value::Number(number) = value else {
                return Err(asm_error(*line, ".load requires a numeric address"));
            };
            result = Some(
                u16::try_from(*number)
                    .map_err(|_| asm_error(*line, ".load address exceeds 0xFFFF"))?,
            );
        }
    }
    Ok(result.unwrap_or(0))
}

fn resolve_entry_address(
    statements: &[Statement],
    labels: &HashMap<String, u32>,
    load_address: u16,
) -> Result<u16, AsmError> {
    let mut result = None;
    for statement in statements {
        if let Statement::Entry(value, line) = statement {
            if result.is_some() {
                return Err(asm_error(*line, ".entry may only be specified once"));
            }
            let number = resolve_value(value, labels, *line)?;
            result = Some(
                u16::try_from(number)
                    .map_err(|_| asm_error(*line, ".entry address exceeds 0xFFFF"))?,
            );
        }
    }
    Ok(result.unwrap_or(load_address))
}

fn encode_instruction(
    instruction: &Instruction,
    labels: &HashMap<String, u32>,
    output: &mut Vec<u8>,
) -> Result<(), AsmError> {
    let spec = instruction_spec(&instruction.mnemonic).ok_or_else(|| {
        asm_error(
            instruction.line,
            format!("unknown instruction '{}'", instruction.mnemonic),
        )
    })?;

    match spec.encoding {
        Encoding::Fixed(opcode) => {
            expect_operands(instruction, 0)?;
            output.push(opcode);
        }
        Encoding::EmbeddedRegister { base } => {
            expect_operands(instruction, 1)?;
            let register = expect_register(&instruction.operands[0], instruction.line)?;
            output.push(base | register);
        }
        Encoding::EmbeddedRegisterImmediate16 { base } => {
            expect_operands(instruction, 2)?;
            let register = expect_register(&instruction.operands[0], instruction.line)?;
            let value = expect_value(&instruction.operands[1], labels, instruction.line)?;
            let immediate = u16::try_from(value).map_err(|_| {
                asm_error(
                    instruction.line,
                    format!("immediate value 0x{value:X} exceeds 0xFFFF"),
                )
            })?;
            output.push(base | register);
            output.extend_from_slice(&immediate.to_le_bytes());
        }
        Encoding::ZeroPage8 { opcode } => {
            expect_operands(instruction, 1)?;
            let value = expect_value(&instruction.operands[0], labels, instruction.line)?;
            let address = u8::try_from(value).map_err(|_| {
                asm_error(
                    instruction.line,
                    format!("zero-page address 0x{value:X} exceeds 0xFF"),
                )
            })?;
            output.push(opcode);
            output.push(address);
        }
        Encoding::ExtendedRegister { opcode } => {
            expect_operands(instruction, 1)?;
            let register = expect_register(&instruction.operands[0], instruction.line)?;
            output.extend_from_slice(&[opcode, register]);
        }
        Encoding::ExtendedRegisterPair { opcode } => {
            let (first, second) = register_pair_for_spec(instruction, spec.form)?;
            let pair = encode_register_pair(first, second)
                .ok_or_else(|| asm_error(instruction.line, "invalid register pair"))?;
            output.extend_from_slice(&[opcode, pair]);
        }
        Encoding::VideoRegisterPair { subcode } => {
            let (first, second) = register_pair_for_spec(instruction, spec.form)?;
            let pair = encode_register_pair(first, second)
                .ok_or_else(|| asm_error(instruction.line, "invalid register pair"))?;
            output.extend_from_slice(&[0x0C, subcode, pair]);
        }
        Encoding::IntegerExtensionPair { subcode } => {
            let (first, second) = register_pair_for_spec(instruction, spec.form)?;
            let pair = encode_register_pair(first, second)
                .ok_or_else(|| asm_error(instruction.line, "invalid register pair"))?;
            output.extend_from_slice(&[0x0D, subcode, pair]);
        }
        Encoding::IntegerExtensionRegister { subcode } => {
            expect_operands(instruction, 1)?;
            let register = expect_register(&instruction.operands[0], instruction.line)?;
            output.extend_from_slice(&[0x0D, subcode, register]);
        }
        Encoding::Address16 { opcode } => {
            expect_operands(instruction, 1)?;
            let value = expect_value(&instruction.operands[0], labels, instruction.line)?;
            let address = u16::try_from(value).map_err(|_| {
                asm_error(
                    instruction.line,
                    format!("address 0x{value:X} exceeds 0xFFFF"),
                )
            })?;
            output.push(opcode);
            output.extend_from_slice(&address.to_le_bytes());
        }
        Encoding::CompactOrGeneral {
            compact_base,
            general,
        } => {
            let (first, second) = register_pair_for_spec(instruction, spec.form)?;
            if let Some(base) = compact_base {
                if let Some(opcode) = encode_compact_pair(base, first, second) {
                    output.push(opcode);
                    return Ok(());
                }
            }
            output.push(general);
            output.push(
                encode_register_pair(first, second)
                    .ok_or_else(|| asm_error(instruction.line, "register encoding overflow"))?,
            );
        }
    }
    Ok(())
}

fn expect_operands(instruction: &Instruction, expected: usize) -> Result<(), AsmError> {
    if instruction.operands.len() != expected {
        return Err(asm_error(
            instruction.line,
            format!(
                "{} expects {expected} operand(s), got {}",
                instruction.mnemonic,
                instruction.operands.len()
            ),
        ));
    }
    Ok(())
}

fn expect_register(operand: &Operand, line: usize) -> Result<u8, AsmError> {
    match operand {
        Operand::Register(register) => Ok(*register),
        _ => Err(asm_error(line, "expected register operand")),
    }
}

fn expect_memory_register(operand: &Operand, line: usize) -> Result<u8, AsmError> {
    match operand {
        Operand::MemoryRegister(register) => Ok(*register),
        _ => Err(asm_error(line, "expected memory operand such as [R0]")),
    }
}

fn expect_memory_register_postinc(operand: &Operand, line: usize) -> Result<u8, AsmError> {
    match operand {
        Operand::MemoryRegisterPostInc(register) => Ok(*register),
        _ => Err(asm_error(
            line,
            "expected post-increment memory operand such as [R0+]",
        )),
    }
}

fn expect_memory_register_predec(operand: &Operand, line: usize) -> Result<u8, AsmError> {
    match operand {
        Operand::MemoryRegisterPreDec(register) => Ok(*register),
        _ => Err(asm_error(
            line,
            "expected pre-decrement memory operand such as [-R0]",
        )),
    }
}

fn expect_value(
    operand: &Operand,
    labels: &HashMap<String, u32>,
    line: usize,
) -> Result<u32, AsmError> {
    match operand {
        Operand::Value(value) => resolve_value(value, labels, line),
        _ => Err(asm_error(line, "expected numeric value or label")),
    }
}

fn resolve_value(
    value: &Value,
    labels: &HashMap<String, u32>,
    line: usize,
) -> Result<u32, AsmError> {
    match value {
        Value::Number(number) => Ok(*number),
        Value::Symbol(symbol) => labels
            .get(symbol)
            .copied()
            .ok_or_else(|| asm_error(line, format!("unknown label '{symbol}'"))),
    }
}

fn asm_error(line: usize, message: impl Into<String>) -> AsmError {
    let message = message.into();
    if line == 0 {
        AsmError::Assembler(message)
    } else {
        AsmError::Assembler(format!("line {line}: {message}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_page_r0_forms_are_two_bytes() {
        assert_eq!(assemble("ZLOAD16 0x20").unwrap().payload, vec![0x04, 0x20]);
        assert_eq!(assemble("ZSTORE8 0x7F").unwrap().payload, vec![0x05, 0x7F]);
        assert!(assemble("ZLOAD8 0x100").is_err());
    }

    #[test]
    fn exact_peepholes_choose_shorter_existing_instructions() {
        let program = assemble("ADDI R2, 1\nSUBI R3, 1\nMOV R4, R4\nHALT").unwrap();
        assert_eq!(
            program.payload,
            vec![
                0x22, // INC R2: embedded register
                0x2B, // DEC R3: embedded register
                0x00, // NOP
                0x01, // HALT
            ]
        );
    }

    #[test]
    fn register_v3_uses_compact_and_but_keeps_subi_for_carry_semantics() {
        assert_eq!(
            assemble("SUBI R0, 2").unwrap().payload,
            vec![0xD0, 0x02, 0x00]
        );
        assert_eq!(assemble("AND R3, R2").unwrap().payload, vec![0xBE]);
        assert_eq!(assemble("XOR R3, R2").unwrap().payload, vec![0xE8, 0x1A]);
    }

    #[test]
    fn assembles_labels_and_embedded_immediates() {
        let source = r#"
            .load 0x0100
            .entry start
        start:
            MOVI R0, 0x8000
            MOVI R1, 224
        loop:
            STORE8 [R0], R1
            ADDI R0, 1
            CMPI R0, 0xC000
            JNZ loop
            HALT
        "#;
        let program = assemble(source).unwrap();
        assert_eq!(program.load_address, 0x0100);
        assert_eq!(program.entry_address, 0x0100);
        assert_eq!(&program.payload[..3], &[0xC0, 0x00, 0x80]);
        assert_eq!(*program.payload.last().unwrap(), 0x01);
    }

    #[test]
    fn chooses_compact_form_for_r0_to_r3() {
        let program = assemble("ADD R3, R2").unwrap();
        assert_eq!(program.payload, vec![0x6E]);
    }

    #[test]
    fn chooses_general_form_when_high_register_is_used() {
        let program = assemble("ADD R3, R7").unwrap();
        assert_eq!(program.payload, vec![0xE1, 0x1F]);
    }

    #[test]
    fn compact_memory_forms_are_one_byte() {
        assert_eq!(assemble("LOAD8 R2, [R1]").unwrap().payload, vec![0x99]);
        assert_eq!(assemble("STORE8 [R2], R3").unwrap().payload, vec![0xAB]);
    }

    #[test]
    fn post_increment_memory_forms_use_two_bytes_and_natural_syntax() {
        assert_eq!(
            assemble("LOAD8 R2, [R1+]").unwrap().payload,
            vec![0xF8, 0x11]
        );
        assert_eq!(
            assemble("STORE8 [R2+], R3").unwrap().payload,
            vec![0xF9, 0x13]
        );
        assert_eq!(
            assemble("LOAD16 R7, [R6+]").unwrap().payload,
            vec![0xFA, 0x3E]
        );
        assert_eq!(
            assemble("STORE16 [R5+], R4").unwrap().payload,
            vec![0xFB, 0x2C]
        );
    }

    #[test]
    fn rejects_same_register_for_post_increment_load() {
        let error = assemble("LOAD8 R2, [R2+]").unwrap_err();
        assert!(error.to_string().contains("distinct"));
    }

    #[test]
    fn rejects_invalid_register() {
        let error = assemble("MOV R8, R0").unwrap_err();
        assert!(error.to_string().contains("R8"));
    }

    #[test]
    fn rejects_multi_digit_register_names_outside_r0_to_r7() {
        let error = assemble("MOV R10, R0").unwrap_err();
        assert!(error.to_string().contains("R10"));
    }

    #[test]
    fn rejects_immediate_larger_than_16_bits() {
        let error = assemble("MOVI R0, 0x10000").unwrap_err();
        assert!(error.to_string().contains("exceeds 0xFFFF"));
    }

    #[test]
    fn rejects_unknown_label() {
        let error = assemble("JMP nowhere").unwrap_err();
        assert!(error.to_string().contains("nowhere"));
    }

    #[test]
    fn clr_and_test_are_zero_cost_pseudo_instructions() {
        let clr = assemble("CLR R3").unwrap();
        assert_eq!(clr.payload, vec![0xE8, 0x1B]); // general XOR R3,R3; compact slot is AND in ISA v3

        let test = assemble("TEST R5").unwrap();
        assert_eq!(test.payload, vec![0xE7, 0x2D]); // general OR R5,R5
    }

    #[test]
    fn labels_account_for_compact_instruction_lengths() {
        let program = assemble("start:\nADD R0,R1\nJMP start").unwrap();
        assert_eq!(program.payload, vec![0x61, 0xF0, 0x00, 0x00]);
    }
}

#[cfg(test)]
mod irq_encoding_tests {
    use super::*;
    #[test]
    fn irq_control_is_one_byte_each() {
        assert_eq!(
            assemble("EI\nDI\nIRET\n").unwrap().payload,
            vec![0x07, 0x08, 0x09]
        );
    }
}

#[cfg(test)]
mod dsp_encoding_tests {
    use super::*;
    #[test]
    fn encodes_dsp_extension() {
        assert_eq!(
            assemble("ASR1 R3\nMULQ15 R2,R5\n").unwrap().payload,
            vec![0x0A, 3, 0x0B, 0x15]
        );
    }
}

#[cfg(test)]
mod video_space_encoding_tests {
    use super::*;
    #[test]
    fn encodes_video_prefix_forms() {
        let p = assemble("VLOAD8 R2, [R0+]\nVSTORE8 [R1+], R2\n").unwrap();
        assert_eq!(p.payload, vec![0x0C, 0x04, 0x10, 0x0C, 0x06, 0x0A]);
    }
}
