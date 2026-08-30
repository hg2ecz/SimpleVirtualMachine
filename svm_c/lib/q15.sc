// Q15 fixed-point helpers.
// Q15 signed values are stored in u16 using two's-complement.
// 0x7fff ~= +1.0, 0x8000 = -1.0.

u16 q15_abs(u16 x) {
    if (x & 0x8000) {
        if (x == 0x8000) { return 0x7fff; }
        return 0 - x;
    }
    return x;
}

u16 q15_neg(u16 x) {
    return 0 - x;
}

u16 q15_mul(u16 a, u16 b) {
    return mul_q15(a, b);
}

u16 q15_div(u16 a, u16 b) {
    u16 negative;
    u16 ua;
    u16 ub;
    u16 q;
    u16 i;

    if (b == 0) {
        if (a & 0x8000) { return 0x8001; }
        return 0x7fff;
    }

    negative = (a ^ b) & 0x8000;
    ua = q15_abs(a);
    ub = q15_abs(b);

    if (ua >= ub) { q = 0x7fff; }
    else {
        q = 0;
        i = 0;
        while (i < 15) {
            q <<= 1;
            ua <<= 1;
            if (ua >= ub) {
                ua -= ub;
                q |= 1;
            }
            i++;
        }
    }

    if (negative) { return 0 - q; }
    return q;
}
