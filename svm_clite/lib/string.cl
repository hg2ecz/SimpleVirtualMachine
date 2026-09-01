// Zero-terminated byte strings.
fn strlen(u8* s) -> u16 {
    u16 n = 0;
    while (s[n] != 0) {
        n = n + 1;
    }
    return n;
}

fn strcmp(u8* a, u8* b) -> i16 {
    u16 i = 0;
    while (a[i] != 0) {
        if (a[i] < b[i]) { return -1; }
        if (a[i] > b[i]) { return 1; }
        i = i + 1;
    }
    if (b[i] != 0) { return -1; }
    return 0;
}
