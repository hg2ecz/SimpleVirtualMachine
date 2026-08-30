include "wide_int.sc";

/* IEEE-754 binary16 software arithmetic on u16 bit patterns.
 * Round mode is truncation toward the retained mantissa in this first version;
 * NaN, infinity, zero and subnormal encodings are recognized.
 */

u16 f16_is_nan(u16 x) { return ((x & 0x7c00) == 0x7c00) && ((x & 0x03ff) != 0); }
u16 f16_is_inf(u16 x) { return (x & 0x7fff) == 0x7c00; }
u16 f16_is_zero(u16 x) { return (x & 0x7fff) == 0; }
u16 f16_neg(u16 x) { return x ^ 0x8000; }
u16 f16_abs(u16 x) { return x & 0x7fff; }

u16 f16_add(u16 a, u16 b) {
    u16 sa;u16 sb;u16 ea;u16 eb;u16 ma;u16 mb;u16 d;u16 m;u16 e;u16 s;
    if (f16_is_nan(a) || f16_is_nan(b)) return 0x7e00;
    if (f16_is_inf(a)) { if (f16_is_inf(b) && ((a^b)&0x8000)) return 0x7e00; return a; }
    if (f16_is_inf(b)) return b;
    sa=a&0x8000; sb=b&0x8000; ea=(a>>10)&31; eb=(b>>10)&31;
    ma=a&0x03ff; mb=b&0x03ff;
    if (ea) ma=ma|0x0400; else ea=1;
    if (eb) mb=mb|0x0400; else eb=1;
    if (ea < eb) {
        d=eb-ea; if (d>11) ma=0; else ma=ma>>d; e=eb;
    } else {
        d=ea-eb; if (d>11) mb=0; else mb=mb>>d; e=ea;
    }
    if (sa==sb) { m=ma+mb; s=sa; }
    else if (ma>=mb) { m=ma-mb; s=sa; }
    else { m=mb-ma; s=sb; }
    if (m==0) return 0;
    if (m&0x0800) { m=m>>1; e=e+1; }
    while ((m&0x0400)==0 && e>1) { m=m<<1; e=e-1; }
    if (e>=31) return s|0x7c00;
    if (e==1 && (m&0x0400)==0) return s|(m&0x03ff);
    return s|(e<<10)|(m&0x03ff);
}

u16 f16_sub(u16 a, u16 b) { return f16_add(a, f16_neg(b)); }

u16 f16_mul(u16 a, u16 b) {
    u16 sa;u16 sb;u16 ea;u16 eb;u16 ma;u16 mb;u16 e;u16 s;u16 lo;u16 hi;u16 m;
    u32 prod;
    if (f16_is_nan(a)||f16_is_nan(b)) return 0x7e00;
    if ((f16_is_inf(a)&&f16_is_zero(b))||(f16_is_inf(b)&&f16_is_zero(a))) return 0x7e00;
    s=(a^b)&0x8000;
    if (f16_is_inf(a)||f16_is_inf(b)) return s|0x7c00;
    if (f16_is_zero(a)||f16_is_zero(b)) return s;
    ea=(a>>10)&31; eb=(b>>10)&31; ma=a&0x03ff; mb=b&0x03ff;
    if (ea) ma=ma|0x0400; else ea=1;
    if (eb) mb=mb|0x0400; else eb=1;
    u16_mul_u32(&prod,ma,mb); lo=load16(&prod); hi=load16(&prod+2);
    e=ea+eb;
    if (e<15) e=0; else e=e-15;
    if (hi&0x0020) { m=(lo>>11)|(hi<<5); e=e+1; }
    else m=(lo>>10)|(hi<<6);
    while ((m&0x0400)==0 && e>1) { m=m<<1; e=e-1; }
    if (e>=31) return s|0x7c00;
    if (e==0 || (e==1 && (m&0x0400)==0)) return s|(m&0x03ff);
    return s|(e<<10)|(m&0x03ff);
}

u16 f16_div(u16 a, u16 b) {
    u16 ea;u16 eb;u16 ma;u16 mb;u16 e;u16 s;u16 q;
    u32 num;u32 den;u32 quo;u32 rem;
    if (f16_is_nan(a)||f16_is_nan(b)|| (f16_is_inf(a)&&f16_is_inf(b)) || (f16_is_zero(a)&&f16_is_zero(b))) return 0x7e00;
    s=(a^b)&0x8000;
    if (f16_is_inf(a)||f16_is_zero(b)) return s|0x7c00;
    if (f16_is_zero(a)||f16_is_inf(b)) return s;
    ea=(a>>10)&31; eb=(b>>10)&31; ma=a&0x03ff; mb=b&0x03ff;
    if (ea) ma=ma|0x0400; else ea=1;
    if (eb) mb=mb|0x0400; else eb=1;
    store16(&num,ma<<10); store16(&num+2,ma>>6); u32_from_u16(&den,mb);
    u32_divmod(&quo,&rem,&num,&den); q=load16(&quo);
    e=ea+15;
    if (e<eb) e=0; else e=e-eb;
    while (q>=0x0800) { q=q>>1; e=e+1; }
    while (q<0x0400 && e>1) { q=q<<1; e=e-1; }
    if (e>=31) return s|0x7c00;
    if (e==0 || (e==1 && q<0x0400)) return s|(q&0x03ff);
    return s|(e<<10)|(q&0x03ff);
}

u16 f16_from_u16(u16 x) {
    u16 p; u16 t; u16 e; u16 m; u16 i;
    if (x == 0) return 0;
    p = 0; t = x;
    while (t > 1) { t = t >> 1; p++; }
    e = p + 15;
    if (e >= 31) return 0x7c00;
    m = x;
    if (p < 10) {
        i = 10 - p;
        while (i) { m = m << 1; i--; }
    } else if (p > 10) {
        i = p - 10;
        while (i) { m = m >> 1; i--; }
    }
    return (e << 10) | (m & 0x03ff);
}

u16 f16_to_u16(u16 x) {
    u16 e; u16 p; u16 m; u16 i;
    if (x & 0x8000) return 0;
    e = (x >> 10) & 31;
    if (e == 0) return 0;
    if (e == 31) return 0xffff;
    if (e < 15) return 0;
    p = e - 15;
    if (p > 15) return 0xffff;
    m = (x & 0x03ff) | 0x0400;
    if (p < 10) {
        i = 10 - p;
        while (i) { m = m >> 1; i--; }
    } else if (p > 10) {
        i = p - 10;
        while (i) { m = m << 1; i--; }
    }
    return m;
}
