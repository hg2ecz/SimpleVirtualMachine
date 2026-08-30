macro_rules! define_opcodes {
    ($( $variant:ident = $raw:literal; )+) => {
        #[repr(u8)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Opcode {
            $( $variant = $raw, )+
        }

        impl TryFrom<u8> for Opcode {
            type Error = u8;

            fn try_from(value: u8) -> Result<Self, Self::Error> {
                match value {
                    $( $raw => Ok(Self::$variant), )+
                    other => Err(other),
                }
            }
        }

        #[cfg(test)]
        pub(crate) const ALL_OPCODES: &[Opcode] = &[
            $( Opcode::$variant, )+
        ];
    };
}

define_opcodes! {
    Nop = 0x00;
    Halt = 0x01;
    Ret = 0x02;
    Dup = 0x03;
    Drop = 0x04;
    Swap = 0x05;
    Over = 0x06;
    Rot = 0x07;

    // ASSEMBLY-ORIENTED CONVENIENCE: these stack shuffles are retained as
    // real one-byte opcodes primarily for hand-written stack/Forth-style
    // assembly. The C backend does not require them; their value is compact,
    // readable manual stack manipulation with no extra hardware datapath.
    Nip = 0x08;
    Tuck = 0x09;
    TwoDup = 0x0A;
    TwoDrop = 0x0B;

    // Linear memory walkers. All are one-byte primitives.
    Load8PostInc = 0x0C;
    Store8PostInc = 0x0D;
    Load16PostInc = 0x0E;
    Store16PostInc = 0x0F;

    Add = 0x10;
    Sub = 0x11;
    Mul = 0x12;
    Div = 0x13;
    Mod = 0x14;
    Neg = 0x15;
    Inc = 0x16;
    Dec = 0x17;
    And = 0x18;
    Or = 0x19;
    Xor = 0x1A;
    Not = 0x1B;
    Shl = 0x1C;
    Shr = 0x1D;
    Shl1 = 0x1E;
    Shr1 = 0x1F;

    Eq = 0x20;
    Ne = 0x21;
    Ult = 0x22;
    Ugt = 0x23;
    Slt = 0x24;
    Sgt = 0x25;
    ZeroEq = 0x26;
    ZeroLt = 0x27;

    Load8 = 0x28;
    Load16 = 0x29;
    Store8 = 0x2A;
    Store16 = 0x2B;
    // ASSEMBLY-ORIENTED STRUCTURED LOOP SUPPORT: retained primarily for
    // hand-written stack/Forth-style assembly. The C backend can express
    // loops with ordinary branches, but DO/I/J/LOOP style code is a natural
    // and compact programming model for this architecture.
    Do = 0x2C;
    I = 0x2D;
    J = 0x2E;
    Unloop = 0x2F;

    // One-byte small literal forms. These consume no immediate byte and
    // are selected automatically by the assembler.
    // Dense one-byte literal window: -1 and 0..10; 11..14 use PUSH8 so four one-byte slots can serve backward memory walkers.
    PushTrue = 0x30;
    Push0 = 0x31;
    Push1 = 0x32;
    Push2 = 0x33;
    Push3 = 0x34;
    Push4 = 0x35;
    Push5 = 0x36;
    Push6 = 0x37;
    Push7 = 0x38;
    Push8Small = 0x39;
    Push9 = 0x3A;
    Push10 = 0x3B;
    // Pre-decrement walkers replace the rarely valuable one-byte 11..14 literals.
    Load8PreDec = 0x3C;
    Store8PreDec = 0x3D;
    Load16PreDec = 0x3E;
    Store16PreDec = 0x3F;

    Push8 = 0x40;
    PushS8 = 0x41;
    Bra8 = 0x42;
    Bz8 = 0x43;
    Bnz8 = 0x44;
    Call8 = 0x45;
    QDo8 = 0x46;
    Loop8 = 0x47;
    PlusLoop8 = 0x48;
    Leave8 = 0x49;
    // ASSEMBLY-ORIENTED DEEP STACK ACCESS: useful mainly in hand-written
    // stack assembly where avoiding explicit temporary storage is valuable.
    // These are not required by the C backend.
    Pick = 0x4A;
    Roll = 0x4B;
    // Zero-page direct memory forms: opcode + 8-bit address.
    Load8Zp = 0x4C;
    Load16Zp = 0x4D;
    Store8Zp = 0x4E;
    Store16Zp = 0x4F;
    // Two-byte system prefix: subcode 0=EI, 1=DI, 2=IRET. Rare control
    // operations pay one extension byte instead of consuming hot one-byte opcodes.
    Sys = 0x50;

    Push16 = 0x80;
    Jmp = 0x81;
    Jz = 0x82;
    Jnz = 0x83;
    Call = 0x84;
    QDo = 0x85;
    Loop = 0x86;
    PlusLoop = 0x87;
    Leave = 0x88;

    // Absolute memory forms. The 16-bit address is encoded in the
    // instruction, avoiding a temporary address push on the data stack.
    Load8Abs = 0x89;
    Load16Abs = 0x8A;
    Store8Abs = 0x8B;
    Store16Abs = 0x8C;
}

impl Opcode {
    pub const fn encoded_len(self) -> usize {
        ((self as u8 >> 6) + 1) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assigned_opcodes_are_unique() {
        let mut seen = [false; 256];
        for opcode in ALL_OPCODES {
            let raw = *opcode as usize;
            assert!(!seen[raw], "duplicate opcode 0x{raw:02X}");
            seen[raw] = true;
        }
    }

    #[test]
    fn decoder_accepts_every_assigned_opcode() {
        for opcode in ALL_OPCODES {
            assert_eq!(Opcode::try_from(*opcode as u8), Ok(*opcode));
        }
    }

    #[test]
    fn instruction_lengths_follow_top_two_bits() {
        for opcode in ALL_OPCODES {
            let raw = *opcode as u8;
            assert_eq!(opcode.encoded_len(), ((raw >> 6) + 1) as usize);
        }
    }
}
