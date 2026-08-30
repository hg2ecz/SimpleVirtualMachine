# SVM-C language overview

SVM-C is a deliberately small freestanding C-like systems language for all nine SVM CPU targets, not ANSI/ISO C.

The current language includes `bool`, `i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64`, `int`, `long`, `void`, static variables and fixed arrays, `[]` indexing, functions, `if/else`, `while`, `for`, `do...while`, `break`, `continue`, short-circuit `&&/||`, statement-level `++/--` and compound assignment, restricted `sizeof`, and VT100-oriented `puts("...")`.

Locals and parameters use static storage rather than stack frames. This keeps the compiler/backends small but deliberately excludes recursion and reentrancy.

The implementation-synchronized reference is **`C_REFERENCE_EN.md`**; the deliberate subset boundaries are documented in that reference.


## Source includes

Reusable source libraries can be included with `include "file.sc";`. This is not a preprocessor: the file is expanded into the same translation unit before lexing/parsing. Relative paths are resolved from the including file, with optional `-I` search directories. Each canonical file is included at most once per compilation.


## Wide numeric objects

`i32/u32` are 4-byte and `i64/u64` are 8-byte storage objects. Wide arithmetic is library-based and address-oriented; `&object` address formation is supported. `int` aliases `i16`, and `long` aliases `i32`. Public `i64/u64` use is intentionally limited to holding full 32×32 multiplication results.
