# SVM-C language reference

This document describes the **currently implemented** SVM-C source language. SVM-C is a C-like freestanding systems-programming language; it is not ANSI/ISO C.

## 1. Types

| Type | Size | Meaning |
|---|---:|---|
| `bool` | 1 byte | logical storage; zero is false, nonzero is true |
| `i8` | 1 byte | 8-bit two's-complement signed bit pattern |
| `u8` | 1 byte | 8-bit unsigned |
| `i16` / `int` | 2 bytes | 16-bit signed/two's-complement value |
| `u16` | 2 bytes | 16-bit unsigned |
| `i32` / `long` | 4 bytes | address-based multiword object |
| `u32` | 4 bytes | address-based multiword object |
| `i64` | 8 bytes | storage for a full signed 32x32 product |
| `u64` | 8 bytes | storage for a full unsigned 32x32 product |
| `void` | - | function return type only |

The CPUs are 16-bit integer machines. `bool/i8/u8/i16/u16` are native scalars. `i32/u32/i64/u64` are **address-only wide objects**: they cannot be used as direct value expressions, by-value parameters, or function return types; wide-integer and `f32` library routines receive `&object` addresses. There is no general 64-bit arithmetic API for `i64/u64`; their primary role is to hold full 32x32 multiplication results.

For the detailed numerical model, see `NUMERIC_TYPES_HU.md`.

## 2. Literals and comments

Integer literals:

```c
1234
0x1234
0XABCD
```

The value must fit in `0..0xFFFF`.

Comments:

```c
// single line
/* multiple lines */
```

Character literals such as `'A'` are not supported. A string literal is supported only as the special argument of `puts("...")`.

Supported string escapes: `\n`, `\r`, `\t`, `\0`, `\\`, `\"`.

## 3. Variables and static storage

Global and function-local variables are supported:

```c
u16 counter;
u8 mode = 3;

u16 main() {
    u16 x = 10;
    return x;
}
```

A global initializer must be a direct numeric constant. A local initializer may be a general expression.

Local variables and parameters live in **statically allocated memory**, not in stack frames. Consequently the current ABI is neither recursive nor reentrant.

By default, the static allocator first uses `0x0000..0x00EF`, then `0xE000..0xFAFF`; `0x00F0..0xDFFF` is not used for ordinary static-object allocation. Two targets reserve additional low memory: MemReg uses `0x000E..0x000F` as compiler-owned hot scratch, while Memory-to-Memory starts user/static allocation at `0x0020` because `0x0000..0x001F` is compiler-owned scratch. The C program image may grow in `0x0100..0xDFFF`; `0xE000..0xFAFF` is the static-overflow area, `0xFB00..0xFEFF` is the runtime stack convention, and `0xFF00..0xFFFF` is MMIO.

There is no block-scope name shadowing inside a function: a local name cannot be redeclared in an inner block.

## 4. Fixed-size arrays

Supported:

```c
u8 bytes[32];
u16 words[16];

bytes[i] = 7;
x = bytes[i];
words[2] = 0x1234;
```

Rules:

- the size must be a positive numeric constant;
- a fixed array may use any storage type; a wide array element cannot be loaded as a value and must be handled through address-based library operations;
- arrays may be global or local;
- array initializers are not supported;
- array parameters are not supported;
- dynamic indexing has no run-time bounds check;
- a compile-time-known constant index outside the array is an error;
- an array name used as a value yields the array's 16-bit base address.

Treating an array name as its base address is an SVM-C-specific rule, not a complete ANSI C array-to-pointer decay model.

## 5. Assignment, `++/--`, and compound assignment

Simple assignment:

```c
x = y + 1;
a[i] = x;
```

As standalone statements, scalars and array elements support:

```c
x++;
x--;
x += 2;
x -= 2;
x *= 3;
x /= 3;
x %= 7;
x &= mask;
x |= mask;
x ^= mask;
x <<= 1;
x >>= 1;

a[i]++;
a[i] += 2;
```

The index of a compound array-element operation must be free of side effects. This is valid:

```c
a[i + 1] += 2;
```

This is currently rejected:

```c
a[getc()] += 2;
```

because the simple lowering could otherwise evaluate the index more than once. Plain `a[getc()] = x;` is allowed.

There is no prefix `++x`/`--x`, and postfix forms are not expressions: `y = x++;` is unsupported.

