use std::collections::HashMap;

const MEMORY_SIZE: usize = 65_536;

use super::{AsmError, Program, instruction::Opcode};

#[derive(Debug, Clone)]
struct Token {
    text: String,
    line: usize,
}

#[derive(Debug, Clone, Copy)]
enum BranchKind {
    Always,
    Zero,
    NonZero,
    Call,
    QDo,
    Loop,
    PlusLoop,
    Leave,
}

impl BranchKind {
    const fn short_opcode(self) -> Opcode {
        match self {
            Self::Always => Opcode::Bra8,
            Self::Zero => Opcode::Bz8,
            Self::NonZero => Opcode::Bnz8,
            Self::Call => Opcode::Call8,
            Self::QDo => Opcode::QDo8,
            Self::Loop => Opcode::Loop8,
            Self::PlusLoop => Opcode::PlusLoop8,
            Self::Leave => Opcode::Leave8,
        }
    }
    const fn long_opcode(self) -> Opcode {
        match self {
            Self::Always => Opcode::Jmp,
            Self::Zero => Opcode::Jz,
            Self::NonZero => Opcode::Jnz,
            Self::Call => Opcode::Call,
            Self::QDo => Opcode::QDo,
            Self::Loop => Opcode::Loop,
            Self::PlusLoop => Opcode::PlusLoop,
            Self::Leave => Opcode::Leave,
        }
    }
}

#[derive(Debug, Clone)]
enum Item {
    Label(String),
    Op(Opcode),
    Imm8(Opcode, u8),
    Imm16(Opcode, u16),
    Branch {
        kind: BranchKind,
        target: String,
        long: bool,
        line: usize,
    },
}

impl Item {
    fn len(&self) -> usize {
        match self {
            Self::Label(_) => 0,
            Self::Op(op) => op.encoded_len(),
            Self::Imm8(op, _) => op.encoded_len(),
            Self::Imm16(op, _) => op.encoded_len(),
            Self::Branch { kind, long, .. } => {
                if *long {
                    kind.long_opcode().encoded_len()
                } else {
                    kind.short_opcode().encoded_len()
                }
            }
        }
    }
}

#[derive(Debug)]
enum Control {
    If {
        false_label: String,
    },
    Else {
        end_label: String,
    },
    Begin {
        begin_label: String,
    },
    While {
        begin_label: String,
        end_label: String,
    },
    Do {
        body_label: String,
        end_label: String,
    },
    Case {
        end_label: String,
    },
    Of {
        case_end: String,
        next_label: String,
    },
}

struct Compiler {
    items: Vec<Item>,
    control: Vec<Control>,
    load_address: u16,
    load_explicit: bool,
    entry: Option<String>,
    unique: usize,
    current_definition: Option<String>,
}

impl Default for Compiler {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            control: Vec::new(),
            load_address: 0,
            load_explicit: false,
            entry: None,
            unique: 0,
            current_definition: None,
        }
    }
}

pub fn assemble(source: &str) -> Result<Program, AsmError> {
    let tokens = tokenize(source);
    let mut compiler = Compiler::default();
    compiler.compile(&tokens)?;
    compiler.finish()
}

fn tokenize(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    for (line_index, line) in source.lines().enumerate() {
        let code = line.split_once('\\').map_or(line, |(code, _)| code);
        for text in code.split_whitespace() {
            tokens.push(Token {
                text: text.to_string(),
                line: line_index + 1,
            });
        }
    }
    tokens
}

