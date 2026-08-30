# Memory-to-Memory assembly programozási kézikönyv

Az ISA célja a közvetlen memóriaoperandusos programozás:

```asm
MOV16 [0x1200], [0x1202]
ADD16 [0x1200], 7
AND16 [0x1200], 0x7FFF
```

Az `A0..A3` címregiszterek pointerekhez használhatók. Nem általános adatregiszterek, így az architektúra nem válik MemReg/Register változattá. A unary műveletek közvetlen read-modify-write műveletek. VRAM külön címtér, `VLD*`/`VST*` utasításokkal.
