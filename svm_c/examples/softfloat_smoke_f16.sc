include "f16.sc";

/* Small soft-float/backend smoke test.
 * Expected: prints F16 SMOKE then OK and exits with 0.
 */
u16 main() {
    u16 a; u16 b; u16 c; u16 d;
    a = 0x3C00; /* +1.0 */
    b = 0x3800; /* +0.5 */
    c = f16_add(a,b);
    if (c != 0x3E00) { puts("F16 SMOKE"); puts("FAIL ADD"); return 1; }
    c = f16_mul(a,b);
    if (c != 0x3800) { puts("F16 SMOKE"); puts("FAIL MUL"); return 2; }
    d = f16_div(a,b);
    if (d != 0x4000) { puts("F16 SMOKE"); puts("FAIL DIV"); return 3; }
    puts("F16 SMOKE"); puts("OK"); return 0;
}
