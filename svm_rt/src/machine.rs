use crate::{
    cpu,
    error::VmError,
    memory::Memory,
    program::{CpuKind, Program},
    video::Video,
};

enum Cpu {
    Register(cpu::register::Cpu),
    Stack(cpu::stack::Cpu),
    Accumulator(cpu::accumulator::Cpu),
    Memreg(cpu::memreg::Cpu),
    LoadStore(cpu::loadstore::Cpu),
    RegMem(cpu::regmem::Cpu),
    Memory2Memory(cpu::memory2memory::Cpu),
    Belt(cpu::belt::Cpu),
    Tta(cpu::tta::Cpu),
}

pub struct Machine {
    cpu: Cpu,
    memory: Memory,
    video: Video,
}

impl Machine {
    pub fn new(kind: CpuKind) -> Self {
        let cpu = match kind {
            CpuKind::Register => Cpu::Register(Default::default()),
            CpuKind::Stack => Cpu::Stack(Default::default()),
            CpuKind::Accumulator => Cpu::Accumulator(Default::default()),
            CpuKind::Memreg => Cpu::Memreg(Default::default()),
            CpuKind::LoadStore => Cpu::LoadStore(Default::default()),
            CpuKind::RegMem => Cpu::RegMem(Default::default()),
            CpuKind::Memory2Memory => Cpu::Memory2Memory(Default::default()),
            CpuKind::Belt => Cpu::Belt(Default::default()),
            CpuKind::Tta => Cpu::Tta(Default::default()),
        };
        Self {
            cpu,
            memory: Memory::default(),
            video: Video,
        }
    }

    pub fn kind(&self) -> CpuKind {
        match &self.cpu {
            Cpu::Register(_) => CpuKind::Register,
            Cpu::Stack(_) => CpuKind::Stack,
            Cpu::Accumulator(_) => CpuKind::Accumulator,
            Cpu::Memreg(_) => CpuKind::Memreg,
            Cpu::LoadStore(_) => CpuKind::LoadStore,
            Cpu::RegMem(_) => CpuKind::RegMem,
            Cpu::Memory2Memory(_) => CpuKind::Memory2Memory,
            Cpu::Belt(_) => CpuKind::Belt,
            Cpu::Tta(_) => CpuKind::Tta,
        }
    }

    pub fn load_program(&mut self, program: &Program) -> Result<(), VmError> {
        if self.kind() != program.cpu {
            *self = Self::new(program.cpu);
        }
        let mut memory = Memory::default();
        program.load_into(&mut memory)?;
        self.memory = memory;
        match &mut self.cpu {
            Cpu::Register(cpu) => cpu.reset(program.entry_address),
            Cpu::Stack(cpu) => cpu.reset(program.entry_address),
            Cpu::Accumulator(cpu) => cpu.reset(program.entry_address),
            Cpu::Memreg(cpu) => cpu.reset(program.entry_address),
            Cpu::LoadStore(cpu) => cpu.reset(program.entry_address),
            Cpu::RegMem(cpu) => cpu.reset(program.entry_address),
            Cpu::Memory2Memory(cpu) => cpu.reset(program.entry_address),
            Cpu::Belt(cpu) => cpu.reset(program.entry_address),
            Cpu::Tta(cpu) => cpu.reset(program.entry_address),
        }
        Ok(())
    }

    pub fn step(&mut self) -> Result<(), VmError> {
        match &mut self.cpu {
            Cpu::Register(cpu) => {
                cpu.step(&mut self.memory)?;
            }
            Cpu::Stack(cpu) => cpu.step(&mut self.memory)?,
            Cpu::Accumulator(cpu) => {
                cpu.step(&mut self.memory)?;
            }
            Cpu::Memreg(cpu) => {
                cpu.step(&mut self.memory)?;
            }
            Cpu::LoadStore(cpu) => cpu.step(&mut self.memory)?,
            Cpu::RegMem(cpu) => cpu.step(&mut self.memory)?,
            Cpu::Memory2Memory(cpu) => cpu.step(&mut self.memory)?,
            Cpu::Belt(cpu) => cpu.step(&mut self.memory)?,
            Cpu::Tta(cpu) => cpu.step(&mut self.memory)?,
        }
        Ok(())
    }

    pub fn halted(&self) -> bool {
        match &self.cpu {
            Cpu::Register(cpu) => cpu.halted(),
            Cpu::Stack(cpu) => cpu.halted(),
            Cpu::Accumulator(cpu) => cpu.halted(),
            Cpu::Memreg(cpu) => cpu.halted(),
            Cpu::LoadStore(cpu) => cpu.halted(),
            Cpu::RegMem(cpu) => cpu.halted(),
            Cpu::Memory2Memory(cpu) => cpu.halted(),
            Cpu::Belt(cpu) => cpu.halted(),
            Cpu::Tta(cpu) => cpu.halted(),
        }
    }

    pub fn render_argb(&self, output: &mut [u32]) -> Result<(), VmError> {
        self.video.to_argb8888(&self.memory, output)
    }

    pub fn set_key(&mut self, key: Option<u8>) {
        self.memory.set_key(key);
    }

    pub fn console_receive(&mut self, byte: u8) {
        self.memory.console_receive(byte);
    }

    pub fn take_console_tx(&mut self) -> Option<u8> {
        self.memory.console_take_tx()
    }

    pub fn video_vsync(&mut self) {
        self.memory.video_vsync();
    }
}
