# C-Lite small standard library

The library under `svm_clite/lib/` is ordinary C-Lite source. It does not add compiler builtins and therefore does not make the compiler more complex.

Modules:

- `memory.cl` – byte/word memory helpers;
- `string.cl` – zero-terminated byte-string helpers;
- `math.cl` – small integer helpers;
- `convert.cl` – simple integer/text conversion helpers;
- `crc.cl` – checksum/CRC examples.

Use the ordinary textual include mechanism, for example:

```c
include "math.cl";

fn main() -> u16 {
    return gcd_u16(84, 30);
}
```

Compile with the library directory on the include path:

```sh
svm-clite -I svm_clite/lib --target register program.cl
```

The design rule is simple: if a feature can be written in C-Lite itself, prefer a `.cl` library routine over a new compiler builtin.
