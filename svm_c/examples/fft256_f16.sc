include "f16.sc";
include "fft_report.sc";

// 256-point radix-2 DIT FFT using IEEE-754 binary16 bit patterns.
// f16 is stored in one u16 word; arithmetic is entirely software.
// real: 0x8000..., imag: 0x8200...

u16 re_addr(u16 i) { return 0x8000 + (i << 1); }
u16 im_addr(u16 i) { return 0x8200 + (i << 1); }

u16 reverse_bits(u16 x, u16 bits) {
    u16 r; u16 i; r = 0; i = 0;
    while (i < bits) { r = (r << 1) | (x & 1); x >>= 1; i++; }
    return r;
}

void stage_root(u16 len, u16 wr, u16 wi) {
    if (len == 2) { store16(wr, 0xBC00); store16(wi, 0x0000); return; }
    if (len == 4) { store16(wr, 0x0000); store16(wi, 0xBC00); return; }
    if (len == 8) { store16(wr, 0x39A8); store16(wi, 0xB9A8); return; }
    if (len == 16) { store16(wr, 0x3B64); store16(wi, 0xB61F); return; }
    if (len == 32) { store16(wr, 0x3BD9); store16(wi, 0xB23E); return; }
    if (len == 64) { store16(wr, 0x3BF6); store16(wi, 0xAE46); return; }
    if (len == 128) { store16(wr, 0x3BFE); store16(wi, 0xAA48); return; }
    if (len == 256) { store16(wr, 0x3BFF); store16(wi, 0xA648); return; }
    store16(wr, 0x3C00); store16(wi, 0);
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
    // Square wave: xy[i] = +1 for the first half and -1 for the second.
    // binary16: +1.0 = 0x3C00, -1.0 = 0xBC00.
    for (i = 0; i < 256; i++) {
        if (i < 128) store16(re_addr(i), 0x3C00);
        else store16(re_addr(i), 0xBC00);
        store16(im_addr(i), 0);
    }
}
void bit_reverse() {
    u16 i; u16 j;
    for (i = 0; i < 256; i++) { j = reverse_bits(i, 8); if (j > i) swap_complex(i, j); }
}

void fft256_f16() {
    u16 len; u16 half; u16 base; u16 j;
    u16 wr; u16 wi; u16 rr; u16 ri; u16 nwr; u16 nwi;
    u16 ar; u16 ai; u16 br; u16 bi; u16 tr; u16 ti;
    u16 p1; u16 p2; u16 s1; u16 s2;
    bit_reverse(); len = 2;
    while (len <= 256) {
        half = len >> 1; stage_root(len, &rr, &ri);
        for (base = 0; base < 256; base += len) {
            wr = 0x3C00; wi = 0; // 1 + 0i
            for (j = 0; j < half; j++) {
                ar = load16(re_addr(base + j)); ai = load16(im_addr(base + j));
                br = load16(re_addr(base + j + half)); bi = load16(im_addr(base + j + half));

                p1 = f16_mul(wr, br); p2 = f16_mul(wi, bi); tr = f16_sub(p1, p2);
                p1 = f16_mul(wr, bi); p2 = f16_mul(wi, br); ti = f16_add(p1, p2);

                s1 = f16_mul(f16_add(ar, tr), 0x3800);
                s2 = f16_mul(f16_add(ai, ti), 0x3800);
                store16(re_addr(base + j), s1); store16(im_addr(base + j), s2);
                s1 = f16_mul(f16_sub(ar, tr), 0x3800);
                s2 = f16_mul(f16_sub(ai, ti), 0x3800);
                store16(re_addr(base + j + half), s1); store16(im_addr(base + j + half), s2);

                p1 = f16_mul(wr, rr); p2 = f16_mul(wi, ri); nwr = f16_sub(p1, p2);
                p1 = f16_mul(wr, ri); p2 = f16_mul(wi, rr); nwi = f16_add(p1, p2);
                wr = nwr; wi = nwi;
            }
        }
        len <<= 1;
    }
}


u16 f16_scaled4(u16 x) {
    u16 scale; u16 y;
    scale = f16_from_u16(10000);
    y = f16_mul(f16_abs(x), scale);
    return f16_to_u16(y);
}
void report_bins() {
    u16 i; u16 r; u16 im; u16 signs;
    fft_report_header();
    i = 0;
    while (i < 6) {
        r = load16(re_addr(i)); im = load16(im_addr(i));
        signs = ((r & 0x8000) != 0) | (((im & 0x8000) != 0) << 1);
        fft_report_row(i, f16_scaled4(r), f16_scaled4(im), signs);
        i++;
    }
}

u16 verify() {
    // Stage scaling makes the result FFT(x)/N. DC must be exactly zero.
    if (!f16_is_zero(load16(re_addr(0)))) return 1;
    if (!f16_is_zero(load16(im_addr(0)))) return 2;
    return 0;
}

u16 main() {
    u16 e;
    u32 is; u32 ie; u32 cs; u32 ce;
    init_input();
    read_instruction_counter(&is); read_cycle_counter(&cs);
    fft256_f16();
    read_instruction_counter(&ie); read_cycle_counter(&ce);
    e = verify();
    puts("FFT256 F16");
    report_bins();
    fft_report_stats(&is, &ie, &cs, &ce);
    if (e == 0) puts("OK"); else puts("FAIL");
    return e;
}
