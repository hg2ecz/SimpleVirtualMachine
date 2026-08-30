# Belt16 assembly programming manual

Belt16 names the eight most recent 16-bit results `b0..b7`; `b0` is always the newest. Every result-producing instruction ages older values by one position. There is no general-purpose register file.

```asm
.load 0x0100
.entry start
start:
    LDI 10
    LDI 20
    ADD b1,b0
    ST16A 0x6000,b0
    HALT
```

Absolute memory uses `LD8A/LD16A` and `ST8A/ST16A`. Pointer memory uses `LD8/LD16 [bN]` and `ST8/ST16 [bA],bV`. Video memory uses `VLD8/VLD16` and `VST8/VST16`.

`PUSH bN` and `POP` are primarily compiler/assembly convenience primitives. `POP` produces a result and therefore places it on the belt.

`CMP bA,bB` also produces a result (`a-b`) and updates `Z/N/C`; the result can be followed by `JZ/JNZ/JC/JNC/JN/JNN`.
