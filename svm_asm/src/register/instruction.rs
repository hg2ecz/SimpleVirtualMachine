pub const REGISTER_COUNT: u8 = 8;
#[cfg(feature = "assembler")]
pub(crate) const COMPACT_REGISTER_COUNT: u8 = 4;

#[cfg(feature = "assembler")]
pub(crate) const fn encode_register_pair(first: u8, second: u8) -> Option<u8> {
    if first < REGISTER_COUNT && second < REGISTER_COUNT {
        Some((first << 3) | second)
    } else {
        None
    }
}

pub const fn decode_register_pair(raw: u8) -> Option<(u8, u8)> {
    if raw & 0xC0 == 0 {
        Some(((raw >> 3) & 0x07, raw & 0x07))
    } else {
        None
    }
}

#[cfg(feature = "assembler")]
pub(crate) const fn encode_compact_pair(base: u8, first: u8, second: u8) -> Option<u8> {
    if first < COMPACT_REGISTER_COUNT && second < COMPACT_REGISTER_COUNT {
        Some(base | (first << 2) | second)
    } else {
        None
    }
}

#[cfg(feature = "assembler")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OperandForm {
    None,
    Register,
    RegisterPair,
    RegisterMemory,
    MemoryRegister,
    RegisterMemoryPostInc,
    MemoryPostIncRegister,
    RegisterMemoryPreDec,
    MemoryPreDecRegister,
    RegisterImmediate16,
    Address16,
    ZeroPage8,
}

#[cfg(feature = "assembler")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Encoding {
    Fixed(u8),
    EmbeddedRegister {
        base: u8,
    },
    CompactOrGeneral {
        compact_base: Option<u8>,
        general: u8,
    },
    EmbeddedRegisterImmediate16 {
        base: u8,
    },
    Address16 {
        opcode: u8,
    },
    ZeroPage8 {
        opcode: u8,
    },
    ExtendedRegister {
        opcode: u8,
    },
    ExtendedRegisterPair {
        opcode: u8,
    },
    VideoRegisterPair {
        subcode: u8,
    },
    IntegerExtensionPair {
        subcode: u8,
    },
    IntegerExtensionRegister {
        subcode: u8,
    },
}

#[cfg(feature = "assembler")]
#[derive(Debug, Clone, Copy)]
pub(crate) struct InstructionSpec {
    pub(crate) mnemonic: &'static str,
    pub(crate) form: OperandForm,
    pub(crate) encoding: Encoding,
}

#[cfg(feature = "assembler")]
macro_rules! spec {
    ($name:literal, $form:ident, $encoding:expr) => {
        InstructionSpec {
            mnemonic: $name,
            form: OperandForm::$form,
            encoding: $encoding,
        }
    };
}

