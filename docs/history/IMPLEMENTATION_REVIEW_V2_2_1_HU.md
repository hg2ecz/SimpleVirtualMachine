# Implementációs felülvizsgálat – v2.2.1

A v2.2 teljes statikus felülvizsgálata a runtime, assembler és SVM-C komponensekre terjedt ki.

## Javított hibák

- SVM-C tesztkódban elavult `crate::lang::compile_source` import maradt; az aktuális `optimized::pipeline::compile_source` útvonalra javítva.
- MemReg assembler első passza a HOT logikai revízió után még `XOR`-t számolt egybájtosnak `AND` helyett; labelcímeket ronthatott.
- Register-Memory és Memory-to-Memory assemblerben egy forward label operandus az első passzban hosszú, a másodikban rövid descriptorra válthatott. Labelekhez most mindig stabil hosszú descriptor készül; rövid forma csak explicit numerikus literálhoz használható.
- Load/Store, Register-Memory, Memory-to-Memory, Belt16 és TTA16 esetén `.entry` hiányában az entry most helyesen az aktuális `.load` cím, nem fixen `0x0100`.
- A runtime CLI usage felsorolja mind a kilenc executable kiterjesztést.
- A magyar C referencia régi, a `long` támogatásával ellentmondó pontja javítva.

## Ellenőrzött integráció

Mind a kilenc target szerepel az assembler CLI-ben, a runtime executable felismerésben és CPU-dispatchban, valamint az optimalizált és `unopt-only` C fordító target-listájában. A Belt16 (`SVB`) és TTA16 (`SVT`) saját executable magicet és runtime magot használ.

## Korlát

A környezetben nincs `cargo`/`rustc`, ezért tényleges Rust fordítás és tesztfuttatás itt nem történt. A v2.2.1 ezért statikusan felülvizsgált javító kiadás; Rust toolchaines környezetben `cargo test --workspace` futtatása továbbra is szükséges.
