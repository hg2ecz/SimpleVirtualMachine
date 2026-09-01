# SVM C-Lite Programmer's Manual

**Version:** 1.0  
**Language goal:** structured, architecture-independent assembly  
**Targets:** Register, Stack, Accumulator, MemReg, Load/Store, Register-Memory, Memory-to-Memory, Belt, TTA

## 1. What is C-Lite?

SVM C-Lite is an intentionally small C-like language for the SimpleVirtualMachine architectures. It is neither full C nor a Rust subset. Its purpose is to write assembly-level programs **without learning target-specific assembly syntax**, using a small amount of structured syntax.

The main design rule is:

> C-Lite is structured, architecture-independent assembly. Simplicity of the language and compiler is more important than C compatibility or optimized code.

A source program remains unchanged across all nine targets:

```c
fn add(u16 a, u16 b) -> u16 {
    return a + b;
}

fn main() -> u16 {
    return add(10, 20);
}
```

Compilation model:

```text
C-Lite source
    ↓
lexer
    ↓
parser
    ↓
simple semantic validation
    ↓
CLIR 0.1
    ↓
mechanical target ASM
    ↓
external svm-asm
    ↓
SVM program
```

The compiler has no optimizer, SSA, register allocator, linker, or embedded assembler.

---

## 2. First program

```c
fn main() -> u16 {
    u16 a = 10;
    u16 b = 20;
    return a + b;
}
```

Check without code generation:

```sh
svm-clite --check hello.cl
```

Generate Register assembly:

```sh
svm-clite --target register hello.cl
```

The default output is `hello.asm`.

Assemble separately:

```sh
svm-asm register hello.asm hello.svm
```

Convenience mode can invoke the external assembler:

```sh
svm-clite --target register --assemble hello.cl hello.svm
```

A custom assembler path may be selected:

```sh
svm-clite --assembler /opt/svm/bin/svm-asm --target stack --assemble hello.cl
```

---

## 3. Supported targets

| Target | Alias |
|---|---|
| `register` | `reg` |
| `stack` | – |
| `accumulator` | `acc` |
| `memreg` | – |
| `loadstore` | – |
| `regmem` | – |
| `memory2memory` | `m2m` |
| `belt` | – |
| `tta` | – |

The same `.cl` source should normally compile unchanged for every target.

---

## 4. Source files and comments

Recommended extension:

```text
.cl
```

Line comment:

```c
// comment
u16 x = 10;
```

Block comment:

```c
/*
   block comment
*/
```

Comments do not enter the AST or generated assembly.

---

## 5. Types

The complete core type set is:

```text
bool
i8
u8
i16
u16
void
```

### 5.1 Integer types

- `u8`: unsigned 8-bit integer
- `i8`: signed 8-bit integer
- `u16`: unsigned 16-bit integer
- `i16`: signed 16-bit integer

C-Lite intentionally has no built-in 32-bit, floating-point, or general promotion system. Wider algorithms may be implemented in libraries as multi-word routines or assembly helpers.

### 5.2 `bool`

```c
bool ready = true;
bool done = false;
```

Stored booleans occupy one byte and contain 0 or 1.

```c
bool flags[16];
```

This occupies 16 bytes; booleans are not bit-packed.

Comparisons return `bool`:

```c
bool smaller = a < b;
```

### 5.3 `void`

`void` is for function return types only. Globals, locals, and parameters cannot be `void`.

A function without `-> type` returns `void`:

```c
fn clear(u8* data, u16 count) {
    // void function
}
```

---

## 6. Numeric literals

Three forms are accepted:

```c
u16 a = 1234;
u16 b = 0x04d2;
u16 c = 0b10101010;
```

The lexer stores numeric literals as `u16`, so they must fit in 0..65535.

A numeric literal may directly initialize or be assigned to `i8`, `u8`, `i16`, or `u16`. The current compiler does not separately range-check a literal against the narrower destination type; this is a low-level language and the programmer is responsible for choosing an appropriate value.

There is no constant folding:

```c
u16 x = 2 + 3 * 4;
```

remains several CLIR operations.

---

## 7. Variables

### 7.1 Local scalar

```c
u16 counter;
u16 start = 10;
u8 ch = 65;
bool active = true;
```

