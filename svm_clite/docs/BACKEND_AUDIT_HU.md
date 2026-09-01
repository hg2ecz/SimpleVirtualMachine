# C-Lite backend audit – 1.0 előtti állapot

## Alapelv

Mind a kilenc backend közvetlenül CLIR-ből generálja a saját ISA assemblyjét. Nincs közös virtuális CPU-modell.

A jó kódgenerálás itt nem optimizer-passokat jelent, hanem természetes reprezentációt és instruction selectiont:

1. a target saját gépmodelljét használjuk;
2. rövid életű CLIR tempet nem írunk RAM-ba, ha a target természetesen életben tudja tartani;
3. csak egyszerű, target-local állapot engedett;
4. nincs SSA, globális liveness, általános regiszterallokátor, scheduler vagy compiler peephole pass.

## Backendek

- **Register** – egy friss temp R0-ban maradhat; szükség esetén spill.
- **Stack** – a CLIR temporaries közvetlenül az adatvermen élnek; nincs temp-RAM.
- **Accumulator** – egy friss temp A-ban maradhat.
- **MemReg** – egy friss temp W-ben maradhat; a W/file-register modell közvetlen.
- **LoadStore** – egy friss temp R0-ban maradhat; natív háromoperandusos ALU és kis logical-immediate formák használata.
- **RegMem** – egy friss temp R0-ban maradhat; natív memória-source operandusok és közvetlen `[0xADDR]` statikus címzés.
- **Memory2Memory** – a temp memóriaoperandus természetes a gépmodellhez; a jelenlegi egyszerű megoldás szándékosan marad.
- **Belt** – a backend a hardver nyolc fizikai `b0..b7` slotját követi; spill csak kiesés vagy vezérlési/hívási határ miatt.
- **TTA** – egy friss temp R0-ban maradhat; közvetlen ALU/MEM/VMEM/CTRL transportok, fölösleges cím- és compare-relék nélkül.

## Jelenlegi kódméret-kontrollpont

A `p2.cl` array/pointer példán a legutóbb külsőleg mért binárisméretek:

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

Ezek nem optimalizációs célértékek. Arra szolgálnak, hogy észrevegyük a gépmodellhez képest nyilvánvalóan pazarló kódgenerálást vagy regressziót.

## Nem cél

Nem vezetünk be csak kódméret kedvéért:

- SSA-t;
- globális liveness analízist;
- graph-coloring regiszterallokátort;
- common subexpression eliminationt;
- constant foldingot;
- instruction schedulert;
- általános optimizer-passokat;
- közös generic target machine-t.

## 1.0 backend release-kritérium

1. Mind a 9 target saját közvetlen CLIR lowerert használ.
2. `cargo test` zöld.
3. A 81 `.cl -> ASM -> binary` integrációs eset zöld.
4. A targetek saját természetes operandusmodelljüket használják.
5. A code-size riportban nincs nyilvánvaló generic-emulációból származó kilógás.
6. További kódméret-javítás csak akkor kerül be, ha a backend egyszerűbb vagy ugyanolyan egyszerű marad.

## Ellenőrzés

```sh
cargo test
svm_clite/scripts/test_9_targets.sh
svm_clite/scripts/report_codegen.sh program.cl
```
