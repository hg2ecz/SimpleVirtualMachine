// Zero-terminated byte-string helpers. String values are u16 addresses.

u16 strlen(u16 s) {
    u16 n;
    n = 0;
    while (load8(s + n) != 0) n = n + 1;
    return n;
}

u16 strcmp(u16 a, u16 b) {
    u16 av;
    u16 bv;
    while (1) {
        av = load8(a);
        bv = load8(b);
        if (av < bv) return 0xffff;
        if (av > bv) return 1;
        if (av == 0) return 0;
        a = a + 1;
        b = b + 1;
    }
}

u16 strncmp(u16 a, u16 b, u16 count) {
    u16 av;
    u16 bv;
    while (count != 0) {
        av = load8(a);
        bv = load8(b);
        if (av < bv) return 0xffff;
        if (av > bv) return 1;
        if (av == 0) return 0;
        a = a + 1;
        b = b + 1;
        count = count - 1;
    }
    return 0;
}

u16 strcpy(u16 dst, u16 src) {
    u16 first;
    u16 c;
    first = dst;
    while (1) {
        c = load8(src);
        store8(dst, c);
        if (c == 0) return first;
        dst = dst + 1;
        src = src + 1;
    }
}

u16 strncpy(u16 dst, u16 src, u16 count) {
    u16 first;
    u16 c;
    first = dst;
    while (count != 0) {
        c = load8(src);
        store8(dst, c);
        dst = dst + 1;
        count = count - 1;
        if (c == 0) {
            while (count != 0) {
                store8(dst, 0);
                dst = dst + 1;
                count = count - 1;
            }
            return first;
        }
        src = src + 1;
    }
    return first;
}

// Returns the address of the first matching byte, or 0.
u16 strchr(u16 s, u16 ch) {
    u16 c;
    ch = ch & 0x00ff;
    while (1) {
        c = load8(s);
        if (c == ch) return s;
        if (c == 0) return 0;
        s = s + 1;
    }
}

u16 streq(u16 a, u16 b) {
    return strcmp(a, b) == 0;
}
