use crate::{
    error::VmError,
    memory::{Memory, STACK_BOTTOM, STACK_TOP_EXCLUSIVE},
};
use svm_asm::accumulator::instruction::op;

const FLAG_Z: u8 = 1;
const FLAG_N: u8 = 2;
const FLAG_C: u8 = 4;
const FLAG_I: u8 = 8;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StepResult {
    pub halted: bool,
}
#[derive(Debug, Clone)]
pub struct Cpu {
    a: u16,
    x: u16,
    y: u16,
    pc: u16,
    sp: u16,
    flags: u8,
    halted: bool,
}
impl Default for Cpu {
    fn default() -> Self {
        let mut c = Self {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            sp: STACK_TOP_EXCLUSIVE,
            flags: 0,
            halted: false,
        };
        c.reset(0);
        c
    }
}
impl Cpu {
    pub fn reset(&mut self, entry: u16) {
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.pc = entry;
        self.sp = STACK_TOP_EXCLUSIVE;
        self.flags = 0;
        self.halted = false
    }
    pub fn halted(&self) -> bool {
        self.halted
    }
    pub fn pc(&self) -> u16 {
        self.pc
    }
    pub fn accumulator(&self) -> u16 {
        self.a
    }
    pub fn index(&self) -> u16 {
        self.x
    }
    pub fn index_y(&self) -> u16 {
        self.y
    }
    pub fn step(&mut self, m: &mut Memory) -> Result<StepResult, VmError> {
        if self.halted {
            return Ok(StepResult { halted: true });
        }
        m.begin_instruction();
        if self.flags & FLAG_I != 0 && m.irq_active() {
            self.enter_irq(m)?;
        }
        let o = self.fetch8(m);
        self.exec(o, m)?;
        m.retire_instruction();
        Ok(StepResult {
            halted: self.halted,
        })
    }
    fn exec(&mut self, o: u8, m: &mut Memory) -> Result<(), VmError> {
        use op::*;
        match o {
            NOP => {}
            HALT => self.halted = true,
            RET => self.pc = self.pop(m)?,
            EI => self.flags |= FLAG_I,
            DI => self.flags &= !FLAG_I,
            IRET => {
                self.flags = self.pop(m)? as u8;
                self.pc = self.pop(m)?
            }
            ASR1 => {
                self.a = ((self.a as i16) >> 1) as u16;
                self.zn(self.a)
            }
            MULQ15X => {
                m.charge_internal_cycles(17);
                self.a = q15_mul(self.a, self.x);
                self.zn(self.a)
            }
            VEXT => {
                let sub = self.fetch8(m);
                self.exec_video(sub, m)?
            }
            TAX => self.x = self.a,
            TXA => {
                self.a = self.x;
                self.zn(self.a)
            }
            PUSHA => self.push(m, self.a)?,
            POPA => {
                self.a = self.pop(m)?;
                self.zn(self.a)
            }
            PUSHX => self.push(m, self.x)?,
            POPX => self.x = self.pop(m)?,
            INC => self.add(1),
            DEC => self.sub(1),
            NEG => {
                let old = self.a;
                self.a = 0u16.wrapping_sub(old);
                self.zn(self.a);
                self.flag_set(FLAG_C, old == 0)
            }
            NOT => {
                self.a = !self.a;
                self.zn(self.a)
            }
            SHL1 => {
                let old = self.a;
                self.a = old.wrapping_shl(1);
                self.zn(self.a);
                self.flag_set(FLAG_C, old & 0x8000 != 0)
            }
            SHR1 => {
                let old = self.a;
                self.a = old.wrapping_shr(1);
                self.zn(self.a);
                self.flag_set(FLAG_C, old & 1 != 0)
            }
            RCR1 => {
                let old = self.a;
                let cin = self.flags & FLAG_C != 0;
                self.a = (old >> 1) | if cin { 0x8000 } else { 0 };
                self.zn(self.a);
                self.flag_set(FLAG_C, old & 1 != 0)
            }
            ADCX => {
                let cin = u16::from(self.flags & FLAG_C != 0);
                let (t, c1) = self.a.overflowing_add(self.x);
                let (v, c2) = t.overflowing_add(cin);
                self.a = v;
                self.zn(v);
                self.flag_set(FLAG_C, c1 || c2)
            }
            SBCX => {
                let bin = u16::from(self.flags & FLAG_C == 0);
                let (t, b1) = self.a.overflowing_sub(self.x);
                let (v, b2) = t.overflowing_sub(bin);
                self.a = v;
                self.zn(v);
                self.flag_set(FLAG_C, !(b1 || b2))
            }
            MULHUX => {
                m.charge_internal_cycles(16);
                self.a = (((self.a as u32) * (self.x as u32)) >> 16) as u16;
                self.zn(self.a);
                self.flag_set(FLAG_C, false)
            }
            INX => self.x = self.x.wrapping_add(1),
            DEX => self.x = self.x.wrapping_sub(1),
            TAY => self.y = self.a,
            TYA => {
                self.a = self.y;
                self.zn(self.a)
            }
            INY => self.y = self.y.wrapping_add(1),
            DEY => self.y = self.y.wrapping_sub(1),
            ADDX => self.add(self.x),
            SUBX => self.sub(self.x),
            MULX => {
                m.charge_internal_cycles(16);
                self.a = self.a.wrapping_mul(self.x);
                self.zn(self.a);
                self.flag_set(FLAG_C, false)
            }
            DIVX | MODX => {
                if self.x == 0 {
                    return Err(VmError::DivisionByZero);
                }
                m.charge_internal_cycles(16);
                self.a = if o == MODX {
                    self.a % self.x
                } else {
                    self.a / self.x
                };
                self.zn(self.a)
            }
            ANDX => {
                self.a &= self.x;
                self.zn(self.a)
            }
            ORX => {
                self.a |= self.x;
                self.zn(self.a)
            }
            XORX => {
                self.a ^= self.x;
                self.zn(self.a)
            }
            SHLX => {
                m.charge_internal_cycles(1);
                self.a = self.a.wrapping_shl(u32::from(self.x & 15));
                self.zn(self.a)
            }
            SHRX => {
                m.charge_internal_cycles(1);
                self.a = self.a.wrapping_shr(u32::from(self.x & 15));
                self.zn(self.a)
            }
            CMPX => self.cmp(self.x),
            LDA8X => {
                self.a = u16::from(m.read_u8(self.x));
                self.zn(self.a)
            }
            LDA16X => {
                self.a = m.read_u16(self.x)?;
                self.zn(self.a)
            }
            STA8X => m.write_u8(self.x, self.a as u8),
            STA16X => m.write_u16(self.x, self.a)?,
            LDA8XP => {
                let q = self.x;
                self.a = u16::from(m.read_u8(q));
                self.x = q.wrapping_add(1);
                self.zn(self.a)
            }
            LDA16XP => {
                let q = self.x;
                self.a = m.read_u16(q)?;
                self.x = q.wrapping_add(2);
                self.zn(self.a)
            }
            STA8XP => {
                let q = self.x;
                m.write_u8(q, self.a as u8);
                self.x = q.wrapping_add(1)
            }
            STA16XP => {
                let q = self.x;
                m.write_u16(q, self.a)?;
                self.x = q.wrapping_add(2)
            }
            STA8Y => m.write_u8(self.y, self.a as u8),
            STA16Y => m.write_u16(self.y, self.a)?,
            STA8YP => {
                let q = self.y;
                m.write_u8(q, self.a as u8);
                self.y = q.wrapping_add(1)
            }
            STA16YP => {
                let q = self.y;
                m.write_u16(q, self.a)?;
                self.y = q.wrapping_add(2)
            }
            LDA8XM => {
                self.x = self.x.wrapping_sub(1);
                self.a = u16::from(m.read_u8(self.x));
                self.zn(self.a)
            }
            LDA16XM => {
                self.x = self.x.wrapping_sub(2);
                self.a = m.read_u16(self.x)?;
                self.zn(self.a)
            }
            STA8YM => {
                self.y = self.y.wrapping_sub(1);
                m.write_u8(self.y, self.a as u8)
            }
            STA16YM => {
                self.y = self.y.wrapping_sub(2);
                m.write_u16(self.y, self.a)?
            }
            LDA8Z => {
                let q = u16::from(self.fetch8(m));
                self.a = u16::from(m.read_u8(q));
                self.zn(self.a)
            }
            LDA16Z => {
                let q = u16::from(self.fetch8(m));
                self.a = m.read_u16(q)?;
                self.zn(self.a)
            }
            STA8Z => {
                let q = u16::from(self.fetch8(m));
                m.write_u8(q, self.a as u8)
            }
            STA16Z => {
                let q = u16::from(self.fetch8(m));
                m.write_u16(q, self.a)?
            }
            LDAI => {
                self.a = self.fetch16(m);
                self.zn(self.a)
            }
            LDXI => self.x = self.fetch16(m),
            LDYI => self.y = self.fetch16(m),
            ADDI => {
                let v = self.fetch16(m);
                self.add(v)
            }
            SUBI => {
                let v = self.fetch16(m);
                self.sub(v)
            }
            CMPI => {
                let v = self.fetch16(m);
                self.cmp(v)
            }
            ANDI => {
                let v = self.fetch16(m);
                self.a &= v;
                self.zn(self.a)
            }
            ORI => {
                let v = self.fetch16(m);
                self.a |= v;
                self.zn(self.a)
            }
            XORI => {
                let v = self.fetch16(m);
                self.a ^= v;
                self.zn(self.a)
            }
            LDA8A => {
                let q = self.fetch16(m);
                self.a = u16::from(m.read_u8(q));
                self.zn(self.a)
            }
            LDA16A => {
                let q = self.fetch16(m);
                self.a = m.read_u16(q)?;
                self.zn(self.a)
            }
            STA8A => {
                let q = self.fetch16(m);
                m.write_u8(q, self.a as u8)
            }
            STA16A => {
                let q = self.fetch16(m);
                m.write_u16(q, self.a)?
            }
            JMP => self.pc = self.fetch16(m),
            CALL => {
                let q = self.fetch16(m);
                self.push(m, self.pc)?;
                self.pc = q
            }
            JZ | JNZ | JC | JNC | JN | JNN => {
                let q = self.fetch16(m);
                let take = match o {
                    JZ => self.flag(FLAG_Z),
                    JNZ => !self.flag(FLAG_Z),
                    JC => self.flag(FLAG_C),
                    JNC => !self.flag(FLAG_C),
                    JN => self.flag(FLAG_N),
                    JNN => !self.flag(FLAG_N),
                    _ => false,
                };
                if take {
                    self.pc = q
                }
            }
            RJMP => {
                let d = self.fetch8(m) as i8;
                self.pc = self.pc.wrapping_add_signed(i16::from(d))
            }
            RCALL => {
                let d = self.fetch8(m) as i8;
                let ret = self.pc;
                self.push(m, ret)?;
                self.pc = self.pc.wrapping_add_signed(i16::from(d))
            }
            RJZ | RJNZ | RJC | RJNC | RJN | RJNN => {
                let d = self.fetch8(m) as i8;
                let take = match o {
                    RJZ => self.flag(FLAG_Z),
                    RJNZ => !self.flag(FLAG_Z),
                    RJC => self.flag(FLAG_C),
                    RJNC => !self.flag(FLAG_C),
                    RJN => self.flag(FLAG_N),
                    RJNN => !self.flag(FLAG_N),
                    _ => false,
                };
                if take {
                    self.pc = self.pc.wrapping_add_signed(i16::from(d))
                }
            }
            _ => return Err(VmError::InvalidOpcode(o)),
        }
        Ok(())
    }

