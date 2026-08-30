/*
 * Compact all-family numeric smoke gate.
 *
 * This intentionally exercises every numeric storage family while selecting
 * only one or two representative operations from each library.  The complete
 * operation-level regression suite is run by run_numeric_smoke.sh, which
 * executes the smaller smoke_*.sc programs separately.  Keeping this program
 * compact makes it suitable for code-dense targets such as Belt/TTA.
 */
include "signed_int.sc";
include "wide_int.sc";
include "trig.sc";
include "f16.sc";
include "f32.sc";
include "arithmetic_int.sc";
include "random.sc";

void cset32(u16 p,u16 lo,u16 hi){store16(p,lo);store16(p+2,hi);}
u16 ceq32(u16 p,u16 lo,u16 hi){return load16(p)==lo && load16(p+2)==hi;}

u16 main(){
    u8 u8v; i8 i8v; i16 i16v;
    u32 a; u32 b; u32 c; u64 w64;
    u16 h;
    puts("COMPACT NUMERIC SMOKE");

    /* bool/u8/i8/u16/i16.  Keep these checks deliberately simple: the
       detailed operation-level cases live in smoke_scalar_int.sc. */
    if ((0xffff + 1) != 0) { puts("FAIL U16"); return 1; }
    u8v=0xa5;
    if (u8v != 0xa5) { puts("FAIL U8"); return 1; }
    i8v=0x80;
    if (i8_sext(i8v) != 0xff80) { puts("FAIL I8"); return 1; }
    i16v=0xff9c;
    if (i16_abs(i16v) != 100) { puts("FAIL I16"); return 1; }
    if (!(7 < 9)) { puts("FAIL BOOL LT"); return 1; }
    if (7 == 9) { puts("FAIL BOOL EQ"); return 1; }

    /* u32/i32 representative carry and signed compare. */
    cset32(&a,0xffff,0xffff);cset32(&b,1,0);u32_add(&c,&a,&b);
    if(!ceq32(&c,0,0)){puts("FAIL U32");return 2;}
    cset32(&a,0xffff,0xffff);cset32(&b,1,0);
    if(!i32_lt(&a,&b)){puts("FAIL I32");return 3;}

    /* u64/i64 storage layout: four little-endian 16-bit words.  Full
       32x32->64 producer arithmetic is covered by smoke_wide_int.sc. */
    store16(&w64,0x1122);store16(&w64+2,0x3344);store16(&w64+4,0x5566);store16(&w64+6,0x7788);
    if(load16(&w64)!=0x1122||load16(&w64+6)!=0x7788){puts("FAIL 64 STORAGE");return 4;}

    /* Q15 + trigonometric approximation. */
    if(q15_mul(0x4000,0x4000)!=0x2000||q15_abs(sin(0x4000)-0x7fff)>4){puts("FAIL Q15");return 5;}

    /* binary16 */
    h=f16_add(0x3c00,0x3800);
    if(h!=0x3e00){puts("FAIL F16");return 6;}

    /* binary32 */
    cset32(&a,0,0x3f80);cset32(&b,0,0x3f00);f32_add(&c,&a,&b);
    if(!ceq32(&c,0,0x3fc0)){puts("FAIL F32");return 7;}

    /* General arithmetic + deterministic PRNG. */
    if(isqrt(65535)!=255){puts("FAIL ARITH");return 8;}
    srand(1);if(rand()!=19511){puts("FAIL RAND");return 9;}

    puts("OK");return 0;
}
