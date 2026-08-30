use crate::{
    error::VmError,
    memory::{Memory, STACK_BOTTOM, STACK_TOP_EXCLUSIVE},
};
use svm_asm::register::instruction::{REGISTER_COUNT, decode_register_pair};

const FLAG_Z: u8 = 1 << 0;
const FLAG_N: u8 = 1 << 1;
const FLAG_C: u8 = 1 << 2;
const FLAG_I: u8 = 1 << 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StepResult {
    pub halted: bool,
}

#[derive(Debug, Clone)]
pub struct Cpu {
    registers: [u16; REGISTER_COUNT as usize],
    pc: u16,
    sp: u16,
    flags: u8,
    halted: bool,
}

impl Default for Cpu {
    fn default() -> Self {
        let mut cpu = Self {
            registers: [0; REGISTER_COUNT as usize],
            pc: 0,
            sp: STACK_TOP_EXCLUSIVE,
            flags: 0,
            halted: false,
        };
        cpu.reset(0);
        cpu
    }
}

impl Cpu {
    pub fn reset(&mut self, entry: u16) {
        self.registers = [0; REGISTER_COUNT as usize];
        self.pc = entry;
        self.sp = STACK_TOP_EXCLUSIVE;
        self.flags = 0;
        self.halted = false;
    }

    pub fn halted(&self) -> bool {
        self.halted
    }

    pub fn pc(&self) -> u16 {
        self.pc
    }

    pub fn register(&self, index: usize) -> Option<u16> {
        self.registers.get(index).copied()
    }

    pub fn step(&mut self, memory: &mut Memory) -> Result<StepResult, VmError> {
        if self.halted {
            return Ok(StepResult { halted: true });
        }

        memory.begin_instruction();
        if self.flags & FLAG_I != 0 && memory.irq_active() {
            self.enter_irq(memory)?;
        }
        let opcode = self.fetch_u8(memory);
        self.execute_raw(opcode, memory)?;
        memory.retire_instruction();
        Ok(StepResult {
            halted: self.halted,
        })
    }

