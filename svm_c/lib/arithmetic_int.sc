// SVM-C integer arithmetic helpers.
// Signed values use 16-bit two's-complement representation in u16.

u16 abs(u16 x) {
    if (x & 0x8000) { return 0 - x; }
    return x;
}

u16 min(u16 a, u16 b) {
    if (a < b) { return a; }
    return b;
}

u16 max(u16 a, u16 b) {
    if (a > b) { return a; }
    return b;
}

u16 clamp(u16 x, u16 lo, u16 hi) {
    if (x < lo) { return lo; }
    if (x > hi) { return hi; }
    return x;
}

u16 isqrt(u16 x) {
    u16 root;
    u16 bit;
    u16 candidate;

    if (x == 0) { return 0; }
    root = 0;
    bit = 128;
    while (bit != 0) {
        candidate = root + bit;
        if (candidate <= x / candidate) { root = candidate; }
        bit >>= 1;
    }
    return root;
}

u16 powu(u16 base, u16 exponent) {
    u16 result;
    result = 1;
    while (exponent != 0) {
        if (exponent & 1) { result = result * base; }
        exponent >>= 1;
        if (exponent != 0) { base = base * base; }
    }
    return result;
}

u16 gcd(u16 a, u16 b) {
    u16 t;
    while (b != 0) {
        t = a % b;
        a = b;
        b = t;
    }
    return a;
}

u16 lcm(u16 a, u16 b) {
    u16 g;
    if (a == 0 || b == 0) { return 0; }
    g = gcd(a, b);
    return (a / g) * b;
}
