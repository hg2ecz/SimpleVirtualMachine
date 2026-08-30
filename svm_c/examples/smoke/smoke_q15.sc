include "trig.sc";

u16 main() {
    puts("Q15 SMOKE");
    if (q15_abs(0x8000) != 0x7fff) { puts("FAIL ABS SAT"); return 1; }
    if (q15_neg(0x4000) != 0xc000) { puts("FAIL NEG"); return 2; }
    if (q15_mul(0x4000,0x4000) != 0x2000) { puts("FAIL MUL"); return 3; }
    if (q15_mul(0xc000,0x4000) != 0xe000) { puts("FAIL MUL SIGN"); return 4; }
    if (q15_div(0x2000,0x4000) != 0x4000) { puts("FAIL DIV"); return 5; }
    if (sin(0x0000) != 0x0000) { puts("FAIL SIN 0"); return 6; }
    if (q15_abs(sin(0x4000) - 0x7fff) > 4) { puts("FAIL SIN 90"); return 7; }
    if (sin(0x8000) != 0x0000) { puts("FAIL SIN 180"); return 8; }
    if (q15_abs(cos(0x0000) - 0x7fff) > 4) { puts("FAIL COS 0"); return 9; }
    puts("OK"); return 0;
}
