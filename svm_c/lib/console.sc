// Common console formatting helpers for SVM-C.
// putc(), puts(), and getc() are compiler built-ins backed by console MMIO.

void newline() {
    putc(13);
    putc(10);
}

void __console_hex_digit(u16 d) {
    d = d & 15;
    if (d < 10) { putc(48 + d); }
    else { putc(55 + d); }
}

void puthex16(u16 v) {
    __console_hex_digit(v >> 12);
    __console_hex_digit(v >> 8);
    __console_hex_digit(v >> 4);
    __console_hex_digit(v);
}

void putu16(u16 v) {
    u16 d;
    u16 q;
    u16 started;
    d = 10000;
    started = 0;
    while (d != 0) {
        q = v / d;
        if (q != 0 || started || d == 1) {
            putc(48 + q);
            started = 1;
        }
        v = v % d;
        d = d / 10;
    }
}
