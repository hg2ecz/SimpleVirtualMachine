fn min_u16(u16 a, u16 b) -> u16 {
    if (a < b) { return a; }
    return b;
}

fn max_u16(u16 a, u16 b) -> u16 {
    if (a > b) { return a; }
    return b;
}

fn abs_i16(i16 x) -> i16 {
    if (x < 0) { return -x; }
    return x;
}

fn gcd_u16(u16 a, u16 b) -> u16 {
    while (b != 0) {
        u16 r = a % b;
        a = b;
        b = r;
    }
    return a;
}
