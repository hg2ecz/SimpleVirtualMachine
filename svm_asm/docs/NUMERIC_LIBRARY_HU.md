# Assembly numerikus segédkönyvtárak

A kézzel írt assemblyhez a könyvtár két külön réteget használ:

- `platform.asm`: architektúrafüggetlen MMIO-konstansnevek `.equ` formában;
- `math.asm` / `format.asm`: az adott ISA természetes programozási modelljére írt rutinok.

## Jelenlegi implementáció

A Register és Stack target már tartalmaz használható referencia-implementációt.

### Register ABI

- `abs16`: `R0 -> R0`;
- `gcd_u16`: `R0=a`, `R1=b` -> `R0=gcd`;
- `putu16`: `R0` előjel nélküli 16 bites szám decimális kiírása;
- `puti16`: `R0` előjeles 16 bites szám decimális kiírása.

A `format.asm` automatikusan behúzza a `console.asm` fájlt.

### Stack ABI

- `abs16 ( n -- |n| )`;
- `min_u16 ( a b -- min )`;
- `max_u16 ( a b -- max )`;
- `gcd_u16 ( a b -- gcd )`;
- `putu16 ( u -- )`;
- `puti16 ( n -- )`.

## További ISA-k

A többi target már rendelkezik azonos nevű standard könyvtárterülettel és `platform.asm` fájllal, de a numerikus rutinokat nem célszerű puszta szintaktikai átiratként lemásolni. Az accumulator, memory-register, memory-to-memory, Belt és TTA gépeknél a természetes ABI és a kódsűrűség eltérő; ezért ezeket célarchitektúránként érdemes optimalizálni.

A C backend később ugyanilyen funkcionális nevekre építhet cél-specifikus intrinsic/library loweringot, de a mostani változtatás ezt még nem kényszeríti rá a C fordítóra.

## Procedure-GC

A ténylegesen implementált assembly rutinok `.proc/.endproc` blokkokban vannak. Egy teljes standard könyvtár include-olása nem kényszeríti a nem használt rutinok gépi kódba építését: az assembler csak az `.entry`, `.keep` vagy élő szimbolikus hivatkozás alapján elérhető eljárásokat tartja meg.