    fn exec_video(&mut self, sub: u8, m: &mut Memory) -> Result<(), VmError> {
        match sub {
            0x00 => {
                self.a = u16::from(m.video_read_u8(self.x));
                self.zn(self.a)
            }
            0x01 => {
                self.a = m.video_read_u16(self.x)?;
                self.zn(self.a)
            }
            0x02 => m.video_write_u8(self.x, self.a as u8),
            0x03 => m.video_write_u16(self.x, self.a)?,
            0x04 => {
                let q = self.x;
                self.a = u16::from(m.video_read_u8(q));
                self.x = q.wrapping_add(1);
                self.zn(self.a)
            }
            0x05 => {
                let q = self.x;
                self.a = m.video_read_u16(q)?;
                self.x = q.wrapping_add(2);
                self.zn(self.a)
            }
            0x06 => {
                let q = self.x;
                m.video_write_u8(q, self.a as u8);
                self.x = q.wrapping_add(1)
            }
            0x07 => {
                let q = self.x;
                m.video_write_u16(q, self.a)?;
                self.x = q.wrapping_add(2)
            }
            0x08 => m.video_write_u8(self.y, self.a as u8),
            0x09 => m.video_write_u16(self.y, self.a)?,
            0x0A => {
                let q = self.y;
                m.video_write_u8(q, self.a as u8);
                self.y = q.wrapping_add(1)
            }
            0x0B => {
                let q = self.y;
                m.video_write_u16(q, self.a)?;
                self.y = q.wrapping_add(2)
            }
            0x0C => {
                self.x = self.x.wrapping_sub(1);
                self.a = u16::from(m.video_read_u8(self.x));
                self.zn(self.a)
            }
            0x0D => {
                self.x = self.x.wrapping_sub(2);
                self.a = m.video_read_u16(self.x)?;
                self.zn(self.a)
            }
            0x0E => {
                self.y = self.y.wrapping_sub(1);
                m.video_write_u8(self.y, self.a as u8)
            }
            0x0F => {
                self.y = self.y.wrapping_sub(2);
                m.video_write_u16(self.y, self.a)?
            }
            _ => return Err(VmError::InvalidOpcode(sub)),
        }
        Ok(())
    }
    fn fetch8(&mut self, m: &Memory) -> u8 {
        let v = m.read_u8(self.pc);
        self.pc = self.pc.wrapping_add(1);
        v
    }
    fn fetch16(&mut self, m: &Memory) -> u16 {
        u16::from_le_bytes([self.fetch8(m), self.fetch8(m)])
    }
    fn add(&mut self, r: u16) {
        let (v, c) = self.a.overflowing_add(r);
        self.a = v;
        self.zn(v);
        self.flag_set(FLAG_C, c)
    }
    fn sub(&mut self, r: u16) {
        let (v, b) = self.a.overflowing_sub(r);
        self.a = v;
        self.zn(v);
        self.flag_set(FLAG_C, !b)
    }
    fn cmp(&mut self, r: u16) {
        let (v, b) = self.a.overflowing_sub(r);
        self.zn(v);
        self.flag_set(FLAG_C, !b)
    }
    fn enter_irq(&mut self, m: &mut Memory) -> Result<(), VmError> {
        let f = self.flags;
        self.push(m, self.pc)?;
        self.push(m, u16::from(f))?;
        self.flags &= !FLAG_I;
        m.charge_internal_cycles(2);
        self.pc = m.irq_vector();
        Ok(())
    }

