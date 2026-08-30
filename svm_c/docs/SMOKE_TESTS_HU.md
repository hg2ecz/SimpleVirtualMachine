# Numerikus smoke és regressziós tesztek

A `svm_c/examples/smoke/` tesztek célja, hogy a nagy benchmarkok (például FFT) előtt gyorsan elkülönítsék a nyelvi, backend-, wide-integer-, Q15- és soft-float regressziókat.

## Részletes tesztek

- `smoke_scalar_int.sc` - bool, u8/i8, u16/i16;
- `smoke_wide_int.sc` - u32/i32, cross-word shift, carry/borrow, div/mod, 32x32 -> 64;
- `smoke_q15.sc` - Q15 és trigonometria;
- `smoke_f16.sc` - binary16 műveletek, konverziók és speciális értékek;
- `smoke_f32.sc` - binary32 műveletek, konverziók és speciális értékek;
- `smoke_arithmetic.sc` - isqrt, powu, gcd/lcm és PRNG.

## Összefoglaló tesztek

`smoke_all.sc` széles, egybinárisos regressziós teszt. `smoke_all_compact.sc` kisebb call graphgal és adathalmazzal minden fő numerikus családot reprezentatívan érint.

A v2.3.17 utáni referenciaállapotban mindkettő mind a kilenc targeten `OK`.

## Teljes futtatás

Az autoritatív részletes regresszió a külön smoke programok targetenkénti futtatása. Ha a repositoryban jelen van a helper script, `run_numeric_smoke.sh` ezt automatizálja. A compact teszt gyors kapu, de nem helyettesíti a részletes teszteket.

## Tesztelési elv

Ahol lehetséges, bitpontos referenciaértékeket használunk. Közelítő algoritmusnál, például Q15 trigonometria esetén dokumentált toleranciát kell használni; a smoke nem írhat elő szigorúbb matematikai tulajdonságot, mint maga a könyvtári specifikáció.
