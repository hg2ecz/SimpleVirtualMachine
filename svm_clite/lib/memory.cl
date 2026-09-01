// Small architecture-neutral byte memory helpers.
fn mem_zero(u8* dst, u16 count) {
    u16 i = 0;
    while (i < count) {
        dst[i] = 0;
        i = i + 1;
    }
}

fn memcpy(u8* dst, u8* src, u16 count) {
    u16 i = 0;
    while (i < count) {
        dst[i] = src[i];
        i = i + 1;
    }
}

fn memcmp(u8* a, u8* b, u16 count) -> i16 {
    u16 i = 0;
    while (i < count) {
        if (a[i] < b[i]) { return -1; }
        if (a[i] > b[i]) { return 1; }
        i = i + 1;
    }
    return 0;
}