### 7.2 Global scalar

```c
u16 ticks;
u8 mode = 1;
bool enabled = false;
```

A global scalar initializer must be a direct integer or boolean literal. This is valid:

```c
u16 mode = 3;
```

This is not a supported global initializer:

```c
u16 mode = 1 + 2;
```

### 7.3 Simple namespace

C-Lite intentionally does not support variable shadowing.

This is invalid:

```c
u16 count;

fn f(u16 count) -> u16 {
    return count;
}
```

Every parameter and local name in a function must be unique, even across separate nested blocks. The direct backend gives each local a simple static storage slot.

---

## 8. Fixed arrays

Arrays use C-like syntax:

```c
u16 values[4];
u8 bytes[128];
i16 samples[32];
bool flags[8];
```

The length is a positive compile-time numeric literal.

Not supported:

- zero-length arrays;
- nested arrays;
- arrays of pointers;
- array initializer lists;
- array parameters.

Pass arrays using pointers:

```c
fn sum(u16* data, u16 count) -> u16 {
    return data[0];
}
```

### 8.1 Indexing

```c
u16 values[4];
values[0] = 10;
values[1] = values[0] + 1;
```

Constant indices are bounds-checked:

```c
values[4] = 1; // error for a 4-element array
```

Dynamic indices have no run-time bounds check:

```c
values[i] = 1;
```

C-Lite is a low-level language; the programmer is responsible for valid addressing.

---

## 9. Pointers

Only one pointer level is supported:

```c
u16* p;
u8* bytes;
bool* flags;
```

There is no:

```text
void*
u16**
function pointer
```

### 9.1 Address-of

```c
u16 x = 10;
u16* p = &x;
```

Address of an array element:

```c
u16 values[4];
u16* p = &values[0];
```

### 9.2 Dereference

```c
u16 x = 10;
u16* p = &x;
*p = 20;
return *p;
```

### 9.3 Pointer indexing

```c
p[i]
```

Element scaling is automatic:

- `u8*`, `i8*`, `bool*`: 1 byte per element
- `u16*`, `i16*`: 2 bytes per element

General C pointer arithmetic is intentionally not a primary language feature. Prefer indexed access.

---

## 10. Functions

### 10.1 Basic form

```c
fn add(u16 a, u16 b) -> u16 {
    return a + b;
}
```

Void function:

```c
fn set_zero(u16* p) {
    *p = 0;
    return;
}
```

### 10.2 Parameters

```c
fn mix(u8 a, i8 b, u16 c, i16 d, bool enabled) -> u16 {
    if (enabled) {
        return c;
    }
    return 0;
}
```

Array parameters are not supported; use pointers.

### 10.3 Recursion

Direct and mutual recursion are rejected:

```c
fn f(u16 x) -> u16 {
    return f(x); // error
}
```

A cycle such as `a -> b -> a` is also an error.

This keeps parameter/local storage static and avoids a general stack-frame ABI.

### 10.4 `main`

Every program must define `main`.

Typical form:

```c
fn main() -> u16 {
    return 0;
}
```

---

## 11. Expressions and operators

### 11.1 Arithmetic

```text
+  -  *  /  %
```

### 11.2 Bit operations

```text
&  |  ^  ~  <<  >>
```

### 11.3 Comparison

```text
==  !=  <  <=  >  >=
```

Comparisons return `bool`.

### 11.4 No `&&`, `||`, or `!`

There are deliberately no short-circuit logical operators.

Write explicit structured conditions instead:

```c
if (a != 0) {
    if (b != 0) {
        // both are non-zero
    }
}
```

`~` is bitwise NOT, not logical NOT.

### 11.5 Operator precedence

Highest to lowest:

1. unary `-`, `~`, `&`, `*`
2. `* / %`
3. `+ -`
4. `<< >>`
5. `< <= > >=`
6. `== !=`
7. bitwise `&`
8. bitwise `^`
9. bitwise `|`

Use parentheses when clarity matters.

---

## 12. Type handling

C-Lite intentionally avoids the full C conversion and integer-promotion rules.

### 12.1 Mixed arithmetic

Arithmetic/bit operators require integer operands, but there is no general C-style promotion system. For predictable low-level code, use matching types deliberately.