    fn push(&mut self, m: &mut Memory, v: u16) -> Result<(), VmError> {
        let n = self.sp.checked_sub(2).ok_or(VmError::StackOverflow)?;
        if n < STACK_BOTTOM {
            return Err(VmError::StackOverflow);
        }
        self.sp = n;
        m.write_u16(n, v)
    }
    fn pop(&mut self, m: &Memory) -> Result<u16, VmError> {
        if self.sp >= STACK_TOP_EXCLUSIVE {
            return Err(VmError::StackUnderflow);
        }
        let v = m.read_u16(self.sp)?;
        self.sp = self.sp.checked_add(2).ok_or(VmError::StackUnderflow)?;
        if self.sp > STACK_TOP_EXCLUSIVE {
            return Err(VmError::StackUnderflow);
        }
        Ok(v)
    }
    fn zn(&mut self, v: u16) {
        self.flag_set(FLAG_Z, v == 0);
        self.flag_set(FLAG_N, v & 0x8000 != 0)
    }
    fn flag(&self, m: u8) -> bool {
        self.flags & m != 0
    }
    fn flag_set(&mut self, m: u8, on: bool) {
        if on {
            self.flags |= m
        } else {
            self.flags &= !m
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use svm_asm::accumulator::assembler::assemble;
    fn run(s: &str) -> (Cpu, Memory) {
        let p = assemble(s).unwrap();
        let mut m = Memory::default();
        m.load(p.load_address, &p.payload).unwrap();
        let mut c = Cpu::default();
        c.reset(p.entry_address);
        while !c.halted() {
            c.step(&mut m).unwrap();
        }
        (c, m)
    }
    #[test]
    fn arithmetic() {
        let (c, _) = run("LDAI 5\nLDXI 7\nADDX\nHALT");
        assert_eq!(c.accumulator(), 12)
    }
    #[test]
    fn post_inc() {
        let (c, m) = run("LDXI 0x2000\nLDAI 0x34\nSTA8 [X+]\nLDAI 0x12\nSTA8 [X+]\nHALT");
        assert_eq!(c.index(), 0x2002);
        assert_eq!(m.read_u16(0x2000).unwrap(), 0x1234)
    }
    #[test]
    fn relaxed_branch_executes() {
        let (c, _) = run("LDAI 0\nCMPI 0\nJZ yes\nLDAI 99\nyes:\nINC\nHALT");
        assert_eq!(c.accumulator(), 1)
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
