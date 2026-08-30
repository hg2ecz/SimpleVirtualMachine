/*
 * SVM-C multiword integer arithmetic.
 *
 * u32/i32 objects are 4 bytes, little-endian words: +0 low, +2 high.
 * u64/i64 objects are 8 bytes: +0,+2,+4,+6 from least to most significant.
 * Public 64-bit arithmetic is intentionally NOT provided.  The only public
 * producers of u64/i64 are the 32x32 -> 64 multiplication routines.
 */

void u32_zero(u16 out) {
    store16(out, 0);
    store16(out + 2, 0);
}

void u32_from_u16(u16 out, u16 x) {
    store16(out, x);
    store16(out + 2, 0);
}

void i32_from_i16(u16 out, u16 x) {
    store16(out, x);
    if (x & 0x8000) store16(out + 2, 0xffff);
    else store16(out + 2, 0);
}

void u32_copy(u16 out, u16 a) {
    store16(out, load16(a));
    store16(out + 2, load16(a + 2));
}

u16 u32_is_zero(u16 a) {
    return (load16(a) | load16(a + 2)) == 0;
}

u16 u32_eq(u16 a, u16 b) {
    return (load16(a) == load16(b)) && (load16(a + 2) == load16(b + 2));
}

u16 u32_lt(u16 a, u16 b) {
    u16 ah; u16 bh; u16 al; u16 bl;
    ah = load16(a + 2); bh = load16(b + 2);
    if (ah < bh) return 1;
    if (ah > bh) return 0;
    al = load16(a); bl = load16(b);
    return al < bl;
}

u16 u32_le(u16 a, u16 b) { return u32_lt(a,b) || u32_eq(a,b); }
u16 u32_gt(u16 a, u16 b) { return u32_lt(b,a); }
u16 u32_ge(u16 a, u16 b) { return u32_gt(a,b) || u32_eq(a,b); }

void u32_add(u16 out, u16 a, u16 b) {
    u16 al; u16 ah; u16 bl; u16 bh; u16 lo; u16 hi; u16 carry;
    al=load16(a); ah=load16(a+2); bl=load16(b); bh=load16(b+2);
    lo = al + bl;
    carry = lo < al;
    hi = ah + bh;
    hi = hi + carry;
    store16(out, lo); store16(out+2, hi);
}

void u32_sub(u16 out, u16 a, u16 b) {
    u16 al; u16 ah; u16 bl; u16 bh; u16 lo; u16 hi; u16 borrow;
    al=load16(a); ah=load16(a+2); bl=load16(b); bh=load16(b+2);
    borrow = al < bl;
    lo = al - bl;
    hi = ah - bh;
    hi = hi - borrow;
    store16(out, lo); store16(out+2, hi);
}

void u32_and(u16 out, u16 a, u16 b) {
    store16(out, load16(a) & load16(b));
    store16(out+2, load16(a+2) & load16(b+2));
}
void u32_or(u16 out, u16 a, u16 b) {
    store16(out, load16(a) | load16(b));
    store16(out+2, load16(a+2) | load16(b+2));
}
void u32_xor(u16 out, u16 a, u16 b) {
    store16(out, load16(a) ^ load16(b));
    store16(out+2, load16(a+2) ^ load16(b+2));
}
void u32_not(u16 out, u16 a) {
    store16(out, ~load16(a));
    store16(out+2, ~load16(a+2));
}

void u32_shl1(u16 out, u16 a) {
    u16 lo; u16 hi;
    lo=load16(a); hi=load16(a+2);
    store16(out, lo << 1);
    store16(out+2, (hi << 1) | (lo >> 15));
}

void u32_shr1(u16 out, u16 a) {
    u16 lo; u16 hi;
    lo=load16(a); hi=load16(a+2);
    store16(out, (lo >> 1) | ((hi & 1) << 15));
    store16(out+2, hi >> 1);
}

void u32_neg(u16 out, u16 a) {
    u16 lo; u16 hi;
    lo = ~load16(a) + 1;
    hi = ~load16(a+2);
    if (lo == 0) hi = hi + 1;
    store16(out,lo); store16(out+2,hi);
}

/* Unsigned restoring division. q and r may be separate from a/b. */
void u32_divmod(u16 q, u16 r, u16 a, u16 b) {
    u32 n; u32 rem; u32 quo; u32 tmp;
    u16 i; u16 bit;
    u32_copy(&n,a); u32_zero(&rem); u32_zero(&quo);
    if (u32_is_zero(b)) { u32_zero(q); u32_zero(r); return; }
    i=0;
    while (i < 32) {
        bit = (load16(&n + 2) >> 15) & 1;
        u32_shl1(&n,&n);
        u32_shl1(&rem,&rem);
        store16(&rem, load16(&rem) | bit);
        u32_shl1(&quo,&quo);
        if (u32_ge(&rem,b)) {
            u32_sub(&tmp,&rem,b); u32_copy(&rem,&tmp);
            store16(&quo, load16(&quo) | 1);
        }
        i=i+1;
    }
    u32_copy(q,&quo); u32_copy(r,&rem);
}