// Register ISA v3 opcode map.
//
// 00..0F  fixed/special and implicit-R0 zero-page forms
// 10..4F  8 x embedded-register unary families: base | rrr
// 50..BF  7 x compact two-register families for R0..R3: base | ddss
// C0..DF  4 x embedded-register immediate16 families: base | rrr, imm16
// E0..EF  full R0..R7 two-register/memory forms: opcode, 00dd dsss
// F0..F7  absolute branch/call forms: opcode, address16
// F8..FB  post-increment memory forms: opcode, 00dd dsss
// FC..FF  pre-decrement memory forms: opcode, 00dd dsss
#[cfg(feature = "assembler")]
pub(crate) const INSTRUCTION_SET: &[InstructionSpec] = &[
    spec!("NOP", None, Encoding::Fixed(0x00)),
    spec!("HALT", None, Encoding::Fixed(0x01)),
    spec!("RET", None, Encoding::Fixed(0x02)),
    // Zero-page fast forms use implicit R0, keeping the common compiler path at 2 bytes.
    spec!("ZLOAD8", ZeroPage8, Encoding::ZeroPage8 { opcode: 0x03 }),
    spec!("ZLOAD16", ZeroPage8, Encoding::ZeroPage8 { opcode: 0x04 }),
    spec!("ZSTORE8", ZeroPage8, Encoding::ZeroPage8 { opcode: 0x05 }),
    spec!("ZSTORE16", ZeroPage8, Encoding::ZeroPage8 { opcode: 0x06 }),
    spec!("EI", None, Encoding::Fixed(0x07)),
    spec!("DI", None, Encoding::Fixed(0x08)),
    spec!("IRET", None, Encoding::Fixed(0x09)),
    spec!(
        "ASR1",
        Register,
        Encoding::ExtendedRegister { opcode: 0x0A }
    ),
    spec!(
        "MULQ15",
        RegisterPair,
        Encoding::ExtendedRegisterPair { opcode: 0x0B }
    ),
    // Separate video-space memory operations. 0x0C is a prefix; the
    // following subcode selects width/direction and the third byte is 00dddsss.
    spec!(
        "ADC",
        RegisterPair,
        Encoding::IntegerExtensionPair { subcode: 0x00 }
    ),
    spec!(
        "SBC",
        RegisterPair,
        Encoding::IntegerExtensionPair { subcode: 0x01 }
    ),
    spec!(
        "MULHU",
        RegisterPair,
        Encoding::IntegerExtensionPair { subcode: 0x02 }
    ),
    spec!(
        "RCR1",
        Register,
        Encoding::IntegerExtensionRegister { subcode: 0x03 }
    ),
    spec!(
        "VLOAD8",
        RegisterMemory,
        Encoding::VideoRegisterPair { subcode: 0x00 }
    ),
    spec!(
        "VLOAD16",
        RegisterMemory,
        Encoding::VideoRegisterPair { subcode: 0x01 }
    ),
    spec!(
        "VSTORE8",
        MemoryRegister,
        Encoding::VideoRegisterPair { subcode: 0x02 }
    ),
    spec!(
        "VSTORE16",
        MemoryRegister,
        Encoding::VideoRegisterPair { subcode: 0x03 }
    ),
    spec!(
        "VLOAD8P",
        RegisterMemoryPostInc,
        Encoding::VideoRegisterPair { subcode: 0x04 }
    ),
    spec!(
        "VLOAD16P",
        RegisterMemoryPostInc,
        Encoding::VideoRegisterPair { subcode: 0x05 }
    ),
    spec!(
        "VSTORE8P",
        MemoryPostIncRegister,
        Encoding::VideoRegisterPair { subcode: 0x06 }
    ),
    spec!(
        "VSTORE16P",
        MemoryPostIncRegister,
        Encoding::VideoRegisterPair { subcode: 0x07 }
    ),
    spec!(
        "VLOAD8M",
        RegisterMemoryPreDec,
        Encoding::VideoRegisterPair { subcode: 0x08 }
    ),
    spec!(
        "VLOAD16M",
        RegisterMemoryPreDec,
        Encoding::VideoRegisterPair { subcode: 0x09 }
    ),
    spec!(
        "VSTORE8M",
        MemoryPreDecRegister,
        Encoding::VideoRegisterPair { subcode: 0x0A }
    ),
    spec!(
        "VSTORE16M",
        MemoryPreDecRegister,
        Encoding::VideoRegisterPair { subcode: 0x0B }
    ),
    spec!("NOT", Register, Encoding::EmbeddedRegister { base: 0x10 }),
    spec!("NEG", Register, Encoding::EmbeddedRegister { base: 0x18 }),
    spec!("INC", Register, Encoding::EmbeddedRegister { base: 0x20 }),
    spec!("DEC", Register, Encoding::EmbeddedRegister { base: 0x28 }),
    spec!("SHL1", Register, Encoding::EmbeddedRegister { base: 0x30 }),
    spec!("SHR1", Register, Encoding::EmbeddedRegister { base: 0x38 }),
    spec!("PUSH", Register, Encoding::EmbeddedRegister { base: 0x40 }),
    spec!("POP", Register, Encoding::EmbeddedRegister { base: 0x48 }),
    spec!(
        "MOV",
        RegisterPair,
        Encoding::CompactOrGeneral {
            compact_base: Some(0x50),
            general: 0xE0
        }
    ),
    spec!(
        "ADD",
        RegisterPair,
        Encoding::CompactOrGeneral {
            compact_base: Some(0x60),
            general: 0xE1
        }
    ),
    spec!(
        "SUB",
        RegisterPair,
        Encoding::CompactOrGeneral {
            compact_base: Some(0x70),
            general: 0xE2
        }
    ),
    spec!(
        "MUL",
        RegisterPair,
        Encoding::CompactOrGeneral {
            compact_base: None,
            general: 0xE3
        }
    ),
    spec!(
        "DIV",
        RegisterPair,
        Encoding::CompactOrGeneral {
            compact_base: None,
            general: 0xE4
        }
    ),
    spec!(
        "MOD",
        RegisterPair,
        Encoding::CompactOrGeneral {
            compact_base: None,
            general: 0xE5
        }
    ),
    spec!(
        "AND",
        RegisterPair,
        Encoding::CompactOrGeneral {
            compact_base: Some(0xB0),
            general: 0xE6
        }
    ),
    spec!(
        "OR",
        RegisterPair,
        Encoding::CompactOrGeneral {
            compact_base: None,
            general: 0xE7
        }
    ),
    spec!(
        "XOR",
        RegisterPair,
        Encoding::CompactOrGeneral {
            compact_base: None,
            general: 0xE8
        }
    ),
    spec!(
        "SHL",
        RegisterPair,
        Encoding::CompactOrGeneral {
            compact_base: None,
            general: 0xE9
        }
    ),
    spec!(
        "SHR",
        RegisterPair,
        Encoding::CompactOrGeneral {
            compact_base: None,
            general: 0xEA
        }
    ),
    spec!(
        "CMP",
        RegisterPair,
        Encoding::CompactOrGeneral {
            compact_base: Some(0x80),
            general: 0xEB
        }
    ),
    spec!(
        "LOAD8",
        RegisterMemory,
        Encoding::CompactOrGeneral {
            compact_base: Some(0x90),
            general: 0xEC
        }
    ),
    spec!(
        "LOAD16",
        RegisterMemory,
        Encoding::CompactOrGeneral {
            compact_base: None,
            general: 0xED
        }
    ),
    spec!(
        "STORE8",
        MemoryRegister,
        Encoding::CompactOrGeneral {
            compact_base: Some(0xA0),
            general: 0xEE
        }
    ),
    spec!(
        "STORE16",
        MemoryRegister,
        Encoding::CompactOrGeneral {
            compact_base: None,
            general: 0xEF
        }
    ),
    // Cost-optimized post-increment indirect memory forms. These reuse the
    // normal memory datapath and add only address-register += access width.
    spec!(
        "LOAD8P",
        RegisterMemoryPostInc,
        Encoding::CompactOrGeneral {
            compact_base: None,
            general: 0xF8
        }
    ),
    spec!(
        "STORE8P",
        MemoryPostIncRegister,
        Encoding::CompactOrGeneral {
            compact_base: None,
            general: 0xF9
        }
    ),
    spec!(
        "LOAD16P",
        RegisterMemoryPostInc,
        Encoding::CompactOrGeneral {
            compact_base: None,
            general: 0xFA
        }
    ),
    spec!(
        "STORE16P",
        MemoryPostIncRegister,
        Encoding::CompactOrGeneral {
            compact_base: None,
            general: 0xFB
        }
    ),
    spec!(
        "LOAD8M",
        RegisterMemoryPreDec,
        Encoding::CompactOrGeneral {
            compact_base: None,
            general: 0xFC
        }
    ),
    spec!(
        "STORE8M",
        MemoryPreDecRegister,
        Encoding::CompactOrGeneral {
            compact_base: None,
            general: 0xFD
        }
    ),
    spec!(
        "LOAD16M",
        RegisterMemoryPreDec,
        Encoding::CompactOrGeneral {
            compact_base: None,
            general: 0xFE
        }
    ),
    spec!(
        "STORE16M",
        MemoryPreDecRegister,
        Encoding::CompactOrGeneral {
            compact_base: None,
            general: 0xFF
        }
    ),
    spec!(
        "MOVI",
        RegisterImmediate16,
        Encoding::EmbeddedRegisterImmediate16 { base: 0xC0 }
    ),
    spec!(
        "ADDI",
        RegisterImmediate16,
        Encoding::EmbeddedRegisterImmediate16 { base: 0xC8 }
    ),
    spec!(
        "SUBI",
        RegisterImmediate16,
        Encoding::EmbeddedRegisterImmediate16 { base: 0xD0 }
    ),
    spec!(
        "CMPI",
        RegisterImmediate16,
        Encoding::EmbeddedRegisterImmediate16 { base: 0xD8 }
    ),
    spec!("JMP", Address16, Encoding::Address16 { opcode: 0xF0 }),
    spec!("CALL", Address16, Encoding::Address16 { opcode: 0xF1 }),
    spec!("JZ", Address16, Encoding::Address16 { opcode: 0xF2 }),
    spec!("JNZ", Address16, Encoding::Address16 { opcode: 0xF3 }),
    spec!("JC", Address16, Encoding::Address16 { opcode: 0xF4 }),
    spec!("JNC", Address16, Encoding::Address16 { opcode: 0xF5 }),
    spec!("JN", Address16, Encoding::Address16 { opcode: 0xF6 }),
    spec!("JNN", Address16, Encoding::Address16 { opcode: 0xF7 }),
];

#[cfg(feature = "assembler")]
pub(crate) fn instruction_spec(mnemonic: &str) -> Option<InstructionSpec> {
    INSTRUCTION_SET
        .iter()
        .find(|spec| spec.mnemonic == mnemonic)
        .copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_register_pair_uses_six_payload_bits() {
        assert_eq!(encode_register_pair(3, 7), Some(0x1F));
        assert_eq!(decode_register_pair(0x1F), Some((3, 7)));
        assert_eq!(decode_register_pair(0x80), None);
    }

    #[test]
    fn compact_pair_is_one_opcode_byte() {
        assert_eq!(encode_compact_pair(0x60, 3, 2), Some(0x6E));
        assert_eq!(encode_compact_pair(0x60, 4, 0), None);
    }

    #[test]
    fn mnemonic_names_are_unique() {
        for (index, spec) in INSTRUCTION_SET.iter().enumerate() {
            assert!(
                !INSTRUCTION_SET[..index]
                    .iter()
                    .any(|previous| previous.mnemonic == spec.mnemonic),
                "duplicate mnemonic {}",
                spec.mnemonic
            );
        }
    }
}