impl Compiler {
    fn compile(&mut self, tokens: &[Token]) -> Result<(), AsmError> {
        let mut i = 0;
        while i < tokens.len() {
            let token = &tokens[i];
            let upper = token.text.to_ascii_uppercase();
            match upper.as_str() {
                ".LOAD" => {
                    if self.load_explicit {
                        return asm_err(token.line, "duplicate .load directive");
                    }
                    let value =
                        next_token(tokens, &mut i, token.line, ".load requires an address")?;
                    self.load_address = parse_u16(&value.text, value.line)?;
                    self.load_explicit = true;
                }
                ".ENTRY" => {
                    if self.entry.is_some() {
                        return asm_err(token.line, "duplicate .entry directive");
                    }
                    let value = next_token(
                        tokens,
                        &mut i,
                        token.line,
                        ".entry requires a label or address",
                    )?;
                    self.entry = Some(value.text.clone());
                }
                ":" => {
                    if self.current_definition.is_some() {
                        return asm_err(token.line, "nested ':' definitions are not allowed");
                    }
                    let name = next_token(tokens, &mut i, token.line, "':' requires a word name")?
                        .text
                        .to_ascii_uppercase();
                    self.items.push(Item::Label(name.clone()));
                    self.current_definition = Some(name);
                }
                ";" => {
                    if self.current_definition.take().is_none() {
                        return asm_err(token.line, "';' outside a ':' definition");
                    }
                    if !self.control.is_empty() {
                        return asm_err(token.line, "unclosed control structure before ';'");
                    }
                    self.items.push(Item::Op(Opcode::Ret));
                }
                "IF" => self.compile_if(),
                "ELSE" => self.compile_else(token.line)?,
                "THEN" => self.compile_then(token.line)?,
                "BEGIN" => self.compile_begin(),
                "AGAIN" => self.compile_again(token.line)?,
                "UNTIL" => self.compile_until(token.line)?,
                "WHILE" => self.compile_while(token.line)?,
                "REPEAT" => self.compile_repeat(token.line)?,
                "DO" => self.compile_do(false),
                "?DO" => self.compile_do(true),
                "LOOP" => self.compile_loop(false, token.line)?,
                "+LOOP" => self.compile_loop(true, token.line)?,
                "LEAVE" => self.compile_leave(token.line)?,
                "CASE" => self.compile_case(),
                "OF" => self.compile_of(token.line)?,
                "ENDOF" => self.compile_endof(token.line)?,
                "ENDCASE" => self.compile_endcase(token.line)?,
                "RECURSE" => {
                    let target = self
                        .current_definition
                        .clone()
                        .ok_or_else(|| assembly(token.line, "RECURSE outside a ':' definition"))?;
                    self.branch(BranchKind::Call, target, token.line);
                }
                "EXIT" => self.compile_exit(),
                "PUSH" => {
                    let value =
                        next_token(tokens, &mut i, token.line, "PUSH requires a numeric value")?;
                    self.push_number(parse_number(&value.text, value.line)?, value.line)?;
                }
                "EI" => self.items.push(Item::Imm8(Opcode::Sys, 0)),
                "DI" => self.items.push(Item::Imm8(Opcode::Sys, 1)),
                "IRET" => self.items.push(Item::Imm8(Opcode::Sys, 2)),
                "ASR1" => self.items.push(Item::Imm8(Opcode::Sys, 3)),
                "MULQ15" => self.items.push(Item::Imm8(Opcode::Sys, 4)),
                "UMUL" | "MUL32" => self.items.push(Item::Imm8(Opcode::Sys, 5)),
                "ADC" => self.items.push(Item::Imm8(Opcode::Sys, 6)),
                "SBC" => self.items.push(Item::Imm8(Opcode::Sys, 7)),
                "RCR1" => self.items.push(Item::Imm8(Opcode::Sys, 8)),
                "VC@" | "VLOAD8" => self.items.push(Item::Imm8(Opcode::Sys, 0x10)),
                "V@" | "VLOAD16" => self.items.push(Item::Imm8(Opcode::Sys, 0x11)),
                "VC!" | "VSTORE8" => self.items.push(Item::Imm8(Opcode::Sys, 0x12)),
                "V!" | "VSTORE16" => self.items.push(Item::Imm8(Opcode::Sys, 0x13)),
                "VC@+" | "VLOAD8+" => self.items.push(Item::Imm8(Opcode::Sys, 0x14)),
                "V@+" | "VLOAD16+" => self.items.push(Item::Imm8(Opcode::Sys, 0x15)),
                "VC!+" | "VSTORE8+" => self.items.push(Item::Imm8(Opcode::Sys, 0x16)),
                "V!+" | "VSTORE16+" => self.items.push(Item::Imm8(Opcode::Sys, 0x17)),
                "VC@-" | "VLOAD8-" => self.items.push(Item::Imm8(Opcode::Sys, 0x18)),
                "V@-" | "VLOAD16-" => self.items.push(Item::Imm8(Opcode::Sys, 0x19)),
                "VC!-" | "VSTORE8-" => self.items.push(Item::Imm8(Opcode::Sys, 0x1A)),
                "V!-" | "VSTORE16-" => self.items.push(Item::Imm8(Opcode::Sys, 0x1B)),
                "PICK" | "ROLL" => {
                    let value = next_token(
                        tokens,
                        &mut i,
                        token.line,
                        "PICK/ROLL requires an 8-bit depth",
                    )?;
                    let depth = parse_u8(&value.text, value.line)?;
                    self.items.push(Item::Imm8(
                        if upper == "PICK" {
                            Opcode::Pick
                        } else {
                            Opcode::Roll
                        },
                        depth,
                    ));
                }
                "JMP" | "JZ" | "JNZ" | "CALL" => {
                    let target =
                        next_token(tokens, &mut i, token.line, "branch requires a target")?
                            .text
                            .to_ascii_uppercase();
                    let kind = match upper.as_str() {
                        "JMP" => BranchKind::Always,
                        "JZ" => BranchKind::Zero,
                        "JNZ" => BranchKind::NonZero,
                        _ => BranchKind::Call,
                    };
                    self.branch(kind, target, token.line);
                }
                _ if token.text.ends_with(':') => {
                    let name = token.text.trim_end_matches(':').to_ascii_uppercase();
                    if name.is_empty() {
                        return asm_err(token.line, "empty label");
                    }
                    self.items.push(Item::Label(name));
                }
                _ => self.compile_atom(token)?,
            }
            i += 1;
        }
        if self.current_definition.is_some() {
            return asm_err(
                tokens.last().map_or(1, |t| t.line),
                "missing ';' at end of definition",
            );
        }
        if !self.control.is_empty() {
            return asm_err(
                tokens.last().map_or(1, |t| t.line),
                "unclosed control structure at end of source",
            );
        }
        Ok(())
    }

