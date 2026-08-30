use crate::{error::VmError, memory::Memory};
const Z: u8 = 1;
const N: u8 = 2;
const C: u8 = 4;
const I: u8 = 8;
#[derive(Clone)]
pub struct Cpu {
    b: [u16; 8],
    pc: u16,
    flags: u8,
    halted: bool,
    control_sp: u16,
    data_sp: u16,
}
impl Default for Cpu {
    fn default() -> Self {
        Self {
            b: [0; 8],
            pc: 0,
            flags: 0,
            halted: false,
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
    fn push_belt(&mut self, v: u16) {
        for i in (1..8).rev() {
            self.b[i] = self.b[i - 1]
        }
        self.b[0] = v;
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
        let maj = w >> 12;
        match maj {
            0 => match w & 0xff {
                0 => {}
                1 => self.halted = true,
                2 => self.pc = self.pop_ctl(m)?,
                3 => self.flags |= I,
                4 => self.flags &= !I,
                5 => {
                    self.flags = self.pop_ctl(m)? as u8;
                    self.pc = self.pop_ctl(m)?
                }
                _ => return Err(VmError::InvalidOpcode((w >> 8) as u8)),
            },
            1 => {
                let v = self.f16(m)?;
                self.push_belt(v)
            }
            2 => {
                let a = self.f16(m)?;
                let v = if w & 0x0800 != 0 {
                    m.read_u16(a)?
                } else {
                    u16::from(m.read_u8(a))
                };
                self.push_belt(v)
            }
            3 => {
                let i = ((w >> 8) & 7) as usize;
                let a = self.f16(m)?;
                if w & 0x0800 != 0 {
                    m.write_u16(a, self.b[i])?
                } else {
                    m.write_u8(a, self.b[i] as u8)
                }
            }
            4 => {
                let f = (w >> 8) & 15;
                let ai = ((w >> 5) & 7) as usize;
                let bi = ((w >> 2) & 7) as usize;
                let x = self.b[ai];
                let y = self.b[bi];
                let v = match f {
                    0 => {
                        let (v, c) = x.overflowing_add(y);
                        self.flag(C, c);
                        v
                    }
                    1 => {
                        let (v, b) = x.overflowing_sub(y);
                        self.flag(C, !b);
                        v
                    }
                    2 => x & y,
                    3 => x | y,
                    4 => x ^ y,
                    5 => {
                        m.charge_internal_cycles(16);
                        x.wrapping_mul(y)
                    }
                    6 => {
                        if y == 0 {
                            return Err(VmError::DivisionByZero);
                        }
                        m.charge_internal_cycles(16);
                        x / y
                    }
                    7 => {
                        if y == 0 {
                            return Err(VmError::DivisionByZero);
                        }
                        m.charge_internal_cycles(16);
                        x % y
                    }
                    8 => x.wrapping_shl((y & 15) as u32),
                    9 => x.wrapping_shr((y & 15) as u32),
                    10 => {
                        let (v, b) = x.overflowing_sub(y);
                        self.flag(C, !b);
                        v
                    }
                    11 => {
                        let cin = u16::from(self.flags & C != 0);
                        let (t, c1) = x.overflowing_add(y);
                        let (v, c2) = t.overflowing_add(cin);
                        self.flag(C, c1 || c2);
                        v
                    }
                    12 => {
                        let bin = u16::from(self.flags & C == 0);
                        let (t, b1) = x.overflowing_sub(y);
                        let (v, b2) = t.overflowing_sub(bin);
                        self.flag(C, !(b1 || b2));
                        v
                    }
                    13 => {
                        m.charge_internal_cycles(16);
                        (((x as u32) * (y as u32)) >> 16) as u16
                    }
                    14 => {
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
                    _ => return Err(VmError::InvalidOpcode((w >> 8) as u8)),
                };
                self.push_belt(v)
            }
            5 => {
                let f = (w >> 8) & 15;
                let i = ((w >> 5) & 7) as usize;
                let x = self.b[i];
                let v = match f {
                    0 => x,
                    1 => !x,
                    2 => 0u16.wrapping_sub(x),
                    3 => (x as i16 >> 1) as u16,
                    4 => {
                        self.flag(C, x & 0x8000 != 0);
                        x << 1
                    }
                    5 => {
                        self.flag(C, x & 1 != 0);
                        x >> 1
                    }
                    6 => {
                        let cin = self.flags & C != 0;
                        let v = (x >> 1) | if cin { 0x8000 } else { 0 };
                        self.flag(C, x & 1 != 0);
                        v
                    }
                    _ => return Err(VmError::InvalidOpcode((w >> 8) as u8)),
                };
                self.push_belt(v)
            }
            6 => {
                let i = ((w >> 8) & 7) as usize;
                let a = self.b[i];
                let v = if w & 0x0800 != 0 {
                    m.read_u16(a)?
                } else {
                    u16::from(m.read_u8(a))
                };
                self.push_belt(v)
            }
            7 => {
                let ai = ((w >> 8) & 7) as usize;
                let vi = ((w >> 5) & 7) as usize;
                let a = self.b[ai];
                if w & 0x0800 != 0 {
                    m.write_u16(a, self.b[vi])?
                } else {
                    m.write_u8(a, self.b[vi] as u8)
                }
            }
            8 => {
                let i = ((w >> 8) & 7) as usize;
                let a = self.b[i];
                let v = if w & 0x0800 != 0 {
                    m.video_read_u16(a)?
                } else {
                    u16::from(m.video_read_u8(a))
                };
                self.push_belt(v)
            }
            9 => {
                let ai = ((w >> 8) & 7) as usize;
                let vi = ((w >> 5) & 7) as usize;
                let a = self.b[ai];
                if w & 0x0800 != 0 {
                    m.video_write_u16(a, self.b[vi])?
                } else {
                    m.video_write_u8(a, self.b[vi] as u8)
                }
            }
            10 => {
                let f = (w >> 8) & 15;
                let a = self.f16(m)?;
                let take = match f {
                    0 => true,
                    1 => {
                        self.push_ctl(m, self.pc)?;
                        true
                    }
                    2 => self.flags & Z != 0,
                    3 => self.flags & Z == 0,
                    4 => self.flags & C != 0,
                    5 => self.flags & C == 0,
                    6 => self.flags & N != 0,
                    7 => self.flags & N == 0,
                    _ => false,
                };
                if take {
                    self.pc = a
                }
            }
            11 => {
                if w & 0x0800 == 0 {
                    let i = ((w >> 8) & 7) as usize;
                    self.push_data(m, self.b[i])?
                } else {
                    let v = self.pop_data(m)?;
                    self.push_belt(v)
                }
            }
            12 => {
                let a = w & 0x00ff;
                let v = if w & 0x0800 != 0 {
                    m.read_u16(a)?
                } else {
                    u16::from(m.read_u8(a))
                };
                self.push_belt(v)
            }
            13 => {
                let i = ((w >> 8) & 7) as usize;
                let a = w & 0x00ff;
                if w & 0x0800 != 0 {
                    m.write_u16(a, self.b[i])?
                } else {
                    m.write_u8(a, self.b[i] as u8)
                }
            }
            _ => return Err(VmError::InvalidOpcode((w >> 8) as u8)),
        };
        m.retire_instruction();
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use svm_asm::belt::assembler::assemble;
    fn run(src: &str) -> (Cpu, Memory) {
        let p = assemble(src).unwrap();
        let mut m = Memory::default();
        m.load(p.load_address, &p.payload).unwrap();
        let mut c = Cpu::default();
        c.reset(p.entry_address);
        while !c.halted() {
            c.step(&mut m).unwrap()
        }
        (c, m)
    }
    #[test]
    fn computes_on_belt() {
        let (c, _) = run("LDI 10\nLDI 20\nADD b1,b0\nHALT");
        assert_eq!(c.b[0], 30);
    }
    #[test]
    fn zero_page_load_store() {
        let (c, m) = run("LDI 0x1234\nZST16 0x0E,b0\nZLD16 0x0E\nHALT");
        assert_eq!(m.read_u16(0x0E).unwrap(), 0x1234);
        assert_eq!(c.b[0], 0x1234);
    }
}
