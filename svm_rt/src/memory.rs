use crate::{error::VmError, video::FONT_8X8};
use std::{cell::Cell, collections::VecDeque};

pub const MEMORY_SIZE: usize = 65_536;
pub const VIDEO_MEMORY_SIZE: usize = 16_384;
pub const MMIO_BASE: u16 = 0xFF00;
pub const MMIO_SIZE: usize = 0x0100;

pub const KEY_STATUS: u16 = 0xFF00;
pub const KEY_CODE: u16 = 0xFF01;
pub const TEXT_X: u16 = 0xFF02;
pub const TEXT_Y: u16 = 0xFF03;
pub const TEXT_FG: u16 = 0xFF04;
pub const TEXT_BG: u16 = 0xFF05;
pub const TEXT_CHAR: u16 = 0xFF06;
// 0xFF07..0xFF0A are reserved for future video control.
pub const VIDEO_VSYNC_COUNTER: u16 = 0xFF0B;
// Each 2-bit pixel selects one of these four slots. Each slot contains a 4-bit
// index into the fixed 16-colour master palette.
pub const VIDEO_PALETTE0: u16 = 0xFF0C;
pub const VIDEO_PALETTE1: u16 = 0xFF0D;
pub const VIDEO_PALETTE2: u16 = 0xFF0E;
pub const VIDEO_PALETTE3: u16 = 0xFF0F;

// 0xFF10..0xFF11 reserved (legacy host output removed; use VT100 console at 0xFF20..0xFF21).
pub const IRQ_ENABLE: u16 = 0xFF12;
pub const IRQ_PENDING: u16 = 0xFF13;
pub const IRQ_ACK: u16 = 0xFF14;
pub const TIMER_CONTROL: u16 = 0xFF15;
pub const TIMER_RELOAD_LO: u16 = 0xFF16;
pub const TIMER_RELOAD_HI: u16 = 0xFF17;
pub const TIMER_COUNT_LO: u16 = 0xFF18;
pub const TIMER_COUNT_HI: u16 = 0xFF19;
pub const CLOCK_TICK_0: u16 = 0xFF1A;
pub const CLOCK_TICK_1: u16 = 0xFF1B;
pub const CLOCK_TICK_2: u16 = 0xFF1C;
pub const CLOCK_TICK_3: u16 = 0xFF1D;
pub const IRQ_VECTOR_LO: u16 = 0xFF1E;
pub const IRQ_VECTOR_HI: u16 = 0xFF1F;
pub const IRQ_TIMER: u8 = 0x01;
pub const IRQ_VSYNC: u8 = 0x02;
pub const IRQ_KEY: u8 = 0x04;
pub const IRQ_CONSOLE_RX: u8 = 0x08;
pub const CONSOLE_DATA: u16 = 0xFF20;
pub const CONSOLE_STATUS: u16 = 0xFF21;
pub const INSTRUCTION_COUNT_0: u16 = 0xFF22;
pub const INSTRUCTION_COUNT_1: u16 = 0xFF23;
pub const INSTRUCTION_COUNT_2: u16 = 0xFF24;
pub const INSTRUCTION_COUNT_3: u16 = 0xFF25;
// Hardware-assisted 16-bit pseudo-random generator. Reading RNG_DATA_LO
// advances the generator and latches one 16-bit sample; RNG_DATA_HI returns
// the high byte of that same sample. A normal 16-bit load from 0xFF26 is
// therefore atomic from the guest's point of view.
pub const RNG_DATA_LO: u16 = 0xFF26;
pub const RNG_DATA_HI: u16 = 0xFF27;
pub const RNG_STATUS: u16 = 0xFF28;
pub const RNG_SEED_LO: u16 = 0xFF29;
pub const RNG_SEED_HI: u16 = 0xFF2A;
pub const RNG_READY: u8 = 0x01;
pub const CONSOLE_RX_READY: u8 = 0x01;
pub const CONSOLE_TX_READY: u8 = 0x02;
pub const CONSOLE_RX_FIFO_CAPACITY: usize = 256;
pub const TIMER_ENABLE: u8 = 0x01;
pub const TIMER_PERIODIC: u8 = 0x02;