    fn compile_atom(&mut self, token: &Token) -> Result<(), AsmError> {
        if token.text.eq_ignore_ascii_case("TRUE") {
            return self.push_number(-1, token.line);
        }
        if token.text.eq_ignore_ascii_case("FALSE") {
            return self.push_number(0, token.line);
        }
        // Recognize symbolic opcodes before numeric literals. Several useful
        // Forth-style mnemonics begin with a digit (0=, 0<, 1+, 1-, 2*, 2/);
        // treating every digit-prefixed token as a number breaks those words.
        if let Some(op) = simple_opcode(&token.text) {
            if let Some(abs_op) = absolute_memory_opcode(op) {
                if let Some(address) = self.items.last().and_then(literal_item_u16) {
                    self.items.pop();
                    if address <= 0x00FF {
                        let zp = match op {
                            Opcode::Load8 => Opcode::Load8Zp,
                            Opcode::Load16 => Opcode::Load16Zp,
                            Opcode::Store8 => Opcode::Store8Zp,
                            Opcode::Store16 => Opcode::Store16Zp,
                            _ => unreachable!(),
                        };
                        self.items.push(Item::Imm8(zp, address as u8));
                    } else {
                        self.items.push(Item::Imm16(abs_op, address));
                    }
                    return Ok(());
                }
            }
            self.items.push(Item::Op(op));
            return Ok(());
        }
        if looks_like_number(&token.text) {
            return self.push_number(parse_number(&token.text, token.line)?, token.line);
        }
        // Forth-like bare word invocation. Unknown names are resolved after all labels are known.
        self.branch(
            BranchKind::Call,
            token.text.to_ascii_uppercase(),
            token.line,
        );
        Ok(())
    }

