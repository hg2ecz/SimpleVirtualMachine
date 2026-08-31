# `backend` – architektúrafüggő kódgenerálás

Minden támogatott ISA külön fájlban található:

- `accumulator.rs`
- `stack.rs`
- `memreg.rs`
- `register.rs`
- `loadstore.rs`
- `regmem.rs`
- `memory2memory.rs`

A `common/` rész nem tartalmaz ISA-utasításválasztást. A targetfüggő assembly
előállítás ebben a könyvtárban történik.

Megjegyzés: a Load/Store és Register-Memory backend jelenlegi implementációja
szándékosan kis adapterréteg a Register közös expression-loweringja fölött. Ez
forráskód-megosztás, nem az ISA-k összemosása: a célassembly formátumot a saját
backend modul állítja elő. Ha később target-specifikus optimalizáció szükséges,
az kizárólag a megfelelő fájlban kerülhet be.

## Kapcsolat az assembler eljárásszintű kódelhagyásával

A backendek a generált C függvényeket `.proc NAME` / `.endproc` blokkokban adják ki.
A `__start` belépési rutin szintén eljárás, és a `.entry __start` a gyökere. Bináris
kimenet készítésekor a C driver meghívja az `svm-asm` közös eljárás-elérhetőségi
passzát, ezért a C forrásban meglévő, de a `main`-ből nem elérhető függvények nem
kerülnek a végső gépi kódba.

A `--emit asm` a választott C-optimalizálási szint utáni, de az assembler procedure-GC
előtti `.proc`-os assemblyt írja ki. `-O0` esetén ezért a C-szintű dead-function
elimination nem fut; `-O1/-O2/-Os` esetén már a C optimizer által szűrt függvénykészlet
látszik. A kimenet újra betáplálható az `svm-asm` programba, ahol a végső
eljárás-elérhetőségi passz fut le.

## Többparaméteres hívási ABI

A korábbi négyelemű skalárparaméter-korlát megszűnt. A lokálisokhoz hasonlóan a
callee paraméterei statikusan kiosztott címeket kapnak. A Register-alapú backendek,
az Accumulator és a MemReg előbb a runtime stackre stagingelik a már kiértékelt
argumentumokat, majd fordított sorrendben a callee paraméterhelyeire írják őket.
Ez megakadályozza, hogy egy későbbi, függvényhívást tartalmazó argumentum
felülírja egy korábbi argumentum értékét. A Stack target természetes adatstackes
átadást használ, és a callee belépésekor másolja a paramétereket statikus helyükre.

Ez az ABI tetszőleges ésszerű számú natív skalár paramétert enged; a gyakorlati
korlátot a statikus RAM és a hívás közbeni stackkapacitás adja. Rekurzió és
reentrancia továbbra sincs, mert a callee paraméterhelyei statikusak.