pub const FRAMEBUFFER_WIDTH: usize = 320;
pub const FRAMEBUFFER_HEIGHT: usize = 200;
pub const FRAMEBUFFER_BPP: usize = 2;
pub const FRAMEBUFFER_SIZE: usize = FRAMEBUFFER_WIDTH * FRAMEBUFFER_HEIGHT * FRAMEBUFFER_BPP / 8; // 16,000 bytes
pub const VIDEO_RESERVED_BYTES: usize = VIDEO_MEMORY_SIZE - FRAMEBUFFER_SIZE; // 384 bytes
pub const TEXT_COLUMNS: usize = FRAMEBUFFER_WIDTH / 8;
pub const TEXT_ROWS: usize = FRAMEBUFFER_HEIGHT / 8;

pub const STACK_BOTTOM: u16 = 0xFB00;
pub const STACK_TOP_EXCLUSIVE: u16 = 0xFF00;

// Runtime stack convention inside the same contiguous RAM. Other CPUs use 0xFB00..0xFEFF as one stack;
// Stack/Belt/TTA split it into 512-byte data and control/return stacks.
pub const DATA_STACK_BOTTOM: u16 = 0xFB00;
pub const DATA_STACK_TOP: u16 = 0xFD00;
pub const RETURN_STACK_BOTTOM: u16 = 0xFD00;
pub const RETURN_STACK_TOP: u16 = 0xFF00;

#[derive(Clone)]
pub struct Memory {
    bytes: Box<[u8; MEMORY_SIZE]>,
    video: Box<[u8; VIDEO_MEMORY_SIZE]>,
    vsync_counter: u8,
    cycle_count: u32,
    instruction_count: u32,
    pending_cycles: Cell<u32>,
    timer_reload: u16,
    timer_count: u16,
    console_rx: VecDeque<u8>,
    console_tx: VecDeque<u8>,
    rng_state: Cell<u32>,
    rng_latch: Cell<u16>,
}

impl Default for Memory {
    fn default() -> Self {
        let mut bytes = Box::new([0u8; MEMORY_SIZE]);
        bytes[TEXT_FG as usize] = 3;
        bytes[TEXT_BG as usize] = 0;
        bytes[VIDEO_PALETTE0 as usize] = 0x00;
        bytes[VIDEO_PALETTE1 as usize] = 0x08;
        bytes[VIDEO_PALETTE2 as usize] = 0x07;
        bytes[VIDEO_PALETTE3 as usize] = 0x0F;
        let video = Box::new([0u8; VIDEO_MEMORY_SIZE]);
        let mut memory = Self {
            bytes,
            video,
            vsync_counter: 0,
            cycle_count: 0,
            instruction_count: 0,
            pending_cycles: Cell::new(0),
            timer_reload: 0,
            timer_count: 0,
            console_rx: VecDeque::new(),
            console_tx: VecDeque::new(),
            rng_state: Cell::new(0x6D2B_79F5),
            rng_latch: Cell::new(0),
        };
        memory.sync_registers();
        memory
    }
}

impl Memory {
    fn sync_registers(&mut self) {
        self.bytes[VIDEO_VSYNC_COUNTER as usize] = self.vsync_counter;
        let cycles = self.cycle_count.to_le_bytes();
        self.bytes[CLOCK_TICK_0 as usize..=CLOCK_TICK_3 as usize].copy_from_slice(&cycles);
        let instructions = self.instruction_count.to_le_bytes();
        self.bytes[INSTRUCTION_COUNT_0 as usize..=INSTRUCTION_COUNT_3 as usize]
            .copy_from_slice(&instructions);
        let reload = self.timer_reload.to_le_bytes();
        self.bytes[TIMER_RELOAD_LO as usize] = reload[0];
        self.bytes[TIMER_RELOAD_HI as usize] = reload[1];
        let count = self.timer_count.to_le_bytes();
        self.bytes[TIMER_COUNT_LO as usize] = count[0];
        self.bytes[TIMER_COUNT_HI as usize] = count[1];
    }

    pub fn video_vsync(&mut self) {
        self.vsync_counter = self.vsync_counter.wrapping_add(1);
        self.raise_irq(IRQ_VSYNC);
        self.sync_registers();
    }

    /// Start accounting one guest instruction. CPU-visible memory accesses
    /// charge bus cycles automatically; internal multi-cycle operations add
    /// their own cost with `charge_internal_cycles`.
    pub fn begin_instruction(&self) {
        self.pending_cycles.set(0);
    }