    fn compile_if(&mut self) {
        let false_label = self.fresh("IF_FALSE");
        self.branch(BranchKind::Zero, false_label.clone(), 0);
        self.control.push(Control::If { false_label });
    }
    fn compile_else(&mut self, line: usize) -> Result<(), AsmError> {
        let Control::If { false_label } = self
            .control
            .pop()
            .ok_or_else(|| assembly(line, "ELSE without matching IF"))?
        else {
            return asm_err(line, "ELSE must match IF");
        };
        let end_label = self.fresh("IF_END");
        self.branch(BranchKind::Always, end_label.clone(), line);
        self.items.push(Item::Label(false_label));
        self.control.push(Control::Else { end_label });
        Ok(())
    }
    fn compile_then(&mut self, line: usize) -> Result<(), AsmError> {
        let control = self
            .control
            .pop()
            .ok_or_else(|| assembly(line, "THEN without matching IF/ELSE"))?;
        let label = match control {
            Control::If { false_label } => false_label,
            Control::Else { end_label } => end_label,
            _ => return asm_err(line, "THEN must match IF or ELSE"),
        };
        self.items.push(Item::Label(label));
        Ok(())
    }
    fn compile_begin(&mut self) {
        let begin_label = self.fresh("BEGIN");
        self.items.push(Item::Label(begin_label.clone()));
        self.control.push(Control::Begin { begin_label });
    }
    fn compile_again(&mut self, line: usize) -> Result<(), AsmError> {
        let Control::Begin { begin_label } = self
            .control
            .pop()
            .ok_or_else(|| assembly(line, "AGAIN without BEGIN"))?
        else {
            return asm_err(line, "AGAIN must close BEGIN");
        };
        self.branch(BranchKind::Always, begin_label, line);
        Ok(())
    }
    fn compile_until(&mut self, line: usize) -> Result<(), AsmError> {
        let Control::Begin { begin_label } = self
            .control
            .pop()
            .ok_or_else(|| assembly(line, "UNTIL without BEGIN"))?
        else {
            return asm_err(line, "UNTIL must close BEGIN");
        };
        self.branch(BranchKind::Zero, begin_label, line);
        Ok(())
    }
    fn compile_while(&mut self, line: usize) -> Result<(), AsmError> {
        let Control::Begin { begin_label } = self
            .control
            .pop()
            .ok_or_else(|| assembly(line, "WHILE without BEGIN"))?
        else {
            return asm_err(line, "WHILE must follow BEGIN");
        };
        let end_label = self.fresh("WHILE_END");
        self.branch(BranchKind::Zero, end_label.clone(), line);
        self.control.push(Control::While {
            begin_label,
            end_label,
        });
        Ok(())
    }
    fn compile_repeat(&mut self, line: usize) -> Result<(), AsmError> {
        let Control::While {
            begin_label,
            end_label,
        } = self
            .control
            .pop()
            .ok_or_else(|| assembly(line, "REPEAT without BEGIN ... WHILE"))?
        else {
            return asm_err(line, "REPEAT must close BEGIN ... WHILE");
        };
        self.branch(BranchKind::Always, begin_label, line);
        self.items.push(Item::Label(end_label));
        Ok(())
    }
    fn compile_do(&mut self, conditional: bool) {
        let body = self.fresh("DO_BODY");
        let end = self.fresh("DO_END");
        if conditional {
            self.branch(BranchKind::QDo, end.clone(), 0);
        } else {
            self.items.push(Item::Op(Opcode::Do));
        }
        self.items.push(Item::Label(body.clone()));
        self.control.push(Control::Do {
            body_label: body,
            end_label: end,
        });
    }
    fn compile_loop(&mut self, plus: bool, line: usize) -> Result<(), AsmError> {
        let Control::Do {
            body_label,
            end_label,
        } = self
            .control
            .pop()
            .ok_or_else(|| assembly(line, "LOOP/+LOOP without DO/?DO"))?
        else {
            return asm_err(line, "LOOP/+LOOP must close DO/?DO");
        };
        self.branch(
            if plus {
                BranchKind::PlusLoop
            } else {
                BranchKind::Loop
            },
            body_label,
            line,
        );
        self.items.push(Item::Label(end_label));
        Ok(())
    }
    fn compile_exit(&mut self) {
        // Loop frames share the return/control stack with return addresses.
        // EXIT may abandon lexical DO/?DO frames, so discard each active
        // loop frame before RET. This keeps the cheap two-stack hardware model
        // from becoming a manual burden on assembly programs.
        let active_loops = self
            .control
            .iter()
            .filter(|c| matches!(c, Control::Do { .. }))
            .count();
        for _ in 0..active_loops {
            self.items.push(Item::Op(Opcode::Unloop));
        }
        self.items.push(Item::Op(Opcode::Ret));
    }

    fn compile_leave(&mut self, line: usize) -> Result<(), AsmError> {
        let target = self
            .control
            .iter()
            .rev()
            .find_map(|c| {
                if let Control::Do { end_label, .. } = c {
                    Some(end_label.clone())
                } else {
                    None
                }
            })
            .ok_or_else(|| assembly(line, "LEAVE outside DO/?DO"))?;
        self.branch(BranchKind::Leave, target, line);
        Ok(())
    }
    fn compile_case(&mut self) {
        let end = self.fresh("CASE_END");
        self.control.push(Control::Case { end_label: end });
    }
    fn compile_of(&mut self, line: usize) -> Result<(), AsmError> {
        let case_end = self
            .control
            .iter()
            .rev()
            .find_map(|c| {
                if let Control::Case { end_label } = c {
                    Some(end_label.clone())
                } else {
                    None
                }
            })
            .ok_or_else(|| assembly(line, "OF without CASE"))?;
        let next = self.fresh("OF_NEXT");
        self.items.push(Item::Op(Opcode::Over));
        self.items.push(Item::Op(Opcode::Eq));
        self.branch(BranchKind::Zero, next.clone(), line);
        self.items.push(Item::Op(Opcode::Drop));
        self.control.push(Control::Of {
            case_end,
            next_label: next,
        });
        Ok(())
    }
    fn compile_endof(&mut self, line: usize) -> Result<(), AsmError> {
        let Control::Of {
            case_end,
            next_label,
        } = self
            .control
            .pop()
            .ok_or_else(|| assembly(line, "ENDOF without OF"))?
        else {
            return asm_err(line, "ENDOF must close OF");
        };
        self.branch(BranchKind::Always, case_end, line);
        self.items.push(Item::Label(next_label));
        Ok(())
    }
    fn compile_endcase(&mut self, line: usize) -> Result<(), AsmError> {
        if matches!(self.control.last(), Some(Control::Of { .. })) {
            return asm_err(line, "OF must be closed by ENDOF before ENDCASE");
        }
        let Control::Case { end_label } = self
            .control
            .pop()
            .ok_or_else(|| assembly(line, "ENDCASE without CASE"))?
        else {
            return asm_err(line, "ENDCASE must close CASE");
        };
        self.items.push(Item::Op(Opcode::Drop));
        self.items.push(Item::Label(end_label));
        Ok(())
    }

