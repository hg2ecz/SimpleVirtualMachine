# Changelog

## 1.0.0-rc30 - release audit cleanup

- refreshed programmer-manual version labels to the stable 1.0 documentation line;
- removed historical rc wording from the active code-generation guide;
- added a CLI help regression test covering all nine target names and the default target;
- consolidated the project to one canonical changelog;
- repaired stale cross-component documentation links and replaced obsolete SVM-C current-state references with the actual C-Lite toolchain where appropriate;
- aligned assembler/runtime compiler-layout notes with the current C-Lite code base (`0x0100`) and static data base (`0x8000`);
- no language, CLIR, ABI, or code-generation behavior changed.

## 1.0.0-rc29 - cleanup compile fix

- imported the shared `layout::parse_fn_header` helper in Register, LoadStore, and RegMem after the rc28 deduplication;
- no language, CLIR, ABI, or code-generation behavior changed.

## 1.0.0-rc28 - cleanup and pre-1.0 consistency

- removed duplicated CLIR storage parsing from Register, LoadStore, and RegMem; all three now use the shared target-neutral `layout.rs`;
- removed duplicated call parsing from the same three backends;
- removed obsolete canonical-migration assertions/test names;
- refreshed backend audit documentation to the current 9/9 direct-lowering state;
- removed stale SVM-C/optimizer material from the repository README;
- no language, CLIR, ABI, or intended generated-code behavior changed.

## 1.0.0-rc27 - targeted Memory2Memory/TTA audit

- kept Memory2Memory deliberately simple after finding no clear non-optimizer simplification;
- removed redundant TTA compare/result relays and unnecessary operand transports;
- retained direct transport-level lowering with no scheduler.

## 1.0.0-rc26 - RegMem native static addressing

- RegMem static loads/stores use native absolute `[0xADDR]` descriptors directly;
- removed needless R7 address relays for static variables, temporaries, globals, and parameter slots.

## 1.0.0-rc25 - physical Belt slot tracking

- Belt backend tracks the eight physical `b0..b7` slots directly;
- spills occur only before a live value would fall off the belt or at control/call boundaries;
- static memory accesses use native absolute Belt forms.

## 1.0.0-rc24 - RegMem regression test correction

- corrected the memory-source unit-test fixture so residency cannot satisfy the tested operand from R0.

## 1.0.0-rc23 - TTA residency

- one fresh CLIR temp may remain in R0;
- static memory addresses move directly to MEM/VMEM address ports;
- removed needless R7 address relays.

## 1.0.0-rc22 - Register-family residency

- LoadStore and RegMem gained one-fresh-temp residency;
- LoadStore retained native three-operand ALU and logical-immediate forms;
- RegMem retained native memory-source operands.

## 1.0.0-rc21 - Register residency

- Register keeps one fresh CLIR temporary in R0 until a spill is actually required.

## 1.0.0-rc20 - native one-temp residency

- Accumulator keeps one fresh temp in A;
- MemReg keeps one fresh temp in W;
- Belt initially gained local fresh-result residency before the later full physical-belt model.

## 1.0.0-rc19 - backend quality audit

- audited all nine direct backends for natural ISA use;
- LoadStore gained native three-operand ALU selection;
- RegMem gained native memory-source instruction selection;
- added the code-size reporting script.

## 1.0.0-rc18 - nine direct CLIR backends

- all nine targets own their CLIR code generator;
- removed the transitional shared CPU-shaped backend completely;
- shared backend code is limited to target-neutral data layout and parsing helpers.

## 1.0.0-rc17 - direct Register, LoadStore, and RegMem lowering

- Register now owns a direct CLIR-to-Register code generator;
- LoadStore now owns a direct CLIR-to-LoadStore code generator;
- RegMem now owns a direct CLIR-to-RegMem code generator;
- Register and LoadStore perform their required logical-immediate legalization locally, without a shared canonical adapter;
- six of nine backends now lower CLIR directly; only Memory2Memory, Belt, and TTA still use the transitional canonical path;
- no optimizer or language feature was added.

## 1.0.0-rc16 - direct Accumulator and MemReg backends

- Accumulator now lowers CLIR directly to its native A/X machine model;
- MemReg now lowers CLIR directly to its native W/F0 machine model;
- added a target-neutral storage-layout helper that emits no instructions and models no CPU;
- Stack, Accumulator, and MemReg no longer pass through the register-shaped canonical machine;
- the nine-target integration harness now rejects canonical R0..R7 leakage on Accumulator and MemReg;
- no language feature or optimizer was added.

## 1.0.0-rc15 - Stack store-order fix

- fixed direct Stack lowering for indexed assignments such as `data[0] = 10`;
- Stack `storemem` now accepts both adjacent CLIR operand orders and emits at most one native `SWAP`;
- added regression coverage for indexed stores, raw store builtins, and the array-pointer example;
- no temp RAM, register emulation, optimizer, or language feature was added.

## 1.0.0-rc14 - direct native Stack backend

