use crate::{error::VmError, memory::*};
use svm_asm::stack::instruction::Opcode;

const TRUE: u16 = 0xFFFF;
const FALSE: u16 = 0x0000;

#[derive(Debug, Clone)]
pub struct Cpu {
    pc: u16,
    dsp: u16,
    // Two-cell lazy stack cache. TOS is always the logical top item. NOS is
    // the second item only when `nos_valid`; otherwise the second item (if any)
    // remains in stack RAM at `dsp` until an instruction actually needs it.
    // `dsp` therefore addresses the first logical item below the cached cells.
    tos: u16,
    tos_valid: bool,
    nos: u16,
    nos_valid: bool,
    // Shared return/control stack pointer. Return addresses and loop frames
    // share one physical stack. This removes the dedicated loop-stack pointer.
    rsp: u16,
    irq_enabled: bool,
    // Minimal arithmetic carry/borrow state. Comparisons still return stack values.
    carry: bool,
    halted: bool,
}

impl Default for Cpu {
    fn default() -> Self {
        Self {
            pc: 0,
            dsp: DATA_STACK_TOP,
            tos: 0,
            tos_valid: false,
            nos: 0,
            nos_valid: false,
            rsp: RETURN_STACK_TOP,
            irq_enabled: false,
            carry: false,
            halted: false,
        }
    }
}

impl Cpu {
    pub fn pc(&self) -> u16 {
        self.pc
    }

    pub fn halted(&self) -> bool {
        self.halted
    }

    pub fn reset(&mut self, entry: u16) {
        self.pc = entry;
        self.dsp = DATA_STACK_TOP;
        self.tos = 0;
        self.tos_valid = false;
        self.nos = 0;
        self.nos_valid = false;
        self.rsp = RETURN_STACK_TOP;
        self.irq_enabled = false;
        self.carry = false;
        self.halted = false;
    }

