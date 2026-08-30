# svm-rt használat

## Indítás

```sh
svm-rt program.svm
svm-rt program.svs
svm-rt program.sva
svm-rt program.svf
svm-rt program.svl
svm-rt program.svr
svm-rt program.svc
svm-rt program.svb
svm-rt program.svt
```

A CPU-t a 4 byte-os executable magic választja ki. A runtime betölti a payloadot a fejlécben megadott címre, majd az entry címen reseteli a megfelelő CPU-magot.

## Host ablak és videó

A referencia futtató 320x200 guest képet 2x host skálázással jelenít meg. A host ablak 60 FPS célfrissítést használ. Egy frame alatt legfeljebb 50 000 guest utasítást hajt végre, majd VSYNC eseményt generál és újrarajzolja a képet. Ez a host scheduler részlete; a guest determinisztikus `cycle_count` modellje ettől különálló.

## Billentyűzet

Az ablakból a runtime alap ASCII billentyűket továbbít (`A..Z`, `0..9`, space, Enter, Escape) a keyboard MMIO felé. A pontos host key-map referencia-runtime részlet, nem ISA-követelmény.

## VT100/RS-232 jellegű konzol

A host stdin bájtjai a guest console RX FIFO-ba kerülnek. A guest `CONSOLE_DATA` írásai a host stdout-ra kerülnek. A terminal raw mode engedélyezése best-effort módon történik.

## Leállás

A futás véget ér, ha:

- a guest CPU HALT állapotba kerül; vagy
- a host ablak bezáródik; vagy
- végrehajtási/programformátum hiba keletkezik.

## Kapcsolódó dokumentumok

- [`EXECUTABLE_FORMAT_HU.md`](EXECUTABLE_FORMAT_HU.md)
- [`CYCLE_MODEL.md`](CYCLE_MODEL.md)
- [`../../docs/MMIO_REFERENCE_HU.md`](../../docs/MMIO_REFERENCE_HU.md)
- [`../../docs/VIDEO_TEXT_REFERENCE_HU.md`](../../docs/VIDEO_TEXT_REFERENCE_HU.md)
