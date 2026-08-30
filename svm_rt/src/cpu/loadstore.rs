use crate::{
    error::VmError,
    memory::{Memory, STACK_BOTTOM, STACK_TOP_EXCLUSIVE},
};
const Z: u8 = 1;
const N: u8 = 2;
const C: u8 = 4;
const I: u8 = 8;
#[derive(Debug, Clone)]
pub struct Cpu {
    r: [u16; 8],
    pc: u16,
    flags: u8,
    halted: bool,
}
impl Default for Cpu {
    fn default() -> Self {
        let mut c = Self {
            r: [0; 8],
            pc: 0,
            flags: 0,
            halted: false,
        };
        c.reset(0);
        c
    }
}
impl Cpu {
    pub fn reset(&mut self, e: u16) {
        self.r = [0; 8];
        self.r[6] = STACK_TOP_EXCLUSIVE;
        self.pc = e;
        self.flags = 0;
        self.halted = false;
    }
    pub fn halted(&self) -> bool {
        self.halted
    }
    fn f8(&mut self, m: &Memory) -> u8 {
        let v = m.read_u8(self.pc);
        self.pc = self.pc.wrapping_add(1);
        v
    }
    fn f16(&mut self, m: &Memory) -> u16 {
        let lo = self.f8(m);
        let hi = self.f8(m);
        u16::from_le_bytes([lo, hi])
    }
    fn push(&mut self, m: &mut Memory, v: u16) -> Result<(), VmError> {
        let next = self.r[6].checked_sub(2).ok_or(VmError::StackOverflow)?;
        if next < STACK_BOTTOM {
            return Err(VmError::StackOverflow);
        }
        self.r[6] = next;
        m.write_u16(self.r[6], v)
    }
    fn pop(&mut self, m: &mut Memory) -> Result<u16, VmError> {
        if self.r[6] >= STACK_TOP_EXCLUSIVE {
            return Err(VmError::StackUnderflow);
        }
        let v = m.read_u16(self.r[6])?;
        self.r[6] = self.r[6].checked_add(2).ok_or(VmError::StackUnderflow)?;
        if self.r[6] > STACK_TOP_EXCLUSIVE {
            return Err(VmError::StackUnderflow);
        }
        Ok(v)
    }
    fn flag(&mut self, b: u8, on: bool) {
        if on {
            self.flags |= b
        } else {
            self.flags &= !b
        }
    }
    fn zn(&mut self, v: u16) {
        self.flag(Z, v == 0);
        self.flag(N, v & 0x8000 != 0)
    }
    fn cmp(&mut self, a: u16, b: u16) {
        let (v, borrow) = a.overflowing_sub(b);
        self.zn(v);
        self.flag(C, !borrow)
    }
    fn addv(&mut self, a: u16, b: u16) -> u16 {
        let (v, c) = a.overflowing_add(b);
        self.zn(v);
        self.flag(C, c);
        v
    }
    fn subv(&mut self, a: u16, b: u16) -> u16 {
        let (v, br) = a.overflowing_sub(b);
        self.zn(v);
        self.flag(C, !br);
        v
    }
    fn enter_irq(&mut self, m: &mut Memory) -> Result<(), VmError> {
        let pc = self.pc;
        let fl = self.flags;
        self.push(m, pc)?;
        self.push(m, u16::from(fl))?;
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
            self.enter_irq(m)?;
        }
        let w = self.f16(m);
        let major = w >> 12;
        match major {
            0 => match w & 0x0fff {
                0 => {}
                1 => self.halted = true,
                2 => self.pc = self.pop(m)?,
                3 => self.flags |= I,
                4 => self.flags &= !I,
                5 => {
                    self.flags = self.pop(m)? as u8;
                    self.pc = self.pop(m)?
                }
                _ => return Err(VmError::InvalidOpcode((w >> 8) as u8)),
            },
            1 => {
                let f = (w >> 9) & 7;
                let d = ((w >> 6) & 7) as usize;
                let a = ((w >> 3) & 7) as usize;
                let b = (w & 7) as usize;
                let av = self.r[a];
                let bv = self.r[b];
                let v = match f {
                    0 => self.addv(av, bv),
                    1 => self.subv(av, bv),
                    2 => {
                        let v = av & bv;
                        self.zn(v);
                        v
                    }
                    3 => {
                        let v = av | bv;
                        self.zn(v);
                        v
                    }
                    4 => {
                        let v = av ^ bv;
                        self.zn(v);
                        v
                    }
                    5 => {
                        m.charge_internal_cycles(16);
                        let v = av.wrapping_mul(bv);
                        self.zn(v);
                        self.flag(C, false);
                        v
                    }
                    6 => {
                        let v = av.wrapping_shl((bv & 15) as u32);
                        self.zn(v);
                        v
                    }
                    7 => {
                        let v = av.wrapping_shr((bv & 15) as u32);
                        self.zn(v);
                        v
                    }
                    _ => unreachable!(),
                };
                self.r[d] = v;
            }
            2 => {
                let f = (w >> 9) & 7;
                let d = ((w >> 6) & 7) as usize;
                let a = ((w >> 3) & 7) as usize;
                let av = self.r[a];
                match f {
                    0 => self.r[d] = av,
                    1 => self.cmp(av, self.r[d]),
                    2 => {
                        let v = !av;
                        self.r[d] = v;
                        self.zn(v)
                    }
                    3 => {
                        let v = 0u16.wrapping_sub(av);
                        self.r[d] = v;
                        self.zn(v);
                        self.flag(C, av == 0)
                    }
                    4 => {
                        let v = ((av as i16) >> 1) as u16;
                        self.r[d] = v;
                        self.zn(v)
                    }
                    _ => return Err(VmError::InvalidOpcode((w >> 8) as u8)),
                }
            }
            3 => {
                let f = (w >> 9) & 7;
                let d = ((w >> 6) & 7) as usize;
                let raw = (w & 0x3f) as u16;
                let sv = (((raw as i16) << 10) >> 10) as u16;
                match f {
                    0 => {
                        let a = self.r[d];
                        self.r[d] = self.addv(a, sv)
                    }
                    1 => {
                        self.r[d] = raw;
                        self.zn(raw)
                    }
                    2 => self.cmp(self.r[d], sv),
                    3 => {
                        self.r[d] &= raw;
                        self.zn(self.r[d])
                    }
                    4 => {
                        self.r[d] |= raw;
                        self.zn(self.r[d])
                    }
                    5 => {
                        self.r[d] ^= raw;
                        self.zn(self.r[d])
                    }
                    6 => {
                        let old = self.r[d];
                        self.r[d] = old.wrapping_shl((raw & 15) as u32);
                        self.zn(self.r[d]);
                        if raw == 1 {
                            self.flag(C, old & 0x8000 != 0)
                        }
                    }
                    7 => {
                        let old = self.r[d];
                        self.r[d] = old.wrapping_shr((raw & 15) as u32);
                        self.zn(self.r[d]);
                        if raw == 1 {
                            self.flag(C, old & 1 != 0)
                        }
                    }
                    _ => {}
                }
            }
            4 | 5 => {
                let d = ((w >> 9) & 7) as usize;
                let a = ((w >> 6) & 7) as usize;
                let off = (((w & 0x3f) as i16) << 10) >> 10;
                let ea = self.r[a].wrapping_add(off as u16);
                self.r[d] = if major == 4 {
                    u16::from(m.read_u8(ea))
                } else {
                    m.read_u16(ea)?
                };
            }
            6 | 7 => {
                let s = ((w >> 9) & 7) as usize;
                let a = ((w >> 6) & 7) as usize;
                let off = (((w & 0x3f) as i16) << 10) >> 10;
                let ea = self.r[a].wrapping_add(off as u16);
                if major == 6 {
                    m.write_u8(ea, self.r[s] as u8)
                } else {
                    m.write_u16(ea, self.r[s])?
                }
            }
            8 => {
                let cond = (w >> 9) & 7;
                let raw = (w & 0x01ff) as i16;
                let rel = (raw << 7) >> 7;
                let take = match cond {
                    0 => true,
                    1 => self.flags & Z != 0,
                    2 => self.flags & Z == 0,
                    3 => self.flags & C != 0,
                    4 => self.flags & C == 0,
                    5 => self.flags & N != 0,
                    6 => self.flags & N == 0,
                    _ => return Err(VmError::InvalidOpcode((w >> 8) as u8)),
                };
                if take {
                    self.pc = self.pc.wrapping_add((rel.wrapping_mul(2)) as u16)
                }
            }
            9 => {
                let f = (w >> 9) & 7;
                let d = ((w >> 6) & 7) as usize;
                let x = self.f16(m);
                match f {
                    0 => self.r[d] = x,
                    1 => {
                        let a = self.r[d];
                        self.r[d] = self.addv(a, x)
                    }
                    2 => self.cmp(self.r[d], x),
                    3 => {
                        let a = self.r[d];
                        self.r[d] = self.subv(a, x)
                    }
                    _ => return Err(VmError::InvalidOpcode((w >> 8) as u8)),
                }
            }
            10 => {
                let f = (w >> 9) & 7;
                let a = self.f16(m);
                let take = match f {
                    0 => true,
                    1 => {
                        self.push(m, self.pc)?;
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
                let f = (w >> 10) & 3;
                let d = ((w >> 7) & 7) as usize;
                let a = ((w >> 4) & 7) as usize;
                let off = ((((w & 15) as i16) << 12) >> 12) as u16;
                let ea = self.r[a].wrapping_add(off);
                match f {
                    0 => self.r[d] = u16::from(m.video_read_u8(ea)),
                    1 => self.r[d] = m.video_read_u16(ea)?,
                    2 => m.video_write_u8(ea, self.r[d] as u8),
                    3 => m.video_write_u16(ea, self.r[d])?,
                    _ => {}
                }
            }
            12 => {
                let f = (w >> 9) & 7;
                let d = ((w >> 6) & 7) as usize;
                let a = ((w >> 3) & 7) as usize;
                let b = (w & 7) as usize;
                let x = self.r[a];
                let y = self.r[b];
                let v = match f {
                    0 => {
                        if y == 0 {
                            return Err(VmError::DivisionByZero);
                        }
                        m.charge_internal_cycles(16);
                        x / y
                    }
                    1 => {
                        if y == 0 {
                            return Err(VmError::DivisionByZero);
                        }
                        m.charge_internal_cycles(16);
                        x % y
                    }
                    2 => {
                        m.charge_internal_cycles(17);
                        let p = (x as i16 as i32) * (y as i16 as i32);
                        let mut q = (p >> 15) as i32;
                        if q > 32767 {
                            q = 32767
                        }
                        if q < -32768 {
                            q = -32768
                        }
                        self.flag(C, false);
                        q as i16 as u16
                    }
                    3 => {
                        let cin = u16::from(self.flags & C != 0);
                        let (t, c1) = x.overflowing_add(y);
                        let (v, c2) = t.overflowing_add(cin);
                        self.flag(C, c1 || c2);
                        v
                    }
                    4 => {
                        let bin = u16::from(self.flags & C == 0);
                        let (t, b1) = x.overflowing_sub(y);
                        let (v, b2) = t.overflowing_sub(bin);
                        self.flag(C, !(b1 || b2));
                        v
                    }
                    5 => {
                        m.charge_internal_cycles(16);
                        self.flag(C, false);
                        (((x as u32) * (y as u32)) >> 16) as u16
                    }
                    6 => {
                        let cin = self.flags & C != 0;
                        let v = (x >> 1) | if cin { 0x8000 } else { 0 };
                        self.flag(C, x & 1 != 0);
                        v
                    }
                    _ => return Err(VmError::InvalidOpcode((w >> 8) as u8)),
                };
                self.r[d] = v;
                self.zn(v)
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
    use svm_asm::loadstore::assembler::assemble;

    #[test]
    fn call_and_expression_push_share_r6_stack() {
        let p = assemble("MOVI R0, 0x1234\nPUSH R0\nCALL f\nPOP R1\nHALT\nf:\nMOVI R0, 7\nRET")
            .unwrap();
        let mut m = Memory::default();
        m.load(p.load_address, &p.payload).unwrap();
        let mut c = Cpu::default();
        c.reset(p.entry_address);
        while !c.halted() {
            c.step(&mut m).unwrap();
        }
        assert_eq!(c.r[1], 0x1234);
        assert_eq!(c.r[6], STACK_TOP_EXCLUSIVE);
    }

    #[test]
    fn subi_preserves_no_borrow_carry_semantics() {
        let p = assemble("MOVI R0,0\nSUBI R0,1\nHALT").unwrap();
        let mut m = Memory::default();
        m.load(p.load_address, &p.payload).unwrap();
        let mut c = Cpu::default();
        c.reset(p.entry_address);
        c.step(&mut m).unwrap();
        c.step(&mut m).unwrap();
        assert_eq!(c.r[0], 0xffff);
        assert_eq!(c.flags & C, 0);
    }
}
