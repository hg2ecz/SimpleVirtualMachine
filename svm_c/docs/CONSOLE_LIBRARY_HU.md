# SVM-C konzolkönyvtár

A fordító beépített konzolfüggvényei:

- `putc(value)` – egy bájt kiírása;
- `puts("literal")` – string-literál kiírása;
- `getc()` – egy bájt beolvasása.

A `svm_c/lib/console.sc` include további formázó segédfüggvényeket ad:

- `newline()` – CR+LF;
- `puthex16(v)` – pontosan négy hexadecimális számjegy;
- `putu16(v)` – előjel nélküli 16 bites decimális szám.

Példa:

```c
include "lib/console.sc";

u16 main() {
    puts("value");
    putu16(12345);
    putc(32);
    puthex16(0xBEEF);
    newline();
    return 0;
}
```

`-O1/-O2/-Os` mellett a nem használt segédfüggvényeket a compiler nem emittálja a programképbe.
