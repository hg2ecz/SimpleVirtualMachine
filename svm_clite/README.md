# svm-clite

`svm-clite` is an intentionally small, C-like structured language for the nine SimpleVirtualMachine architectures. Its goal is not full C compatibility: it is **architecture-independent assembly with a little structure**, so programs can be written without learning nine target-specific assembly syntaxes.

## Documentation language policy

For SVM C-Lite 1.0, **English is the primary normative documentation language**. The Hungarian documents are maintained as complete companion translations and must describe the same language, CLIR, ABI, backend rules, and limitations.

Start with the [documentation index](docs/README.md) or the primary manual:

- [`docs/PROGRAMMING_MANUAL_EN.md`](docs/PROGRAMMING_MANUAL_EN.md) - primary programmer's manual
- [`docs/PROGRAMMING_MANUAL_HU.md`](docs/PROGRAMMING_MANUAL_HU.md) - teljes magyar programozói kézikönyv

If an English/Hungarian pair ever disagrees, the implementation and tests decide correctness first; the English document is the normative text to repair, and the Hungarian counterpart must then be brought back into parity.

## Model

```text
C-Lite -> CLIR 0.1 -> target ASM -> external svm-asm
```

There is no optimizer, SSA, general register allocator, linker, embedded assembler, SVM-C layer, or shared generic CPU backend. Each of the nine targets lowers CLIR directly to its natural machine model.

## Language core

- `bool`, `i8`, `u8`, `i16`, `u16`, `void`
- scalar variables, fixed arrays, one pointer level
- `fn`, parameters, `return`
- `if / else`
- `while`, `break`, `continue`
- textual `include`
- `//` and `/* ... */` comments
- arithmetic, bitwise and comparison operations
- `load8/load16/store8/store16`
- `vload8/vload16/vstore8/vstore16` for volatile/MMIO access

Recursion is forbidden. A stored `bool` occupies one byte and is not bit-packed.

## Usage

```sh
svm-clite --check program.cl
svm-clite --emit ir program.cl
svm-clite --target register program.cl
svm-asm register program.asm program.svm
```

`--assemble` is only a convenience wrapper around an external `svm-asm` invocation.

Use `svm-clite --help` or `svm-clite -h` for all command-line options and target names.

## Primary documentation

English normative documents:

- [`docs/PROGRAMMING_MANUAL_EN.md`](docs/PROGRAMMING_MANUAL_EN.md) - complete programmer's manual
- [`docs/LANGUAGE_EN.md`](docs/LANGUAGE_EN.md) - compact language reference
- [`docs/CLIR_0_1_EN.md`](docs/CLIR_0_1_EN.md) - architecture-independent CLIR 0.1 reference
- [`docs/CODEGEN_EN.md`](docs/CODEGEN_EN.md) - direct target code-generation model
- [`docs/DESIGN_RULES_EN.md`](docs/DESIGN_RULES_EN.md) - simplicity and scope rules
- [`docs/ONE_ZERO_SCOPE_EN.md`](docs/ONE_ZERO_SCOPE_EN.md) - 1.0 language boundary
- [`docs/STDLIB_EN.md`](docs/STDLIB_EN.md) - small C-Lite standard library
- [`docs/BACKEND_AUDIT_EN.md`](docs/BACKEND_AUDIT_EN.md) - backend design/status audit

Complete Hungarian companion documents use the corresponding `_HU.md` filenames.

## Nine-target integration check

After building the workspace:

```sh
svm_clite/scripts/test_9_targets.sh
svm_clite/scripts/report_codegen.sh svm_clite/tests/programs/array_pointer.cl
```

The integration script uses the separate `svm-clite` and `svm-asm` executables and compiles/assembles the same small C-Lite programs for all nine targets.
