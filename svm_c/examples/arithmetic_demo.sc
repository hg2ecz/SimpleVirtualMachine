include "arithmetic.sc";

u16 main() {
    u16 a;
    u16 r;

    a = sin(0x4000);      // approximately +1.0 Q15
    r = isqrt(144);       // 12
    srand(1234);
    r += rand_range(10);  // 0..9
    r += q15_abs(a) >> 12;
    return r;
}
