// Byte-addressed memory helpers. Addresses are u16 values.

void mem_zero(u16 dst, u16 count) {
    while (count != 0) {
        store8(dst, 0);
        dst = dst + 1;
        count = count - 1;
    }
}

void memset(u16 dst, u16 value, u16 count) {
    value = value & 0x00ff;
    while (count != 0) {
        store8(dst, value);
        dst = dst + 1;
        count = count - 1;
    }
}

void memcpy(u16 dst, u16 src, u16 count) {
    while (count != 0) {
        store8(dst, load8(src));
        dst = dst + 1;
        src = src + 1;
        count = count - 1;
    }
}

void memmove(u16 dst, u16 src, u16 count) {
    if (count == 0 || dst == src) return;
    if (dst < src || dst >= src + count) {
        memcpy(dst, src, count);
        return;
    }
    dst = dst + count;
    src = src + count;
    while (count != 0) {
        dst = dst - 1;
        src = src - 1;
        store8(dst, load8(src));
        count = count - 1;
    }
}

// Returns 0 when equal, 0xffff when a<b, 1 when a>b.
u16 memcmp(u16 a, u16 b, u16 count) {
    u16 av;
    u16 bv;
    while (count != 0) {
        av = load8(a);
        bv = load8(b);
        if (av < bv) return 0xffff;
        if (av > bv) return 1;
        a = a + 1;
        b = b + 1;
        count = count - 1;
    }
    return 0;
}
