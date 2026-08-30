//! Cost-oriented memory-register ISA.
//! W is the working register. Direct file operands address zero page (0x00..0xFF).
//! FSR0/FSR1 provide full 16-bit indirect addressing with hold/post-inc/pre-dec modes.
pub mod op {
    pub const NOP: u8 = 0x00;
    pub const HALT: u8 = 0x01;
    pub const RET: u8 = 0x02;
    pub const PUSHW: u8 = 0x03;
    pub const POPW: u8 = 0x04;
    pub const INCW: u8 = 0x05;
    pub const DECW: u8 = 0x06;
    pub const NEGW: u8 = 0x07;
    pub const NOTW: u8 = 0x08;
    pub const SHL1W: u8 = 0x09;
    pub const SHR1W: u8 = 0x0A;
    pub const W2F0: u8 = 0x0B;
    pub const W2F1: u8 = 0x0C;
    pub const F02W: u8 = 0x0D;
    pub const F12W: u8 = 0x0E;
    pub const ASR1W: u8 = 0x0F;

    pub const LDI: u8 = 0x10;
    pub const VEXT: u8 = 0x1C;
    pub const EI: u8 = 0x19;
    pub const DI: u8 = 0x1A;
    pub const IRET: u8 = 0x1B;
    pub const FSR0I: u8 = 0x11;
    pub const FSR1I: u8 = 0x12;
    pub const ADDI: u8 = 0x13;
    pub const SUBI: u8 = 0x14;
    pub const CMPI: u8 = 0x15;
    pub const ANDI: u8 = 0x16;
    pub const ORI: u8 = 0x17;
    pub const XORI: u8 = 0x18;

    // direct zero-page, opcode + file8. d=0 forms write W; d=1 forms write file where meaningful.
    pub const MOV8_FW: u8 = 0x20;
    pub const MOV8_WF: u8 = 0x21;
    pub const MOV16_FW: u8 = 0x22;
    pub const MOV16_WF: u8 = 0x23;
    pub const ADD_FW: u8 = 0x24;
    pub const ADD_FF: u8 = 0x25;
    pub const SUB_FW: u8 = 0x26;
    pub const SUB_FF: u8 = 0x27;
    pub const AND_FW: u8 = 0x28;
    pub const AND_FF: u8 = 0x29;
    pub const OR_FW: u8 = 0x2A;
    pub const OR_FF: u8 = 0x2B;
    pub const XOR_FW: u8 = 0x2C;
    pub const XOR_FF: u8 = 0x2D;
    pub const CMP_F: u8 = 0x2E;
    pub const INC_F: u8 = 0x2F; // word file increment; DEC_F is 0x4F

    // FSR0 hold/post-inc/pre-dec, byte then word load/store.
    pub const LDB0: u8 = 0x30;
    pub const LDW0: u8 = 0x31;
    pub const STB0: u8 = 0x32;
    pub const STW0: u8 = 0x33;
    pub const LDB0P: u8 = 0x34;
    pub const LDW0P: u8 = 0x35;
    pub const STB0P: u8 = 0x36;
    pub const STW0P: u8 = 0x37;
    pub const LDB0M: u8 = 0x38;
    pub const LDW0M: u8 = 0x39;
    pub const STB0M: u8 = 0x3A;
    pub const STW0M: u8 = 0x3B;
    // FSR1 same modes.
    pub const LDB1: u8 = 0x3C;
    pub const LDW1: u8 = 0x3D;
    pub const STB1: u8 = 0x3E;
    pub const STW1: u8 = 0x3F;
    pub const LDB1P: u8 = 0x40;
    pub const LDW1P: u8 = 0x41;
    pub const STB1P: u8 = 0x42;
    pub const STW1P: u8 = 0x43;
    pub const LDB1M: u8 = 0x44;
    pub const LDW1M: u8 = 0x45;
    pub const STB1M: u8 = 0x46;
    pub const STW1M: u8 = 0x47;
    pub const SHL_FW: u8 = 0x48;
    pub const SHR_FW: u8 = 0x49;
    pub const MUL_FW: u8 = 0x4A;
    pub const DIV_FW: u8 = 0x4B;
    pub const MOD_FW: u8 = 0x4C;
    pub const MULQ15_FW: u8 = 0x4D;
    pub const DEC_F: u8 = 0x4F;
    pub const ADC_FW: u8 = 0x50;
    pub const ADC_FF: u8 = 0x51;
    pub const SBC_FW: u8 = 0x52;
    pub const SBC_FF: u8 = 0x53;
    pub const MULHU_FW: u8 = 0x54;
    pub const RCR1W: u8 = 0x55;

    pub const JMP: u8 = 0x60;
    pub const CALL: u8 = 0x61;
    pub const JZ: u8 = 0x62;
    pub const JNZ: u8 = 0x63;
    pub const JC: u8 = 0x64;
    pub const JNC: u8 = 0x65;
    pub const JN: u8 = 0x66;
    pub const JNN: u8 = 0x67;
    pub const RJMP: u8 = 0x68;
    pub const RCALL: u8 = 0x69;
    pub const RJZ: u8 = 0x6A;
    pub const RJNZ: u8 = 0x6B;
    pub const RJC: u8 = 0x6C;
    pub const RJNC: u8 = 0x6D;
    pub const RJN: u8 = 0x6E;
    pub const RJNN: u8 = 0x6F;

    // 1-byte hot file forms, address = low nibble (0x00..0x0F).
    pub const HOT_LD8: u8 = 0x80;
    pub const HOT_ST8: u8 = 0x90;
    pub const HOT_LD16: u8 = 0xA0;
    pub const HOT_ST16: u8 = 0xB0;
    pub const HOT_ADDW: u8 = 0xC0;
    pub const HOT_ADDF: u8 = 0xD0;
    pub const HOT_ANDW: u8 = 0xE0;
    pub const HOT_ANDF: u8 = 0xF0;
}
