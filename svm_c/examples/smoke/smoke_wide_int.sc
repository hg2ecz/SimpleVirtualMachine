include "wide_int.sc";

void set32(u16 p, u16 lo, u16 hi) { store16(p,lo); store16(p+2,hi); }
u16 eq32(u16 p, u16 lo, u16 hi) { return load16(p)==lo && load16(p+2)==hi; }
u16 eq64lo(u16 p, u16 w0, u16 w1) { return load16(p)==w0 && load16(p+2)==w1; }
u16 eq64hi(u16 p, u16 w2, u16 w3) { return load16(p+4)==w2 && load16(p+6)==w3; }

u16 main() {
    u32 a; u32 b; u32 c; u32 q; u32 r; i32 sa; i32 sb;
    u64 p; i64 sp;
    puts("WIDE INT SMOKE");

    set32(&a,0xffff,0xffff); set32(&b,1,0);
    u32_add(&c,&a,&b);
    if (!eq32(&c,0,0)) { puts("FAIL U32 ADD CARRY"); return 1; }

    set32(&a,0,0); set32(&b,1,0);
    u32_sub(&c,&a,&b);
    if (!eq32(&c,0xffff,0xffff)) { puts("FAIL U32 SUB BORROW"); return 2; }

    set32(&a,1,1);
    u32_shl1(&c,&a);
    if (!eq32(&c,2,2)) { puts("FAIL U32 SHL1"); return 3; }
    u32_shr1(&c,&a);
    if (!eq32(&c,0x8000,0)) { puts("FAIL U32 SHR1"); return 4; }

    set32(&a,0x86a0,0x0001); /* 100000 */
    set32(&b,300,0);
    u32_divmod(&q,&r,&a,&b);
    if (!eq32(&q,333,0)) { puts("FAIL U32 DIV"); return 5; }
    if (!eq32(&r,100,0)) { puts("FAIL U32 MOD"); return 6; }

    set32(&a,2,1); set32(&b,4,3);
    u32_mul_u64(&p,&a,&b); /* 0x00010002 * 0x00030004 = 0x00000003000a0008 */
    if (!eq64lo(&p,0x0008,0x000a) || !eq64hi(&p,0x0003,0x0000)) { puts("FAIL U32 MUL U64"); return 7; }

    set32(&sa,0xfc18,0xffff); /* -1000 */
    set32(&sb,7,0);
    i32_div(&q,&sa,&sb);
    if (!eq32(&q,0xff72,0xffff)) { puts("FAIL I32 DIV"); return 8; } /* -142 */
    i32_mod(&r,&sa,&sb);
    if (!eq32(&r,0xfffa,0xffff)) { puts("FAIL I32 MOD"); return 9; } /* -6 */
    if (!i32_lt(&sa,&sb)) { puts("FAIL I32 LT"); return 10; }

    set32(&sa,2,0); set32(&sb,0xfffd,0xffff); /* 2 * -3 = -6 */
    i32_mul_i64(&sp,&sa,&sb);
    if (!eq64lo(&sp,0xfffa,0xffff) || !eq64hi(&sp,0xffff,0xffff)) { puts("FAIL I32 MUL I64"); return 11; }

    puts("OK"); return 0;
}
