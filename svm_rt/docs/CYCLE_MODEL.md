# SVM cycle model

The runtime uses one shared deterministic cycle-accounting model for all nine CPU cores. The model is intentionally simple: it is a VM cost model, not a transistor-accurate timing model.

## Instruction accounting

Before each guest instruction the runtime clears the pending-cycle accumulator. CPU-visible memory and VRAM accesses charge cycles while the instruction executes. Multi-cycle internal operations may add explicit internal cost. At retirement, the instruction consumes the accumulated cost, with a minimum of one cycle, and the retired-instruction counter advances by one.

Consequently, instruction encoding length and data-memory traffic both matter: instruction fetches use the ordinary CPU memory read path, so fetched bytes contribute to the cost.

## Memory costs

- CPU-address-space byte read/write: 1 cycle per byte access.
- CPU-address-space 16-bit read/write: two byte accesses.
- VRAM byte read/write: 1 cycle.
- VRAM 16-bit read/write: 2 cycles.
- MMIO uses the same CPU-address-space access mechanism; device side effects do not create a separate hidden timing model unless the runtime explicitly charges internal cycles.

## Internal multi-cycle operations

Expensive integer operations add explicit internal cost in the CPU cores. The current implementations use additional deterministic charges for operations such as multiply, divide/modulo, `MULHU`, and `MULQ15`. The exact instruction-level behavior is defined by the runtime source for each CPU core; all architectures use the same accounting API.

The model deliberately does not emulate general-purpose instruction/data caches, speculative execution, or variable external-memory latency. It **does** model architecturally relevant internal operand storage when it removes a real memory access. In particular, the Stack CPU uses a two-cell `TOS`/`NOS` stack cache with lazy NOS refill. TOS/NOS register operations add no memory cycle; RAM is charged only when a cached cell spills, NOS is demanded from RAM, or a pop must expose a RAM-backed new TOS. This is a microarchitectural cost model choice, not a new ISA-visible feature.

## Counters, timer and interrupts

`cycle_count` and `instruction_count` are separate. The timer advances according to retired VM cycles, not merely retired instructions. Thus an instruction with larger memory/internal cost advances the timer by more than a minimal instruction.

This document is the shared reference linked from the individual assembler instruction references.