void u32_div(u16 out, u16 a, u16 b) { u32 rem; u32_divmod(out,&rem,a,b); }
void u32_mod(u16 out, u16 a, u16 b) { u32 quo; u32_divmod(&quo,out,a,b); }

/* Internal helpers used only to form a 64-bit multiplication result. */
void __u64_zero(u16 out) {
    store16(out,0); store16(out+2,0); store16(out+4,0); store16(out+6,0);
}
void __u64_from_u32(u16 out, u16 a) {
    store16(out,load16(a)); store16(out+2,load16(a+2));
    store16(out+4,0); store16(out+6,0);
}
void __u64_add(u16 out, u16 a, u16 b) {
    u16 i; u16 av; u16 bv; u16 sum; u16 carry; u16 next;
    i=0; carry=0;
    while (i < 8) {
        av=load16(a+i); bv=load16(b+i);
        sum=av+bv; next = sum < av;
        sum=sum+carry;
        if (sum < carry) next=1;
        store16(out+i,sum); carry=next; i=i+2;
    }
}
void __u64_shl1(u16 out, u16 a) {
    u16 w0;u16 w1;u16 w2;u16 w3;
    w0=load16(a);w1=load16(a+2);w2=load16(a+4);w3=load16(a+6);
    store16(out,w0<<1);
    store16(out+2,(w1<<1)|(w0>>15));
    store16(out+4,(w2<<1)|(w1>>15));
    store16(out+6,(w3<<1)|(w2>>15));
}
void __u64_neg(u16 out) {
    u16 i; u16 w; u16 carry;
    i=0; carry=1;
    while (i < 8) {
        w=~load16(out+i);
        if (carry) { w=w+1; if (w != 0) carry=0; }
        store16(out+i,w); i=i+2;
    }
}

void u32_mul_u64(u16 out, u16 a, u16 b) {
    u64 acc; u64 mcand; u64 tmp64; u32 mult; u32 tmp32;
    u16 i;
    __u64_zero(&acc); __u64_from_u32(&mcand,a); u32_copy(&mult,b);
    i=0;
    while (i < 32) {
        if (load16(&mult) & 1) { __u64_add(&tmp64,&acc,&mcand); __u64_copy(&acc,&tmp64); }
        __u64_shl1(&tmp64,&mcand); __u64_copy(&mcand,&tmp64);
        u32_shr1(&tmp32,&mult); u32_copy(&mult,&tmp32);
        i=i+1;
    }
    __u64_copy(out,&acc);
}

void __u64_copy(u16 out, u16 a) {
    store16(out,load16(a)); store16(out+2,load16(a+2));
    store16(out+4,load16(a+4)); store16(out+6,load16(a+6));
}

u16 i32_negative(u16 a) { return (load16(a+2) & 0x8000) != 0; }
void i32_abs_u32(u16 out, u16 a) { if (i32_negative(a)) u32_neg(out,a); else u32_copy(out,a); }
void i32_add(u16 out, u16 a, u16 b) { u32_add(out,a,b); }
void i32_sub(u16 out, u16 a, u16 b) { u32_sub(out,a,b); }

u16 i32_lt(u16 a, u16 b) {
    u16 sa; u16 sb;
    sa=i32_negative(a); sb=i32_negative(b);
    if (sa != sb) return sa;
    if (sa) return u32_lt(b,a);
    return u32_lt(a,b);
}

void i32_divmod(u16 q, u16 r, u16 a, u16 b) {
    u32 aa; u32 bb; u32 uq; u32 ur;
    u16 sa; u16 sb;
    sa=i32_negative(a); sb=i32_negative(b);
    i32_abs_u32(&aa,a); i32_abs_u32(&bb,b);
    u32_divmod(&uq,&ur,&aa,&bb);
    if (sa != sb) u32_neg(q,&uq); else u32_copy(q,&uq);
    if (sa) u32_neg(r,&ur); else u32_copy(r,&ur);
}
void i32_div(u16 out, u16 a, u16 b) { i32 rem; i32_divmod(out,&rem,a,b); }
void i32_mod(u16 out, u16 a, u16 b) { i32 quo; i32_divmod(&quo,out,a,b); }

void i32_mul_i64(u16 out, u16 a, u16 b) {
    u32 aa; u32 bb;
    u16 neg;
    neg=i32_negative(a) ^ i32_negative(b);
    i32_abs_u32(&aa,a); i32_abs_u32(&bb,b);
    u32_mul_u64(out,&aa,&bb);
    if (neg) __u64_neg(out);
}

/* Exact 16x16 -> 32 unsigned product, useful to soft-float code. */
void u16_mul_u32(u16 out, u16 a, u16 b) {
    u16 a0;u16 a1;u16 b0;u16 b1;u16 p0;u16 p1;u16 p2;u16 p3;
    u16 lo;u16 hi;u16 mid;u16 carry;
    a0=a&0xff; a1=a>>8; b0=b&0xff; b1=b>>8;
    p0=a0*b0; p1=a0*b1; p2=a1*b0; p3=a1*b1;
    mid=(p0>>8)+(p1&0xff)+(p2&0xff);
    lo=(p0&0xff)|(mid<<8);
    carry=mid>>8;
    hi=p3+(p1>>8)+(p2>>8)+carry;
    store16(out,lo); store16(out+2,hi);
}
