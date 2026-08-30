include "wide_int.sc";

/* IEEE-754 binary32 software arithmetic.
 * f32 values are stored in u32 objects and passed by address.
 * First implementation uses truncation rather than full guard/round/sticky
 * round-to-nearest-even. NaN/Inf/zero encodings are handled explicitly.
 */

u16 f32_sign(u16 a) { return load16(a+2) & 0x8000; }
u16 f32_exp(u16 a) { return (load16(a+2) >> 7) & 0x00ff; }
u16 f32_frac_nonzero(u16 a) { return (load16(a) | (load16(a+2)&0x007f)) != 0; }
u16 f32_is_nan(u16 a) { return (f32_exp(a)==255) && f32_frac_nonzero(a); }
u16 f32_is_inf(u16 a) { return (f32_exp(a)==255) && !f32_frac_nonzero(a); }
u16 f32_is_zero(u16 a) { return (load16(a)==0) && ((load16(a+2)&0x7fff)==0); }

void f32_copy(u16 out, u16 a) { u32_copy(out,a); }
void f32_neg(u16 out, u16 a) { u32_copy(out,a); store16(out+2,load16(out+2)^0x8000); }
void f32_abs(u16 out, u16 a) { u32_copy(out,a); store16(out+2,load16(out+2)&0x7fff); }
void f32_nan(u16 out) { store16(out,0); store16(out+2,0x7fc0); }
void f32_inf(u16 out, u16 sign) { store16(out,0); store16(out+2,(sign&0x8000)|0x7f80); }
void f32_zero(u16 out, u16 sign) { store16(out,0); store16(out+2,sign&0x8000); }

void __f32_unpack_mant(u16 out, u16 a) {
    u16 e;
    e=f32_exp(a);
    store16(out,load16(a));
    if (e==0) store16(out+2,load16(a+2)&0x007f);
    else store16(out+2,(load16(a+2)&0x007f)|0x0080);
}

void __f32_pack(u16 out, u16 sign, u16 e, u16 m) {
    u16 mh;
    if (e>=255) { f32_inf(out,sign); return; }
    mh=load16(m+2);
    if (e==0 || (e==1 && (mh&0x0080)==0)) {
        store16(out,load16(m));
        store16(out+2,(sign&0x8000)|(mh&0x007f));
        return;
    }
    store16(out,load16(m));
    store16(out+2,(sign&0x8000)|(e<<7)|(mh&0x007f));
}

void f32_add(u16 out, u16 a, u16 b) {
    u16 sa;u16 sb;u16 ea;u16 eb;u16 d;u16 e;u16 s;
    u16 alo;u16 ahi;u16 blo;u16 bhi;u16 lo;u16 hi;u16 carry;u16 borrow;
    if (f32_is_nan(a)||f32_is_nan(b)) { f32_nan(out); return; }
    if (f32_is_inf(a)) { if (f32_is_inf(b)&&(f32_sign(a)!=f32_sign(b))) f32_nan(out); else f32_copy(out,a); return; }
    if (f32_is_inf(b)) { f32_copy(out,b); return; }
    sa=f32_sign(a); sb=f32_sign(b); ea=f32_exp(a); eb=f32_exp(b);
    alo=load16(a); ahi=load16(a+2)&0x007f;
    blo=load16(b); bhi=load16(b+2)&0x007f;
    if (ea!=0) ahi=ahi|0x0080; else ea=1;
    if (eb!=0) bhi=bhi|0x0080; else eb=1;

    if (ea<eb) {
        d=eb-ea; e=eb;
        if (d>25) { alo=0; ahi=0; }
        else while(d) {
            alo=(alo>>1)|((ahi&1)<<15);
            ahi=ahi>>1;
            d=d-1;
        }
    } else {
        d=ea-eb; e=ea;
        if (d>25) { blo=0; bhi=0; }
        else while(d) {
            blo=(blo>>1)|((bhi&1)<<15);
            bhi=bhi>>1;
            d=d-1;
        }
    }

    if (sa==sb) {
        lo=alo+blo; carry=lo<alo;
        hi=ahi+bhi+carry; s=sa;
        if (hi&0x0100) {
            lo=(lo>>1)|((hi&1)<<15);
            hi=hi>>1;
            e=e+1;
        }
    } else {
        if (ahi>bhi || (ahi==bhi && alo>=blo)) {
            borrow=alo<blo; lo=alo-blo; hi=ahi-bhi-borrow; s=sa;
        } else {
            borrow=blo<alo; lo=blo-alo; hi=bhi-ahi-borrow; s=sb;
        }
        if ((lo|hi)==0) { f32_zero(out,0); return; }
        while ((hi&0x0080)==0 && e>1) {
            hi=(hi<<1)|(lo>>15);
            lo=lo<<1;
            e=e-1;
        }
    }

    if (e>=255) { f32_inf(out,s); return; }
    if (e==0 || (e==1 && (hi&0x0080)==0)) {
        store16(out,lo);
        store16(out+2,(s&0x8000)|(hi&0x007f));
        return;
    }
    store16(out,lo);
    store16(out+2,(s&0x8000)|(e<<7)|(hi&0x007f));
}

