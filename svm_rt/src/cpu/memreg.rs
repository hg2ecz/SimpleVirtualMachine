use crate::{
    error::VmError,
    memory::{Memory, STACK_BOTTOM, STACK_TOP_EXCLUSIVE},
};
use svm_asm::memreg::instruction::op;
const Z: u8 = 1;
const N: u8 = 2;
const C: u8 = 4;
const I: u8 = 8;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StepResult {
    pub halted: bool,
}
#[derive(Debug, Clone)]
pub struct Cpu {
    w: u16,
    fsr: [u16; 2],
    pc: u16,
    sp: u16,
    flags: u8,
    halted: bool,
}
impl Default for Cpu {
    fn default() -> Self {
        let mut c = Self {
            w: 0,
            fsr: [0; 2],
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
    pub fn reset(&mut self, e: u16) {
        self.w = 0;
        self.fsr = [0; 2];
        self.pc = e;
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
    pub fn w(&self) -> u16 {
        self.w
    }
    pub fn fsr(&self, n: usize) -> Option<u16> {
        self.fsr.get(n).copied()
    }
    pub fn step(&mut self, m: &mut Memory) -> Result<StepResult, VmError> {
        if self.halted {
            return Ok(StepResult { halted: true });
        }
        m.begin_instruction();
        if self.flags & I != 0 && m.irq_active() {
            self.enter_irq(m)?;
        }
        let o = self.f8(m);
        self.exec(o, m)?;
        m.retire_instruction();
        Ok(StepResult {
            halted: self.halted,
        })
    }
    fn exec(&mut self, o: u8, m: &mut Memory) -> Result<(), VmError> {
        use op::*;
        if o >= 0x80 {
            let f = u16::from(o & 0x0f);
            match o & 0xf0 {
                0x80 => {
                    self.w = u16::from(m.read_u8(f));
                    self.zn(self.w)
                }
                0x90 => m.write_u8(f, self.w as u8),
                0xA0 => {
                    self.w = m.read_u16(f)?;
                    self.zn(self.w)
                }
                0xB0 => m.write_u16(f, self.w)?,
                0xC0 => self.add(m.read_u16(f)?),
                0xD0 => {
                    let v = m.read_u16(f)?.wrapping_add(self.w);
                    m.write_u16(f, v)?;
                    self.zn(v)
                }
                0xE0 => {
                    self.w &= m.read_u16(f)?;
                    self.zn(self.w)
                }
                0xF0 => {
                    let v = m.read_u16(f)? & self.w;
                    m.write_u16(f, v)?;
                    self.zn(v)
                }
                _ => {}
            }
            return Ok(());
        }
        match o {
            NOP => {}
            HALT => self.halted = true,
            RET => self.pc = self.pop(m)?,
            VEXT => {
                let sub = self.f8(m);
                self.vind(sub, m)?
            }
            EI => self.flags |= I,
            DI => self.flags &= !I,
            IRET => {
                self.flags = self.pop(m)? as u8;
                self.pc = self.pop(m)?
            }
            PUSHW => self.push(m, self.w)?,
            POPW => {
                self.w = self.pop(m)?;
                self.zn(self.w)
            }
            INCW => self.add(1),
            DECW => self.sub(1),
            NEGW => {
                let q = self.w;
                self.w = 0u16.wrapping_sub(q);
                self.zn(self.w);
                self.sf(C, q == 0)
            }
            NOTW => {
                self.w = !self.w;
                self.zn(self.w)
            }
            SHL1W => {
                let old = self.w;
                self.w = old.wrapping_shl(1);
                self.zn(self.w);
                self.sf(C, old & 0x8000 != 0)
            }
            SHR1W => {
                let old = self.w;
                self.w = old.wrapping_shr(1);
                self.zn(self.w);
                self.sf(C, old & 1 != 0)
            }
            RCR1W => {
                let old = self.w;
                let cin = self.flags & C != 0;
                self.w = (old >> 1) | if cin { 0x8000 } else { 0 };
                self.zn(self.w);
                self.sf(C, old & 1 != 0)
            }
            W2F0 => self.fsr[0] = self.w,
            W2F1 => self.fsr[1] = self.w,
            F02W => {
                self.w = self.fsr[0];
                self.zn(self.w)
            }
            F12W => {
                self.w = self.fsr[1];
                self.zn(self.w)
            }
            ASR1W => {
                self.w = ((self.w as i16) >> 1) as u16;
                self.zn(self.w)
            }
            LDI => {
                self.w = self.f16(m);
                self.zn(self.w)
            }
            FSR0I => self.fsr[0] = self.f16(m),
            FSR1I => self.fsr[1] = self.f16(m),
            ADDI => {
                let v = self.f16(m);
                self.add(v)
            }
            SUBI => {
                let v = self.f16(m);
                self.sub(v)
            }
            CMPI => {
                let v = self.f16(m);
                self.cmp(v)
            }
            ANDI => {
                let v = self.f16(m);
                self.w &= v;
                self.zn(self.w)
            }
            ORI => {
                let v = self.f16(m);
                self.w |= v;
                self.zn(self.w)
            }
            XORI => {
                let v = self.f16(m);
                self.w ^= v;
                self.zn(self.w)
            }
            MOV8_FW => {
                let f = u16::from(self.f8(m));
                self.w = u16::from(m.read_u8(f));
                self.zn(self.w)
            }
            MOV8_WF => {
                let f = u16::from(self.f8(m));
                m.write_u8(f, self.w as u8)
            }
            MOV16_FW => {
                let f = u16::from(self.f8(m));
                self.w = m.read_u16(f)?;
                self.zn(self.w)
            }
            MOV16_WF => {
                let f = u16::from(self.f8(m));
                m.write_u16(f, self.w)?
            }
            ADD_FW | ADD_FF | SUB_FW | SUB_FF | AND_FW | AND_FF | OR_FW | OR_FF | XOR_FW
            | XOR_FF => {
                let f = u16::from(self.f8(m));
                let a = m.read_u16(f)?;
                let to_f = matches!(o, ADD_FF | SUB_FF | AND_FF | OR_FF | XOR_FF);
                let (v, c) = match o {
                    ADD_FW => self.w.overflowing_add(a),
                    ADD_FF => a.overflowing_add(self.w),
                    SUB_FW => {
                        let (v, b) = self.w.overflowing_sub(a);
                        (v, !b)
                    }
                    SUB_FF => {
                        let (v, b) = a.overflowing_sub(self.w);
                        (v, !b)
                    }
                    AND_FW => (self.w & a, false),
                    AND_FF => (a & self.w, false),
                    OR_FW => (self.w | a, false),
                    OR_FF => (a | self.w, false),
                    XOR_FW => (self.w ^ a, false),
                    _ => (a ^ self.w, false),
                };
                if to_f {
                    m.write_u16(f, v)?
                } else {
                    self.w = v
                }
                self.zn(v);
                if matches!(o, ADD_FW | ADD_FF | SUB_FW | SUB_FF) {
                    self.sf(C, c)
                }
            }
            SHL_FW | SHR_FW | MUL_FW | MULQ15_FW | DIV_FW | MOD_FW => {
                let f = u16::from(self.f8(m));
                let a = m.read_u16(f)?;
                if matches!(o, DIV_FW | MOD_FW) && a == 0 {
                    return Err(VmError::DivisionByZero);
                }
                if matches!(o, MUL_FW) {
                    m.charge_internal_cycles(16)
                } else if matches!(o, MULQ15_FW) {
                    m.charge_internal_cycles(17)
                } else if matches!(o, DIV_FW | MOD_FW) {
                    m.charge_internal_cycles(16)
                } else {
                    m.charge_internal_cycles(1)
                }
                self.w = match o {
                    SHL_FW => self.w.wrapping_shl(u32::from(a & 15)),
                    SHR_FW => self.w.wrapping_shr(u32::from(a & 15)),
                    MUL_FW => self.w.wrapping_mul(a),
                    MULQ15_FW => q15_mul(self.w, a),
                    DIV_FW => self.w / a,
                    _ => self.w % a,
                };
                self.zn(self.w)
            }
            CMP_F => {
                let f = u16::from(self.f8(m));
                self.cmp(m.read_u16(f)?)
            }
            INC_F | DEC_F => {
                let f = u16::from(self.f8(m));
                let v = if o == INC_F {
                    m.read_u16(f)?.wrapping_add(1)
                } else {
                    m.read_u16(f)?.wrapping_sub(1)
                };
                m.write_u16(f, v)?;
                self.zn(v)
            }
            LDB0 | LDW0 | STB0 | STW0 | LDB0P | LDW0P | STB0P | STW0P | LDB0M | LDW0M | STB0M
            | STW0M => self.ind(o, 0, m)?,
            LDB1 | LDW1 | STB1 | STW1 | LDB1P | LDW1P | STB1P | STW1P | LDB1M | LDW1M | STB1M
            | STW1M => self.ind(o, 1, m)?,
            ADC_FW | ADC_FF | SBC_FW | SBC_FF | MULHU_FW => {
                let f = u16::from(self.f8(m));
                let fv = m.read_u16(f)?;
                let to_f = matches!(o, ADC_FF | SBC_FF);
                let (lhs, rhs) = if to_f { (fv, self.w) } else { (self.w, fv) };
                let v = if o == MULHU_FW {
                    m.charge_internal_cycles(16);
                    self.sf(C, false);
                    (((lhs as u32) * (rhs as u32)) >> 16) as u16
                } else if matches!(o, ADC_FW | ADC_FF) {
                    let cin = u16::from(self.flags & C != 0);
                    let (t, c1) = lhs.overflowing_add(rhs);
                    let (v, c2) = t.overflowing_add(cin);
                    self.sf(C, c1 || c2);
                    v
                } else {
                    let bin = u16::from(self.flags & C == 0);
                    let (t, b1) = lhs.overflowing_sub(rhs);
                    let (v, b2) = t.overflowing_sub(bin);
                    self.sf(C, !(b1 || b2));
                    v
                };
                if to_f {
                    m.write_u16(f, v)?
                } else {
                    self.w = v
                };
                self.zn(v)
            }
            JMP => self.pc = self.f16(m),
            CALL => {
                let q = self.f16(m);
                self.push(m, self.pc)?;
                self.pc = q
            }
            JZ | JNZ | JC | JNC | JN | JNN => {
                let q = self.f16(m);
                if self.take(o) {
                    self.pc = q
                }
            }
            RJMP => {
                let d = self.f8(m) as i8;
                self.pc = self.pc.wrapping_add_signed(i16::from(d))
            }
            RCALL => {
                let d = self.f8(m) as i8;
                let r = self.pc;
                self.push(m, r)?;
                self.pc = self.pc.wrapping_add_signed(i16::from(d))
            }
            RJZ | RJNZ | RJC | RJNC | RJN | RJNN => {
                let d = self.f8(m) as i8;
                if self.take(o) {
                    self.pc = self.pc.wrapping_add_signed(i16::from(d))
                }
            }
            _ => return Err(VmError::InvalidOpcode(o)),
        }
        Ok(())
    }

    fn vind(&mut self, sub: u8, m: &mut Memory) -> Result<(), VmError> {
        let n = if sub < 0x0C { 0 } else { 1 };
        let k = if sub < 0x0C { sub } else { sub - 0x0C };
        let word = matches!(k, 0x01 | 0x03 | 0x05 | 0x07 | 0x09 | 0x0B);
        let store = matches!(k, 0x02 | 0x03 | 0x06 | 0x07 | 0x0A | 0x0B);
        let post = matches!(k, 0x04 | 0x05 | 0x06 | 0x07);
        let pre = matches!(k, 0x08 | 0x09 | 0x0A | 0x0B);
        if k > 0x0B {
            return Err(VmError::InvalidOpcode(sub));
        }
        let step = if word { 2 } else { 1 };
        if pre {
            self.fsr[n] = self.fsr[n].wrapping_sub(step)
        }
        let a = self.fsr[n];
        if store {
            if word {
                m.video_write_u16(a, self.w)?
            } else {
                m.video_write_u8(a, self.w as u8)
            }
        } else {
            self.w = if word {
                m.video_read_u16(a)?
            } else {
                u16::from(m.video_read_u8(a))
            };
            self.zn(self.w)
        }
        if post {
            self.fsr[n] = a.wrapping_add(step)
        }
        Ok(())
    }
    fn ind(&mut self, o: u8, n: usize, m: &mut Memory) -> Result<(), VmError> {
        use op::*;
        let word = matches!(
            o,
            LDW0 | STW0
                | LDW0P
                | STW0P
                | LDW0M
                | STW0M
                | LDW1
                | STW1
                | LDW1P
                | STW1P
                | LDW1M
                | STW1M
        );
        let pre = matches!(
            o,
            LDB0M | LDW0M | STB0M | STW0M | LDB1M | LDW1M | STB1M | STW1M
        );
        let post = matches!(
            o,
            LDB0P | LDW0P | STB0P | STW0P | LDB1P | LDW1P | STB1P | STW1P
        );
        let load = matches!(
            o,
            LDB0 | LDW0
                | LDB0P
                | LDW0P
                | LDB0M
                | LDW0M
                | LDB1
                | LDW1
                | LDB1P
                | LDW1P
                | LDB1M
                | LDW1M
        );
        let step = if word { 2 } else { 1 };
        if pre {
            self.fsr[n] = self.fsr[n].wrapping_sub(step)
        }
        let a = self.fsr[n];
        if load {
            self.w = if word {
                m.read_u16(a)?
            } else {
                u16::from(m.read_u8(a))
            };
            self.zn(self.w)
        } else if word {
            m.write_u16(a, self.w)?
        } else {
            m.write_u8(a, self.w as u8)
        }
        if post {
            self.fsr[n] = a.wrapping_add(step)
        }
        Ok(())
    }
    fn enter_irq(&mut self, m: &mut Memory) -> Result<(), VmError> {
        let f = self.flags;
        self.push(m, self.pc)?;
        self.push(m, u16::from(f))?;
        self.flags &= !I;
        m.charge_internal_cycles(2);
        self.pc = m.irq_vector();
        Ok(())
    }
    fn take(&self, o: u8) -> bool {
        use op::*;
        match o {
            JZ | RJZ => self.fl(Z),
            JNZ | RJNZ => !self.fl(Z),
            JC | RJC => self.fl(C),
            JNC | RJNC => !self.fl(C),
            JN | RJN => self.fl(N),
            JNN | RJNN => !self.fl(N),
            _ => false,
        }
    }
    fn f8(&mut self, m: &Memory) -> u8 {
        let v = m.read_u8(self.pc);
        self.pc = self.pc.wrapping_add(1);
        v
    }
    fn f16(&mut self, m: &Memory) -> u16 {
        u16::from_le_bytes([self.f8(m), self.f8(m)])
    }
    fn add(&mut self, r: u16) {
        let (v, c) = self.w.overflowing_add(r);
        self.w = v;
        self.zn(v);
        self.sf(C, c)
    }
    fn sub(&mut self, r: u16) {
        let (v, b) = self.w.overflowing_sub(r);
        self.w = v;
        self.zn(v);
        self.sf(C, !b)
    }
    fn cmp(&mut self, r: u16) {
        let (v, b) = self.w.overflowing_sub(r);
        self.zn(v);
        self.sf(C, !b)
    }
    fn zn(&mut self, v: u16) {
        self.sf(Z, v == 0);
        self.sf(N, v & 0x8000 != 0)
    }
    fn fl(&self, x: u8) -> bool {
        self.flags & x != 0
    }
    fn sf(&mut self, x: u8, on: bool) {
        if on {
            self.flags |= x
        } else {
            self.flags &= !x
        }
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
        self.sp += 2;
        Ok(v)
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
