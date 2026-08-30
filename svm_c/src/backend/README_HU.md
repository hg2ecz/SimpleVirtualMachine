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