void f32_sub(u16 out, u16 a, u16 b) { u32 nb; f32_neg(&nb,b); f32_add(out,a,&nb); }

void f32_mul(u16 out, u16 a, u16 b) {
    u16 s;u16 ea;u16 eb;u16 e;u16 w1;u16 w2;u16 w3;u16 lo;u16 hi;
    u32 ma;u32 mb;u32 m;u64 p;
    if (f32_is_nan(a)||f32_is_nan(b)) { f32_nan(out); return; }
    if ((f32_is_inf(a)&&f32_is_zero(b))||(f32_is_inf(b)&&f32_is_zero(a))) { f32_nan(out); return; }
    s=f32_sign(a)^f32_sign(b);
    if (f32_is_inf(a)||f32_is_inf(b)) { f32_inf(out,s); return; }
    if (f32_is_zero(a)||f32_is_zero(b)) { f32_zero(out,s); return; }
    ea=f32_exp(a);eb=f32_exp(b);__f32_unpack_mant(&ma,a);__f32_unpack_mant(&mb,b);
    if(ea==0)ea=1;if(eb==0)eb=1;
    u32_mul_u64(&p,&ma,&mb);
    w1=load16(&p+2);w2=load16(&p+4);w3=load16(&p+6);
    e=ea+eb; if(e<127)e=0;else e=e-127;
    if(w2&0x8000) { lo=(w1>>8)|(w2<<8);hi=w2>>8;e=e+1; }
    else { lo=(w1>>7)|(w2<<9);hi=w2>>7; }
    store16(&m,lo);store16(&m+2,hi&0x00ff);
    while((load16(&m+2)&0x0080)==0 && e>1){u32 tmp;u32_shl1(&tmp,&m);u32_copy(&m,&tmp);e=e-1;}
    __f32_pack(out,s,e,&m);
}

void f32_div(u16 out, u16 a, u16 b) {
    u16 s;u16 ea;u16 eb;u16 e;u16 i;u16 bit;
    u32 ma;u32 mb;u32 rem;u32 q;u32 tmp;
    if (f32_is_nan(a)||f32_is_nan(b)||(f32_is_inf(a)&&f32_is_inf(b))||(f32_is_zero(a)&&f32_is_zero(b))) { f32_nan(out); return; }
    s=f32_sign(a)^f32_sign(b);
    if (f32_is_inf(a)||f32_is_zero(b)) { f32_inf(out,s); return; }
    if (f32_is_zero(a)||f32_is_inf(b)) { f32_zero(out,s); return; }
    ea=f32_exp(a);eb=f32_exp(b);__f32_unpack_mant(&ma,a);__f32_unpack_mant(&mb,b);
    if(ea==0)ea=1;if(eb==0)eb=1;
    e=ea+127; if(e<eb)e=0;else e=e-eb;
    if(u32_lt(&ma,&mb)){u32_shl1(&tmp,&ma);u32_copy(&ma,&tmp);if(e)e=e-1;}
    u32_sub(&rem,&ma,&mb);u32_from_u16(&q,1);i=0;
    while(i<23){
        u32_shl1(&tmp,&rem);u32_copy(&rem,&tmp);
        u32_shl1(&tmp,&q);u32_copy(&q,&tmp);
        bit=0;if(u32_ge(&rem,&mb)){u32_sub(&tmp,&rem,&mb);u32_copy(&rem,&tmp);bit=1;}
        if(bit)store16(&q,load16(&q)|1);
        i=i+1;
    }
    __f32_pack(out,s,e,&q);
}

void f32_from_u16(u16 out, u16 x) {
    u16 p; u16 t; u16 e; u16 shift;
    u32 mant; u32 tmp;
    if (x == 0) { f32_zero(out,0); return; }
    p = 0; t = x;
    while (t > 1) { t = t >> 1; p++; }
    e = p + 127;
    u32_from_u16(&mant, x);
    shift = 23 - p;
    while (shift) { u32_shl1(&tmp,&mant); u32_copy(&mant,&tmp); shift--; }
    __f32_pack(out,0,e,&mant);
}

u16 f32_to_u16(u16 a) {
    u16 e;u16 mh;u16 lo;u16 shift;u32 m;u32 tmp;
    if(f32_sign(a))return 0;if(f32_is_nan(a)||f32_is_inf(a))return 0xffff;
    e=f32_exp(a);if(e<127)return 0;__f32_unpack_mant(&m,a);
    if(e>142)return 0xffff;
    shift=150-e;
    while(shift){u32_shr1(&tmp,&m);u32_copy(&m,&tmp);shift=shift-1;}
    lo=load16(&m);mh=load16(&m+2);if(mh)return 0xffff;return lo;
}
