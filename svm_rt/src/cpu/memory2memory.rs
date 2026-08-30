use crate::{
    error::VmError,
    memory::{Memory, STACK_BOTTOM, STACK_TOP_EXCLUSIVE},
};
const Z: u8 = 1;
const N: u8 = 2;
const C: u8 = 4;
const I: u8 = 8;
#[derive(Clone, Debug)]
pub struct Cpu {
    a: [u16; 4],
    pc: u16,
    flags: u8,
    halted: bool,
}
#[derive(Clone, Copy)]
struct Ref {
    addr: u16,
    post: Option<(usize, u16)>,
}
#[derive(Clone, Copy)]
enum Source {
    Imm(u16),
    Mem(Ref),
}
impl Default for Cpu {
    fn default() -> Self {
        let mut c = Self {
            a: [0; 4],
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
        self.a = [0; 4];
        self.a[3] = STACK_TOP_EXCLUSIVE;
        self.pc = e;
        self.flags = 0;
        self.halted = false
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
        let l = self.f8(m);
        let h = self.f8(m);
        u16::from_le_bytes([l, h])
    }
    fn push(&mut self, m: &mut Memory, v: u16) -> Result<(), VmError> {
        let next = self.a[3].checked_sub(2).ok_or(VmError::StackOverflow)?;
        if next < STACK_BOTTOM {
            return Err(VmError::StackOverflow);
        }
        self.a[3] = next;
        m.write_u16(self.a[3], v)
    }
    fn pop(&mut self, m: &mut Memory) -> Result<u16, VmError> {
        if self.a[3] >= STACK_TOP_EXCLUSIVE {
            return Err(VmError::StackUnderflow);
        }
        let v = m.read_u16(self.a[3])?;
        self.a[3] = self.a[3].checked_add(2).ok_or(VmError::StackUnderflow)?;
        if self.a[3] > STACK_TOP_EXCLUSIVE {
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
    fn zn(&mut self, v: u16, w: u8) {
        self.flag(Z, v == 0);
        self.flag(
            N,
            if w == 1 {
                v & 0x80 != 0
            } else {
                v & 0x8000 != 0
            },
        )
    }
    fn cmp(&mut self, x: u16, y: u16, w: u8) {
        let mask = if w == 1 { 0xff } else { 0xffff };
        let (a, b) = (x & mask, y & mask);
        let (v, br) = a.overflowing_sub(b);
        self.zn(v & mask, w);
        self.flag(C, !br)
    }
    fn memref(&mut self, m: &Memory, d: u8, w: u8) -> Result<Ref, VmError> {
        let inc = u16::from(w);
        match d {
            0x00..=0x7f => Ok(Ref {
                addr: u16::from(d),
                post: None,
            }),
            0x80..=0x83 => Ok(Ref {
                addr: self.a[(d & 3) as usize],
                post: None,
            }),
            0x84..=0x87 => {
                let r = (d & 3) as usize;
                Ok(Ref {
                    addr: self.a[r],
                    post: Some((r, inc)),
                })
            }
            0x88..=0x8b => {
                let r = (d & 3) as usize;
                self.a[r] = self.a[r].wrapping_sub(inc);
                Ok(Ref {
                    addr: self.a[r],
                    post: None,
                })
            }
            0x8c..=0x8f => {
                let r = (d & 3) as usize;
                let off = self.f8(m) as i8 as i16;
                Ok(Ref {
                    addr: self.a[r].wrapping_add(off as u16),
                    post: None,
                })
            }
            0xf0 => Ok(Ref {
                addr: self.f16(m),
                post: None,
            }),
            _ => Err(VmError::InvalidOpcode(d)),
        }
    }
    fn finish(&mut self, r: Ref) {
        if let Some((i, n)) = r.post {
            self.a[i] = self.a[i].wrapping_add(n)
        }
    }
    fn source(&mut self, m: &Memory, d: u8, w: u8) -> Result<Source, VmError> {
        match d {
            0xf1 => Ok(Source::Imm(self.f16(m))),
            0xf2 => Ok(Source::Imm(u16::from(self.f8(m)))),
            _ => Ok(Source::Mem(self.memref(m, d, w)?)),
        }
    }
    fn source_value(&self, m: &Memory, s: Source, w: u8) -> Result<u16, VmError> {
        match s {
            Source::Imm(v) => Ok(v),
            Source::Mem(r) => {
                if w == 1 {
                    Ok(u16::from(m.read_u8(r.addr)))
                } else {
                    m.read_u16(r.addr)
                }
            }
        }
    }
    fn enter_irq(&mut self, m: &mut Memory) -> Result<(), VmError> {
        let p = self.pc;
        let f = self.flags;
        self.push(m, p)?;
        self.push(m, u16::from(f))?;
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
        let op = self.f8(m);
        match op {
            0 => {}
            1 => self.halted = true,
            2 => self.pc = self.pop(m)?,
            3 => self.flags |= I,
            4 => self.flags &= !I,
            5 => {
                self.flags = self.pop(m)? as u8;
                self.pc = self.pop(m)?
            }
            0x10..=0x16 | 0x20..=0x2f => {
                let w = if op < 0x20 { 1 } else { 2 };
                let dd = self.f8(m);
                let dr = self.memref(m, dd, w)?;
                let sd = self.f8(m);
                let src = self.source(m, sd, w)?;
                let dv = if op == 0x10 || op == 0x20 {
                    0
                } else if w == 1 {
                    u16::from(m.read_u8(dr.addr))
                } else {
                    m.read_u16(dr.addr)?
                };
                let sv = self.source_value(m, src, w)?;
                let mask = if w == 1 { 0xff } else { 0xffff };
                let mut write = true;
                let v = match op {
                    0x10 | 0x20 => sv,
                    0x11 => {
                        let sum = (dv & 0xff) + (sv & 0xff);
                        self.flag(C, sum > 0xff);
                        sum
                    }
                    0x21 => {
                        let (v, c) = dv.overflowing_add(sv);
                        self.flag(C, c);
                        v
                    }
                    0x12 => {
                        let a = dv & 0xff;
                        let b = sv & 0xff;
                        self.flag(C, a >= b);
                        a.wrapping_sub(b)
                    }
                    0x22 => {
                        let (v, b) = dv.overflowing_sub(sv);
                        self.flag(C, !b);
                        v
                    }
                    0x13 | 0x23 => dv & sv,
                    0x14 | 0x24 => dv | sv,
                    0x15 | 0x25 => dv ^ sv,
                    0x16 | 0x26 => {
                        self.cmp(dv, sv, w);
                        write = false;
                        dv
                    }
                    0x27 => {
                        m.charge_internal_cycles(16);
                        dv.wrapping_mul(sv)
                    }
                    0x28 => {
                        if sv == 0 {
                            return Err(VmError::DivisionByZero);
                        }
                        m.charge_internal_cycles(16);
                        dv / sv
                    }
                    0x29 => {
                        if sv == 0 {
                            return Err(VmError::DivisionByZero);
                        }
                        m.charge_internal_cycles(16);
                        dv % sv
                    }
                    0x2a => dv.wrapping_shl((sv & 15) as u32),
                    0x2b => dv.wrapping_shr((sv & 15) as u32),
                    0x2c => {
                        m.charge_internal_cycles(17);
                        let p = (dv as i16 as i32) * (sv as i16 as i32);
                        ((p >> 15).clamp(-32768, 32767) as i16) as u16
                    }
                    0x2d => {
                        let cin = u16::from(self.flags & C != 0);
                        let (t, c1) = dv.overflowing_add(sv);
                        let (v, c2) = t.overflowing_add(cin);
                        self.flag(C, c1 || c2);
                        v
                    }
                    0x2e => {
                        let bin = u16::from(self.flags & C == 0);
                        let (t, b1) = dv.overflowing_sub(sv);
                        let (v, b2) = t.overflowing_sub(bin);
                        self.flag(C, !(b1 || b2));
                        v
                    }
                    0x2f => {
                        m.charge_internal_cycles(16);
                        self.flag(C, false);
                        (((dv as u32) * (sv as u32)) >> 16) as u16
                    }
                    _ => dv,
                } & mask;
                if write {
                    if w == 1 {
                        m.write_u8(dr.addr, v as u8)
                    } else {
                        m.write_u16(dr.addr, v)?
                    }
                    if op != 0x10 && op != 0x20 {
                        self.zn(v, w)
                    }
                }
                self.finish(dr);
                if let Source::Mem(r) = src {
                    self.finish(r)
                }
            }
            0x30..=0x33 | 0x38..=0x3f => {
                let w = if op < 0x38 { 1 } else { 2 };
                let d = self.f8(m);
                let r = self.memref(m, d, w)?;
                let x = if w == 1 {
                    u16::from(m.read_u8(r.addr))
                } else {
                    m.read_u16(r.addr)?
                };
                let mask = if w == 1 { 0xff } else { 0xffff };
                let v = match op {
                    0x30 | 0x38 => x.wrapping_add(1),
                    0x31 | 0x39 => x.wrapping_sub(1),
                    0x32 | 0x3a => !x,
                    0x33 | 0x3b => 0u16.wrapping_sub(x),
                    0x3c => ((x as i16) >> 1) as u16,
                    0x3d => {
                        let cin = self.flags & C != 0;
                        let v = (x >> 1) | if cin { 0x8000 } else { 0 };
                        self.flag(C, x & 1 != 0);
                        v
                    }
                    0x3e => {
                        self.flag(C, x & 0x8000 != 0);
                        x << 1
                    }
                    0x3f => {
                        self.flag(C, x & 1 != 0);
                        x >> 1
                    }
                    _ => x,
                } & mask;
                if w == 1 {
                    m.write_u8(r.addr, v as u8)
                } else {
                    m.write_u16(r.addr, v)?
                }
                self.zn(v, w);
                self.finish(r)
            }
            0x40..=0x43 => {
                let r = (op & 3) as usize;
                self.a[r] = self.f16(m)
            }
            0x44..=0x47 => {
                let r = (op & 3) as usize;
                let x = self.f8(m) as i8 as i16;
                self.a[r] = self.a[r].wrapping_add(x as u16)
            }
            0x48..=0x4b => {
                let r = (op & 3) as usize;
                let d = self.f8(m);
                let mr = self.memref(m, d, 2)?;
                self.a[r] = m.read_u16(mr.addr)?;
                self.finish(mr)
            }
            0x4c..=0x4f => {
                let r = (op & 3) as usize;
                let d = self.f8(m);
                let mr = self.memref(m, d, 2)?;
                m.write_u16(mr.addr, self.a[r])?;
                self.finish(mr)
            }
            0x50..=0x57 => {
                let rel = self.f8(m) as i8 as i16;
                let take = match op {
                    0x50 => true,
                    0x51 => self.flags & Z != 0,
                    0x52 => self.flags & Z == 0,
                    0x53 => self.flags & C != 0,
                    0x54 => self.flags & C == 0,
                    0x55 => self.flags & N != 0,
                    0x56 => self.flags & N == 0,
                    0x57 => {
                        self.push(m, self.pc)?;
                        true
                    }
                    _ => false,
                };
                if take {
                    self.pc = self.pc.wrapping_add(rel as u16)
                }
            }
            0x58..=0x5f => {
                let a = self.f16(m);
                let take = match op {
                    0x58 => true,
                    0x59 => self.flags & Z != 0,
                    0x5a => self.flags & Z == 0,
                    0x5b => self.flags & C != 0,
                    0x5c => self.flags & C == 0,
                    0x5d => self.flags & N != 0,
                    0x5e => self.flags & N == 0,
                    0x5f => {
                        self.push(m, self.pc)?;
                        true
                    }
                    _ => false,
                };
                if take {
                    self.pc = a
                }
            }
            0x60 | 0x61 => {
                let w = if op == 0x60 { 1 } else { 2 };
                let dd = self.f8(m);
                let dr = self.memref(m, dd, w)?;
                let sd = self.f8(m);
                let sr = self.memref(m, sd, w)?;
                let v = if w == 1 {
                    u16::from(m.video_read_u8(sr.addr))
                } else {
                    m.video_read_u16(sr.addr)?
                };
                if w == 1 {
                    m.write_u8(dr.addr, v as u8)
                } else {
                    m.write_u16(dr.addr, v)?
                }
                self.finish(dr);
                self.finish(sr)
            }
            0x62 | 0x63 => {
                let w = if op == 0x62 { 1 } else { 2 };
                let dd = self.f8(m);
                let dr = self.memref(m, dd, w)?;
                let sd = self.f8(m);
                let src = self.source(m, sd, w)?;
                let v = self.source_value(m, src, w)?;
                if w == 1 {
                    m.video_write_u8(dr.addr, v as u8)
                } else {
                    m.video_write_u16(dr.addr, v)?
                }
                self.finish(dr);
                if let Source::Mem(r) = src {
                    self.finish(r)
                }
            }
            _ => return Err(VmError::InvalidOpcode(op)),
        }
        m.retire_instruction();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn call_stack_uses_a3() {
        let mut m = Memory::default();
        let mut c = Cpu::default();
        c.reset(0x1234);
        c.push(&mut m, 0xBEEF).unwrap();
        assert_eq!(c.a[3], STACK_TOP_EXCLUSIVE - 2);
        assert_eq!(c.pop(&mut m).unwrap(), 0xBEEF);
        assert_eq!(c.a[3], STACK_TOP_EXCLUSIVE);
    }
}