    fn execute_raw(&mut self, opcode: u8, memory: &mut Memory) -> Result<(), VmError> {
        match opcode {
            0x00 => return Ok(()),
            0x01 => {
                self.halted = true;
                return Ok(());
            }
            0x02 => {
                self.pc = self.pop_u16(memory)?;
                return Ok(());
            }
            0x03 => {
                let a = u16::from(self.fetch_u8(memory));
                self.registers[0] = u16::from(memory.read_u8(a));
                return Ok(());
            }
            0x04 => {
                let a = u16::from(self.fetch_u8(memory));
                self.registers[0] = memory.read_u16(a)?;
                return Ok(());
            }
            0x05 => {
                let a = u16::from(self.fetch_u8(memory));
                memory.write_u8(a, self.registers[0] as u8);
                return Ok(());
            }
            0x06 => {
                let a = u16::from(self.fetch_u8(memory));
                memory.write_u16(a, self.registers[0])?;
                return Ok(());
            }
            0x07 => {
                self.flags |= FLAG_I;
                return Ok(());
            }
            0x08 => {
                self.flags &= !FLAG_I;
                return Ok(());
            }
            0x09 => {
                self.flags = self.pop_u16(memory)? as u8;
                self.pc = self.pop_u16(memory)?;
                return Ok(());
            }
            0x0A => {
                let r = usize::from(self.fetch_u8(memory));
                if r >= self.registers.len() {
                    return Err(VmError::InvalidOpcode(opcode));
                }
                let v = ((self.registers[r] as i16) >> 1) as u16;
                self.registers[r] = v;
                self.set_zn(v);
                return Ok(());
            }
            0x0B => {
                let (d, q) = self.fetch_register_pair(memory)?;
                memory.charge_internal_cycles(17);
                let v = q15_mul(self.registers[d], self.registers[q]);
                self.registers[d] = v;
                self.set_zn(v);
                self.set_flag(FLAG_C, false);
                return Ok(());
            }
            0x0C => {
                let sub = self.fetch_u8(memory);
                let (first, second) = self.fetch_register_pair(memory)?;
                self.execute_video_memory(sub, first, second, memory)?;
                return Ok(());
            }
            0x0D => {
                let sub = self.fetch_u8(memory);
                let raw = self.fetch_u8(memory);
                if sub == 3 {
                    let r = usize::from(raw);
                    if r >= self.registers.len() {
                        return Err(VmError::InvalidOpcode(opcode));
                    }
                    let old = self.registers[r];
                    let cin = self.flag(FLAG_C);
                    let v = (old >> 1) | if cin { 0x8000 } else { 0 };
                    self.registers[r] = v;
                    self.set_zn(v);
                    self.set_flag(FLAG_C, old & 1 != 0);
                    return Ok(());
                }
                let (a, b) =
                    decode_register_pair(raw).ok_or(VmError::InvalidRegisterEncoding(raw))?;
                let (a, b) = (usize::from(a), usize::from(b));
                let x = self.registers[a];
                let y = self.registers[b];
                let v = match sub {
                    0 => {
                        let cin = u16::from(self.flag(FLAG_C));
                        let (t, c1) = x.overflowing_add(y);
                        let (v, c2) = t.overflowing_add(cin);
                        self.set_flag(FLAG_C, c1 || c2);
                        v
                    }
                    1 => {
                        let bin = u16::from(!self.flag(FLAG_C));
                        let (t, b1) = x.overflowing_sub(y);
                        let (v, b2) = t.overflowing_sub(bin);
                        self.set_flag(FLAG_C, !(b1 || b2));
                        v
                    }
                    2 => {
                        memory.charge_internal_cycles(16);
                        self.set_flag(FLAG_C, false);
                        (((x as u32) * (y as u32)) >> 16) as u16
                    }
                    _ => return Err(VmError::InvalidOpcode(opcode)),
                };
                self.registers[a] = v;
                self.set_zn(v);
                return Ok(());
            }
            0x0D..=0x0F => return Err(VmError::InvalidOpcode(opcode)),
            _ => {}
        }

        // 0x10..0x4F: unary families, lower 3 bits are the register.
        if (0x10..=0x4F).contains(&opcode) {
            let register = usize::from(opcode & 0x07);
            match opcode & 0xF8 {
                0x10 => {
                    let value = !self.registers[register];
                    self.registers[register] = value;
                    self.set_zn(value);
                }
                0x18 => {
                    let original = self.registers[register];
                    let value = 0u16.wrapping_sub(original);
                    self.registers[register] = value;
                    self.set_zn(value);
                    self.set_flag(FLAG_C, original == 0);
                }
                0x20 => self.add(register, 1),
                0x28 => self.sub(register, 1),
                0x30 => {
                    let old = self.registers[register];
                    let value = old.wrapping_shl(1);
                    self.registers[register] = value;
                    self.set_zn(value);
                    self.set_flag(FLAG_C, old & 0x8000 != 0);
                }
                0x38 => {
                    let old = self.registers[register];
                    let value = old.wrapping_shr(1);
                    self.registers[register] = value;
                    self.set_zn(value);
                    self.set_flag(FLAG_C, old & 1 != 0);
                }
                0x40 => self.push_u16(memory, self.registers[register])?,
                0x48 => self.registers[register] = self.pop_u16(memory)?,
                _ => unreachable!(),
            }
            return Ok(());
        }

        // 0x50..0xBF: compact R0..R3 two-register forms.
        if (0x50..=0xBF).contains(&opcode) {
            let destination = usize::from((opcode >> 2) & 0x03);
            let source = usize::from(opcode & 0x03);
            match opcode & 0xF0 {
                0x50 => self.registers[destination] = self.registers[source],
                0x60 => self.add(destination, self.registers[source]),
                0x70 => self.sub(destination, self.registers[source]),
                0x80 => self.compare(self.registers[destination], self.registers[source]),
                0x90 => {
                    self.registers[destination] = u16::from(memory.read_u8(self.registers[source]));
                }
                0xA0 => memory.write_u8(self.registers[destination], self.registers[source] as u8),
                0xB0 => {
                    let value = self.registers[destination] & self.registers[source];
                    self.registers[destination] = value;
                    self.set_zn(value);
                }
                _ => unreachable!(),
            }
            return Ok(());
        }

        // 0xC0..0xDF: immediate16 families, lower 3 bits are the register.
        if (0xC0..=0xDF).contains(&opcode) {
            let register = usize::from(opcode & 0x07);
            let value = self.fetch_u16(memory);
            match opcode & 0xF8 {
                0xC0 => self.registers[register] = value,
                0xC8 => self.add(register, value),
                0xD0 => self.sub(register, value),
                0xD8 => self.compare(self.registers[register], value),
                _ => unreachable!(),
            }
            return Ok(());
        }

        // 0xE0..0xEF: full R0..R7 two-register/memory forms.
        if (0xE0..=0xEF).contains(&opcode) {
            let (first, second) = self.fetch_register_pair(memory)?;
            match opcode {
                0xE0 => self.registers[first] = self.registers[second],
                0xE1 => self.add(first, self.registers[second]),
                0xE2 => self.sub(first, self.registers[second]),
                0xE3 => {
                    memory.charge_internal_cycles(16);
                    let value = self.registers[first].wrapping_mul(self.registers[second]);
                    self.registers[first] = value;
                    self.set_zn(value);
                    self.set_flag(FLAG_C, false);
                }
                0xE4 | 0xE5 => {
                    let rhs = self.registers[second];
                    if rhs == 0 {
                        return Err(VmError::DivisionByZero);
                    }
                    memory.charge_internal_cycles(16);
                    let value = if opcode == 0xE5 {
                        self.registers[first] % rhs
                    } else {
                        self.registers[first] / rhs
                    };
                    self.registers[first] = value;
                    self.set_zn(value);
                }
                0xE6 => {
                    let value = self.registers[first] & self.registers[second];
                    self.registers[first] = value;
                    self.set_zn(value);
                }
                0xE7 => {
                    let value = self.registers[first] | self.registers[second];
                    self.registers[first] = value;
                    self.set_zn(value);
                }
                0xE8 => {
                    let value = self.registers[first] ^ self.registers[second];
                    self.registers[first] = value;
                    self.set_zn(value);
                }
                0xE9 | 0xEA => {
                    memory.charge_internal_cycles(1);
                    let count = u32::from(self.registers[second] & 15);
                    let original = self.registers[first];
                    let value = if opcode == 0xE9 {
                        original.wrapping_shl(count)
                    } else {
                        original.wrapping_shr(count)
                    };
                    self.registers[first] = value;
                    self.set_zn(value);
                }
                0xEB => self.compare(self.registers[first], self.registers[second]),
                0xEC => {
                    self.registers[first] = u16::from(memory.read_u8(self.registers[second]));
                }
                0xED => self.registers[first] = memory.read_u16(self.registers[second])?,
                0xEE => memory.write_u8(self.registers[first], self.registers[second] as u8),
                0xEF => memory.write_u16(self.registers[first], self.registers[second])?,
                _ => unreachable!(),
            }
            return Ok(());
        }

        // 0xF8..0xFB: post-increment indirect memory forms.
        if (0xF8..=0xFB).contains(&opcode) {
            let (first, second) = self.fetch_register_pair(memory)?;
            match opcode {
                0xF8 => {
                    if first == second {
                        return Err(VmError::InvalidRegisterEncoding(
                            ((first as u8) << 3) | second as u8,
                        ));
                    }
                    let address = self.registers[second];
                    self.registers[first] = u16::from(memory.read_u8(address));
                    self.registers[second] = address.wrapping_add(1);
                }
                0xF9 => {
                    let address = self.registers[first];
                    memory.write_u8(address, self.registers[second] as u8);
                    self.registers[first] = address.wrapping_add(1);
                }
                0xFA => {
                    if first == second {
                        return Err(VmError::InvalidRegisterEncoding(
                            ((first as u8) << 3) | second as u8,
                        ));
                    }
                    let address = self.registers[second];
                    self.registers[first] = memory.read_u16(address)?;
                    self.registers[second] = address.wrapping_add(2);
                }
                0xFB => {
                    let address = self.registers[first];
                    memory.write_u16(address, self.registers[second])?;
                    self.registers[first] = address.wrapping_add(2);
                }
                _ => unreachable!(),
            }
            return Ok(());
        }

        // 0xFC..0xFF: pre-decrement indirect memory forms.
        // The address register is decremented by access width before the access.
        if (0xFC..=0xFF).contains(&opcode) {
            let (first, second) = self.fetch_register_pair(memory)?;
            match opcode {
                0xFC => {
                    if first == second {
                        return Err(VmError::InvalidRegisterEncoding(
                            ((first as u8) << 3) | second as u8,
                        ));
                    }
                    let address = self.registers[second].wrapping_sub(1);
                    self.registers[second] = address;
                    self.registers[first] = u16::from(memory.read_u8(address));
                }
                0xFD => {
                    let address = self.registers[first].wrapping_sub(1);
                    self.registers[first] = address;
                    memory.write_u8(address, self.registers[second] as u8);
                }
                0xFE => {
                    if first == second {
                        return Err(VmError::InvalidRegisterEncoding(
                            ((first as u8) << 3) | second as u8,
                        ));
                    }
                    let address = self.registers[second].wrapping_sub(2);
                    self.registers[second] = address;
                    self.registers[first] = memory.read_u16(address)?;
                }
                0xFF => {
                    let address = self.registers[first].wrapping_sub(2);
                    self.registers[first] = address;
                    memory.write_u16(address, self.registers[second])?;
                }
                _ => unreachable!(),
            }
            return Ok(());
        }

        // 0xF0..0xF7: absolute branches and CALL.
        match opcode {
            0xF0 => self.pc = self.fetch_u16(memory),
            0xF1 => {
                let target = self.fetch_u16(memory);
                self.push_u16(memory, self.pc)?;
                self.pc = target;
            }
            0xF2 => self.branch(memory, self.flag(FLAG_Z)),
            0xF3 => self.branch(memory, !self.flag(FLAG_Z)),
            0xF4 => self.branch(memory, self.flag(FLAG_C)),
            0xF5 => self.branch(memory, !self.flag(FLAG_C)),
            0xF6 => self.branch(memory, self.flag(FLAG_N)),
            0xF7 => self.branch(memory, !self.flag(FLAG_N)),
            _ => return Err(VmError::InvalidOpcode(opcode)),
        }
        Ok(())
    }