- Stack now lowers CLIR directly instead of receiving register-shaped canonical assembly;
- CLIR temporaries live on the Stack VM data stack and no longer get fake R0..R7/static-temp slots on this target;
- added explicit CLIR `drop` for unused expression results; Stack maps it to native `DROP`, register-style legacy lowering treats it as no-op;
- signed Stack division/modulo uses three simple compiler-private static scratch words rather than register emulation;
- no language feature or optimizer was added;
- this is the first staged step toward direct CLIR lowering for all nine ISAs before 1.0.

## 1.0.0-rc13 - CLI help

- added `-h` and `--help`;
- help output lists all options, all nine targets, the default target, and short examples;
- no language, CLIR, ABI, backend, or assembler behavior changed.

## 1.0.0-rc12 - nine-target external integration harness

- added file-based C-Lite smoke programs for arithmetic, loops, arrays/pointers, calls, control flow, memory/MMIO, bool, globals, and textual include;
- added `scripts/test_9_targets.sh`, which drives the standalone `svm-clite` and standalone `svm-asm` executables across all nine targets;
- the harness checks 81 `.cl -> target ASM -> binary` cases and keeps the compiler/assembler boundary explicit;
- removed the stray `canonical.old.rs` development artifact;
- no language, CLIR, ABI, optimizer, or backend semantics changed.

## 1.0.0-rc11 - assembler test fixture correction

- corrected the Belt MMIO-overlap test address so the test actually crosses the `0xFF00` MMIO boundary;
- no assembler behavior or C-Lite behavior changed.

## 1.0.0-rc10 - test correctness cleanup

- fixed four test fixtures that violated the intentionally strict C-Lite type/main rules;
- kept implicit `u8` to `u16` return conversion forbidden;
- byte-parameter and byte-MMIO tests now exercise lowering without requiring widening;
- signed-comparison IR test now uses a `bool` return and includes the required `main()`.

## 1.0.0-rc9 - readability cleanup

- reformatted and split the canonical CLIR-to-ASM lowering into small named helpers;
- reformatted the Accumulator and MemReg adapters without changing their translation rules;
- reformatted the parser and CLIR emitter for readability;
- removed the obsolete `allow_return` parser parameter left from the old `for` implementation;
- corrected an outdated CLIR comment: comparisons produce `bool`;
- no language, CLIR, ABI, optimizer, or generated-code semantics intentionally changed.

## 1.0.0-rc8 - code cleanup

- deduplicated the identical Register/LoadStore logical-immediate expansion;
- reduced the public Rust API surface by keeping the target module private while re-exporting `Target`;
- no language, CLIR, ABI, or generated-code semantics changed.

## 1.0.0-rc7 - explicit nine-target adapters

- added explicit `register`, `loadstore`, and `regmem` backend adapters;
- fixed Register output that previously emitted unsupported `ANDI`/`ORI`/`XORI`;
- fixed LoadStore output for logical immediates larger than its native immediate field;
- reserved R5 as a tiny adapter scratch register for Register and LoadStore;
- no language feature, optimizer, or compiler pass was added.

## 1.0.0-rc6

- no new language feature; release-stabilization cleanup only;
- centralized type names and CLIR suffixes in `Ty`;
- removed the unused string token/string-literal lexer path because `include` is expanded before lexing;
- removed a trivial CLI target-parser wrapper;
- corrected the array-element diagnostic to include `bool`;
- added Hungarian and English 1.0 release checklists.

## 1.0.0-rc5

- Added complete Hungarian and English programmer manuals.
- Documented the actual rc4/rc5 syntax, type rules, CLI, arrays, pointers, functions, control flow, MMIO, includes, standard library, CLIR relationship, assembler boundary, diagnostics, and deliberate language omissions.
- No language or code-generation feature was added.

## 1.0.0-rc4

- documentation cleanup only; no language or code-generation feature added;
- removed duplicate/outdated IR and backend documents;
- one CLIR 0.1 specification, one code-generation document and one learning path remain;
- added learning examples for function calls, MMIO and `bool`;
- kept the compiler model unchanged: no optimizer, no SVM-C, no embedded assembler.

## 1.0.0-rc3

- removed unused `Ty::scalar_c`;
- removed unused canonical backend return metadata;
- removed unused canonical backend `reg_slot` helper.

## 1.0.0-rc2

- added minimal `bool`, `true` and `false`;
- comparisons return `bool` in CLIR;
- stored `bool` is one byte, never bit-packed;
- fixed 8-bit function parameter stores;
- added explicit simplicity design rules.

## 1.0.0-rc1

- include-once with cycle detection;
- clearer include and lexer errors;
- 9-target smoke coverage for arithmetic, control flow, arrays/pointers, calls and MMIO;
- small C-Lite source libraries;
- CLIR 0.1 documented and frozen for the 1.0 line.

## Direct compiler baseline

The current 1.0 line intentionally has:

- no optimizer or constant folding;
- no SVM-C dependency;
- no embedded assembler;
- direct `C-Lite -> CLIR -> target ASM` translation;
- external `svm-asm` ownership of assembly includes and `.proc/.endproc` reachability filtering.