    pub fn step(&mut self, memory: &mut Memory) -> Result<(), VmError> {
        if self.halted {
            return Ok(());
        }
        memory.begin_instruction();
        if self.irq_enabled && memory.irq_active() {
            self.enter_irq(memory)?;
        }
        let raw = self.fetch_u8(memory);
        let opcode = Opcode::try_from(raw).map_err(VmError::InvalidOpcode)?;
        use Opcode::*;
        match opcode {
            Nop => {}
            Halt => self.halted = true,
            Ret => self.pc = self.rpop(memory)?,
            Dup => {
                let v = self.peek(memory, 0)?;
                self.push(memory, v)?;
            }
            Drop => {
                self.drop_cells(memory, 1)?;
            }
            Swap => {
                self.ensure_nos(memory)?;
                std::mem::swap(&mut self.tos, &mut self.nos);
            }
            Over => {
                let v = self.peek(memory, 1)?;
                self.push(memory, v)?;
            }
            Rot => {
                self.ensure_nos(memory)?;
                self.require_data_depth(3)?;
                let third = memory.read_u16(self.dsp)?;
                memory.write_u16(self.dsp, self.nos)?;
                self.nos = self.tos;
                self.tos = third;
            }
            // Assembly-oriented convenience primitive; kept for compact
            // hand-written stack/Forth-style code rather than C codegen.
            Nip => {
                self.ensure_nos(memory)?;
                // Remove NOS and keep TOS. The next lower item stays lazy in RAM.
                self.nos_valid = false;
            }
            Tuck => {
                self.ensure_nos(memory)?;
                self.ensure_push_capacity(1)?;
                // ( a b -- b a b ): TOS/NOS already contain b/a. Only the
                // new third item must spill to RAM.
                let next = self.dsp.checked_sub(2).ok_or(VmError::DataStackOverflow)?;
                if next < DATA_STACK_BOTTOM {
                    return Err(VmError::DataStackOverflow);
                }
                memory.write_u16(next, self.tos)?;
                self.dsp = next;
            }
            TwoDup => {
                let a = self.peek(memory, 1)?;
                let b = self.peek(memory, 0)?;
                self.ensure_push_capacity(2)?;
                self.push(memory, a)?;
                self.push(memory, b)?;
            }
            TwoDrop => {
                self.drop_two(memory)?;
            }
            Load8PostInc => {
                // ( addr -- addr+1 value )
                self.require_data_depth(1)?;
                let addr = self.tos;
                let value = u16::from(memory.read_u8(addr));
                self.tos = addr.wrapping_add(1);
                self.push(memory, value)?;
            }
            Store8PostInc => {
                // ( value addr -- addr+1 )
                self.ensure_nos(memory)?;
                let addr = self.tos;
                let value = self.nos;
                memory.write_u8(addr, value as u8);
                self.nos_valid = false;
                self.tos = addr.wrapping_add(1);
            }
            Load16PostInc => {
                // ( addr -- addr+2 value )
                self.require_data_depth(1)?;
                let addr = self.tos;
                let value = memory.read_u16(addr)?;
                self.tos = addr.wrapping_add(2);
                self.push(memory, value)?;
            }
            Store16PostInc => {
                // ( value addr -- addr+2 )
                self.ensure_nos(memory)?;
                let addr = self.tos;
                let value = self.nos;
                memory.write_u16(addr, value)?;
                self.nos_valid = false;
                self.tos = addr.wrapping_add(2);
            }
            Load8PreDec => {
                // ( addr -- addr-1 value )
                self.require_data_depth(1)?;
                let addr = self.tos.wrapping_sub(1);
                let value = u16::from(memory.read_u8(addr));
                self.tos = addr;
                self.push(memory, value)?;
            }
            Store8PreDec => {
                // ( value addr -- addr-1 )
                self.ensure_nos(memory)?;
                let addr = self.tos.wrapping_sub(1);
                let value = self.nos;
                memory.write_u8(addr, value as u8);
                self.nos_valid = false;
                self.tos = addr;
            }
            Load16PreDec => {
                // ( addr -- addr-2 value )
                self.require_data_depth(1)?;
                let addr = self.tos.wrapping_sub(2);
                let value = memory.read_u16(addr)?;
                self.tos = addr;
                self.push(memory, value)?;
            }
            Store16PreDec => {
                // ( value addr -- addr-2 )
                self.ensure_nos(memory)?;
                let addr = self.tos.wrapping_sub(2);
                let value = self.nos;
                memory.write_u16(addr, value)?;
                self.nos_valid = false;
                self.tos = addr;
            }
            Add => {
                self.ensure_nos(memory)?;
                let (result, carry) = self.nos.overflowing_add(self.tos);
                self.tos = result;
                self.nos_valid = false;
                self.carry = carry;
            }
            Sub => {
                self.ensure_nos(memory)?;
                let (result, borrow) = self.nos.overflowing_sub(self.tos);
                self.tos = result;
                self.nos_valid = false;
                self.carry = !borrow;
            }
            Mul => {
                memory.charge_internal_cycles(16);
                self.binary(memory, u16::wrapping_mul)?;
            }
            Div => {
                memory.charge_internal_cycles(16);
                self.divmod(memory, false)?;
            }
            Mod => {
                memory.charge_internal_cycles(16);
                self.divmod(memory, true)?;
            }
            Neg => self.unary(memory, |v| 0u16.wrapping_sub(v))?,
            Inc => self.unary(memory, |v| v.wrapping_add(1))?,
            Dec => self.unary(memory, |v| v.wrapping_sub(1))?,
            And => self.binary(memory, |a, b| a & b)?,
            Or => self.binary(memory, |a, b| a | b)?,
            Xor => self.binary(memory, |a, b| a ^ b)?,
            Not => self.unary(memory, |v| !v)?,
            Shl => {
                memory.charge_internal_cycles(1);
                self.binary(memory, |a, b| a.wrapping_shl(u32::from(b & 15)))?;
            }
            Shr => {
                memory.charge_internal_cycles(1);
                self.binary(memory, |a, b| a.wrapping_shr(u32::from(b & 15)))?;
            }
            Shl1 => {
                self.require_data_depth(1)?;
                let value = self.tos;
                self.carry = (value & 0x8000) != 0;
                self.tos = value.wrapping_shl(1);
            }
            Shr1 => {
                self.require_data_depth(1)?;
                let value = self.tos;
                self.carry = (value & 1) != 0;
                self.tos = value >> 1;
            }
            Eq => self.compare(memory, |a, b| a == b)?,
            Ne => self.compare(memory, |a, b| a != b)?,
            Ult => self.compare(memory, |a, b| a < b)?,
            Ugt => self.compare(memory, |a, b| a > b)?,
            Slt => self.compare(memory, |a, b| (a as i16) < (b as i16))?,
            Sgt => self.compare(memory, |a, b| (a as i16) > (b as i16))?,
            ZeroEq => self.unary(memory, |v| flag(v == 0))?,
            ZeroLt => self.unary(memory, |v| flag((v as i16) < 0))?,
            Load8 => {
                let addr = self.peek(memory, 0)?;
                self.tos = u16::from(memory.read_u8(addr));
            }
            Load16 => {
                let addr = self.peek(memory, 0)?;
                self.tos = memory.read_u16(addr)?;
            }
            Store8 => {
                self.ensure_nos(memory)?;
                memory.write_u8(self.tos, self.nos as u8);
                self.drop_two(memory)?;
            }
            Store16 => {
                self.ensure_nos(memory)?;
                memory.write_u16(self.tos, self.nos)?;
                self.drop_two(memory)?;
            }
            // Assembly-oriented structured loop primitive. C codegen does
            // not depend on DO/LOOP hardware; manual stack assembly benefits.
            Do => {
                let start = self.peek(memory, 0)?;
                let limit = self.peek(memory, 1)?;
                self.loop_push(memory, limit, start)?;
                self.drop_two(memory)?;
            }
            I => {
                let (_, index) = self.loop_peek(memory, 0)?;
                self.push(memory, index)?;
            }
            J => {
                let (_, index) = self.loop_peek(memory, 1)?;
                self.push(memory, index)?;
            }
            Unloop => {
                self.loop_pop(memory)?;
            }
            PushTrue => self.push(memory, TRUE)?,
            Push0 => self.push(memory, 0)?,
            Push1 => self.push(memory, 1)?,
            Push2 => self.push(memory, 2)?,
            Push3 => self.push(memory, 3)?,
            Push4 => self.push(memory, 4)?,
            Push5 => self.push(memory, 5)?,
            Push6 => self.push(memory, 6)?,
            Push7 => self.push(memory, 7)?,
            Push8Small => self.push(memory, 8)?,
            Push9 => self.push(memory, 9)?,
            Push10 => self.push(memory, 10)?,
            Push8 => {
                let v = u16::from(self.fetch_u8(memory));
                self.push(memory, v)?;
            }
            PushS8 => {
                let v = self.fetch_u8(memory) as i8 as i16 as u16;
                self.push(memory, v)?;
            }
            Bra8 => self.branch_rel8(memory, true, false)?,
            Bz8 => {
                let take = self.pop(memory)? == 0;
                self.branch_rel8(memory, take, false)?;
            }
            Bnz8 => {
                let take = self.pop(memory)? != 0;
                self.branch_rel8(memory, take, false)?;
            }
            Call8 => self.branch_rel8(memory, true, true)?,
            QDo8 => self.qdo_rel8(memory)?,
            Loop8 => self.loop_rel8(memory, false)?,
            PlusLoop8 => self.loop_rel8(memory, true)?,
            Leave8 => self.leave_rel8(memory)?,
            // Assembly-oriented deep-stack access primitive.
            Pick => {
                let depth = self.fetch_u8(memory) as usize;
                let v = self.peek(memory, depth)?;
                self.push(memory, v)?;
            }
            Roll => {
                let depth = self.fetch_u8(memory) as usize;
                self.roll(memory, depth)?;
            }
            Load8Zp => {
                let addr = u16::from(self.fetch_u8(memory));
                self.push(memory, u16::from(memory.read_u8(addr)))?;
            }
            Load16Zp => {
                let addr = u16::from(self.fetch_u8(memory));
                let value = memory.read_u16(addr)?;
                self.push(memory, value)?;
            }
            Store8Zp => {
                let addr = u16::from(self.fetch_u8(memory));
                let value = self.peek(memory, 0)?;
                memory.write_u8(addr, value as u8);
                self.drop_cells(memory, 1)?;
            }
            Store16Zp => {
                let addr = u16::from(self.fetch_u8(memory));
                let value = self.peek(memory, 0)?;
                memory.write_u16(addr, value)?;
                self.drop_cells(memory, 1)?;
            }
            Sys => match self.fetch_u8(memory) {
                0 => self.irq_enabled = true,
                1 => self.irq_enabled = false,
                2 => {
                    self.irq_enabled = self.rpop(memory)? != 0;
                    self.pc = self.rpop(memory)?;
                }
                3 => {
                    self.unary(memory, |v| ((v as i16) >> 1) as u16)?;
                }
                4 => {
                    memory.charge_internal_cycles(17);
                    self.binary(memory, q15_mul)?;
                }
                5 => {
                    self.ensure_nos(memory)?;
                    memory.charge_internal_cycles(16);
                    let p = (self.nos as u32) * (self.tos as u32);
                    self.nos = p as u16;
                    self.tos = (p >> 16) as u16;
                    self.nos_valid = true;
                }
                6 => {
                    // ( a b -- a+b+C ), updates C with carry-out.
                    self.ensure_nos(memory)?;
                    let carry_in = u16::from(self.carry);
                    let (partial, c1) = self.nos.overflowing_add(self.tos);
                    let (result, c2) = partial.overflowing_add(carry_in);
                    self.tos = result;
                    self.nos_valid = false;
                    self.carry = c1 || c2;
                }
                7 => {
                    // C=1 means no borrow. ( a b -- a-b-(1-C) ).
                    self.ensure_nos(memory)?;
                    let borrow_in = u16::from(!self.carry);
                    let (partial, b1) = self.nos.overflowing_sub(self.tos);
                    let (result, b2) = partial.overflowing_sub(borrow_in);
                    self.tos = result;
                    self.nos_valid = false;
                    self.carry = !(b1 || b2);
                }
                8 => {
                    // ( a -- r ), old C enters bit15, old bit0 becomes new C.
                    self.require_data_depth(1)?;
                    let value = self.tos;
                    let carry_in = self.carry;
                    self.carry = (value & 1) != 0;
                    self.tos = (value >> 1) | if carry_in { 0x8000 } else { 0 };
                }
                0x10 => {
                    let a = self.peek(memory, 0)?;
                    self.tos = u16::from(memory.video_read_u8(a));
                }
                0x11 => {
                    let a = self.peek(memory, 0)?;
                    self.tos = memory.video_read_u16(a)?;
                }
                0x12 => {
                    self.ensure_nos(memory)?;
                    memory.video_write_u8(self.tos, self.nos as u8);
                    self.drop_two(memory)?;
                }
                0x13 => {
                    self.ensure_nos(memory)?;
                    memory.video_write_u16(self.tos, self.nos)?;
                    self.drop_two(memory)?;
                }
                0x14 => {
                    let a = self.tos;
                    let v = u16::from(memory.video_read_u8(a));
                    self.tos = a.wrapping_add(1);
                    self.push(memory, v)?;
                }
                0x15 => {
                    let a = self.tos;
                    let v = memory.video_read_u16(a)?;
                    self.tos = a.wrapping_add(2);
                    self.push(memory, v)?;
                }
                0x16 => {
                    self.ensure_nos(memory)?;
                    let a = self.tos;
                    let v = self.nos;
                    memory.video_write_u8(a, v as u8);
                    self.nos_valid = false;
                    self.tos = a.wrapping_add(1);
                }
                0x17 => {
                    self.ensure_nos(memory)?;
                    let a = self.tos;
                    let v = self.nos;
                    memory.video_write_u16(a, v)?;
                    self.nos_valid = false;
                    self.tos = a.wrapping_add(2);
                }
                0x18 => {
                    let a = self.tos.wrapping_sub(1);
                    let v = u16::from(memory.video_read_u8(a));
                    self.tos = a;
                    self.push(memory, v)?;
                }
                0x19 => {
                    let a = self.tos.wrapping_sub(2);
                    let v = memory.video_read_u16(a)?;
                    self.tos = a;
                    self.push(memory, v)?;
                }
                0x1A => {
                    self.ensure_nos(memory)?;
                    let a = self.tos.wrapping_sub(1);
                    let v = self.nos;
                    memory.video_write_u8(a, v as u8);
                    self.nos_valid = false;
                    self.tos = a;
                }
                0x1B => {
                    self.ensure_nos(memory)?;
                    let a = self.tos.wrapping_sub(2);
                    let v = self.nos;
                    memory.video_write_u16(a, v)?;
                    self.nos_valid = false;
                    self.tos = a;
                }
                other => return Err(VmError::InvalidOpcode(other)),
            },
            Push16 => {
                let v = self.fetch_u16(memory);
                self.push(memory, v)?;
            }
            Jmp => {
                let target = self.fetch_u16(memory);
                self.pc = target;
            }
            Jz => {
                let target = self.fetch_u16(memory);
                if self.pop(memory)? == 0 {
                    self.pc = target;
                }
            }
            Jnz => {
                let target = self.fetch_u16(memory);
                if self.pop(memory)? != 0 {
                    self.pc = target;
                }
            }
            Call => {
                let target = self.fetch_u16(memory);
                let ret = self.pc;
                self.rpush(memory, ret)?;
                self.pc = target;
            }
            QDo => {
                let target = self.fetch_u16(memory);
                self.qdo_abs(memory, target)?;
            }
            Loop => {
                let target = self.fetch_u16(memory);
                self.loop_abs(memory, target, false)?;
            }
            PlusLoop => {
                let target = self.fetch_u16(memory);
                self.loop_abs(memory, target, true)?;
            }
            Leave => {
                let target = self.fetch_u16(memory);
                self.loop_pop(memory)?;
                self.pc = target;
            }
            Load8Abs => {
                let addr = self.fetch_u16(memory);
                self.push(memory, u16::from(memory.read_u8(addr)))?;
            }
            Load16Abs => {
                let addr = self.fetch_u16(memory);
                let value = memory.read_u16(addr)?;
                self.push(memory, value)?;
            }
            Store8Abs => {
                let addr = self.fetch_u16(memory);
                let value = self.peek(memory, 0)?;
                memory.write_u8(addr, value as u8);
                self.drop_cells(memory, 1)?;
            }
            Store16Abs => {
                let addr = self.fetch_u16(memory);
                let value = self.peek(memory, 0)?;
                memory.write_u16(addr, value)?;
                self.drop_cells(memory, 1)?;
            }
        }
        memory.retire_instruction();
        Ok(())
    }

