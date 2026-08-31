# Assembly examples

Az összes kézzel írt assembly példa az assembler crate alatt található, architektúránként külön könyvtárban:

- `register/`
- `stack/`
- `accumulator/`
- `memreg/`
- `loadstore/`
- `regmem/`
- `memory2memory/`
- `belt/`
- `tta/`

Példa fordítás:

```sh
cargo run -p svm-asm -- register svm_asm/examples/register/text_demo.asm
cargo run -p svm-rt -- svm_asm/examples/register/text_demo.svm
```

A példák a közös platformot használják; a runtime implementációja a `svm_rt/` crate-ben van, de ott már nincs külön `examples/` könyvtár.

- `register/console_library.asm` - using the include-able Register console helpers.
## Eljárásforma a példákban

A példák az új assembler-modellt követik: a belépési pont és minden külön hívható rutin `.proc NAME` / `.endproc` blokk. A ciklus- és elágazási címkék a megfelelő eljáráson belül maradnak. A belépési pont nélküli, önálló könyvtári töredékek `.keep` direktívával jelzik, mely rutinokat kell közvetlen összeállításkor megtartani. Emiatt a példák egyúttal a procedure-GC helyes használatát is demonstrálják.

