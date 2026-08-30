# SVM MMIO referencia

A CPU-címtér `0xFF00..0xFFFF` tartománya a közös platform MMIO-lapja. A RAM ezért egyetlen összefüggő `0x0000..0xFEFF` tartomány. A VRAM külön 16 KiB-os adatcímtér, nem része ennek a lapnak.

## Regisztertérkép

| Cím | Név | Hozzáférés | Jelentés |
|---:|---|---|---|
| `0xFF00` | `KEY_STATUS` | R | billentyűzet állapot |
| `0xFF01` | `KEY_CODE` | R | aktuális ASCII billentyűkód |
| `0xFF02` | `TEXT_X` | R/W | karakteroszlop, `0..39` |
| `0xFF03` | `TEXT_Y` | R/W | karaktersor, `0..24` |
| `0xFF04` | `TEXT_FG` | R/W | előtér palettaslot, `0..3` |
| `0xFF05` | `TEXT_BG` | R/W | háttér palettaslot, `0..3` |
| `0xFF06` | `TEXT_CHAR` | W | 8x8 glyph kirajzolása az aktuális cellába |
| `0xFF07..0xFF0A` | reserved | - | jövőbeli videovezérlés |
| `0xFF0B` | `VIDEO_VSYNC_COUNTER` | R | 8 bites VSYNC számláló |
| `0xFF0C..0xFF0F` | `VIDEO_PALETTE0..3` | R/W | a négy 2 bites framebuffer-slot 0..15 master-paletta indexe |
| `0xFF10..0xFF11` | reserved | - | korábbi host-output helye; ne használd |
| `0xFF12` | `IRQ_ENABLE` | R/W | engedélyezett IRQ-források bitmaszkja |
| `0xFF13` | `IRQ_PENDING` | R | függő IRQ-források bitmaszkja |
| `0xFF14` | `IRQ_ACK` | W | a beírt 1 bitek törlik a megfelelő pending biteket |
| `0xFF15` | `TIMER_CONTROL` | R/W | bit0 enable, bit1 periodic |
| `0xFF16..0xFF17` | `TIMER_RELOAD` | R/W | 16 bites little-endian újratöltési érték |
| `0xFF18..0xFF19` | `TIMER_COUNT` | R/W | 16 bites aktuális timer érték |
| `0xFF1A..0xFF1D` | `CLOCK_TICK` | R | 32 bites little-endian VM ciklusszámláló |
| `0xFF1E..0xFF1F` | `IRQ_VECTOR` | R/W | 16 bites little-endian megszakítási belépési cím |
| `0xFF20` | `CONSOLE_DATA` | R/W | VT100/RS-232 jellegű konzol adat |
| `0xFF21` | `CONSOLE_STATUS` | R/W | RX-ready/TX-ready állapot, RX fogyasztás |
| `0xFF22..0xFF25` | `INSTRUCTION_COUNT` | R | 32 bites little-endian retired-instruction számláló |
| `0xFF26..0xFF27` | `RNG_DATA` | R | 16 bites PRNG minta; LO olvasás új mintát latch-el |
| `0xFF28` | `RNG_STATUS` | R | bit0 `RNG_READY`, jelenleg mindig 1 |
| `0xFF29..0xFF2A` | `RNG_SEED` | W | 16 bites little-endian seed |
| `0xFF2B..0xFFFF` | reserved | - | jövőbeli perifériák |

## Billentyűzet

`KEY_STATUS` értéke 1, amíg a referencia host-ablak egy támogatott billentyűt lenyomva lát, különben 0. `KEY_CODE` az aktuális ASCII kód. Új lenyomási élkor keyboard IRQ keletkezik. A referencia host key-map `A..Z`, `0..9`, space, Enter és Escape billentyűket támogat; egy fizikai implementáció ugyanazt az MMIO ABI-t más billentyűforrással is megvalósíthatja.

## IRQ bitek

- bit0 (`0x01`): timer
- bit1 (`0x02`): VSYNC
- bit2 (`0x04`): billentyűzet
- bit3 (`0x08`): konzol RX

`IRQ_PENDING & IRQ_ENABLE != 0` esetén a CPU-mag a saját ISA-specifikus interrupt belépési szabályai szerint használja az `IRQ_VECTOR` címet.

## Konzol státusz

`CONSOLE_STATUS`:

- bit0 (`0x01`): RX adat áll rendelkezésre;
- bit1 (`0x02`): TX kész; a referencia-runtime-ban mindig kész.

`CONSOLE_DATA` olvasása az RX FIFO első bájtját adja. A FIFO elem tényleges eltávolításához `CONSOLE_STATUS` bit0-ját 1-gyel kell írni. `CONSOLE_DATA` írása a host stdout felé kerülő TX FIFO-ba teszi a bájtot.

## Timer

A timer a VM ciklusszámlálóval együtt halad, nem pusztán utasításonként. `TIMER_ENABLE` mellett minden nyugdíjazott VM ciklus csökkenti a számlálót. Nullánál timer IRQ keletkezik. Periodikus módban a számláló újratöltődik, one-shot módban az enable bit törlődik.

## RNG

A referencia-VM determinisztikus `xorshift32` állapotgépet használ. Ez reprodukálható, kis hardverköltségű PRNG modell, nem entrópiaforrás és nem kriptográfiai RNG. A `0xFF26` alsó bájt olvasása új 16 bites mintát készít és latch-el; a `0xFF27` ugyanennek a mintának a felső bájtját adja. Egy 16 bites load `0xFF26` címről ezért konzisztens mintát kap.

## Ciklusmodell

Minden MMIO byte read/write ugyanazt az alap CPU-memória-hozzáférési költséget viseli, mint egy RAM byte. A perifériahatás csak akkor ad további belső ciklust, ha a runtime ezt külön implementálja. Lásd: [`../svm_rt/docs/CYCLE_MODEL.md`](../svm_rt/docs/CYCLE_MODEL.md).
