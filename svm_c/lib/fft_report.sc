/* Small reporting helpers for the FFT examples.
 * No general printf is provided. Values are printed as signed decimal values
 * scaled by 10000, which gives a printf-like %16.4f presentation.
 *
 * Reporting deliberately uses only 16-bit operations. The measured FFT may
 * use wide integers/soft-float, but formatting must not depend on u32 division.
 */

void print_nl() { putc(13); putc(10); }

u16 __digits_u16(u16 v) {
    if (v >= 10000) return 5;
    if (v >= 1000) return 4;
    if (v >= 100) return 3;
    if (v >= 10) return 2;
    return 1;
}

void print_u16_dec(u16 v) {
    u16 d; u16 q; u16 started;
    d = 10000; started = 0;
    while (d != 0) {
        q = v / d;
        if (q != 0 || started || d == 1) { putc(48 + q); started = 1; }
        v = v % d; d = d / 10;
    }
}

void print_u16_width(u16 v, u16 width) {
    u16 n; n = __digits_u16(v);
    while (width > n) { putc(32); width--; }
    print_u16_dec(v);
}

void __print_frac4(u16 frac) {
    putc(48 + ((frac / 1000) % 10));
    putc(48 + ((frac / 100) % 10));
    putc(48 + ((frac / 10) % 10));
    putc(48 + (frac % 10));
}

void print_fixed4_scaled(u16 negative, u16 scaled_abs, u16 width) {
    u16 whole; u16 frac; u16 n;
    whole = scaled_abs / 10000; frac = scaled_abs % 10000;
    n = __digits_u16(whole) + 5;
    if (negative && scaled_abs != 0) n++;
    while (width > n) { putc(32); width--; }
    if (negative && scaled_abs != 0) putc(45);
    print_u16_dec(whole); putc(46); __print_frac4(frac);
}

/* Exact unsigned 16x16 -> 32 as two words. */
void __mul16_words(u16 a, u16 b, u16 plo, u16 phi) {
    u16 a0; u16 a1; u16 b0; u16 b1;
    u16 p0; u16 p1; u16 p2; u16 p3; u16 mid;
    a0=a&0xff; a1=a>>8; b0=b&0xff; b1=b>>8;
    p0=a0*b0; p1=a0*b1; p2=a1*b0; p3=a1*b1;
    mid=(p0>>8)+(p1&0xff)+(p2&0xff);
    store16(plo,(p0&0xff)|(mid<<8));
    store16(phi,p3+(p1>>8)+(p2>>8)+(mid>>8));
}

u16 __ge32(u16 ah,u16 al,u16 bh,u16 bl) {
    if (ah > bh) return 1;
    if (ah < bh) return 0;
    return al >= bl;
}

/* sqrt(ar*ar + ai*ai), with ar/ai scaled by 10000. */
u16 fft_abs_scaled4(u16 ar, u16 ai) {
    u16 p1l;u16 p1h;u16 p2l;u16 p2h;u16 sl;u16 sh;u16 carry;
    u16 ql;u16 qh;u16 lo;u16 hi;u16 mid;
    __mul16_words(ar,ar,&p1l,&p1h); __mul16_words(ai,ai,&p2l,&p2h);
    sl=p1l+p2l; carry=sl<p1l; sh=p1h+p2h+carry;
    lo=0; hi=14143;
    while (lo < hi) {
        mid=lo+((hi-lo+1)>>1); __mul16_words(mid,mid,&ql,&qh);
        if (__ge32(sh,sl,qh,ql)) lo=mid; else hi=mid-1;
    }
    return lo;
}

void fft_report_header() { puts("bin        real             imag           absval"); }

void fft_report_row(u16 bin, u16 rscaled, u16 iscaled, u16 signbits) {
    u16 mag; u16 rneg; u16 ineg;
    rneg=signbits&1; ineg=(signbits>>1)&1; mag=fft_abs_scaled4(rscaled,iscaled);
    print_u16_width(bin,3); putc(32);
    print_fixed4_scaled(rneg,rscaled,16); putc(32);
    print_fixed4_scaled(ineg,iscaled,16); putc(32);
    print_fixed4_scaled(0,mag,16); print_nl();
}

void read_instruction_counter(u16 out) {
    u16 h1;u16 h2;u16 lo;
    h1=instr_hi(); lo=instr_lo(); h2=instr_hi();
    if (h1!=h2) { lo=instr_lo(); h1=h2; }
    store16(out,lo); store16(out+2,h1);
}
void read_cycle_counter(u16 out) {
    u16 h1;u16 h2;u16 lo;
    h1=clock_hi(); lo=clock_lo(); h2=clock_hi();
    if (h1!=h2) { lo=clock_lo(); h1=h2; }
    store16(out,lo); store16(out+2,h1);
}

/* Divide a 32-bit (hi:lo) value by 10 using a 16-bit long-division
 * step for the low word. Returns the remainder and stores the quotient. */
u16 __div32_10(u16 lo,u16 hi,u16 qlo_addr,u16 qhi_addr) {
    u16 qh;u16 ql;u16 rem;u16 i;u16 bit;
    qh=hi/10; rem=hi%10; ql=0; i=0;
    while (i<16) {
        bit=(lo>>(15-i))&1;
        rem=(rem<<1)|bit;
        ql=ql<<1;
        if (rem>=10) { rem=rem-10; ql=ql|1; }
        i++;
    }
    store16(qlo_addr,ql); store16(qhi_addr,qh); return rem;
}

/* Decimal printing without any 32-bit compare/divide helper. Formatting is
 * outside the measured FFT interval, so portability is preferred over speed. */
void print_u32_words(u16 lo,u16 hi) {
    u8 digits[10]; u16 n;u16 ql;u16 qh;u16 rem;
    if (lo==0 && hi==0) { putc(48); return; }
    n=0;
    while (lo!=0 || hi!=0) {
        rem=__div32_10(lo,hi,&ql,&qh);
        store8(&digits+n,rem);
        lo=ql; hi=qh; n++;
    }
    while (n!=0) { n--; putc(48+load8(&digits+n)); }
}

void fft_report_stats(u16 instr_start,u16 instr_end,u16 cycle_start,u16 cycle_end) {
    u16 sl;u16 sh;u16 el;u16 eh;u16 dl;u16 dh;u16 borrow;
    sl=load16(instr_start); sh=load16(instr_start+2);
    el=load16(instr_end); eh=load16(instr_end+2);
    borrow=el<sl; dl=el-sl; dh=eh-sh-borrow;
    puts("instruction_count"); print_u32_words(dl,dh); print_nl();
    sl=load16(cycle_start); sh=load16(cycle_start+2);
    el=load16(cycle_end); eh=load16(cycle_end+2);
    borrow=el<sl; dl=el-sl; dh=eh-sh-borrow;
    puts("time_cycles"); print_u32_words(dl,dh); print_nl();
}