    fn execute_video_memory(
        &mut self,
        sub: u8,
        first: usize,
        second: usize,
        memory: &mut Memory,
    ) -> Result<(), VmError> {
        match sub {
            0x00 => self.registers[first] = u16::from(memory.video_read_u8(self.registers[second])),
            0x01 => self.registers[first] = memory.video_read_u16(self.registers[second])?,
            0x02 => memory.video_write_u8(self.registers[first], self.registers[second] as u8),
            0x03 => memory.video_write_u16(self.registers[first], self.registers[second])?,
            0x04 => {
                if first == second {
                    return Err(VmError::InvalidRegisterEncoding(
                        ((first as u8) << 3) | second as u8,
                    ));
                }
                let a = self.registers[second];
                self.registers[first] = u16::from(memory.video_read_u8(a));
                self.registers[second] = a.wrapping_add(1);
            }
            0x05 => {
                if first == second {
                    return Err(VmError::InvalidRegisterEncoding(
                        ((first as u8) << 3) | second as u8,
                    ));
                }
                let a = self.registers[second];
                self.registers[first] = memory.video_read_u16(a)?;
                self.registers[second] = a.wrapping_add(2);
            }
            0x06 => {
                let a = self.registers[first];
                memory.video_write_u8(a, self.registers[second] as u8);
                self.registers[first] = a.wrapping_add(1);
            }
            0x07 => {
                let a = self.registers[first];
                memory.video_write_u16(a, self.registers[second])?;
                self.registers[first] = a.wrapping_add(2);
            }
            0x08 => {
                if first == second {
                    return Err(VmError::InvalidRegisterEncoding(
                        ((first as u8) << 3) | second as u8,
                    ));
                }
                let a = self.registers[second].wrapping_sub(1);
                self.registers[second] = a;
                self.registers[first] = u16::from(memory.video_read_u8(a));
            }
            0x09 => {
                if first == second {
                    return Err(VmError::InvalidRegisterEncoding(
                        ((first as u8) << 3) | second as u8,
                    ));
                }
                let a = self.registers[second].wrapping_sub(2);
                self.registers[second] = a;
                self.registers[first] = memory.video_read_u16(a)?;
            }
            0x0A => {
                let a = self.registers[first].wrapping_sub(1);
                self.registers[first] = a;
                memory.video_write_u8(a, self.registers[second] as u8);
            }
            0x0B => {
                let a = self.registers[first].wrapping_sub(2);
                self.registers[first] = a;
                memory.video_write_u16(a, self.registers[second])?;
            }
            _ => return Err(VmError::InvalidOpcode(0x0C)),
        }
        Ok(())
    }
    fn fetch_u8(&mut self, memory: &Memory) -> u8 {
        let value = memory.read_u8(self.pc);
        self.pc = self.pc.wrapping_add(1);
        value
    }