    fn push_number(&mut self, value: i32, line: usize) -> Result<(), AsmError> {
        let small = match value {
            -1 => Some(Opcode::PushTrue),
            0 => Some(Opcode::Push0),
            1 => Some(Opcode::Push1),
            2 => Some(Opcode::Push2),
            3 => Some(Opcode::Push3),
            4 => Some(Opcode::Push4),
            5 => Some(Opcode::Push5),
            6 => Some(Opcode::Push6),
            7 => Some(Opcode::Push7),
            8 => Some(Opcode::Push8Small),
            9 => Some(Opcode::Push9),
            10 => Some(Opcode::Push10),
            _ => None,
        };
        if let Some(opcode) = small {
            self.items.push(Item::Op(opcode));
        } else if (-128..=-1).contains(&value) {
            self.items
                .push(Item::Imm8(Opcode::PushS8, value as i8 as u8));
        } else if (0..=255).contains(&value) {
            self.items.push(Item::Imm8(Opcode::Push8, value as u8));
        } else if (-32768..=65535).contains(&value) {
            self.items
                .push(Item::Imm16(Opcode::Push16, value as i16 as u16));
        } else {
            return asm_err(line, "literal does not fit a 16-bit SVM-S cell");
        }
        Ok(())
    }
    fn branch(&mut self, kind: BranchKind, target: String, line: usize) {
        self.items.push(Item::Branch {
            kind,
            target,
            long: false,
            line,
        });
    }
    fn fresh(&mut self, prefix: &str) -> String {
        let value = format!("__{prefix}_{}", self.unique);
        self.unique += 1;
        value
    }

    fn finish(mut self) -> Result<Program, AsmError> {
        // Monotonic branch relaxation: start short, promote only branches that cannot reach.
        loop {
            let labels = layout(&self.items, self.load_address)?;
            let mut changed = false;
            let mut pc = self.load_address;
            for item in &mut self.items {
                match item {
                    Item::Label(_) => {}
                    Item::Branch {
                        target, long, line, ..
                    } if !*long => {
                        let target_addr = resolve_target(target, *line, &labels)?;
                        let next = pc.wrapping_add(2);
                        if relative_i8(next, target_addr).is_none() {
                            *long = true;
                            changed = true;
                        }
                        pc = pc.wrapping_add(if *long { 3 } else { 2 });
                    }
                    _ => pc = pc.wrapping_add(item.len() as u16),
                }
            }
            if !changed {
                break;
            }
        }
        let labels = layout(&self.items, self.load_address)?;
        let entry = match self.entry {
            Some(ref value) if looks_like_number(value) => parse_u16(value, 1)?,
            Some(ref value) => resolve_target(value, 1, &labels)?,
            None => self.load_address,
        };
        let mut payload = Vec::new();
        let mut pc = self.load_address;
        for item in self.items {
            match item {
                Item::Label(_) => {}
                Item::Op(op) => {
                    payload.push(op as u8);
                    pc = pc.wrapping_add(1);
                }
                Item::Imm8(op, v) => {
                    payload.extend_from_slice(&[op as u8, v]);
                    pc = pc.wrapping_add(2);
                }
                Item::Imm16(op, v) => {
                    payload.push(op as u8);
                    payload.extend_from_slice(&v.to_le_bytes());
                    pc = pc.wrapping_add(3);
                }
                Item::Branch {
                    kind,
                    target,
                    long,
                    line,
                } => {
                    let target = resolve_target(&target, line, &labels)?;
                    if long {
                        payload.push(kind.long_opcode() as u8);
                        payload.extend_from_slice(&target.to_le_bytes());
                        pc = pc.wrapping_add(3);
                    } else {
                        payload.push(kind.short_opcode() as u8);
                        let next = pc.wrapping_add(2);
                        let rel = relative_i8(next, target)
                            .ok_or_else(|| assembly(line, "internal branch relaxation error"))?;
                        payload.push(rel as u8);
                        pc = next;
                    }
                }
            }
        }
        Ok(Program {
            load_address: self.load_address,
            entry_address: entry,
            payload,
        })
    }
}

fn layout(items: &[Item], load: u16) -> Result<HashMap<String, u16>, AsmError> {
    let mut labels = HashMap::new();
    let mut cursor = load as usize;

    for item in items {
        if let Item::Label(name) = item {
            let address =
                u16::try_from(cursor).map_err(|_| assembly(1, "label lies beyond 0xFFFF"))?;
            if labels.insert(name.clone(), address).is_some() {
                return asm_err(1, &format!("duplicate label '{name}'"));
            }
            continue;
        }

        cursor = cursor
            .checked_add(item.len())
            .ok_or_else(|| assembly(1, "program layout size overflow"))?;
        if cursor > MEMORY_SIZE {
            return asm_err(1, "program crosses the end of the 64 KiB address space");
        }
    }

    Ok(labels)
}