    pub fn charge_internal_cycles(&self, cycles: u32) {
        self.pending_cycles
            .set(self.pending_cycles.get().wrapping_add(cycles));
    }

    pub fn retire_instruction(&mut self) {
        let cycles = self.pending_cycles.replace(0).max(1);
        self.cycle_count = self.cycle_count.wrapping_add(cycles);
        self.instruction_count = self.instruction_count.wrapping_add(1);
        for _ in 0..cycles {
            if self.bytes[TIMER_CONTROL as usize] & TIMER_ENABLE != 0 {
                if self.timer_count == 0 {
                    self.timer_count = self.timer_reload;
                }
                if self.timer_count != 0 {
                    self.timer_count = self.timer_count.wrapping_sub(1);
                    if self.timer_count == 0 {
                        self.raise_irq(IRQ_TIMER);
                        if self.bytes[TIMER_CONTROL as usize] & TIMER_PERIODIC != 0 {
                            self.timer_count = self.timer_reload;
                        } else {
                            self.bytes[TIMER_CONTROL as usize] &= !TIMER_ENABLE;
                        }
                    }
                }
            }
        }
        self.sync_registers();
    }

    pub fn cycle_count(&self) -> u32 {
        self.cycle_count
    }
    pub fn instruction_count(&self) -> u32 {
        self.instruction_count
    }

    pub fn irq_active(&self) -> bool {
        self.bytes[IRQ_PENDING as usize] & self.bytes[IRQ_ENABLE as usize] != 0
    }
    pub fn irq_vector(&self) -> u16 {
        u16::from_le_bytes([
            self.bytes[IRQ_VECTOR_LO as usize],
            self.bytes[IRQ_VECTOR_HI as usize],
        ])
    }
    pub fn raise_irq(&mut self, source: u8) {
        self.bytes[IRQ_PENDING as usize] |= source;
    }

    fn rng_next_u16(&self) -> u16 {
        // xorshift32: tiny hardware-friendly state machine, non-cryptographic.
        let mut x = self.rng_state.get();
        if x == 0 {
            x = 0x6D2B_79F5;
        }
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng_state.set(x);
        let value = ((x >> 16) as u16) ^ (x as u16);
        self.rng_latch.set(value);
        value
    }

    fn rng_seed_from_registers(&self) {
        let seed16 = u16::from_le_bytes([
            self.bytes[RNG_SEED_LO as usize],
            self.bytes[RNG_SEED_HI as usize],
        ]);
        let seed = if seed16 == 0 {
            0x6D2B_79F5
        } else {
            // Expand a 16-bit guest seed into a non-zero 32-bit state.
            0xA5A5_0000u32 ^ seed16 as u32 ^ ((seed16 as u32) << 16)
        };
        self.rng_state.set(seed);
        self.rng_latch.set(0);
    }