### 12.2 Literals

Numeric literals are represented as `u16` internally and may be assigned directly to the integer scalar types.

### 12.3 Pointer element type

Pointer arguments must have a matching element type:

```c
fn first(u16* p) -> u16 {
    return p[0];
}

fn main() -> u16 {
    u8 data[4];
    return first(&data[0]); // type error
}
```

---

## 13. Conditional execution

```c
if (condition) {
    // ...
} else {
    // ...
}
```

`else if` is supported as parser sugar:

```c
if (x == 0) {
    return 0;
} else if (x == 1) {
    return 1;
} else {
    return 2;
}
```

A condition may be any scalar value (`bool`, integer, or pointer). Zero is false and non-zero is true. This condition rule does not imply that integers may be assigned to `bool` variables.

---

## 14. Loops

`while` is the only loop construct:

```c
u16 i = 0;
while (i < 10) {
    i = i + 1;
}
```

There is no `for`, `do/while`, or `goto`.

### 14.1 `break`

```c
while (true) {
    if (ready) {
        break;
    }
}
```

### 14.2 `continue`

```c
while (i < n) {
    i = i + 1;
    if (i == 2) {
        continue;
    }
    sum = sum + i;
}
```

`break` and `continue` are valid only inside loops.

---

## 15. Raw memory and MMIO

Direct target-neutral memory access is an important part of the architecture-independent assembly model.

### 15.1 Normal memory

```c
u8 a = load8(0x1000);
u16 b = load16(0x2000);

store8(0x1000, a);
store16(0x2000, b);
```

Logical signatures:

```text
load8(u16 address) -> u8
load16(u16 address) -> u16
store8(u16 address, u8 value) -> void
store16(u16 address, u16 value) -> void
```

### 15.2 Volatile/MMIO

```c
vstore8(0xff00, 65);
u8 status = vload8(0xff01);
```

Available operations:

```text
vload8
vload16
vstore8
vstore16
```

The `v` form expresses volatile/MMIO access. Device addresses belong to the platform documentation rather than the C-Lite language itself.

---

## 16. Include

```c
include "math.cl";
```

Includes are deliberately simple textual include-once expansion.

Search order:

1. directory of the current source file;
2. directories passed with `-I`.

Example:

```sh
svm-clite -I svm_clite/lib --target register program.cl
```

Each canonical file is expanded at most once. Include cycles are errors.

There is no macro preprocessor, `#define`, conditional compilation, or include-guard syntax.

---

## 17. Small standard library

The standard library is written in C-Lite itself.

### `memory.cl`

```text
mem_zero
memcpy
memcmp
```

### `string.cl`

```text
strlen
strcmp
```

Strings are zero-terminated `u8` sequences.

### `math.cl`

```text
min_u16
max_u16
abs_i16
gcd_u16
```

### `convert.cl`

```text
hex_digit
u16_to_hex
```

### `crc.cl`

```text
crc8
```

Example:

```c
include "math.cl";

fn main() -> u16 {
    return gcd_u16(84, 30);
}
```

---

## 18. CLIR: architecture-independent assembly

The internal intermediate language is CLIR 0.1. It can be emitted for learning and debugging:

```sh
svm-clite --emit ir program.cl
```

C-Lite:

```c
u16 x = a + b;
```

Corresponding style of CLIR:

```text
load.u16 %0, a
load.u16 %1, b
add.u16 %2, %0, %1
store.u16 x, %2
```

`%0`, `%1`, ... are virtual temporary values, not physical CPU registers.

### 18.1 Main CLIR operations

```text
const.T
load.T
store.T
addr
index
loadmem.T
storemem.T
loadmemv.T
storememv.T

add.T sub.T mul.T div.T mod.T
and.T or.T xor.T shl.T shr.T
neg.T not.T

eq.T ne.T lt.T le.T gt.T ge.T

jz
jmp
call
ret
```

See `CLIR_0_1_EN.md` for the full reference.

---

## 19. Lowering structured control flow

### 19.1 `if`

C-Lite:

```c
if (a < b) {
    x = 1;
} else {
    x = 2;
}
```

Conceptual CLIR:

```text
load.u16 %0, a
load.u16 %1, b
lt.u16 %2, %0, %1
jz %2, else_0
const.u16 %3, 1
store.u16 x, %3
jmp endif_1
else_0:
const.u16 %4, 2
store.u16 x, %4
endif_1:
```

### 19.2 `while`

C-Lite:

```c
while (i < n) {
    i = i + 1;
}
```

Conceptual CLIR:

```text
while_test_0:
load.u16 %0, i
load.u16 %1, n
lt.u16 %2, %0, %1
jz %2, while_end_1
load.u16 %3, i
const.u16 %4, 1
add.u16 %5, %3, %4
store.u16 i, %5
jmp while_test_0
while_end_1:
```

This is why C-Lite can be viewed as structured assembly: structured constructs lower to a small number of explicit branches.

---

## 20. Generated assembly and `.proc`

C-Lite emits target assembly that may contain `.proc/.endproc` procedure blocks. C-Lite does not decide which procedures are live.

The external `svm-asm` is responsible for:

- assembly include processing;
- `.equ` constants;
- `.proc/.endproc` reachability analysis;
- excluding unreachable procedures;
- final binary generation.

This is an assembler responsibility, not a C-Lite optimization pass.

---

## 21. Errors and `--check`

Use:

```sh
svm-clite --check program.cl
```

This performs parsing and semantic validation without generating assembly.

Checks include:

- lexical/parser errors;
- unknown types or variables;
- duplicate names;
- pointer type mismatches;
- constant array bounds;
- `break/continue` placement;
- function argument count/types;
- return type;
- direct and mutual recursion;
- division/modulo by literal zero.

Lexer errors include line/column information. Include errors include source file and line number.

---

## 22. Complete example: summing an array

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

Only the target option changes:

```sh
svm-clite --target register sum.cl
svm-clite --target stack sum.cl
svm-clite --target accumulator sum.cl
svm-clite --target memreg sum.cl
svm-clite --target loadstore sum.cl
svm-clite --target regmem sum.cl
svm-clite --target memory2memory sum.cl
svm-clite --target belt sum.cl
svm-clite --target tta sum.cl
```

---

## 23. Complete example: MMIO and boolean logic

```c
fn main() -> u16 {
    u8 status = vload8(0xff01);
    bool ready = status != 0;

    if (ready) {
        vstore8(0xff00, 65);
        return 1;
    }

    return 0;
}
```

There is no Register/Stack/Accumulator-specific syntax in the source.

---

## 24. Deliberately unsupported C features

The 1.0 line intentionally does not contain:

```text
for
do/while
switch
goto
struct
union
enum
typedef
macro/#define
++ --
+= -= etc.
?:
&& || !
function pointer
pointer-to-pointer
void*
varargs
malloc/free
recursion
full C integer-promotion rules
optimizer
SSA
register allocator
```

These are not merely unfinished items; their absence is part of the simplicity goal. A new feature should only be added when it lowers directly and clearly to a few existing CLIR operations.

---

## 25. Recommended programming style

Prefer:

- short, explicit functions;
- explicit scalar types;
- simple `while` loops;
- passing arrays as pointer + length;
- `vload*`/`vstore*` for MMIO;
- several simple expressions instead of one dense expression;
- viewing CLIR while learning or debugging.

Instead of:

```c
result = (a + b) * (c - d) ^ mask;
```

this can be easier to follow:

```c
u16 x = a + b;
u16 y = c - d;
u16 z = x * y;
result = z ^ mask;
```

Because there is no optimizer, the relationship between source, CLIR, and assembly stays visible.

---

## 26. Further documentation

- `LANGUAGE_EN.md` – concise language reference
- `CLIR_0_1_EN.md` – CLIR 0.1 specification
- `CODEGEN_EN.md` – code-generation model
- `LEARNING_EN.md` – step-by-step learning path
- `DESIGN_RULES_EN.md` – design constraints
- `ONE_ZERO_SCOPE_EN.md` – deliberate 1.0 scope
- `STDLIB_EN.md` – small standard library
- `examples/` – compilable examples

Recommended learning order:

```text
PROGRAMMING_MANUAL_EN.md
    ↓
LEARNING_EN.md + examples/
    ↓
CLIR_0_1_EN.md
    ↓
CODEGEN_EN.md
    ↓
target-specific assembly documentation
```
