include "f32.sc";
include "fft_report.sc";

// 4096-point radix-2 DIT FFT using software IEEE-754 binary32 arithmetic.
// Each f32 is a four-byte u32 object in explicit high RAM.
// real: 0x8000..., imag: 0xC000...

u16 re_addr(u16 i) { return 0x8000 + (i << 2); }
u16 im_addr(u16 i) { return 0xC000 + (i << 2); }

void f32_bits(u16 out, u16 lo, u16 hi) { store16(out, lo); store16(out + 2, hi); }

u16 reverse_bits(u16 x, u16 bits) {
    u16 r; u16 i; r = 0; i = 0;
    while (i < bits) { r = (r << 1) | (x & 1); x >>= 1; i++; }
    return r;
}

void stage_root(u16 len, u16 wr, u16 wi) {
    if (len == 2) { f32_bits(wr, 0x0000, 0xBF80); f32_bits(wi, 0x0000, 0x0000); return; }
    if (len == 4) { f32_bits(wr, 0x0000, 0x0000); f32_bits(wi, 0x0000, 0xBF80); return; }
    if (len == 8) { f32_bits(wr, 0x04F3, 0x3F35); f32_bits(wi, 0x04F3, 0xBF35); return; }
    if (len == 16) { f32_bits(wr, 0x835E, 0x3F6C); f32_bits(wi, 0xEF15, 0xBEC3); return; }
    if (len == 32) { f32_bits(wr, 0x14BE, 0x3F7B); f32_bits(wi, 0xC5C2, 0xBE47); return; }
    if (len == 64) { f32_bits(wr, 0xC46D, 0x3F7E); f32_bits(wi, 0xBD36, 0xBDC8); return; }
    if (len == 128) { f32_bits(wr, 0xB10F, 0x3F7F); f32_bits(wi, 0xFB30, 0xBD48); return; }
    if (len == 256) { f32_bits(wr, 0xEC43, 0x3F7F); f32_bits(wi, 0x0AB0, 0xBCC9); return; }
    if (len == 512) { f32_bits(wr, 0xFB11, 0x3F7F); f32_bits(wi, 0x0E90, 0xBC49); return; }
    if (len == 1024) { f32_bits(wr, 0xFEC4, 0x3F7F); f32_bits(wi, 0x0F88, 0xBBC9); return; }
    if (len == 2048) { f32_bits(wr, 0xFFB1, 0x3F7F); f32_bits(wi, 0x0FC6, 0xBB49); return; }
    if (len == 4096) { f32_bits(wr, 0xFFEC, 0x3F7F); f32_bits(wi, 0x0FD5, 0xBAC9); return; }
    f32_bits(wr, 0x0000, 0x3F80); f32_bits(wi, 0x0000, 0x0000);
}

void swap_f32(u16 a, u16 b) {
    u32 t; f32_copy(&t, a); f32_copy(a, b); f32_copy(b, &t);
}

void swap_complex(u16 a, u16 b) {
    swap_f32(re_addr(a), re_addr(b));
    swap_f32(im_addr(a), im_addr(b));
}

void init_input() {
    u16 i;
    // Square wave: xy[i] = +1 for the first half and -1 for the second.
    // binary32: +1.0 = 0x3F800000, -1.0 = 0xBF800000.
    for (i = 0; i < 4096; i++) {
        if (i < 2048) f32_bits(re_addr(i), 0x0000, 0x3F80);
        else f32_bits(re_addr(i), 0x0000, 0xBF80);
        f32_bits(im_addr(i), 0, 0);
    }
}
void bit_reverse() {
    u16 i; u16 j;
    for (i = 0; i < 4096; i++) { j = reverse_bits(i, 12); if (j > i) swap_complex(i, j); }
}

void fft4096_f32() {
    u16 len; u16 half; u16 base; u16 j;
    u32 wr; u32 wi; u32 rr; u32 ri; u32 nwr; u32 nwi;
    u32 ar; u32 ai; u32 br; u32 bi; u32 tr; u32 ti;
    u32 p1; u32 p2; u32 s1; u32 s2; u32 halfv;
    f32_bits(&halfv, 0x0000, 0x3F00);
    bit_reverse(); len = 2;
    while (len <= 4096) {
        half = len >> 1; stage_root(len, &rr, &ri);
        for (base = 0; base < 4096; base += len) {
            f32_bits(&wr, 0x0000, 0x3F80); f32_bits(&wi, 0, 0);
            for (j = 0; j < half; j++) {
                f32_copy(&ar, re_addr(base + j)); f32_copy(&ai, im_addr(base + j));
                f32_copy(&br, re_addr(base + j + half)); f32_copy(&bi, im_addr(base + j + half));

                f32_mul(&p1, &wr, &br); f32_mul(&p2, &wi, &bi); f32_sub(&tr, &p1, &p2);
                f32_mul(&p1, &wr, &bi); f32_mul(&p2, &wi, &br); f32_add(&ti, &p1, &p2);

                f32_add(&s1, &ar, &tr); f32_mul(&s2, &s1, &halfv); f32_copy(re_addr(base + j), &s2);
                f32_add(&s1, &ai, &ti); f32_mul(&s2, &s1, &halfv); f32_copy(im_addr(base + j), &s2);
                f32_sub(&s1, &ar, &tr); f32_mul(&s2, &s1, &halfv); f32_copy(re_addr(base + j + half), &s2);
                f32_sub(&s1, &ai, &ti); f32_mul(&s2, &s1, &halfv); f32_copy(im_addr(base + j + half), &s2);

                f32_mul(&p1, &wr, &rr); f32_mul(&p2, &wi, &ri); f32_sub(&nwr, &p1, &p2);
                f32_mul(&p1, &wr, &ri); f32_mul(&p2, &wi, &rr); f32_add(&nwi, &p1, &p2);
                f32_copy(&wr, &nwr); f32_copy(&wi, &nwi);
            }
        }
        len <<= 1;
    }
}


u16 f32_scaled4(u16 a) {
    u32 ax; u32 scale; u32 y;
    f32_abs(&ax, a);
    f32_from_u16(&scale, 10000);
    f32_mul(&y, &ax, &scale);
    return f32_to_u16(&y);
}
void report_bins() {
    u16 i; u16 signs;
    fft_report_header();
    i = 0;
    while (i < 6) {
        signs = (f32_sign(re_addr(i)) != 0) | ((f32_sign(im_addr(i)) != 0) << 1);
        fft_report_row(i, f32_scaled4(re_addr(i)), f32_scaled4(im_addr(i)), signs);
        i++;
    }
}

u16 verify() {
    // Stage scaling makes the result FFT(x)/N. DC must be exactly zero.
    if (!f32_is_zero(re_addr(0))) return 1;
    if (!f32_is_zero(im_addr(0))) return 2;
    return 0;
}

u16 main() {
    u16 e;
    u32 is; u32 ie; u32 cs; u32 ce;
    init_input();
    read_instruction_counter(&is); read_cycle_counter(&cs);
    fft4096_f32();
    read_instruction_counter(&ie); read_cycle_counter(&ce);
    e = verify();
    puts("FFT4096 F32");
    report_bins();
    fft_report_stats(&is, &ie, &cs, &ce);
    if (e == 0) puts("OK"); else puts("FAIL");
    return e;
}