## 6. Expressions and operator precedence

From strongest to weakest:

| Level | Operators |
|---|---|
| unary | `- ~ !` |
| multiply | `* / %` |
| add | `+ -` |
| shift | `<< >>` |
| relational | `< > <= >=` |
| equality | `== !=` |
| bit AND | `&` |
| bit XOR | `^` |
| bit OR | `|` |
| logical AND | `&&` |
| logical OR | `||` |

`&&` and `||` are **short-circuiting**: the right-hand side executes only if it is needed to determine the result. Logical results are 0 or 1.

There is no `?:`, comma operator, or general assignment expression.

## 7. `sizeof`

Supported forms:

```c
sizeof(u8)       // 1
sizeof(u16)      // 2
sizeof(int)      // 2
sizeof(x)
sizeof(buffer)
```

The parenthesized operand may be a type name or **one object name**. A general expression such as `sizeof(a + b)` is currently unsupported. For arrays it returns the total size in bytes.

## 8. Control flow

### `if / else`

```c
if (x == 0) {
    y = 1;
} else {
    y = 2;
}
```

Zero is false and nonzero is true.

### `while`

```c
while (i < 100) {
    i++;
}
```

### `do ... while`

```c
do {
    i++;
} while (i < 100);
```

### `for`

```c
for (u16 i = 0; i < 10; i++) {
    sum += i;
}
```

All three header fields may be omitted. The init and step fields use SVM-C's simple statement forms: declaration, assignment, `++/--`, compound assignment, or expression statement.

### `break` and `continue`

Both are supported in all three loop forms. Using them outside a loop is a compile error.

The `continue` target is:

- `while`: re-check the condition;
- `for`: execute the step, then check the condition;
- `do...while`: check the condition.

There is no `switch`, `goto`, or label syntax.

## 9. Functions

```c
u16 add(u16 a, u16 b) {
    return a + b;
}

void hello() {
    puts("hello");
    return;
}
```

Rules:

- at most 4 parameters;
- a parameter cannot be `void` or an array;
- there are no separate prototypes/declarations;
- a `void` result cannot be used as a value;
- the program must define `main()`;
- direct and indirect recursion are rejected;
- variadic functions and function pointers are not supported.

## 10. `puts()` and strings

String support is deliberately narrow:

```c
puts("Hello VT100");
```

`puts()` takes exactly one string literal. The compiler emits its bytes to the VT100 console and then emits `CR` + `LF`.

There are no general string objects, string variables, `char *`, string pointers, or ordinary address values derived from string literals.

## 11. Builtins

### Normal 64 KiB system address space

| Builtin | Returns | Meaning |
|---|---|---|
| `load8(addr)` | `u8` | read one byte |
| `load16(addr)` | `u16` | read little-endian 16-bit value |
| `store8(addr,val)` | `void` | write one byte |
| `store16(addr,val)` | `void` | write little-endian 16-bit value |

### Separate 16 KiB VRAM

| Builtin | Returns | Meaning |
|---|---|---|
| `vload8(addr)` | `u8` | read one VRAM byte |
| `vload16(addr)` | `u16` | read one 16-bit VRAM value |
| `vstore8(addr,val)` | `void` | write one VRAM byte |
| `vstore16(addr,val)` | `void` | write one 16-bit VRAM value |

### VT100 / RS-232

| Builtin | Returns | Meaning |
|---|---|---|
| `putc(ch)` | `void` | transmit one byte |
| `puts("...")` | `void` | transmit a string literal followed by CR/LF |
| `getc()` | `u8` | blocking byte receive |

### Performance counters

| Builtin | Meaning |
|---|---|
| `clock_lo()` / `clock_hi()` | halves of the 32-bit VM cycle counter |
| `instr_lo()` / `instr_hi()` | halves of the 32-bit retired-instruction counter |

### Fixed-point / DSP

| Builtin | Meaning |
|---|---|
| `asr1(x)` | signed arithmetic right shift by one bit |
| `mul_q15(a,b)` | signed Q15xQ15 with a 32-bit intermediate and Q15 rescaling |

`mul_q15()` saturates the special `-32768 * -32768` case to `0x7FFF`.

## 12. Deliberately unsupported