    fn fetch_u16(&mut self, memory: &Memory) -> u16 {
        u16::from_le_bytes([self.fetch_u8(memory), self.fetch_u8(memory)])
    }

    fn fetch_register_pair(&mut self, memory: &Memory) -> Result<(usize, usize), VmError> {
        let raw = self.fetch_u8(memory);
        let (first, second) =
            decode_register_pair(raw).ok_or(VmError::InvalidRegisterEncoding(raw))?;
        Ok((usize::from(first), usize::from(second)))
    }

    fn add(&mut self, destination: usize, rhs: u16) {
        let lhs = self.registers[destination];
        let (value, carry) = lhs.overflowing_add(rhs);
        self.registers[destination] = value;
        self.set_zn(value);
        self.set_flag(FLAG_C, carry);
    }

    fn sub(&mut self, destination: usize, rhs: u16) {
        let lhs = self.registers[destination];
        let (value, borrow) = lhs.overflowing_sub(rhs);
        self.registers[destination] = value;
        self.set_zn(value);
        self.set_flag(FLAG_C, !borrow);
    }

    fn compare(&mut self, lhs: u16, rhs: u16) {
        let (value, borrow) = lhs.overflowing_sub(rhs);
        self.set_zn(value);
        self.set_flag(FLAG_C, !borrow);
    }