    // Ordinary CPU address space. Instruction fetches use only these methods.
    pub fn read_u8(&self, address: u16) -> u8 {
        self.charge_internal_cycles(1);
        match address {
            RNG_DATA_LO => (self.rng_next_u16() & 0x00FF) as u8,
            RNG_DATA_HI => (self.rng_latch.get() >> 8) as u8,
            RNG_STATUS => RNG_READY,
            CONSOLE_DATA => self.console_rx.front().copied().unwrap_or(0),
            CONSOLE_STATUS => {
                (if self.console_rx.is_empty() {
                    0
                } else {
                    CONSOLE_RX_READY
                }) | CONSOLE_TX_READY
            }
            _ => self.bytes[address as usize],
        }
    }
    pub fn write_u8(&mut self, address: u16, value: u8) {
        self.charge_internal_cycles(1);
        match address {
            CONSOLE_DATA => {
                self.console_tx.push_back(value);
            }
            CONSOLE_STATUS => {
                if value & CONSOLE_RX_READY != 0 {
                    self.console_rx.pop_front();
                    if !self.console_rx.is_empty() {
                        self.raise_irq(IRQ_CONSOLE_RX);
                    }
                }
            }
            VIDEO_VSYNC_COUNTER | IRQ_PENDING | CLOCK_TICK_0 | CLOCK_TICK_1 | CLOCK_TICK_2
            | CLOCK_TICK_3 | INSTRUCTION_COUNT_0 | INSTRUCTION_COUNT_1 | INSTRUCTION_COUNT_2
            | INSTRUCTION_COUNT_3 | RNG_DATA_LO | RNG_DATA_HI | RNG_STATUS => {}
            RNG_SEED_LO | RNG_SEED_HI => {
                self.bytes[address as usize] = value;
                self.rng_seed_from_registers();
            }
            IRQ_ACK => {
                self.bytes[IRQ_PENDING as usize] &= !value;
                if value & IRQ_CONSOLE_RX != 0 && !self.console_rx.is_empty() {
                    self.bytes[IRQ_PENDING as usize] |= IRQ_CONSOLE_RX;
                }
            }
            TIMER_CONTROL => {
                self.bytes[TIMER_CONTROL as usize] = value & (TIMER_ENABLE | TIMER_PERIODIC);
                if value & TIMER_ENABLE != 0 && self.timer_count == 0 {
                    self.timer_count = self.timer_reload;
                }
                self.sync_registers();
            }
            TIMER_RELOAD_LO | TIMER_RELOAD_HI => {
                self.bytes[address as usize] = value;
                self.timer_reload = u16::from_le_bytes([
                    self.bytes[TIMER_RELOAD_LO as usize],
                    self.bytes[TIMER_RELOAD_HI as usize],
                ]);
                self.sync_registers();
            }
            TIMER_COUNT_LO | TIMER_COUNT_HI => {
                self.bytes[address as usize] = value;
                self.timer_count = u16::from_le_bytes([
                    self.bytes[TIMER_COUNT_LO as usize],
                    self.bytes[TIMER_COUNT_HI as usize],
                ]);
                self.sync_registers();
            }
            VIDEO_PALETTE0 | VIDEO_PALETTE1 | VIDEO_PALETTE2 | VIDEO_PALETTE3 => {
                self.bytes[address as usize] = value & 0x0F;
            }
            _ => {
                self.bytes[address as usize] = value;
                if address == TEXT_CHAR {
                    self.draw_text_char(value);
                }
            }
        }
    }

    pub fn read_u16(&self, address: u16) -> Result<u16, VmError> {
        let hi = address
            .checked_add(1)
            .ok_or(VmError::InvalidMemoryAccess { address, width: 2 })?;
        Ok(u16::from_le_bytes([
            self.read_u8(address),
            self.read_u8(hi),
        ]))
    }
    pub fn write_u16(&mut self, address: u16, value: u16) -> Result<(), VmError> {
        let hi = address
            .checked_add(1)
            .ok_or(VmError::InvalidMemoryAccess { address, width: 2 })?;
        let [lo, hi_byte] = value.to_le_bytes();
        self.write_u8(address, lo);
        self.write_u8(hi, hi_byte);
        Ok(())
    }

    // Separate video data space. There is no instruction fetch path from this space.
    pub fn video_read_u8(&self, address: u16) -> u8 {
        self.charge_internal_cycles(1);
        self.video.get(address as usize).copied().unwrap_or(0)
    }
    pub fn video_write_u8(&mut self, address: u16, value: u8) {
        self.charge_internal_cycles(1);
        if let Some(slot) = self.video.get_mut(address as usize) {
            *slot = value;
        }
    }
    pub fn video_read_u16(&self, address: u16) -> Result<u16, VmError> {
        self.charge_internal_cycles(2);
        let start = address as usize;
        if start + 1 >= VIDEO_MEMORY_SIZE {
            return Err(VmError::InvalidMemoryAccess { address, width: 2 });
        }
        Ok(u16::from_le_bytes([
            self.video[start],
            self.video[start + 1],
        ]))
    }
    pub fn video_write_u16(&mut self, address: u16, value: u16) -> Result<(), VmError> {
        self.charge_internal_cycles(2);
        let start = address as usize;
        if start + 1 >= VIDEO_MEMORY_SIZE {
            return Err(VmError::InvalidMemoryAccess { address, width: 2 });
        }
        let [lo, hi] = value.to_le_bytes();
        self.video[start] = lo;
        self.video[start + 1] = hi;
        Ok(())
    }