    fn enter_irq(&mut self, memory: &mut Memory) -> Result<(), VmError> {
        let was_enabled = self.irq_enabled;
        self.rpush(memory, self.pc)?;
        self.rpush(memory, u16::from(was_enabled))?;
        self.irq_enabled = false;
        memory.charge_internal_cycles(2);
        self.pc = memory.irq_vector();
        Ok(())
    }

    fn fetch_u8(&mut self, memory: &Memory) -> u8 {
        let value = memory.read_u8(self.pc);
        self.pc = self.pc.wrapping_add(1);
        value
    }

    fn fetch_u16(&mut self, memory: &Memory) -> u16 {
        let lo = self.fetch_u8(memory);
        let hi = self.fetch_u8(memory);
        u16::from_le_bytes([lo, hi])
    }

    fn relative_target(&self, offset: i8) -> u16 {
        self.pc.wrapping_add_signed(i16::from(offset))
    }

    fn cached_depth(&self) -> usize {
        usize::from(self.tos_valid) + usize::from(self.nos_valid)
    }

    fn data_depth(&self) -> usize {
        let backed = (DATA_STACK_TOP - self.dsp) as usize / 2;
        backed + self.cached_depth()
    }

    /// Materialize the logical second stack item into NOS only when it is
    /// actually needed. Loading it removes that cell from the RAM-backed part
    /// of the representation, so no eager refill is required after ALU ops.
    fn ensure_nos(&mut self, memory: &Memory) -> Result<(), VmError> {
        self.require_data_depth(2)?;
        if !self.nos_valid {
            self.nos = memory.read_u16(self.dsp)?;
            self.dsp += 2;
            self.nos_valid = true;
        }
        Ok(())
    }

