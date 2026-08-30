/* Signed helpers for the native 8/16-bit storage types. */
u16 i8_sext(u8 x) {
    if (x & 0x80) return 0xff00 | x;
    return x;
}

u16 i16_abs(i16 x) { if (x & 0x8000) return 0 - x; return x; }
u16 i16_negative(i16 x) { return (x & 0x8000) != 0; }

i16 i16_div(i16 a, i16 b) {
    u16 aa; u16 bb; u16 q;
    aa=i16_abs(a); bb=i16_abs(b);
    if (bb==0) return 0;
    q=aa/bb;
    if (i16_negative(a) ^ i16_negative(b)) return 0-q;
    return q;
}

i16 i16_mod(i16 a, i16 b) {
    u16 aa; u16 bb; u16 r;
    aa=i16_abs(a); bb=i16_abs(b);
    if (bb==0) return 0;
    r=aa%bb;
    if (i16_negative(a)) return 0-r;
    return r;
}

bool i16_lt(i16 a, i16 b) {
    u16 sa; u16 sb;
    sa=i16_negative(a); sb=i16_negative(b);
    if (sa != sb) return sa;
    if (sa) return b < a;
    return a < b;
}

/* Arithmetic right shift by one. */
i16 i16_asr1(i16 x) {
    u16 y;
    y=x>>1;
    if (x & 0x8000) y=y|0x8000;
    return y;
}
