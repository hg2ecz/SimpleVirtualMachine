// 16-bit bit-manipulation helpers.

u16 rotl16(u16 x, u16 n) {
    n = n & 15;
    if (n == 0) return x;
    return (x << n) | (x >> (16 - n));
}

u16 rotr16(u16 x, u16 n) {
    n = n & 15;
    if (n == 0) return x;
    return (x >> n) | (x << (16 - n));
}

u16 popcount16(u16 x) {
    u16 n;
    n = 0;
    while (x != 0) {
        n = n + (x & 1);
        x = x >> 1;
    }
    return n;
}

u16 parity16(u16 x) {
    return popcount16(x) & 1;
}

u16 clz16(u16 x) {
    u16 n;
    if (x == 0) return 16;
    n = 0;
    while ((x & 0x8000) == 0) {
        n = n + 1;
        x = x << 1;
    }
    return n;
}

u16 ctz16(u16 x) {
    u16 n;
    if (x == 0) return 16;
    n = 0;
    while ((x & 1) == 0) {
        n = n + 1;
        x = x >> 1;
    }
    return n;
}

u16 bitreverse16(u16 x) {
    u16 r;
    u16 i;
    r = 0;
    i = 0;
    while (i < 16) {
        r = (r << 1) | (x & 1);
        x = x >> 1;
        i = i + 1;
    }
    return r;
}

u16 bswap16(u16 x) {
    return (x << 8) | (x >> 8);
}