    fn push(&mut self, memory: &mut Memory, value: u16) -> Result<(), VmError> {
        const CAPACITY: usize = ((DATA_STACK_TOP - DATA_STACK_BOTTOM) as usize) / 2;
        if self.data_depth() >= CAPACITY {
            return Err(VmError::DataStackOverflow);
        }
        if !self.tos_valid {
            self.tos = value;
            self.tos_valid = true;
            return Ok(());
        }
        if !self.nos_valid {
            self.nos = self.tos;
            self.nos_valid = true;
            self.tos = value;
            return Ok(());
        }
        let next = self.dsp.checked_sub(2).ok_or(VmError::DataStackOverflow)?;
        if next < DATA_STACK_BOTTOM {
            return Err(VmError::DataStackOverflow);
        }
        memory.write_u16(next, self.nos)?;
        self.dsp = next;
        self.nos = self.tos;
        self.tos = value;
        Ok(())
    }

    fn pop(&mut self, memory: &Memory) -> Result<u16, VmError> {
        if !self.tos_valid {
            return Err(VmError::DataStackUnderflow);
        }
        let value = self.tos;
        if self.nos_valid {
            self.tos = self.nos;
            self.nos_valid = false;
        } else if self.dsp < DATA_STACK_TOP {
            // TOS itself cannot be lazy: after pop the new logical top must be
            // available. NOS, however, remains lazy.
            self.tos = memory.read_u16(self.dsp)?;
            self.dsp += 2;
        } else {
            self.tos_valid = false;
        }
        Ok(value)
    }

