include "f32.sc";

void bits(u16 out, u16 lo, u16 hi) { store16(out,lo); store16(out+2,hi); }
u16 eqbits(u16 a, u16 lo, u16 hi) { return load16(a)==lo && load16(a+2)==hi; }

u16 main() {
    u32 a; u32 b; u32 c; u32 d; u32 x;
    puts("F32 SMOKE");
    bits(&a,0x0000,0x3f80); bits(&b,0x0000,0x3f00);
    f32_add(&c,&a,&b); if (!eqbits(&c,0x0000,0x3fc0)) { puts("FAIL ADD"); return 1; }
    f32_sub(&c,&a,&b); if (!eqbits(&c,0x0000,0x3f00)) { puts("FAIL SUB"); return 2; }
    f32_mul(&c,&a,&b); if (!eqbits(&c,0x0000,0x3f00)) { puts("FAIL MUL"); return 3; }
    f32_div(&d,&a,&b); if (!eqbits(&d,0x0000,0x4000)) { puts("FAIL DIV"); return 4; }
    f32_neg(&c,&a); if (!eqbits(&c,0x0000,0xbf80)) { puts("FAIL NEG"); return 5; }
    f32_abs(&d,&c); if (!eqbits(&d,0x0000,0x3f80)) { puts("FAIL ABS"); return 6; }
    bits(&c,0,0x7f80); if (!f32_is_inf(&c)) { puts("FAIL INF"); return 7; }
    bits(&c,1,0x7fc0); if (!f32_is_nan(&c)) { puts("FAIL NAN"); return 8; }
    bits(&c,0,0x8000); if (!f32_is_zero(&c)) { puts("FAIL ZERO"); return 9; }
    f32_from_u16(&x,10000); if (!eqbits(&x,0x4000,0x461c)) { puts("FAIL FROM U16"); return 10; }
    if (f32_to_u16(&x)!=10000) { puts("FAIL TO U16"); return 11; }
    puts("OK"); return 0;
}
