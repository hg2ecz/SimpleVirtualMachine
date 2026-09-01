# C-Lite backend audit – pre-1.0 state

## Principle

All nine backends lower CLIR directly to their own ISA assembly. There is no shared virtual CPU model.

Good code generation here means natural representation and instruction selection, not optimizer passes:

1. use the target's own machine model;
2. do not materialize a short-lived CLIR temporary in RAM when the target can naturally keep it alive;
3. allow only small target-local state;
4. no SSA, global liveness, general register allocation, scheduler, or compiler peephole pass.

## Backends

- **Register** – one fresh temp may remain in R0 and spills only when required.
- **Stack** – CLIR temporaries live directly on the data stack; no temp RAM.
- **Accumulator** – one fresh temp may remain in A.
- **MemReg** – one fresh temp may remain in W; native W/file-register model.
- **LoadStore** – one fresh temp may remain in R0; native three-operand ALU and small logical-immediate forms.
- **RegMem** – one fresh temp may remain in R0; native memory-source operands and direct `[0xADDR]` static addressing.
- **Memory2Memory** – memory temporaries are natural to the ISA; the deliberately simple representation is retained.
- **Belt** – the backend tracks the eight physical `b0..b7` slots; spills occur only before a value falls off or at control-flow/call boundaries.
- **TTA** – one fresh temp may remain in R0; direct ALU/MEM/VMEM/CTRL transports without needless address or compare relays.

## Current code-size checkpoint

Latest externally measured binary sizes for the `p2.cl` array/pointer example:

```text
target            bin_bytes
register                210
stack                    67
accumulator             155
memreg                  255
loadstore               268
regmem                  270
memory2memory           326
belt                    192
tta                     320
```

These are regression indicators, not optimization targets.

## Out of scope

Do not add only for code size:

- SSA;
- global liveness analysis;
- graph-coloring register allocation;
- common subexpression elimination;
- constant folding;
- instruction scheduling;
- general optimizer passes;
- a shared generic target machine.

## 1.0 backend release criteria

1. All nine targets own a direct CLIR lowerer.
2. `cargo test` is green.
3. All 81 `.cl -> ASM -> binary` integration cases are green.
4. Each target uses its natural operand model.
5. The code-size report shows no obvious generic-emulation outlier.
6. Further size work is accepted only when the backend remains equally simple or becomes simpler.

## Checks

```sh
cargo test
svm_clite/scripts/test_9_targets.sh
svm_clite/scripts/report_codegen.sh program.cl
```
