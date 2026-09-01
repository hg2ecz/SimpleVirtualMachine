# SVM C-Lite 1.0 language

SVM C-Lite is an intentionally small, structured, **C-like** language for SimpleVirtualMachine. It is not intended to clone standard C or Rust. C influences declarations, expressions, arrays, pointers and the memory model; the deliberately simple `fn ... -> type` function syntax is inspired by Rust and other modern languages.

```c
fn sum(u16* data, u16 count) -> u16 {
    u16 i = 0;
    u16 result = 0;
    while (i < count) {
        result = result + data[i];
        i = i + 1;
    }
    return result;
}

fn main() -> u16 {
    u16 values[4];
    values[0] = 10;
    values[1] = 20;
    values[2] = 30;
    values[3] = 40;
    return sum(&values[0], 4);
}
```

Core types are `i8`, `u8`, `i16`, `u16`, and `void`. One pointer level is supported (`u16* p`); `void*`, pointer-to-pointer, function pointers, and complex C declarators are intentionally absent.

Fixed arrays use C-like syntax:

```c
u16 values[4];
u8 bytes[128];
```

Both local and global variables/arrays are supported. Global scalar initializers are limited to integer constants . Constant array indices are bounds checked.

Control flow consists of `if/else`, `while`, `break`, `continue`, and `return`. `while` is the only loop construct.

Pointers and indexed accesses are lowered to byte-addressed SVM memory operations with element-size scaling performed by the compiler.

The language remains architecture-independent and targets all nine SVM architectures through CLIR. Each target has its own direct CLIR-to-native-ISA backend; no shared generic CPU backend or optimizer is involved.

## Simple namespace

C-Lite intentionally avoids name shadowing. A local or parameter may not reuse a visible global or another variable name in the same function. This differs from C but keeps diagnostics and static storage allocation simple.

## Target-neutral assembly-like IR

`svm-clite --emit ir program.cl` exposes the deliberately small target-neutral IR. It contains virtual temporaries, memory operations, arithmetic, labels/jumps, calls and returns but no physical ISA registers or target-specific stack/belt/TTA details. See `CLIR_0_1_EN.md`.

Raw target-neutral memory access is available through `load8`, `load16`, `store8`, `store16`; `vload8`, `vload16`, `vstore8`, and `vstore16` are the volatile/MMIO forms. Direct and mutual recursion are rejected to keep static allocation and call handling simple.

## Simple control flow and constants

`while` is the only loop construct. `else if` is parser sugar for nested `if` statements.


Decimal, hexadecimal and binary literals are accepted: `1234`, `0x04d2`, `0b1010`.

C-Lite deliberately performs no optimization, including no constant folding. Literal arithmetic remains visible as separate CLIR operations; constant division/modulo by zero is still rejected as a semantic error.


## Includes and comments

Includes are deliberately textual:

```c
include "math.cl";
```

There is no macro preprocessor. An include occupies its own line. Both `// ...` and `/* ... */` comments are accepted; comments do not enter the AST or affect generated code.

## 1.0 simplicity rules

Includes are textual include-once: a file is expanded at most once per compilation and include cycles are errors. There is no macro system.

Local visibility remains block-based, but every local name must be unique within a function. The direct backend intentionally gives each local name one static storage slot; it does not implement scope-based slot reuse.

The compiler performs no optimization. Source operations remain visible as CLIR operations and are then lowered mechanically to assembly.

## Boolean type

`bool` has the literals `true` and `false`. Comparisons return `bool`. Stored booleans use one byte containing 0 or 1; there is no bit packing and integers do not implicitly convert to `bool`.