    fn ensure_push_capacity(&self, count: usize) -> Result<(), VmError> {
        const CAPACITY: usize = ((DATA_STACK_TOP - DATA_STACK_BOTTOM) as usize) / 2;
        if self
            .data_depth()
            .checked_add(count)
            .map_or(true, |d| d > CAPACITY)
        {
            return Err(VmError::DataStackOverflow);
        }
        Ok(())
    }

    fn drop_cells(&mut self, memory: &Memory, count: usize) -> Result<(), VmError> {
        self.require_data_depth(count)?;
        for _ in 0..count {
            self.pop(memory)?;
        }
        Ok(())
    }

    fn drop_two(&mut self, memory: &Memory) -> Result<(), VmError> {
        self.require_data_depth(2)?;
        if self.nos_valid {
            self.nos_valid = false;
        } else {
            // The logical second item is still in RAM: discard it without
            // first refilling NOS.
            self.dsp += 2;
        }
        if self.dsp < DATA_STACK_TOP {
            self.tos = memory.read_u16(self.dsp)?;
            self.dsp += 2;
            self.tos_valid = true;
        } else {
            self.tos_valid = false;
        }
        Ok(())
    }

    fn peek(&self, memory: &Memory, depth: usize) -> Result<u16, VmError> {
        if depth >= self.data_depth() {
            return Err(VmError::DataStackUnderflow);
        }
        if depth == 0 {
            return Ok(self.tos);
        }
        if self.nos_valid && depth == 1 {
            return Ok(self.nos);
        }
        memory.read_u16(self.backing_cell_address(depth)?)
    }

    fn backing_cell_address(&self, depth: usize) -> Result<u16, VmError> {
        let cached = self.cached_depth();
        if depth < cached || depth >= self.data_depth() {
            return Err(VmError::DataStackUnderflow);
        }
        let offset = depth
            .checked_sub(cached)
            .and_then(|d| d.checked_mul(2))
            .ok_or(VmError::DataStackUnderflow)?;
        let address = (self.dsp as usize)
            .checked_add(offset)
            .ok_or(VmError::DataStackUnderflow)?;
        if address + 1 >= DATA_STACK_TOP as usize {
            return Err(VmError::DataStackUnderflow);
        }
        Ok(address as u16)
    }

    fn set_depth(&mut self, memory: &mut Memory, depth: usize, value: u16) -> Result<(), VmError> {
        if depth >= self.data_depth() {
            return Err(VmError::DataStackUnderflow);
        }
        if depth == 0 {
            self.tos = value;
            return Ok(());
        }
        if self.nos_valid && depth == 1 {
            self.nos = value;
            return Ok(());
        }
        let address = self.backing_cell_address(depth)?;
        memory.write_u16(address, value)
    }

    fn roll(&mut self, memory: &mut Memory, depth: usize) -> Result<(), VmError> {
        if depth == 0 {
            self.require_data_depth(1)?;
            return Ok(());
        }
        self.require_data_depth(depth + 1)?;
        let selected = self.peek(memory, depth)?;
        for current_depth in (1..=depth).rev() {
            let above = self.peek(memory, current_depth - 1)?;
            self.set_depth(memory, current_depth, above)?;
        }
        self.tos = selected;
        Ok(())
    }

    fn rpush(&mut self, memory: &mut Memory, value: u16) -> Result<(), VmError> {
        let next = self
            .rsp
            .checked_sub(2)
            .ok_or(VmError::ReturnStackOverflow)?;
        if next < RETURN_STACK_BOTTOM {
            return Err(VmError::ReturnStackOverflow);
        }
        memory.write_u16(next, value)?;
        self.rsp = next;
        Ok(())
    }

    fn rpop(&mut self, memory: &Memory) -> Result<u16, VmError> {
        if self.rsp >= RETURN_STACK_TOP {
            return Err(VmError::ReturnStackUnderflow);
        }
        let value = memory.read_u16(self.rsp)?;
        self.rsp += 2;
        Ok(value)
    }
    fn loop_push(&mut self, memory: &mut Memory, limit: u16, index: u16) -> Result<(), VmError> {
        // A loop frame is two cells on the shared return/control stack.
        // Layout at RSP: [index][limit]. CALL may temporarily push a return
        // address above it, but that address is gone before execution returns
        // to the lexical loop body and reaches I/J/LOOP.
        let next = self
            .rsp
            .checked_sub(4)
            .ok_or(VmError::ReturnStackOverflow)?;
        if next < RETURN_STACK_BOTTOM {
            return Err(VmError::ReturnStackOverflow);
        }
        memory.write_u16(next, index)?;
        memory.write_u16(next + 2, limit)?;
        self.rsp = next;
        Ok(())
    }

    fn loop_pop(&mut self, _memory: &Memory) -> Result<(), VmError> {
        if self.rsp > RETURN_STACK_TOP.saturating_sub(4) {
            return Err(VmError::ReturnStackUnderflow);
        }
        self.rsp += 4;
        Ok(())
    }

