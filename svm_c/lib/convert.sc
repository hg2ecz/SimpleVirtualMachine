// Integer/text conversion helpers. Buffers contain zero-terminated bytes.

u16 digit_value(u16 ch) {
    if (ch >= 48 && ch <= 57) return ch - 48;
    if (ch >= 65 && ch <= 70) return ch - 55;
    if (ch >= 97 && ch <= 102) return ch - 87;
    return 0xffff;
}

u16 parse_u16_dec(u16 s) {
    u16 v;
    u16 d;
    v = 0;
    while (1) {
        d = digit_value(load8(s));
        if (d > 9) return v;
        v = v * 10 + d;
        s = s + 1;
    }
}

// Returns a two's-complement i16 bit pattern in u16.
u16 parse_i16_dec(u16 s) {
    u16 neg;
    u16 v;
    neg = 0;
    if (load8(s) == 45) {
        neg = 1;
        s = s + 1;
    } else if (load8(s) == 43) {
        s = s + 1;
    }
    v = parse_u16_dec(s);
    if (neg) return 0 - v;
    return v;
}

u16 parse_hex16(u16 s) {
    u16 v;
    u16 d;
    v = 0;
    if (load8(s) == 48 && (load8(s + 1) == 120 || load8(s + 1) == 88)) s = s + 2;
    while (1) {
        d = digit_value(load8(s));
        if (d > 15) return v;
        v = (v << 4) | d;
        s = s + 1;
    }
}

u16 __hexchar(u16 d) {
    d = d & 15;
    if (d < 10) return 48 + d;
    return 55 + d;
}

// Writes exactly four hex digits plus NUL. Returns dst.
u16 u16_to_hex(u16 dst, u16 v) {
    store8(dst, __hexchar(v >> 12));
    store8(dst + 1, __hexchar(v >> 8));
    store8(dst + 2, __hexchar(v >> 4));
    store8(dst + 3, __hexchar(v));
    store8(dst + 4, 0);
    return dst;
}

// Writes unsigned decimal plus NUL. Buffer must hold at least 6 bytes.
u16 u16_to_dec(u16 dst, u16 v) {
    u8 digits[5];
    u16 n;
    u16 first;
    first = dst;
    n = 0;
    if (v == 0) {
        store8(dst, 48);
        store8(dst + 1, 0);
        return first;
    }
    while (v != 0) {
        store8(&digits + n, 48 + (v % 10));
        n = n + 1;
        v = v / 10;
    }
    while (n != 0) {
        n = n - 1;
        store8(dst, load8(&digits + n));
        dst = dst + 1;
    }
    store8(dst, 0);
    return first;
}

// Writes signed i16 decimal plus NUL. Buffer must hold at least 7 bytes.
u16 i16_to_dec(u16 dst, u16 v) {
    u16 first;
    first = dst;
    if (v & 0x8000) {
        store8(dst, 45);
        dst = dst + 1;
        v = 0 - v;
    }
    u16_to_dec(dst, v);
    return first;
}