    pub fn load(&mut self, address: u16, payload: &[u8]) -> Result<(), VmError> {
        let start = address as usize;
        let end = start
            .checked_add(payload.len())
            .filter(|e| *e <= MEMORY_SIZE)
            .ok_or(VmError::InvalidMemoryRange {
                address,
                length: payload.len(),
            })?;
        if start < MMIO_BASE as usize && end > MMIO_BASE as usize {
            return Err(VmError::InvalidMemoryRange {
                address,
                length: payload.len(),
            });
        }
        if start >= MMIO_BASE as usize && !payload.is_empty() {
            return Err(VmError::InvalidMemoryRange {
                address,
                length: payload.len(),
            });
        }
        for (i, &byte) in payload.iter().enumerate() {
            self.write_u8((start + i) as u16, byte);
        }
        Ok(())
    }

    pub fn framebuffer(&self) -> &[u8] {
        &self.video[..FRAMEBUFFER_SIZE]
    }
    pub fn palette(&self) -> [u8; 4] {
        [
            self.bytes[VIDEO_PALETTE0 as usize] & 0x0F,
            self.bytes[VIDEO_PALETTE1 as usize] & 0x0F,
            self.bytes[VIDEO_PALETTE2 as usize] & 0x0F,
            self.bytes[VIDEO_PALETTE3 as usize] & 0x0F,
        ]
    }

    pub fn console_receive(&mut self, byte: u8) {
        if self.console_rx.len() < CONSOLE_RX_FIFO_CAPACITY {
            let was_empty = self.console_rx.is_empty();
            self.console_rx.push_back(byte);
            if was_empty {
                self.raise_irq(IRQ_CONSOLE_RX);
            }
        }
    }

    pub fn console_take_tx(&mut self) -> Option<u8> {
        self.console_tx.pop_front()
    }