    fn loop_peek(&self, memory: &Memory, depth: usize) -> Result<(u16, u16), VmError> {
        let offset = depth.checked_mul(4).ok_or(VmError::ReturnStackUnderflow)?;
        let addr = (self.rsp as usize)
            .checked_add(offset)
            .ok_or(VmError::ReturnStackUnderflow)?;
        if addr + 3 >= RETURN_STACK_TOP as usize {
            return Err(VmError::ReturnStackUnderflow);
        }
        let base = addr as u16;
        let index = memory.read_u16(base)?;
        let limit = memory.read_u16(base + 2)?;
        Ok((limit, index))
    }

    fn loop_set_index(&self, memory: &mut Memory, index: u16) -> Result<(), VmError> {
        if self.rsp > RETURN_STACK_TOP.saturating_sub(4) {
            return Err(VmError::ReturnStackUnderflow);
        }
        memory.write_u16(self.rsp, index)
    }

    fn unary<F: FnOnce(u16) -> u16>(
        &mut self,
        _memory: &mut Memory,
        operation: F,
    ) -> Result<(), VmError> {
        self.require_data_depth(1)?;
        self.tos = operation(self.tos);
        Ok(())
    }

    fn binary<F: FnOnce(u16, u16) -> u16>(
        &mut self,
        memory: &mut Memory,
        operation: F,
    ) -> Result<(), VmError> {
        self.ensure_nos(memory)?;
        self.tos = operation(self.nos, self.tos);
        self.nos_valid = false;
        Ok(())
    }

    fn compare<F: FnOnce(u16, u16) -> bool>(
        &mut self,
        memory: &mut Memory,
        predicate: F,
    ) -> Result<(), VmError> {
        self.ensure_nos(memory)?;
        self.tos = flag(predicate(self.nos, self.tos));
        self.nos_valid = false;
        Ok(())
    }

    fn divmod(&mut self, memory: &mut Memory, modulo: bool) -> Result<(), VmError> {
        self.ensure_nos(memory)?;
        let divisor = self.tos;
        if divisor == 0 {
            return Err(VmError::DivisionByZero);
        }
        let dividend = self.nos;
        self.tos = if modulo {
            dividend % divisor
        } else {
            dividend / divisor
        };
        self.nos_valid = false;
        Ok(())
    }

    fn require_data_depth(&self, count: usize) -> Result<(), VmError> {
        if self.data_depth() < count {
            Err(VmError::DataStackUnderflow)
        } else {
            Ok(())
        }
    }

    fn branch_rel8(&mut self, memory: &mut Memory, take: bool, call: bool) -> Result<(), VmError> {
        let offset = self.fetch_u8(memory) as i8;
        let target = self.relative_target(offset);
        if take {
            if call {
                self.rpush(memory, self.pc)?;
            }
            self.pc = target;
        }
        Ok(())
    }
    fn qdo_rel8(&mut self, memory: &mut Memory) -> Result<(), VmError> {
        let off = self.fetch_u8(memory) as i8;
        let target = self.relative_target(off);
        self.qdo_abs(memory, target)
    }
    fn qdo_abs(&mut self, memory: &mut Memory, target: u16) -> Result<(), VmError> {
        let start = self.peek(memory, 0)?;
        let limit = self.peek(memory, 1)?;
        if start == limit {
            self.drop_two(memory)?;
            self.pc = target;
        } else {
            self.loop_push(memory, limit, start)?;
            self.drop_two(memory)?;
        }
        Ok(())
    }
    fn loop_rel8(&mut self, memory: &mut Memory, plus: bool) -> Result<(), VmError> {
        let off = self.fetch_u8(memory) as i8;
        let target = self.relative_target(off);
        self.loop_abs(memory, target, plus)
    }
    fn loop_abs(&mut self, memory: &mut Memory, target: u16, plus: bool) -> Result<(), VmError> {
        let (limit, index) = self.loop_peek(memory, 0)?;
        let step = if plus { self.pop(memory)? } else { 1 };
        let next = index.wrapping_add(step);
        let continue_loop = if plus {
            plus_loop_continues(index, limit, step)
        } else {
            next != limit
        };
        if continue_loop {
            self.loop_set_index(memory, next)?;
            self.pc = target;
        } else {
            self.loop_pop(memory)?;
        }
        Ok(())
    }
    fn leave_rel8(&mut self, memory: &mut Memory) -> Result<(), VmError> {
        let off = self.fetch_u8(memory) as i8;
        let target = self.relative_target(off);
        self.loop_pop(memory)?;
        self.pc = target;
        Ok(())
    }
}

fn flag(value: bool) -> u16 {
    if value { TRUE } else { FALSE }
}