    fn branch(&mut self, memory: &Memory, take: bool) {
        let target = self.fetch_u16(memory);
        if take {
            self.pc = target;
        }
    }

    fn enter_irq(&mut self, memory: &mut Memory) -> Result<(), VmError> {
        let saved_flags = self.flags;
        self.push_u16(memory, self.pc)?;
        self.push_u16(memory, u16::from(saved_flags))?;
        self.flags &= !FLAG_I;
        memory.charge_internal_cycles(2);
        self.pc = memory.irq_vector();
        Ok(())
    }

    fn push_u16(&mut self, memory: &mut Memory, value: u16) -> Result<(), VmError> {
        let next_sp = self.sp.checked_sub(2).ok_or(VmError::StackOverflow)?;
        if next_sp < STACK_BOTTOM {
            return Err(VmError::StackOverflow);
        }
        self.sp = next_sp;
        memory.write_u16(self.sp, value)
    }

    fn pop_u16(&mut self, memory: &Memory) -> Result<u16, VmError> {
        if self.sp >= STACK_TOP_EXCLUSIVE {
            return Err(VmError::StackUnderflow);
        }
        let value = memory.read_u16(self.sp)?;
        self.sp = self.sp.checked_add(2).ok_or(VmError::StackUnderflow)?;
        if self.sp > STACK_TOP_EXCLUSIVE {
            return Err(VmError::StackUnderflow);
        }
        Ok(value)
    }

    fn set_zn(&mut self, value: u16) {
        self.set_flag(FLAG_Z, value == 0);
        self.set_flag(FLAG_N, (value & 0x8000) != 0);
    }

    fn flag(&self, mask: u8) -> bool {
        self.flags & mask != 0
    }

