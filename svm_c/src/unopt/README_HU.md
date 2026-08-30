# `unopt` – szándékosan optimalizáció nélküli fordítási út

Ez a könyvtár a `svm-c-unopt-only` demonstrációs fordító saját pipeline-ját tartalmazza.

A cél nem egy második teljes C frontend fenntartása. A lexer, parser, AST, szemantikai
ellenőrzés és memória-layout nyelvi infrastruktúra, ezért a `../common/` könyvtárból
közös. A kilenc ISA kódgenerátora szintén ugyanaz a `../backend/` réteg, hogy az
optimalizált és optimalizálatlan fordítás összehasonlításakor ne két eltérő codegen
implementációt mérjünk.

Az `unopt/pipeline.rs` szándékosan:

- nem importál optimizer modult;
- nem fogad `OptLevel` paramétert;
- nem futtat konstanshajtást vagy AST-optimalizálást;
- a kötelező nyelvi lowering után közvetlenül a kiválasztott ISA-backendet hívja.

A `svm-c-unopt-only` parancssori program a `src/bin/svm-c-unopt-only.rs` fájlban van.
