# SVM executable formátum

A kilenc assembler/compiler cél azonos, 12 byte-os konténerfejlécet használ. A CPU-t a magic választja ki, ezért az `svm-rt` futtatáskor nem kér külön target paramétert.

## Fejléc

| Offset | Méret | Jelentés |
|---:|---:|---|
| 0 | 4 | CPU-specifikus magic |
| 4 | 2 | load address, little-endian |
| 6 | 2 | entry address, little-endian |
| 8 | 4 | payload méret byte-ban, little-endian |
| 12 | N | nyers program payload |

A fájlméretnek pontosan `12 + payload_size` értékűnek kell lennie. A load tartománynak bele kell férnie a 64 KiB CPU-címtérbe; a compiler/assembler ezen felül saját platformvédelmet is alkalmazhat az MMIO és ABI-fenntartások miatt.

## Magic és tipikus kiterjesztés

| Target | Magic | Kiterjesztés |
|---|---|---|
| Register | `SVM\x09` | `.svm` |
| Stack | `SVS\x08` | `.svs` |
| Accumulator | `SVA\x06` | `.sva` |
| MemReg | `SVF\x04` | `.svf` |
| Load/Store | `SVL\x01` | `.svl` |
| Register-Memory | `SVR\x01` | `.svr` |
| Memory-to-Memory | `SVC\x01` | `.svc` |
| Belt16 | `SVB\x01` | `.svb` |
| TTA16 | `SVT\x01` | `.svt` |

A kiterjesztés kényelmi konvenció; a runtime a magic alapján választ CPU-magot.
