include "f32.sc";

/* Bit-exact binary32 smoke test.
 * This exercises the wide-object ABI independently of FFT/reporting.
 */
void bits(u16 out, u16 lo, u16 hi) { store16(out,lo); store16(out+2,hi); }
u16 eqbits(u16 a, u16 lo, u16 hi) { return load16(a)==lo && load16(a+2)==hi; }

u16 main() {
    u32 a; u32 b; u32 c; u32 d; u32 x;
    bits(&a,0x0000,0x3F80); /* +1.0 */
    bits(&b,0x0000,0x3F00); /* +0.5 */

    f32_add(&c,&a,&b);
    if (!eqbits(&c,0x0000,0x3FC0)) { puts("F32 SMOKE"); puts("FAIL ADD"); return 1; }

    f32_mul(&c,&a,&b);
    if (!eqbits(&c,0x0000,0x3F00)) { puts("F32 SMOKE"); puts("FAIL MUL"); return 2; }

    f32_div(&d,&a,&b);
    if (!eqbits(&d,0x0000,0x4000)) { puts("F32 SMOKE"); puts("FAIL DIV"); return 3; }

    bits(&b,0x0000,0xBF80); /* -1.0 */
    f32_sub(&c,&a,&b);
    if (!eqbits(&c,0x0000,0x4000)) { puts("F32 SMOKE"); puts("FAIL SUB"); return 4; }

    f32_from_u16(&x,10000);
    if (!eqbits(&x,0x4000,0x461C)) { puts("F32 SMOKE"); puts("FAIL FROM_U16"); return 5; }
    if (f32_to_u16(&x)!=10000) { puts("F32 SMOKE"); puts("FAIL TO_U16"); return 6; }

    /* Reporting-scale path: 0.5 * 10000 must convert back to 5000. */
    bits(&a,0x0000,0x3F00);
    f32_from_u16(&b,10000);
    f32_mul(&c,&a,&b);
    if (f32_to_u16(&c)!=5000) { puts("F32 SMOKE"); puts("FAIL SCALE"); return 7; }

    puts("F32 SMOKE"); puts("OK"); return 0;
}