    pub fn set_key(&mut self, key: Option<u8>) {
        let was_down = self.bytes[KEY_STATUS as usize] != 0;
        match key {
            Some(c) => {
                self.bytes[KEY_STATUS as usize] = 1;
                self.bytes[KEY_CODE as usize] = c;
                if !was_down {
                    self.raise_irq(IRQ_KEY);
                }
            }
            None => {
                self.bytes[KEY_STATUS as usize] = 0;
                self.bytes[KEY_CODE as usize] = 0;
            }
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: u8) {
        if x >= FRAMEBUFFER_WIDTH || y >= FRAMEBUFFER_HEIGHT {
            return;
        }
        let pixel = y * FRAMEBUFFER_WIDTH + x;
        let byte_offset = pixel / 4;
        let shift = 6 - ((pixel & 3) * 2);
        let mask = !(0x03u8 << shift);
        self.video[byte_offset] = (self.video[byte_offset] & mask) | ((color & 0x03) << shift);
    }

    fn draw_text_char(&mut self, ch: u8) {
        let x = self.bytes[TEXT_X as usize] as usize;
        let y = self.bytes[TEXT_Y as usize] as usize;
        if x >= TEXT_COLUMNS || y >= TEXT_ROWS {
            return;
        }
        let fg = self.bytes[TEXT_FG as usize] & 3;
        let bg = self.bytes[TEXT_BG as usize] & 3;
        let code = if (0x20..=0x7f).contains(&ch) {
            ch
        } else {
            b'?'
        };
        let glyph = (code - 0x20) as usize * 8;
        for row in 0..8 {
            let bits = FONT_8X8[glyph + row];
            for col in 0..8 {
                let c = if bits & (0x80 >> col) != 0 { fg } else { bg };
                self.set_pixel(x * 8 + col, y * 8 + row, c);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn framebuffer_is_16000_bytes() {
        assert_eq!(Memory::default().framebuffer().len(), 16_000);
    }
    #[test]
    fn video_ram_is_exactly_16k() {
        assert_eq!(VIDEO_MEMORY_SIZE, 16_384);
        assert_eq!(VIDEO_RESERVED_BYTES, 384);
    }
    #[test]
    fn system_space_no_longer_maps_framebuffer() {
        let mut m = Memory::default();
        m.write_u8(0x8000, 0x5A);
        assert_eq!(m.read_u8(0x8000), 0x5A);
        assert_eq!(m.video_read_u8(0), 0);
    }
    #[test]
    fn video_out_of_range_is_safe() {
        let mut m = Memory::default();
        m.video_write_u8(0x4000, 0xAA);
        assert_eq!(m.video_read_u8(0x4000), 0);
    }
    #[test]
    fn upper_memory_is_normal_ram() {
        let mut m = Memory::default();
        m.write_u8(0xE000, 0x5A);
        m.write_u8(0xF800, 0xA5);
        assert_eq!(m.read_u8(0xE000), 0x5A);
        assert_eq!(m.read_u8(0xF800), 0xA5);
    }

    #[test]
    fn top_page_is_mmio_and_ram_ends_at_feff() {
        let mut m = Memory::default();
        m.write_u8(0xFEFF, 0x5A);
        assert_eq!(m.read_u8(0xFEFF), 0x5A);
        assert_eq!(MMIO_BASE, 0xFF00);
    }

    #[test]
    fn loader_rejects_program_crossing_into_mmio() {
        let mut m = Memory::default();
        assert!(m.load(0xFEFE, &[1, 2]).is_ok());
        assert!(m.load(0xFEFF, &[1, 2]).is_err());
        assert!(m.load(0xFF00, &[1]).is_err());
    }

    #[test]
    fn text_char_uses_internal_font_rom() {
        let mut m = Memory::default();
        m.write_u8(TEXT_X, 0);
        m.write_u8(TEXT_Y, 0);
        m.write_u8(TEXT_FG, 3);
        m.write_u8(TEXT_BG, 0);
        m.write_u8(TEXT_CHAR, b'A');
        assert!(m.framebuffer()[..16].iter().any(|&b| b != 0));
    }

    #[test]
    fn palette_selectors_are_four_bit() {
        let mut m = Memory::default();
        m.write_u8(VIDEO_PALETTE0, 0xFF);
        assert_eq!(m.palette()[0], 0x0F);
    }
    #[test]
    fn cycle_and_instruction_counters_are_separate() {
        let mut m = Memory::default();
        m.begin_instruction();
        let _ = m.read_u8(0x0100);
        let _ = m.read_u16(0x0200).unwrap();
        m.charge_internal_cycles(16);
        m.retire_instruction();
        assert_eq!(m.cycle_count, 19);
        assert_eq!(m.instruction_count, 1);
    }

    #[test]
    fn rng_mmio_is_repeatable_after_seed() {
        let mut a = Memory::default();
        let mut b = Memory::default();
        a.write_u16(RNG_SEED_LO, 0x1234).unwrap();
        b.write_u16(RNG_SEED_LO, 0x1234).unwrap();
        assert_eq!(
            a.read_u16(RNG_DATA_LO).unwrap(),
            b.read_u16(RNG_DATA_LO).unwrap()
        );
        assert_eq!(
            a.read_u16(RNG_DATA_LO).unwrap(),
            b.read_u16(RNG_DATA_LO).unwrap()
        );
        assert_eq!(a.read_u8(RNG_STATUS) & RNG_READY, RNG_READY);
    }

    #[test]
    fn rng_word_read_uses_one_latched_sample() {
        let mut m = Memory::default();
        m.write_u16(RNG_SEED_LO, 1).unwrap();
        let lo = m.read_u8(RNG_DATA_LO);
        let hi = m.read_u8(RNG_DATA_HI);
        let first = u16::from_le_bytes([lo, hi]);
        m.write_u16(RNG_SEED_LO, 1).unwrap();
        assert_eq!(m.read_u16(RNG_DATA_LO).unwrap(), first);
    }

    #[test]
    fn console_mmio_round_trip() {
        let mut m = Memory::default();
        assert_eq!(
            m.read_u8(CONSOLE_STATUS) & CONSOLE_TX_READY,
            CONSOLE_TX_READY
        );
        m.console_receive(b'A');
        assert_ne!(m.read_u8(CONSOLE_STATUS) & CONSOLE_RX_READY, 0);
        assert_eq!(m.read_u8(CONSOLE_DATA), b'A');
        m.write_u8(CONSOLE_STATUS, CONSOLE_RX_READY);
        assert_eq!(m.read_u8(CONSOLE_STATUS) & CONSOLE_RX_READY, 0);
        m.write_u8(CONSOLE_DATA, b'B');
        assert_eq!(m.console_take_tx(), Some(b'B'));
    }
}