- `char`, `short`, the C `signed` keyword, and standalone `signed` declaration syntax (`long` is supported as an alias of `i32`);
- `float`, `double`;
- general pointer declarations, dereference (`*p`), and pointer-pointer operations; `&object` address formation is supported for the address-only wide-object/library ABI;
- `struct`, `union`, `enum`, `typedef`;
- character literals;
- general string/pointer semantics;
- array initializers, array parameters, VLA;
- `switch/case/default`, `goto`;
- prefix `++/--` and value-producing postfix `++/--`;
- `?:`, comma operator, assignment expressions;
- cast syntax;
- general `sizeof(expression)`;
- preprocessor and header system;
- `static`/`extern`/linkage model;
- variadic functions;
- dynamic allocation;
- automatic stack-frame locals, recursion, reentrancy.

## 13. Short practical example

```c
u8 data[16];

u16 main() {
    u16 i = 0;
    u16 sum = 0;

    puts("SVM-C example");

    for (i = 0; i < sizeof(data); i++) {
        data[i] = i;
        if ((i & 1) == 0) {
            continue;
        }
        sum += data[i];
        if (sum > 40) {
            break;
        }
    }

    return sum;
}
```

## 14. Optimization and unopt-only mode

The `svm-c` optimization levels `-O0`, `-O1`, `-O2`, and `-Os` change generated code, not source-language syntax. The separate `svm-c-unopt-only` binary uses the same frontend and backends but runs no AST optimizer pass at all and accepts no `-O` option. Internal optimizer AST forms such as `Inc1/Dec1/Shl1/Shr1` are **not** source-language operators.

With `-O1`, `-O2`, and `-Os`, the compiler transitively walks the direct call graph rooted at `main()` and removes unreachable functions **before static-memory layout**. Consequently unused routines from an included library occupy neither code space nor static RAM. `-O0` and `svm-c-unopt-only` keep every parsed function for educational comparability. Function pointers are not supported, so the direct call graph is complete.

## Source includes

User libraries may be brought in with `include "file.sc";`. This is not a preprocessor: the included source becomes part of the same translation unit before compilation. Paths are relative to the including file, and additional `-I` search directories may be supplied. A source file is included at most once per compilation.

## 15. Target architectures and ISA-sensitive code generation

The compiler supports nine target architectures: `register`, `stack`, `accumulator`, `memreg`, `loadstore`, `regmem`, `memory2memory`, `belt`, and `tta`. The source language and semantics are common; only backend code generation differs.

Important code-generation rules of the current ISA revision:

- **Register:** the compact logical operation for `R0..R3` is `AND`; `XOR` remains a full normal ALU operation. `SUBI` is native because carry/no-borrow semantics must be preserved. Constant masking may use `MOVI` plus compact `AND` where profitable.
- **MemReg:** the one-byte logical acceleration for the `0x00..0x0F` hot-file window is `AND`, not `XOR`; XOR remains available in normal form. Compiler-owned 16-bit scratch is `0x000E..0x000F`, allowing frequent temporary `MOV16`, `ADD`, and `AND` operations to use short hot encodings.
- **Load/Store:** strict load/store model; the long-immediate form of `SUBI` has its own `SUBI16` decode to preserve correct carry/no-borrow semantics. There is no automatic post-increment load/store.
- **Stack:** several stack-manipulation and structured-loop instructions remain in the core ISA for hand-written assembly/Forth usability. Multiword arithmetic uses only a minimal hidden `C` state; comparisons still produce explicit stack values.
- **Accumulator, Register-Memory, Memory-to-Memory:** use the natural forms of their own operand models; no target has hardware floating point.

`f16` and `f32` arithmetic is software on every target. 32-bit integer and soft-float code can benefit from integer assists such as `ADC/SBC/MULHU/RCR1`, while the CPUs remain 16-bit integer machines.

### Belt16 target

The `belt` / `belt16` target uses an eight-element (`b0..b7`) implicit result belt. The current C backend lowers shared virtual temporaries into the compiler-owned `0x0000..0x000F` memory window, so Belt C static objects start at `0x0020`. This is conservative reference lowering; later belt-specific optimization may keep short-lived results directly on the belt. There is no hardware floating point.

### TTA16 target

The `tta` / `tta16` target emits transport-triggered code. ALU operations are explicit transports: the first operand goes to `ALU.X`, the second to the corresponding `ALU.*` trigger port, and the result is read from `ALU.OUT`. C-language semantics do not change; there is no hardware floating point.