fn relative_i8(next_pc: u16, target: u16) -> Option<i8> {
    let delta = target.wrapping_sub(next_pc) as i16;
    i8::try_from(delta).ok()
}
fn resolve_target(text: &str, line: usize, labels: &HashMap<String, u16>) -> Result<u16, AsmError> {
    if looks_like_number(text) {
        return parse_u16(text, line);
    }
    labels
        .get(&text.to_ascii_uppercase())
        .copied()
        .ok_or_else(|| assembly(line, &format!("unknown target/word '{text}'")))
}
fn next_token<'a>(
    tokens: &'a [Token],
    i: &mut usize,
    line: usize,
    message: &str,
) -> Result<&'a Token, AsmError> {
    *i += 1;
    tokens.get(*i).ok_or_else(|| assembly(line, message))
}
fn looks_like_number(text: &str) -> bool {
    let text = text.trim_start_matches('-');
    text.chars().next().is_some_and(|ch| ch.is_ascii_digit())
}

fn parse_number(text: &str, line: usize) -> Result<i32, AsmError> {
    let clean = text.replace('_', "");
    let result = if let Some(hex) = clean
        .strip_prefix("-0x")
        .or_else(|| clean.strip_prefix("-0X"))
    {
        i32::from_str_radix(hex, 16).map(|v| -v)
    } else if let Some(hex) = clean
        .strip_prefix("0x")
        .or_else(|| clean.strip_prefix("0X"))
    {
        i32::from_str_radix(hex, 16)
    } else {
        clean.parse::<i32>()
    };
    result.map_err(|_| assembly(line, &format!("invalid number '{text}'")))
}
fn parse_u16(text: &str, line: usize) -> Result<u16, AsmError> {
    let v = parse_number(text, line)?;
    u16::try_from(v).map_err(|_| assembly(line, "value does not fit u16"))
}
fn parse_u8(text: &str, line: usize) -> Result<u8, AsmError> {
    let v = parse_number(text, line)?;
    u8::try_from(v).map_err(|_| assembly(line, "value does not fit u8"))
}
fn assembly(line: usize, message: &str) -> AsmError {
    AsmError::Assembly {
        line,
        message: message.to_string(),
    }
}
fn asm_err<T>(line: usize, message: &str) -> Result<T, AsmError> {
    Err(assembly(line, message))
}

fn absolute_memory_opcode(op: Opcode) -> Option<Opcode> {
    match op {
        Opcode::Load8 => Some(Opcode::Load8Abs),
        Opcode::Load16 => Some(Opcode::Load16Abs),
        Opcode::Store8 => Some(Opcode::Store8Abs),
        Opcode::Store16 => Some(Opcode::Store16Abs),
        _ => None,
    }
}

fn literal_item_u16(item: &Item) -> Option<u16> {
    use Opcode::*;
    match item {
        Item::Op(PushTrue) => Some(0xFFFF),
        Item::Op(Push0) => Some(0),
        Item::Op(Push1) => Some(1),
        Item::Op(Push2) => Some(2),
        Item::Op(Push3) => Some(3),
        Item::Op(Push4) => Some(4),
        Item::Op(Push5) => Some(5),
        Item::Op(Push6) => Some(6),
        Item::Op(Push7) => Some(7),
        Item::Op(Push8Small) => Some(8),
        Item::Op(Push9) => Some(9),
        Item::Op(Push10) => Some(10),
        Item::Imm8(Push8, v) => Some(u16::from(*v)),
        Item::Imm8(PushS8, v) => Some((*v as i8 as i16) as u16),
        Item::Imm16(Push16, v) => Some(*v),
        _ => None,
    }
}

