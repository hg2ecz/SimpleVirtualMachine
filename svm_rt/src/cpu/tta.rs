use crate::{error::VmError, memory::Memory};

const Z: u8 = 1;
const N: u8 = 2;
const C: u8 = 4;
const I: u8 = 8;

#[derive(Clone)]
pub struct Cpu {
    r: [u16; 8],
    pc: u16,
    flags: u8,
    halted: bool,
    alu_x: u16,
    alu_out: u16,
    mem_addr: u16,
    vmem_addr: u16,
    control_sp: u16,
    data_sp: u16,
}
impl Default for Cpu {
    fn default() -> Self {
        Self {
            r: [0; 8],
            pc: 0,
            flags: 0,
            halted: false,
            alu_x: 0,
            alu_out: 0,
            mem_addr: 0,
            vmem_addr: 0,
            control_sp: 0xFF00,
            data_sp: 0xFD00,
        }
    }
}
impl Cpu {
    pub fn reset(&mut self, entry: u16) {
        *self = Self::default();
        self.pc = entry
    }
    pub fn halted(&self) -> bool {
        self.halted
    }
    fn f16(&mut self, m: &Memory) -> Result<u16, VmError> {
        let v = m.read_u16(self.pc)?;
        self.pc = self.pc.wrapping_add(2);
        Ok(v)
    }
    fn flag(&mut self, f: u8, on: bool) {
        if on {
            self.flags |= f
        } else {
            self.flags &= !f
        }
    }
    fn zn(&mut self, v: u16) {
        self.flag(Z, v == 0);
        self.flag(N, v & 0x8000 != 0)
    }
    fn set_out(&mut self, v: u16) {
        self.alu_out = v;
        self.zn(v)
    }
    fn push_ctl(&mut self, m: &mut Memory, v: u16) -> Result<(), VmError> {
        if self.control_sp < 0xFD02 {
            return Err(VmError::StackOverflow);
        }
        self.control_sp = self.control_sp.wrapping_sub(2);
        m.write_u16(self.control_sp, v)
    }
    fn pop_ctl(&mut self, m: &Memory) -> Result<u16, VmError> {
        if self.control_sp >= 0xFF00 {
            return Err(VmError::StackUnderflow);
        }
        let v = m.read_u16(self.control_sp)?;
        self.control_sp = self.control_sp.wrapping_add(2);
        Ok(v)
    }
    fn push_data(&mut self, m: &mut Memory, v: u16) -> Result<(), VmError> {
        if self.data_sp < 0xFB02 {
            return Err(VmError::StackOverflow);
        }
        self.data_sp = self.data_sp.wrapping_sub(2);
        m.write_u16(self.data_sp, v)
    }
    fn pop_data(&mut self, m: &Memory) -> Result<u16, VmError> {
        if self.data_sp >= 0xFD00 {
            return Err(VmError::StackUnderflow);
        }
        let v = m.read_u16(self.data_sp)?;
        self.data_sp = self.data_sp.wrapping_add(2);
        Ok(v)
    }
    fn enter_irq(&mut self, m: &mut Memory) -> Result<(), VmError> {
        let pc = self.pc;
        let fl = self.flags;
        self.push_ctl(m, pc)?;
        self.push_ctl(m, u16::from(fl))?;
        self.flags &= !I;
        self.pc = m.irq_vector();
        Ok(())
    }
    fn source(&mut self, m: &mut Memory, s: u8) -> Result<u16, VmError> {
        Ok(match s {
            0..=7 => self.r[s as usize],
            8 => self.alu_out,
            9 => u16::from(m.read_u8(self.mem_addr)),
            10 => m.read_u16(self.mem_addr)?,
            11 => u16::from(m.video_read_u8(self.vmem_addr)),
            12 => m.video_read_u16(self.vmem_addr)?,
            13 => self.pop_data(m)?,
            14 => self.pop_ctl(m)?,
            15 => {
                self.flags = self.pop_ctl(m)? as u8;
                self.pop_ctl(m)?
            }
            16 => u16::from(self.flags),
            17 => 0,
            _ => return Err(VmError::InvalidOpcode(s)),
        })
    }
    fn alu_binary(&mut self, m: &mut Memory, d: u8, y: u16) -> Result<(), VmError> {
        let x = self.alu_x;
        let v = match d {
            9 => {
                let (v, c) = x.overflowing_add(y);
                self.flag(C, c);
                v
            }
            10 => {
                let cin = u16::from(self.flags & C != 0);
                let (t, c1) = x.overflowing_add(y);
                let (v, c2) = t.overflowing_add(cin);
                self.flag(C, c1 || c2);
                v
            }
            11 => {
                let (v, b) = x.overflowing_sub(y);
                self.flag(C, !b);
                v
            }
            12 => {
                let bin = u16::from(self.flags & C == 0);
                let (t, b1) = x.overflowing_sub(y);
                let (v, b2) = t.overflowing_sub(bin);
                self.flag(C, !(b1 || b2));
                v
            }
            13 => x & y,
            14 => x | y,
            15 => x ^ y,
            16 => {
                m.charge_internal_cycles(16);
                x.wrapping_mul(y)
            }
            17 => {
                m.charge_internal_cycles(16);
                (((x as u32) * (y as u32)) >> 16) as u16
            }
            18 => {
                m.charge_internal_cycles(17);
                let p = (x as i16 as i32) * (y as i16 as i32);
                let mut q = p >> 15;
                if q > 32767 {
                    q = 32767
                }
                if q < -32768 {
                    q = -32768
                }
                q as i16 as u16
            }
            19 => {
                if y == 0 {
                    return Err(VmError::DivisionByZero);
                }
                m.charge_internal_cycles(16);
                x / y
            }
            20 => {
                if y == 0 {
                    return Err(VmError::DivisionByZero);
                }
                m.charge_internal_cycles(16);
                x % y
            }
            21 => x.wrapping_shl((y & 15) as u32),
            22 => x.wrapping_shr((y & 15) as u32),
            23 => {
                let (v, b) = x.overflowing_sub(y);
                self.flag(C, !b);
                v
            }
            _ => return Err(VmError::InvalidOpcode(d)),
        };
        self.set_out(v);
        Ok(())
    }
    fn dest(&mut self, m: &mut Memory, d: u8, v: u16) -> Result<(), VmError> {
        match d {
            0..=7 => self.r[d as usize] = v,
            8 => self.alu_x = v,
            9..=23 => self.alu_binary(m, d, v)?,
            24 => self.set_out(!v),
            25 => self.set_out(0u16.wrapping_sub(v)),
            26 => self.set_out((v as i16 >> 1) as u16),
            27 => {
                self.flag(C, v & 0x8000 != 0);
                self.set_out(v << 1)
            }
            28 => {
                self.flag(C, v & 1 != 0);
                self.set_out(v >> 1)
            }
            29 => {
                let cin = self.flags & C != 0;
                let out = (v >> 1) | if cin { 0x8000 } else { 0 };
                self.flag(C, v & 1 != 0);
                self.set_out(out)
            }
            30 => self.mem_addr = v,
            31 => m.write_u8(self.mem_addr, v as u8),
            32 => m.write_u16(self.mem_addr, v)?,
            33 => self.vmem_addr = v,
            34 => m.video_write_u8(self.vmem_addr, v as u8),
            35 => m.video_write_u16(self.vmem_addr, v)?,
            36 => self.pc = v,
            37 => {
                if self.flags & Z != 0 {
                    self.pc = v
                }
            }
            38 => {
                if self.flags & Z == 0 {
                    self.pc = v
                }
            }
            39 => {
                if self.flags & C != 0 {
                    self.pc = v
                }
            }
            40 => {
                if self.flags & C == 0 {
                    self.pc = v
                }
            }
            41 => {
                if self.flags & N != 0 {
                    self.pc = v
                }
            }
            42 => {
                if self.flags & N == 0 {
                    self.pc = v
                }
            }
            43 => {
                self.push_ctl(m, self.pc)?;
                self.pc = v
            }
            44 => self.halted = true,
            45 => self.flags |= I,
            46 => self.flags &= !I,
            47 => self.push_data(m, v)?,
            _ => return Err(VmError::InvalidOpcode(d)),
        }
        Ok(())
    }
    pub fn step(&mut self, m: &mut Memory) -> Result<(), VmError> {
        if self.halted {
            return Ok(());
        }
        m.begin_instruction();
        if self.flags & I != 0 && m.irq_active() {
            self.enter_irq(m)?
        }
        let w = m.read_u16(self.pc)?;
        self.pc = self.pc.wrapping_add(2);
        let s = ((w >> 6) & 0x3f) as u8;
        let d = (w & 0x3f) as u8;
        let v = if s == 63 {
            self.f16(m)?
        } else {
            self.source(m, s)?
        };
        self.dest(m, d, v)?;
        m.retire_instruction();
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use svm_asm::tta::assembler::assemble;
    #[test]
    fn transported_add() {
        let p = assemble("MOV 10,R0\nMOV R0,ALU.X\nMOV 20,ALU.ADD\nMOV ALU.OUT,R1\nHALT").unwrap();
        let mut m = Memory::default();
        m.load(p.load_address, &p.payload).unwrap();
        let mut c = Cpu::default();
        c.reset(p.entry_address);
        while !c.halted() {
            c.step(&mut m).unwrap()
        }
        assert_eq!(c.r[1], 30);
    }
}
