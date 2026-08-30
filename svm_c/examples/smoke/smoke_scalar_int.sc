include "signed_int.sc";

u16 main() {
    u8 b8; i8 s8; i16 s16;
    puts("SCALAR INT SMOKE");

    if ((0xffff + 1) != 0) { puts("FAIL U16 WRAP ADD"); return 1; }
    if ((0 - 1) != 0xffff) { puts("FAIL U16 WRAP SUB"); return 2; }
    if ((123 * 17) != 2091) { puts("FAIL U16 MUL"); return 3; }
    if ((1000 / 7) != 142) { puts("FAIL U16 DIV"); return 4; }
    if ((1000 % 7) != 6) { puts("FAIL U16 MOD"); return 5; }
    if ((0x8001 >> 1) != 0x4000) { puts("FAIL U16 SHR"); return 6; }
    if ((0x4001 << 1) != 0x8002) { puts("FAIL U16 SHL"); return 7; }
    if ((0x55aa & 0x0ff0) != 0x05a0) { puts("FAIL AND"); return 8; }
    if ((0x5500 | 0x00aa) != 0x55aa) { puts("FAIL OR"); return 9; }
    if ((0x55aa ^ 0xffff) != 0xaa55) { puts("FAIL XOR"); return 10; }

    b8 = 0xff; b8 = b8 + 2;
    if (b8 != 1) { puts("FAIL U8 TRUNC"); return 11; }
    s8 = 0x80;
    if (i8_sext(s8) != 0xff80) { puts("FAIL I8 SEXT"); return 12; }

    s16 = 0xff9c; /* -100 */
    if (!i16_negative(s16)) { puts("FAIL I16 SIGN"); return 13; }
    if (i16_abs(s16) != 100) { puts("FAIL I16 ABS"); return 14; }
    if (i16_div(0xff9c,7) != 0xfff2) { puts("FAIL I16 DIV"); return 15; } /* -14 */
    if (i16_mod(0xff9c,7) != 0xfffe) { puts("FAIL I16 MOD"); return 16; } /* -2 */
    if (!i16_lt(0xffff,1)) { puts("FAIL I16 LT"); return 17; }
    if (i16_asr1(0x8000) != 0xc000) { puts("FAIL I16 ASR"); return 18; }

    puts("OK"); return 0;
}
