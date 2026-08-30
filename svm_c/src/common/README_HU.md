# `common` – architektúrafüggetlen C frontend

Ide tartozik minden, ami a C-szerű nyelv jelentését írja le, nem valamely CPU-t:

- `model.rs` – AST, típusok, közös modellek;
- `frontend.rs` – lexer és parser;
- `semantic.rs` – szemantikai ellenőrzések;
- `layout.rs` – közös memória-layout és kötelező nyelvi lowering.

ISA-specifikus utasításkódolás vagy assembly-emisszió nem kerülhet ebbe a
könyvtárba.
