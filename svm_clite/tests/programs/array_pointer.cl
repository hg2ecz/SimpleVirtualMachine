fn sum(u16* p, u16 n) -> u16 {
    u16 i = 0;
    u16 s = 0;
    while (i < n) {
        s = s + p[i];
        i = i + 1;
    }
    return s;
}

fn main() -> u16 {
    u16 a[4];
    a[0] = 1;
    a[1] = 2;
    a[2] = 3;
    a[3] = 4;
    return sum(&a[0], 4);
}