fn simple_opcode(text: &str) -> Option<Opcode> {
    use Opcode::*;
    Some(match text.to_ascii_uppercase().as_str() {
        "NOP" => Nop,
        "HALT" => Halt,
        "RET" => Ret,
        "DUP" => Dup,
        "DROP" => Drop,
        "SWAP" => Swap,
        "OVER" => Over,
        "ROT" => Rot,
        "NIP" => Nip,
        "TUCK" => Tuck,
        "2DUP" => TwoDup,
        "2DROP" => TwoDrop,
        "C@+" | "LOAD8+" => Load8PostInc,
        "C!+" | "STORE8+" => Store8PostInc,
        "@+" | "LOAD16+" => Load16PostInc,
        "!+" | "STORE16+" => Store16PostInc,
        "C@-" | "LOAD8-" => Load8PreDec,
        "C!-" | "STORE8-" => Store8PreDec,
        "@-" | "LOAD16-" => Load16PreDec,
        "!-" | "STORE16-" => Store16PreDec,
        "ADD" => Add,
        "+" => Add,
        "SUB" => Sub,
        "-" => Sub,
        "MUL" => Mul,
        "*" => Mul,
        "DIV" => Div,
        "/" => Div,
        "MOD" => Mod,
        "NEG" => Neg,
        "NEGATE" => Neg,
        "INC" => Inc,
        "1+" => Inc,
        "DEC" => Dec,
        "1-" => Dec,
        "AND" => And,
        "OR" => Or,
        "XOR" => Xor,
        "NOT" => Not,
        "SHL" => Shl,
        "SHR" => Shr,
        "SHL1" => Shl1,
        "2*" => Shl1,
        "SHR1" => Shr1,
        "2/" => Shr1,
        "EQ" => Eq,
        "=" => Eq,
        "NE" => Ne,
        "<>" => Ne,
        "ULT" => Ult,
        "U<" => Ult,
        "UGT" => Ugt,
        "U>" => Ugt,
        "SLT" => Slt,
        "<" => Slt,
        "SGT" => Sgt,
        ">" => Sgt,
        "0=" => ZeroEq,
        "0<" => ZeroLt,
        "LOAD8" => Load8,
        "C@" => Load8,
        "LOAD16" => Load16,
        "@" => Load16,
        "STORE8" => Store8,
        "C!" => Store8,
        "STORE16" => Store16,
        "!" => Store16,
        "I" => I,
        "J" => J,
        "UNLOOP" => Unloop,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digit_prefixed_forth_words_are_not_parsed_as_numbers() {
        let p = assemble("0= 0< 1+ 1- 2* 2/ HALT").unwrap();
        assert_eq!(p.payload[0], Opcode::ZeroEq as u8);
        assert_eq!(p.payload[1], Opcode::ZeroLt as u8);
        assert_eq!(p.payload[2], Opcode::Inc as u8);
        assert_eq!(p.payload[3], Opcode::Dec as u8);
        assert_eq!(p.payload[4], Opcode::Shl1 as u8);
        assert_eq!(p.payload[5], Opcode::Shr1 as u8);
        assert_eq!(p.payload[6], Opcode::Halt as u8);
    }

    #[test]
    fn zero_page_constant_memory_access_is_two_bytes() {
        let p = assemble("0x20 @ 0x21 ! HALT").unwrap();
        assert_eq!(p.payload[0], Opcode::Load16Zp as u8);
        assert_eq!(p.payload[1], 0x20);
        assert_eq!(p.payload[2], Opcode::Store16Zp as u8);
        assert_eq!(p.payload[3], 0x21);
    }

    #[test]
    fn chooses_short_literals() {
        let program = assemble("10 -1 1000 HALT").unwrap();
        assert_eq!(program.payload[0], Opcode::Push10 as u8);
        assert_eq!(program.payload[1], Opcode::PushTrue as u8);
        assert_eq!(program.payload[2], Opcode::Push16 as u8);
    }

    #[test]
    fn minus_one_through_ten_use_single_byte_literals() {
        let program = assemble("TRUE 0 1 2 3 4 5 6 7 8 9 10 HALT").unwrap();
        assert_eq!(
            program.payload,
            vec![
                Opcode::PushTrue as u8,
                Opcode::Push0 as u8,
                Opcode::Push1 as u8,
                Opcode::Push2 as u8,
                Opcode::Push3 as u8,
                Opcode::Push4 as u8,
                Opcode::Push5 as u8,
                Opcode::Push6 as u8,
                Opcode::Push7 as u8,
                Opcode::Push8Small as u8,
                Opcode::Push9 as u8,
                Opcode::Push10 as u8,
                Opcode::Halt as u8,
            ]
        );
    }

    #[test]
    fn pre_decrement_memory_words_are_single_byte_primitives() {
        let p = assemble("C@- C!- @- !- HALT").unwrap();
        assert_eq!(
            p.payload,
            vec![
                Opcode::Load8PreDec as u8,
                Opcode::Store8PreDec as u8,
                Opcode::Load16PreDec as u8,
                Opcode::Store16PreDec as u8,
                Opcode::Halt as u8,
            ]
        );
    }

    #[test]
    fn post_increment_memory_words_are_single_byte_primitives() {
        let p = assemble("C@+ C!+ @+ !+ HALT").unwrap();
        assert_eq!(
            p.payload,
            vec![
                Opcode::Load8PostInc as u8,
                Opcode::Store8PostInc as u8,
                Opcode::Load16PostInc as u8,
                Opcode::Store16PostInc as u8,
                Opcode::Halt as u8,
            ]
        );
    }

    #[test]
    fn absolute_memory_peephole_removes_literal_address_push() {
        let p = assemble("72 0xFF06 C! 0xFF06 C@ HALT").unwrap();
        assert!(p.payload.contains(&(Opcode::Store8Abs as u8)));
        assert!(p.payload.contains(&(Opcode::Load8Abs as u8)));
    }

    #[test]
    fn if_then_uses_short_branch_when_near() {
        let program = assemble("1 IF 2 DROP THEN HALT").unwrap();
        assert!(program.payload.contains(&(Opcode::Bz8 as u8)));
    }

    #[test]
    fn long_branch_is_promoted_when_needed() {
        let source = format!("1 IF {} THEN HALT", "NOP ".repeat(200));
        let program = assemble(&source).unwrap();
        assert!(program.payload.contains(&(Opcode::Jz as u8)));
    }

    #[test]
    fn colon_definition_and_bare_call() {
        let program = assemble(".entry main : twice DUP + ; : main 7 twice HALT ;").unwrap();
        assert!(
            program
                .payload
                .iter()
                .any(|byte| *byte == Opcode::Call8 as u8 || *byte == Opcode::Call as u8)
        );
    }

    #[test]
    fn exit_inside_nested_loops_unloops_before_ret() {
        let program = assemble(".entry f : f 3 0 DO 2 0 DO EXIT LOOP LOOP ;").unwrap();
        assert!(program.payload.windows(3).any(|w| w
            == [
                Opcode::Unloop as u8,
                Opcode::Unloop as u8,
                Opcode::Ret as u8,
            ]));
    }

    #[test]
    fn do_loop_compiles() {
        let program = assemble(".entry main : main 10 0 DO I DROP LOOP HALT ;").unwrap();
        assert!(program.payload.contains(&(Opcode::Do as u8)));
        assert!(
            program
                .payload
                .iter()
                .any(|byte| *byte == Opcode::Loop8 as u8 || *byte == Opcode::Loop as u8)
        );
    }

    #[test]
    fn final_byte_of_address_space_is_usable() {
        let program = assemble(".load 0xFFFF HALT").unwrap();
        assert_eq!(program.load_address, 0xFFFF);
        assert_eq!(program.payload, vec![Opcode::Halt as u8]);
    }

    #[test]
    fn short_branch_can_end_at_address_space_boundary() {
        let program = assemble(".load 0xFFFE JMP 0xFFFF").unwrap();
        assert_eq!(program.payload, vec![Opcode::Bra8 as u8, 0xFF]);
    }

    #[test]
    fn relative_branch_distance_uses_wrapping_pc_arithmetic() {
        assert_eq!(relative_i8(0x0000, 0xFFFF), Some(-1));
        assert_eq!(relative_i8(0xFFFF, 0x0001), Some(2));
        assert_eq!(relative_i8(0x0000, 0x0080), None);
    }

    #[test]
    fn duplicate_directives_are_rejected() {
        assert!(assemble(".load 0 .load 1 HALT").is_err());
        assert!(assemble(".entry 0 .entry 1 HALT").is_err());
    }

    #[test]
    fn malformed_control_structure_is_rejected() {
        assert!(assemble("ELSE HALT").is_err());
        assert!(assemble("BEGIN HALT").is_err());
        assert!(assemble("CASE 1 OF HALT ENDCASE").is_err());
    }
}

#[cfg(test)]
mod irq_encoding_tests {
    use super::*;
    #[test]
    fn irq_control_uses_two_byte_system_prefix() {
        assert_eq!(
            assemble("EI DI IRET").unwrap().payload,
            vec![
                Opcode::Sys as u8,
                0,
                Opcode::Sys as u8,
                1,
                Opcode::Sys as u8,
                2
            ]
        );
    }
}

#[cfg(test)]
mod dsp_encoding_tests {
    use super::*;
    #[test]
    fn encodes_dsp_system_extensions() {
        assert_eq!(
            assemble("ASR1 MULQ15").unwrap().payload,
            vec![Opcode::Sys as u8, 3, Opcode::Sys as u8, 4]
        );
    }
}

#[cfg(test)]
mod video_space_encoding_tests {
    use super::*;
    #[test]
    fn encodes_video_system_extensions() {
        assert_eq!(
            assemble("VC@+\nVC!+\nVC@-\nVC!-\n").unwrap().payload,
            vec![
                Opcode::Sys as u8,
                0x14,
                Opcode::Sys as u8,
                0x16,
                Opcode::Sys as u8,
                0x18,
                Opcode::Sys as u8,
                0x1A
            ]
        );
    }
}