fn plus_loop_continues(index: u16, limit: u16, step: u16) -> bool {
    // Forth +LOOP terminates when adding the increment crosses the circular
    // boundary between limit-1 and limit. Biasing (index-limit) by MIN-INT
    // maps that boundary to the two's-complement signed overflow boundary.
    let biased = index.wrapping_sub(limit).wrapping_add(0x8000) as i16;
    let (_, crossed_boundary) = biased.overflowing_add(step as i16);
    !crossed_boundary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arithmetic_uses_stack_order() {
        let mut mem = Memory::default();
        mem.load(0, &[0x40, 10, 0x40, 3, 0x11, 0x01]).unwrap();
        let mut cpu = Cpu::default();
        while !cpu.halted() {
            cpu.step(&mut mem).unwrap();
        }
        assert_eq!(cpu.pop(&mem).unwrap(), 7);
    }

    #[test]
    fn carry_chains_through_adc() {
        let mut mem = Memory::default();
        // FFFF 1 ADD => 0000, C=1; 1234 0000 ADC => 1235.
        mem.load(
            0,
            &[
                Opcode::Push16 as u8,
                0xFF,
                0xFF,
                Opcode::Push1 as u8,
                Opcode::Add as u8,
                Opcode::Drop as u8,
                Opcode::Push16 as u8,
                0x34,
                0x12,
                Opcode::Push0 as u8,
                Opcode::Sys as u8,
                6,
                Opcode::Halt as u8,
            ],
        )
        .unwrap();
        let mut cpu = Cpu::default();
        while !cpu.halted() {
            cpu.step(&mut mem).unwrap();
        }
        assert_eq!(cpu.pop(&mem).unwrap(), 0x1235);
        assert!(!cpu.carry);
    }

    #[test]
    fn borrow_chains_through_sbc() {
        let mut mem = Memory::default();
        // 0 - 1 => FFFF, C=0; 1234 - 0000 - 1 => 1233.
        mem.load(
            0,
            &[
                Opcode::Push0 as u8,
                Opcode::Push1 as u8,
                Opcode::Sub as u8,
                Opcode::Drop as u8,
                Opcode::Push16 as u8,
                0x34,
                0x12,
                Opcode::Push0 as u8,
                Opcode::Sys as u8,
                7,
                Opcode::Halt as u8,
            ],
        )
        .unwrap();
        let mut cpu = Cpu::default();
        while !cpu.halted() {
            cpu.step(&mut mem).unwrap();
        }
        assert_eq!(cpu.pop(&mem).unwrap(), 0x1233);
        assert!(cpu.carry);
    }

    #[test]
    fn shift_and_rcr_use_minimal_carry_state() {
        let mut mem = Memory::default();
        // 8001 SHR1 => 4000, C=1; 0002 RCR1 => 8001, C=0.
        mem.load(
            0,
            &[
                Opcode::Push16 as u8,
                0x01,
                0x80,
                Opcode::Shr1 as u8,
                Opcode::Drop as u8,
                Opcode::Push2 as u8,
                Opcode::Sys as u8,
                8,
                Opcode::Halt as u8,
            ],
        )
        .unwrap();
        let mut cpu = Cpu::default();
        while !cpu.halted() {
            cpu.step(&mut mem).unwrap();
        }
        assert_eq!(cpu.pop(&mem).unwrap(), 0x8001);
        assert!(!cpu.carry);
    }

    #[test]
    fn optimized_stack_shuffles_preserve_forth_order() {
        let mut memory = Memory::default();
        // 1 2 SWAP -> 2 1; 3 ROT transforms 2 1 3 -> 1 3 2; NIP -> 1 2
        memory
            .load(
                0,
                &[
                    Opcode::Push8 as u8,
                    1,
                    Opcode::Push8 as u8,
                    2,
                    Opcode::Swap as u8,
                    Opcode::Push8 as u8,
                    3,
                    Opcode::Rot as u8,
                    Opcode::Nip as u8,
                    Opcode::Halt as u8,
                ],
            )
            .unwrap();
        let mut cpu = Cpu::default();
        while !cpu.halted() {
            cpu.step(&mut memory).unwrap();
        }
        assert_eq!(cpu.pop(&memory).unwrap(), 2);
        assert_eq!(cpu.pop(&memory).unwrap(), 1);
    }

    #[test]
    fn binary_underflow_preserves_existing_item() {
        let mut mem = Memory::default();
        mem.load(0, &[0x40, 7, 0x10]).unwrap();
        let mut cpu = Cpu::default();
        cpu.step(&mut mem).unwrap();
        assert!(matches!(
            cpu.step(&mut mem),
            Err(VmError::DataStackUnderflow)
        ));
        assert_eq!(cpu.pop(&mem).unwrap(), 7);
    }

    #[test]
    fn division_by_zero_preserves_operands() {
        let mut mem = Memory::default();
        mem.load(0, &[0x40, 9, 0x40, 0, 0x13]).unwrap();
        let mut cpu = Cpu::default();
        cpu.step(&mut mem).unwrap();
        cpu.step(&mut mem).unwrap();
        assert!(matches!(cpu.step(&mut mem), Err(VmError::DivisionByZero)));
        assert_eq!(cpu.pop(&mem).unwrap(), 0);
        assert_eq!(cpu.pop(&mem).unwrap(), 9);
    }

    #[test]
    fn plus_loop_detects_positive_boundary_crossing() {
        assert!(plus_loop_continues(1, 4, 1));
        assert!(plus_loop_continues(2, 4, 1));
        assert!(!plus_loop_continues(3, 4, 1));
    }

    #[test]
    fn plus_loop_detects_negative_boundary_crossing() {
        let minus_one = (-1_i16) as u16;
        assert!(plus_loop_continues(4, 1, minus_one));
        assert!(plus_loop_continues(3, 1, minus_one));
        assert!(plus_loop_continues(2, 1, minus_one));
        assert!(!plus_loop_continues(1, 1, minus_one));
    }

    #[test]
    fn plus_loop_does_not_exit_early_when_start_is_past_limit() {
        assert!(plus_loop_continues(4, 1, 1));
        assert!(plus_loop_continues(1, 4, (-1_i16) as u16));
    }

    #[test]
    fn zero_plus_loop_step_never_crosses_the_boundary() {
        assert!(plus_loop_continues(123, 456, 0));
    }

    #[test]
    fn comparison_uses_forth_boolean() {
        let mut mem = Memory::default();
        mem.load(0, &[0x40, 7, 0x40, 7, 0x20, 0x01]).unwrap();
        let mut cpu = Cpu::default();
        while !cpu.halted() {
            cpu.step(&mut mem).unwrap();
        }
        assert_eq!(cpu.pop(&mem).unwrap(), 0xFFFF);
    }
    #[test]
    fn one_bit_shifts_are_single_opcode_stack_operations() {
        let mut mem = Memory::default();
        mem.load(
            0,
            &[
                Opcode::Push8 as u8,
                6,
                Opcode::Shl1 as u8,
                Opcode::Shr1 as u8,
                Opcode::Halt as u8,
            ],
        )
        .unwrap();
        let mut cpu = Cpu::default();
        while !cpu.halted() {
            cpu.step(&mut mem).unwrap();
        }
        assert_eq!(cpu.pop(&mem).unwrap(), 6);
    }

    #[test]
    fn tos_nos_cache_preserves_logical_stack_order() {
        let mut mem = Memory::default();
        let mut cpu = Cpu::default();
        cpu.push(&mut mem, 1).unwrap();
        cpu.push(&mut mem, 2).unwrap();
        assert_eq!(cpu.tos, 2);
        assert_eq!(cpu.nos, 1);
        assert!(cpu.nos_valid);
        cpu.push(&mut mem, 3).unwrap();
        assert_eq!(cpu.tos, 3);
        assert_eq!(cpu.nos, 2);
        assert!(cpu.nos_valid);
        assert_eq!(cpu.pop(&mem).unwrap(), 3);
        assert_eq!(cpu.pop(&mem).unwrap(), 2);
        assert_eq!(cpu.pop(&mem).unwrap(), 1);
        assert!(matches!(cpu.pop(&mem), Err(VmError::DataStackUnderflow)));
    }

    #[test]
    fn two_cached_operands_make_add_fetch_only() {
        let mut mem = Memory::default();
        mem.load(
            0,
            &[
                Opcode::Push1 as u8,
                Opcode::Push2 as u8,
                Opcode::Add as u8,
                Opcode::Halt as u8,
            ],
        )
        .unwrap();
        let mut cpu = Cpu::default();
        while !cpu.halted() {
            cpu.step(&mut mem).unwrap();
        }
        assert_eq!(cpu.pop(&mem).unwrap(), 3);
        // Four one-byte instruction fetches; PUSH/PUSH/ADD never touch data-stack RAM.
        assert_eq!(mem.cycle_count(), 4);
    }

    #[test]
    fn binary_result_leaves_nos_lazy_until_needed() {
        let mut mem = Memory::default();
        let mut cpu = Cpu::default();
        cpu.push(&mut mem, 1).unwrap();
        cpu.push(&mut mem, 2).unwrap();
        cpu.push(&mut mem, 3).unwrap();
        // Representation: TOS=3, NOS=2, RAM contains 1.
        cpu.binary(&mut mem, u16::wrapping_add).unwrap();
        assert_eq!(cpu.tos, 5);
        assert!(!cpu.nos_valid);
        assert_eq!(cpu.peek(&mem, 1).unwrap(), 1);
        // A unary operation must not refill NOS.
        cpu.unary(&mut mem, |v| !v).unwrap();
        assert!(!cpu.nos_valid);
    }

    #[test]
    fn absolute_store_and_load_use_tos_value() {
        let mut mem = Memory::default();
        mem.load(
            0,
            &[
                Opcode::Push8 as u8,
                0x5A,
                Opcode::Store8Abs as u8,
                0x00,
                0x20,
                Opcode::Load8Abs as u8,
                0x00,
                0x20,
                Opcode::Halt as u8,
            ],
        )
        .unwrap();
        let mut cpu = Cpu::default();
        while !cpu.halted() {
            cpu.step(&mut mem).unwrap();
        }
        assert_eq!(cpu.pop(&mem).unwrap(), 0x5A);
    }

    #[test]
    fn post_increment_memory_primitives_preserve_updated_address() {
        let mut mem = Memory::default();
        mem.write_u8(0x2000, 0x5A);
        mem.write_u16(0x2001, 0x1234).unwrap();
        mem.load(
            0,
            &[
                Opcode::Push16 as u8,
                0x00,
                0x20,
                Opcode::Load8PostInc as u8,
                Opcode::Drop as u8,
                Opcode::Load16PostInc as u8,
                Opcode::Halt as u8,
            ],
        )
        .unwrap();
        let mut cpu = Cpu::default();
        while !cpu.halted() {
            cpu.step(&mut mem).unwrap();
        }
        assert_eq!(cpu.pop(&mem).unwrap(), 0x1234);
        assert_eq!(cpu.pop(&mem).unwrap(), 0x2003);

        let mut mem = Memory::default();
        mem.load(
            0,
            &[
                Opcode::Push8 as u8,
                0x77,
                Opcode::Push16 as u8,
                0x00,
                0x21,
                Opcode::Store8PostInc as u8,
                Opcode::Halt as u8,
            ],
        )
        .unwrap();
        let mut cpu = Cpu::default();
        while !cpu.halted() {
            cpu.step(&mut mem).unwrap();
        }
        assert_eq!(mem.read_u8(0x2100), 0x77);
        assert_eq!(cpu.pop(&mem).unwrap(), 0x2101);
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
