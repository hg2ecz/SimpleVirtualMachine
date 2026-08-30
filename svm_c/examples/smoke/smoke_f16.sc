include "f16.sc";

u16 main() {
    u16 a; u16 b; u16 c;
    puts("F16 SMOKE");
    a=0x3c00; b=0x3800;
    if (f16_add(a,b)!=0x3e00) { puts("FAIL ADD"); return 1; }
    if (f16_sub(a,b)!=0x3800) { puts("FAIL SUB"); return 2; }
    if (f16_mul(a,b)!=0x3800) { puts("FAIL MUL"); return 3; }
    if (f16_div(a,b)!=0x4000) { puts("FAIL DIV"); return 4; }
    if (f16_neg(a)!=0xbc00) { puts("FAIL NEG"); return 5; }
    if (f16_abs(0xbc00)!=0x3c00) { puts("FAIL ABS"); return 6; }
    if (!f16_is_zero(0x8000)) { puts("FAIL ZERO"); return 7; }
    if (!f16_is_inf(0x7c00)) { puts("FAIL INF"); return 8; }
    if (!f16_is_nan(0x7e00)) { puts("FAIL NAN"); return 9; }
    c=f16_from_u16(10000);
    if (c!=0x70e2) { puts("FAIL FROM U16"); return 10; }
    if (f16_to_u16(c)!=10000) { puts("FAIL TO U16"); return 11; }
    puts("OK"); return 0;
}