    fn set_flag(&mut self, mask: u8, set: bool) {
        if set {
            self.flags |= mask;
        } else {
            self.flags &= !mask;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use svm_asm::register::assembler::assemble;

    fn run_asm(source: &str) -> (Cpu, Memory) {
        let program = assemble(source).unwrap();
        let mut memory = Memory::default();
        memory.load(program.load_address, &program.payload).unwrap();
        let mut cpu = Cpu::default();
        cpu.reset(program.entry_address);
        while !cpu.halted() {
            cpu.step(&mut memory).unwrap();
        }
        (cpu, memory)
    }

    #[test]
    fn compact_and_general_register_forms_execute_identically() {
        let (cpu, _) =
            run_asm("MOVI R0, 1\nMOVI R1, 2\nADD R0, R1\nMOVI R6, 4\nMOVI R7, 5\nADD R6, R7\nHALT");
        assert_eq!(cpu.register(0), Some(3));
        assert_eq!(cpu.register(6), Some(9));
    }

    #[test]
    fn register_v3_compact_and_and_subi_execute() {
        let (cpu, _) = run_asm("MOVI R0,0xF0F0\nMOVI R1,0x00F0\nAND R0,R1\nSUBI R0,0x0010\nHALT");
        assert_eq!(cpu.register(0), Some(0x00E0));
    }

    #[test]
    fn immediate_register_is_embedded_in_opcode() {
        let (cpu, _) = run_asm("MOVI R7, 0x1234\nINC R7\nHALT");
        assert_eq!(cpu.register(7), Some(0x1235));
    }

    #[test]
    fn video_store8_draws_into_framebuffer() {
        let (cpu, memory) = run_asm("MOVI R0, 0x0000\nMOVI R1, 0x00E0\nVSTORE8 [R0], R1\nHALT");
        assert!(cpu.halted());
        assert_eq!(memory.framebuffer()[0], 0xE0);
    }

    #[test]
    fn full_load16_store16_are_little_endian() {
        let (cpu, memory) =
            run_asm("MOVI R4, 0x2000\nMOVI R5, 0x1234\nSTORE16 [R4], R5\nLOAD16 R6, [R4]\nHALT");
        assert_eq!(cpu.register(6), Some(0x1234));
        assert_eq!(memory.read_u8(0x2000), 0x34);
        assert_eq!(memory.read_u8(0x2001), 0x12);
    }

    #[test]
    fn division_by_zero_is_an_error() {
        let program = assemble("MOVI R4, 10\nMOVI R5, 0\nDIV R4, R5").unwrap();
        let mut memory = Memory::default();
        memory.load(program.load_address, &program.payload).unwrap();
        let mut cpu = Cpu::default();
        cpu.step(&mut memory).unwrap();
        cpu.step(&mut memory).unwrap();
        assert!(matches!(
            cpu.step(&mut memory),
            Err(VmError::DivisionByZero)
        ));
    }

    #[test]
    fn reserved_opcode_is_rejected() {
        let mut memory = Memory::default();
        memory.load(0, &[0x0E]).unwrap();
        let mut cpu = Cpu::default();
        assert!(matches!(
            cpu.step(&mut memory),
            Err(VmError::InvalidOpcode(0x0E))
        ));
    }

    #[test]
    fn reserved_bits_in_full_register_pair_are_rejected() {
        let mut memory = Memory::default();
        memory.load(0, &[0xE1, 0x80]).unwrap();
        let mut cpu = Cpu::default();
        assert!(matches!(
            cpu.step(&mut memory),
            Err(VmError::InvalidRegisterEncoding(0x80))
        ));
    }

    #[test]
    fn stack_is_confined_to_its_reserved_region() {
        let mut memory = Memory::default();
        let mut cpu = Cpu::default();
        let capacity = ((STACK_TOP_EXCLUSIVE - STACK_BOTTOM) / 2) as usize;
        for _ in 0..capacity {
            cpu.push_u16(&mut memory, 0x1234).unwrap();
        }
        assert!(matches!(
            cpu.push_u16(&mut memory, 0x5678),
            Err(VmError::StackOverflow)
        ));
    }

    #[test]
    fn program_counter_wraps_during_instruction_fetch() {
        let mut memory = Memory::default();
        memory.write_u8(0xFFFF, 0x00);
        let mut cpu = Cpu::default();
        cpu.reset(0xFFFF);
        cpu.step(&mut memory).unwrap();
        assert_eq!(cpu.pc(), 0x0000);
    }
    #[test]
    fn post_increment_memory_access_updates_address_register_by_width() {
        let (cpu, memory) = run_asm(
            "MOVI R0,0x2000\nMOVI R1,0x005A\nSTORE8 [R0+],R1\nMOVI R2,0x1234\nSTORE16 [R0+],R2\nMOVI R3,0x2000\nLOAD8 R4,[R3+]\nLOAD16 R5,[R3+]\nHALT",
        );
        assert_eq!(memory.read_u8(0x2000), 0x5A);
        assert_eq!(memory.read_u16(0x2001).unwrap(), 0x1234);
        assert_eq!(cpu.register(0), Some(0x2003));
        assert_eq!(cpu.register(3), Some(0x2003));
        assert_eq!(cpu.register(4), Some(0x005A));
        assert_eq!(cpu.register(5), Some(0x1234));
    }
}

fn q15_mul(a: u16, b: u16) -> u16 {
    let p = (a as i16 as i32) * (b as i16 as i32);
    if p == 1073741824 {
        0x7FFF
    } else {
        (p >> 15) as i16 as u16
    }
}
