# Memory-to-Memory assembly programming manual

The ISA is designed for direct memory operands:

```asm
MOV16 [0x1200], [0x1202]
ADD16 [0x1200], 7
AND16 [0x1200], 0x7FFF
```

`A0..A3` are pointer/address registers only, deliberately not general data registers, so the architecture remains a true memory-to-memory endpoint. Unary operations are direct read-modify-write operations. VRAM is separate and uses `VLD*`/`VST*`.
