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
        let a = self.f8(m);
        let b = self.f8(m);
        u16::from_le_bytes([a, b])
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
        let (v, br) = a.overflowing_sub(b);
        self.zn(v);
        self.flag(C, !br)
    }
    fn src(&mut self, m: &mut Memory, d: u8, width: u8) -> Result<u16, VmError> {
        match d {
            0x00..=0x07 => Ok(self.r[d as usize]),
            0x10..=0x17 => {
                let a = self.r[(d & 7) as usize];
                if width == 1 {
                    Ok(u16::from(m.read_u8(a)))
                } else {
                    m.read_u16(a)
                }
            }
            0x18..=0x1f => {
                let r = (d & 7) as usize;
                let a = self.r[r];
                let v = if width == 1 {
                    u16::from(m.read_u8(a))
                } else {
                    m.read_u16(a)?
                };
                self.r[r] = a.wrapping_add(width as u16);
                Ok(v)
            }
            0x28..=0x2f => {
                let r = (d & 7) as usize;
                self.r[r] = self.r[r].wrapping_sub(width as u16);
                let a = self.r[r];
                if width == 1 {
                    Ok(u16::from(m.read_u8(a)))
                } else {
                    m.read_u16(a)
                }
            }
            0x20..=0x27 => {
                let r = (d & 7) as usize;
                let off = self.f8(m) as i8 as i16;
                let a = self.r[r].wrapping_add(off as u16);
                if width == 1 {
                    Ok(u16::from(m.read_u8(a)))
                } else {
                    m.read_u16(a)
                }
            }
            0xe0 => {
                let a = u16::from(self.f8(m));
                if width == 1 {
                    Ok(u16::from(m.read_u8(a)))
                } else {
                    m.read_u16(a)
                }
            }
            0xe1 => {
                let a = self.f16(m);
                if width == 1 {
                    Ok(u16::from(m.read_u8(a)))
                } else {
                    m.read_u16(a)
                }
            }
            0xf0 => Ok(u16::from(self.f8(m))),
            0xf1 => Ok(self.f16(m)),
            _ => Err(VmError::InvalidOpcode(d)),
        }
    }
    fn addr(
        &mut self,
        m: &Memory,
        d: u8,
        width: u8,
    ) -> Result<(u16, Option<(usize, i16)>), VmError> {
        match d {
            0x10..=0x17 => Ok((self.r[(d & 7) as usize], None)),
            0x18..=0x1f => {
                let r = (d & 7) as usize;
                Ok((self.r[r], Some((r, width as i16))))
            }
            0x28..=0x2f => {
                let r = (d & 7) as usize;
                let a = self.r[r].wrapping_sub(width as u16);
                self.r[r] = a;
                Ok((a, None))
            }
            0x20..=0x27 => {
                let r = (d & 7) as usize;
                let off = self.f8(m) as i8 as i16;
                Ok((self.r[r].wrapping_add(off as u16), None))
            }
            0xe0 => Ok((u16::from(self.f8(m)), None)),
            0xe1 => Ok((self.f16(m), None)),
            _ => Err(VmError::InvalidOpcode(d)),
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
            0x20..=0x2f => {
                let d = self.f8(m) as usize;
                if d >= 8 {
                    return Err(VmError::InvalidOpcode(op));
                }
                let sd = self.f8(m);
                let b = self.src(m, sd, 2)?;
                let a = self.r[d];
                match op {
                    0x20 => self.r[d] = b,
                    0x21 => {
                        let (v, c) = a.overflowing_add(b);
                        self.r[d] = v;
                        self.zn(v);
                        self.flag(C, c)
                    }
                    0x22 => {
                        let (v, br) = a.overflowing_sub(b);
                        self.r[d] = v;
                        self.zn(v);
                        self.flag(C, !br)
                    }
                    0x23 => {
                        self.r[d] = a & b;
                        self.zn(self.r[d])
                    }
                    0x24 => {
                        self.r[d] = a | b;
                        self.zn(self.r[d])
                    }
                    0x25 => {
                        self.r[d] = a ^ b;
                        self.zn(self.r[d])
                    }
                    0x26 => self.cmp(a, b),
                    0x27 => {
                        m.charge_internal_cycles(16);
                        self.r[d] = a.wrapping_mul(b);
                        self.zn(self.r[d])
                    }
                    0x28 => {
                        if b == 0 {
                            return Err(VmError::DivisionByZero);
                        }
                        m.charge_internal_cycles(16);
                        self.r[d] = a / b;
                        self.zn(self.r[d])
                    }
                    0x29 => {
                        if b == 0 {
                            return Err(VmError::DivisionByZero);
                        }
                        m.charge_internal_cycles(16);
                        self.r[d] = a % b;
                        self.zn(self.r[d])
                    }
                    0x2a => {
                        self.r[d] = a.wrapping_shl((b & 15) as u32);
                        self.zn(self.r[d])
                    }
                    0x2b => {
                        self.r[d] = a.wrapping_shr((b & 15) as u32);
                        self.zn(self.r[d])
                    }
                    0x2c => {
                        m.charge_internal_cycles(17);
                        let p = (a as i16 as i32) * (b as i16 as i32);
                        self.r[d] = ((p >> 15).clamp(-32768, 32767) as i16) as u16;
                        self.zn(self.r[d])
                    }
                    0x2d => {
                        let cin = u16::from(self.flags & C != 0);
                        let (t, c1) = a.overflowing_add(b);
                        let (v, c2) = t.overflowing_add(cin);
                        self.r[d] = v;
                        self.zn(v);
                        self.flag(C, c1 || c2)
                    }
                    0x2e => {
                        let bin = u16::from(self.flags & C == 0);
                        let (t, b1) = a.overflowing_sub(b);
                        let (v, b2) = t.overflowing_sub(bin);
                        self.r[d] = v;
                        self.zn(v);
                        self.flag(C, !(b1 || b2))
                    }
                    0x2f => {
                        m.charge_internal_cycles(16);
                        self.r[d] = (((a as u32) * (b as u32)) >> 16) as u16;
                        self.zn(self.r[d]);
                        self.flag(C, false)
                    }
                    _ => {}
                }
            }
            0x30..=0x37 => {
                let d = self.f8(m) as usize;
                if d >= 8 {
                    return Err(VmError::InvalidOpcode(op));
                }
                let a = self.r[d];
                let v = match op {
                    0x30 => !a,
                    0x31 => 0u16.wrapping_sub(a),
                    0x32 => a.wrapping_add(1),
                    0x33 => a.wrapping_sub(1),
                    0x34 => ((a as i16) >> 1) as u16,
                    0x35 => {
                        self.flag(C, a & 0x8000 != 0);
                        a << 1
                    }
                    0x36 => {
                        self.flag(C, a & 1 != 0);
                        a >> 1
                    }
                    _ => {
                        let cin = self.flags & C != 0;
                        let v = (a >> 1) | if cin { 0x8000 } else { 0 };
                        self.flag(C, a & 1 != 0);
                        v
                    }
                };
                self.r[d] = v;
                self.zn(v)
            }
            0x40 | 0x41 => {
                let d = self.f8(m) as usize;
                let md = self.f8(m);
                self.r[d] = self.src(m, md, if op == 0x40 { 1 } else { 2 })?
            }
            0x42 | 0x43 => {
                let md = self.f8(m);
                let width = if op == 0x42 { 1 } else { 2 };
                let (a, post) = self.addr(m, md, width)?;
                let r = self.f8(m) as usize;
                if width == 1 {
                    m.write_u8(a, self.r[r] as u8)
                } else {
                    m.write_u16(a, self.r[r])?
                }
                if let Some((rr, inc)) = post {
                    self.r[rr] = self.r[rr].wrapping_add(inc as u16)
                }
            }
            0x60..=0x67 => {
                let a = self.f16(m);
                let take = match op {
                    0x60 => true,
                    0x61 => self.flags & Z != 0,
                    0x62 => self.flags & Z == 0,
                    0x63 => self.flags & C != 0,
                    0x64 => self.flags & C == 0,
                    0x65 => self.flags & N != 0,
                    0x66 => self.flags & N == 0,
                    0x67 => {
                        self.push(m, self.pc)?;
                        true
                    }
                    _ => false,
                };
                if take {
                    self.pc = a
                }
            }
            0x70 | 0x71 => {
                let d = self.f8(m) as usize;
                let md = self.f8(m);
                let width = if op == 0x70 { 1 } else { 2 };
                let (a, post) = self.addr(m, md, width)?;
                self.r[d] = if width == 1 {
                    u16::from(m.video_read_u8(a))
                } else {
                    m.video_read_u16(a)?
                };
                if let Some((rr, inc)) = post {
                    self.r[rr] = self.r[rr].wrapping_add(inc as u16)
                }
            }
            0x72 | 0x73 => {
                let md = self.f8(m);
                let width = if op == 0x72 { 1 } else { 2 };
                let (a, post) = self.addr(m, md, width)?;
                let r = self.f8(m) as usize;
                if width == 1 {
                    m.video_write_u8(a, self.r[r] as u8)
                } else {
                    m.video_write_u16(a, self.r[r])?
                }
                if let Some((rr, inc)) = post {
                    self.r[rr] = self.r[rr].wrapping_add(inc as u16)
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
    use svm_asm::regmem::assembler::assemble;

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
}
