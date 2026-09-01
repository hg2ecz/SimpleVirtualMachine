# C-Lite kis standard library

A standard library maga is C-Lite forrás. A fordító nem kap külön beépített algoritmusokat.

Modulok:

- `memory.cl`: `mem_zero`, `memcpy`, `memcmp`
- `string.cl`: `strlen`, `strcmp`
- `math.cl`: `min_u16`, `max_u16`, `abs_i16`, `gcd_u16`
- `convert.cl`: `hex_digit`, `u16_to_hex`
- `crc.cl`: `crc8`

Használat:

```c
include "math.cl";

fn main() -> u16 {
    return gcd_u16(84, 30);
}
```

Fordítás a projekt gyökeréből például:

```sh
svm-clite -I svm_clite/lib --target register program.cl
```

Az include egyszerű textual include-once. Nincs macro-preprocessor.
