//! Cost-optimized accumulator ISA.
//! A is the 16-bit accumulator; X and Y are low-cost 16-bit address/index registers.

pub mod op {
    pub const NOP: u8 = 0x00;
    pub const HALT: u8 = 0x01;
    pub const RET: u8 = 0x02;
    pub const TAX: u8 = 0x03;
    pub const TXA: u8 = 0x04;
    pub const PUSHA: u8 = 0x05;
    pub const POPA: u8 = 0x06;
    pub const PUSHX: u8 = 0x07;
    pub const POPX: u8 = 0x08;
    pub const INC: u8 = 0x09;
    pub const DEC: u8 = 0x0A;
    pub const NEG: u8 = 0x0B;
    pub const NOT: u8 = 0x0C;
    pub const SHL1: u8 = 0x0D;
    pub const SHR1: u8 = 0x0E;
    pub const INX: u8 = 0x0F;
    pub const DEX: u8 = 0x10;

    pub const ADDX: u8 = 0x11;
    pub const SUBX: u8 = 0x12;
    pub const MULX: u8 = 0x13;
    pub const DIVX: u8 = 0x14;
    pub const MODX: u8 = 0x15;
    pub const ANDX: u8 = 0x16;
    pub const ORX: u8 = 0x17;
    pub const XORX: u8 = 0x18;
    pub const SHLX: u8 = 0x19;
    pub const SHRX: u8 = 0x1A;
    pub const CMPX: u8 = 0x1B;

    pub const LDA8X: u8 = 0x1C;
    pub const LDA16X: u8 = 0x1D;
    pub const STA8X: u8 = 0x1E;
    pub const STA16X: u8 = 0x1F;
    pub const LDA8XP: u8 = 0x20;
    pub const LDA16XP: u8 = 0x21;
    pub const STA8XP: u8 = 0x22;
    pub const STA16XP: u8 = 0x23;
    pub const TAY: u8 = 0x24;
    pub const TYA: u8 = 0x25;
    pub const INY: u8 = 0x26;
    pub const DEY: u8 = 0x27;
    pub const STA8Y: u8 = 0x28;
    pub const STA16Y: u8 = 0x29;
    pub const STA8YP: u8 = 0x2A;
    pub const STA16YP: u8 = 0x2B;
    pub const LDA8XM: u8 = 0x2C;
    pub const LDA16XM: u8 = 0x2D;
    pub const STA8YM: u8 = 0x2E;
    pub const STA16YM: u8 = 0x2F;

    // Zero-page direct forms: opcode + 8-bit address.
    pub const LDA8Z: u8 = 0x30;
    pub const LDA16Z: u8 = 0x31;
    pub const STA8Z: u8 = 0x32;
    pub const STA16Z: u8 = 0x33;
    pub const EI: u8 = 0x34;
    pub const DI: u8 = 0x35;
    pub const IRET: u8 = 0x36;
    pub const ASR1: u8 = 0x37;
    pub const MULQ15X: u8 = 0x38;
    pub const VEXT: u8 = 0x39; // video-space memory prefix
    pub const ADCX: u8 = 0x3A;
    pub const SBCX: u8 = 0x3B;
    pub const MULHUX: u8 = 0x3C;
    pub const RCR1: u8 = 0x3D;

    pub const LDAI: u8 = 0x40;
    pub const LDXI: u8 = 0x41;
    pub const ADDI: u8 = 0x42;
    pub const SUBI: u8 = 0x43;
    pub const CMPI: u8 = 0x44;
    pub const ANDI: u8 = 0x45;
    pub const ORI: u8 = 0x46;
    pub const XORI: u8 = 0x47;
    pub const LDYI: u8 = 0x48;

    pub const LDA8A: u8 = 0x50;
    pub const LDA16A: u8 = 0x51;
    pub const STA8A: u8 = 0x52;
    pub const STA16A: u8 = 0x53;

    pub const JMP: u8 = 0x60;
    pub const CALL: u8 = 0x61;
    pub const JZ: u8 = 0x62;
    pub const JNZ: u8 = 0x63;
    pub const JC: u8 = 0x64;
    pub const JNC: u8 = 0x65;
    pub const JN: u8 = 0x66;
    pub const JNN: u8 = 0x67;

    // 8-bit PC-relative short control transfers. The displacement is
    // relative to the PC immediately after the two-byte instruction.
    pub const RJMP: u8 = 0x68;
    pub const RCALL: u8 = 0x69;
    pub const RJZ: u8 = 0x6A;
    pub const RJNZ: u8 = 0x6B;
    pub const RJC: u8 = 0x6C;
    pub const RJNC: u8 = 0x6D;
    pub const RJN: u8 = 0x6E;
    pub const RJNN: u8 = 0x6F;
}
