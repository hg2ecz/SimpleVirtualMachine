include "fft_report.sc";

// 256-point radix-2 DIT FFT, Q15/u16 storage.
// Data workspace is explicit high RAM so the 4096-point case is not limited
// by the compiler's deliberately small static-object area.
// real: 0x8000..., imag: 0x8200...

u16 re_addr(u16 i) { return 0x8000 + (i << 1); }
u16 im_addr(u16 i) { return 0x8200 + (i << 1); }

u16 reverse_bits(u16 x, u16 bits) {
    u16 r; u16 i;
    r = 0; i = 0;
    while (i < bits) { r = (r << 1) | (x & 1); x >>= 1; i++; }
    return r;
}

void stage_root(u16 len, u16 wr, u16 wi) {
    if (len == 2) { store16(wr, 0x8001); store16(wi, 0x0000); return; }
    if (len == 4) { store16(wr, 0x0000); store16(wi, 0x8001); return; }
    if (len == 8) { store16(wr, 0x5A82); store16(wi, 0xA57E); return; }
    if (len == 16) { store16(wr, 0x7641); store16(wi, 0xCF05); return; }
    if (len == 32) { store16(wr, 0x7D89); store16(wi, 0xE707); return; }
    if (len == 64) { store16(wr, 0x7F61); store16(wi, 0xF374); return; }
    if (len == 128) { store16(wr, 0x7FD8); store16(wi, 0xF9B8); return; }
    if (len == 256) { store16(wr, 0x7FF5); store16(wi, 0xFCDC); return; }
    store16(wr, 0x7FFF); store16(wi, 0);
}

void swap_complex(u16 a, u16 b) {
    u16 t; u16 aa; u16 bb;
    aa = re_addr(a); bb = re_addr(b);
    t = load16(aa); store16(aa, load16(bb)); store16(bb, t);
    aa = im_addr(a); bb = im_addr(b);
    t = load16(aa); store16(aa, load16(bb)); store16(bb, t);
}

void init_input() {
    u16 i;
    // Square wave: first half +1, second half -1, imag = 0.
    // Q15 cannot represent +1.0 exactly, so use the symmetric pair
    // +0x7FFF and -0x7FFF (0x8001). This keeps the DC component exact.
    for (i = 0; i < 256; i++) {
        if (i < 128) store16(re_addr(i), 0x7FFF);
        else store16(re_addr(i), 0x8001);
        store16(im_addr(i), 0);
    }
}
void bit_reverse() {
    u16 i; u16 j;
    for (i = 0; i < 256; i++) {
        j = reverse_bits(i, 8);
        if (j > i) swap_complex(i, j);
    }
}

void fft256_q15() {
    u16 len; u16 half; u16 base; u16 j;
    u16 wr; u16 wi; u16 rr; u16 ri; u16 nwr; u16 nwi;
    u16 ar; u16 ai; u16 br; u16 bi; u16 tr; u16 ti;
    bit_reverse();
    len = 2;
    while (len <= 256) {
        half = len >> 1;
        stage_root(len, &rr, &ri);
        for (base = 0; base < 256; base += len) {
            wr = 0x7FFF; wi = 0;
            for (j = 0; j < half; j++) {
                ar = load16(re_addr(base + j));
                ai = load16(im_addr(base + j));
                br = load16(re_addr(base + j + half));
                bi = load16(im_addr(base + j + half));

                tr = mul_q15(wr, br) - mul_q15(wi, bi);
                ti = mul_q15(wr, bi) + mul_q15(wi, br);

                store16(re_addr(base + j),        asr1(ar + tr));
                store16(im_addr(base + j),        asr1(ai + ti));
                store16(re_addr(base + j + half), asr1(ar - tr));
                store16(im_addr(base + j + half), asr1(ai - ti));

                nwr = mul_q15(wr, rr) - mul_q15(wi, ri);
                nwi = mul_q15(wr, ri) + mul_q15(wi, rr);
                wr = nwr; wi = nwi;
            }
        }
        len <<= 1;
    }
}


u16 q15_negative(u16 x) { return (x & 0x8000) != 0; }
u16 q15_scaled4(u16 x) {
    u16 a;
    u32 p; u32 d; u32 q;
    if (q15_negative(x)) a = (~x) + 1; else a = x;
    u16_mul_u32(&p, a, 10000);
    u32_from_u16(&d, 32767);
    u32_div(&q, &p, &d);
    return load16(&q);
}
void report_bins() {
    u16 i; u16 r; u16 im; u16 signs;
    fft_report_header();
    i = 0;
    while (i < 6) {
        r = load16(re_addr(i)); im = load16(im_addr(i));
        signs = q15_negative(r) | (q15_negative(im) << 1);
        fft_report_row(i, q15_scaled4(r), q15_scaled4(im), signs);
        i++;
    }
}

u16 verify() {
    // The FFT is scaled by 1/2 at every stage, so this is FFT(x)/N.
    // The requested +1/-1 square wave has exactly zero DC.
    if (load16(re_addr(0)) != 0) return 1;
    if (load16(im_addr(0)) != 0) return 2;
    return 0;
}

u16 main() {
    u16 e;
    u32 is; u32 ie; u32 cs; u32 ce;
    init_input();
    read_instruction_counter(&is); read_cycle_counter(&cs);
    fft256_q15();
    read_instruction_counter(&ie); read_cycle_counter(&ce);
    e = verify();
    puts("FFT256 Q15");
    report_bins();
    fft_report_stats(&is, &ie, &cs, &ce);
    if (e == 0) puts("OK"); else puts("FAIL");
    return e;
}
