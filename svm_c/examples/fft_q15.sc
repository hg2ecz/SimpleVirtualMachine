// 16-point radix-2 Q15 FFT benchmark for all nine SVM targets.
// Uses native SVM-C+ arrays, Q15 DSP builtins, cycle/instruction counters,
// the 320x200 display using the video device internal character ROM and the VT100 console.

u16 real[16];
u16 imag[16];
u16 twre[8];
u16 twim[8];

u16 gx = 0;
u16 gy = 0;
u16 errors = 0;

u16 reverse4(u16 x) {
    return ((x & 1) << 3) | ((x & 2) << 1) | ((x & 4) >> 1) | ((x & 8) >> 3);
}

void swap_bin(u16 a, u16 b) {
    u16 t;
    t = real[a]; real[a] = real[b]; real[b] = t;
    t = imag[a]; imag[a] = imag[b]; imag[b] = t;
}

void init_twiddles() {
    twre[0]=0x7FFF; twim[0]=0x0000;
    twre[1]=0x7642; twim[1]=0xCF04;
    twre[2]=0x5A82; twim[2]=0xA57E;
    twre[3]=0x30FC; twim[3]=0x89BE;
    twre[4]=0x0000; twim[4]=0x8001;
    twre[5]=0xCF04; twim[5]=0x89BE;
    twre[6]=0xA57E; twim[6]=0xA57E;
    twre[7]=0x89BE; twim[7]=0xCF04;
}

void init_input() {
    u16 i;
    for (i = 0; i < 16; i++) {
        real[i] = 0;
        imag[i] = 0;
    }
    real[0] = 0x4000;
}

void bit_reverse() {
    u16 i;
    u16 j;
    for (i = 0; i < 16; i++) {
        j = reverse4(i);
        if (j > i) {
            swap_bin(i, j);
        }
    }
}

void fft16() {
    u16 len;
    u16 half;
    u16 step;
    u16 base;
    u16 j;
    u16 k;
    u16 ar; u16 ai; u16 br; u16 bi;
    u16 wr; u16 wi; u16 tr; u16 ti;

    bit_reverse();
    len = 2;
    while (len <= 16) {
        half = len >> 1;
        step = 16 / len;

        for (base = 0; base < 16; base += len) {
            for (j = 0; j < half; j++) {
                k = j * step;
                wr = twre[k];
                wi = twim[k];
                br = real[base + j + half];
                bi = imag[base + j + half];

                tr = mul_q15(wr, br) - mul_q15(wi, bi);
                ti = mul_q15(wr, bi) + mul_q15(wi, br);
                ar = real[base + j];
                ai = imag[base + j];

                real[base + j]        = asr1(ar + tr);
                imag[base + j]        = asr1(ai + ti);
                real[base + j + half] = asr1(ar - tr);
                imag[base + j + half] = asr1(ai - ti);
            }
        }
        len <<= 1;
    }
}

u16 verify() {
    u16 i;
    errors = 0;
    for (i = 0; i < 16; i++) {
        if (real[i] != 0x0400) { errors++; }
        if (imag[i] != 0) { errors++; }
    }
    return errors;
}

// --- Graphical text output: 40x25 characters, ROM 8x8 font ---
void gfx_at(u16 x, u16 y) {
    gx = x;
    gy = y;
}

void gfx_putc(u16 ch) {
    store8(0xFF02, gx);
    store8(0xFF03, gy);
    store8(0xFF06, ch);
    gx++;
}

void gfx_hex_digit(u16 x) {
    if (x < 10) { gfx_putc(48 + x); }
    else { gfx_putc(55 + x); }
}

void gfx_hex16(u16 x) {
    gfx_hex_digit((x >> 12) & 15);
    gfx_hex_digit((x >> 8) & 15);
    gfx_hex_digit((x >> 4) & 15);
    gfx_hex_digit(x & 15);
}

void gfx_clear() {
    u16 x;
    u16 y;

    store8(0xFF04, 3); // foreground slot
    store8(0xFF05, 0); // background slot
    for (y = 0; y < 25; y++) {
        for (x = 0; x < 40; x++) {
            store8(0xFF02, x);
            store8(0xFF03, y);
            store8(0xFF06, 32);
        }
    }
}

void gfx_title() {
    gfx_at(0,0);
    gfx_putc(70); gfx_putc(70); gfx_putc(84); gfx_putc(32);
    gfx_putc(81); gfx_putc(49); gfx_putc(53); gfx_putc(32);
    gfx_putc(78); gfx_putc(61); gfx_putc(49); gfx_putc(54);
}

void gfx_timing(u16 cycles_hi, u16 cycles_lo, u16 instr_hi_count, u16 instr_lo_count) {
    gfx_at(0,1);
    gfx_putc(67); gfx_putc(61);
    gfx_hex16(cycles_hi); gfx_hex16(cycles_lo);
    gfx_at(0,2);
    gfx_putc(73); gfx_putc(61);
    gfx_hex16(instr_hi_count); gfx_hex16(instr_lo_count);
    gfx_at(0,3);
    gfx_putc(69); gfx_putc(82); gfx_putc(82); gfx_putc(61);
    gfx_hex16(errors);
}

void gfx_bin(u16 row, u16 i) {
    gfx_at(0,row);
    gfx_hex_digit(i);
    gfx_putc(58);
    gfx_hex16(real[i]);
    gfx_putc(32);
    gfx_hex16(imag[i]);
}

void gfx_results(u16 cycles_hi, u16 cycles_lo, u16 instr_hi_count, u16 instr_lo_count) {
    u16 i;

    gfx_clear();
    gfx_title();
    gfx_timing(cycles_hi, cycles_lo, instr_hi_count, instr_lo_count);
    gfx_at(0,4);
    gfx_putc(66); gfx_putc(73); gfx_putc(78); gfx_putc(32);
    gfx_putc(82); gfx_putc(69); gfx_putc(32); gfx_putc(73); gfx_putc(77);

    for (i = 0; i < 16; i++) {
        gfx_bin(5 + i, i);
    }

    gfx_at(0,22);
    gfx_putc(80); gfx_putc(82); gfx_putc(69); gfx_putc(83); gfx_putc(83); gfx_putc(32);
    gfx_putc(75); gfx_putc(69); gfx_putc(89); gfx_putc(32);
    gfx_putc(84); gfx_putc(79); gfx_putc(32);
    gfx_putc(69); gfx_putc(88); gfx_putc(73); gfx_putc(84);
}

// --- VT100 / RS-232 console output ---
void console_crlf() {
    putc(13);
    putc(10);
}

void console_hex_digit(u16 x) {
    if (x < 10) { putc(48 + x); }
    else { putc(55 + x); }
}

void console_hex16(u16 x) {
    console_hex_digit((x >> 12) & 15);
    console_hex_digit((x >> 8) & 15);
    console_hex_digit((x >> 4) & 15);
    console_hex_digit(x & 15);
}

void console_bin(u16 i) {
    console_hex_digit(i);
    putc(58);
    console_hex16(real[i]);
    putc(32);
    console_hex16(imag[i]);
    console_crlf();
}

void console_results(u16 cycles_hi, u16 cycles_lo, u16 instr_hi_count, u16 instr_lo_count) {
    u16 i;

    puts("FFT Q15 N=16");
    putc(67); putc(61); // C=
    console_hex16(cycles_hi); console_hex16(cycles_lo); console_crlf();
    putc(73); putc(61); // I=
    console_hex16(instr_hi_count); console_hex16(instr_lo_count); console_crlf();
    putc(69); putc(82); putc(82); putc(61); // ERR=
    console_hex16(errors); console_crlf();
    puts("BIN RE IM");

    for (i = 0; i < 16; i++) {
        console_bin(i);
    }
}

void wait_key() {
    while (load8(0xFF00) != 0) { }
    while (load8(0xFF00) == 0) { }
    while (load8(0xFF00) != 0) { }
}

u16 main() {
    u16 start_lo;
    u16 start_hi;
    u16 start_hi2;
    u16 end_lo;
    u16 end_hi;
    u16 end_hi2;
    u16 cycles_lo;
    u16 cycles_hi;
    u16 instr_start_lo;
    u16 instr_start_hi;
    u16 instr_start_hi2;
    u16 instr_end_lo;
    u16 instr_end_hi;
    u16 instr_end_hi2;
    u16 instr_count_lo;
    u16 instr_count_hi;
    u16 borrow;

    init_twiddles();
    init_input();

    instr_start_hi = instr_hi();
    instr_start_lo = instr_lo();
    instr_start_hi2 = instr_hi();
    while (instr_start_hi != instr_start_hi2) {
        instr_start_hi = instr_start_hi2;
        instr_start_lo = instr_lo();
        instr_start_hi2 = instr_hi();
    }
    instr_start_hi = instr_start_hi2;

    start_hi = clock_hi();
    start_lo = clock_lo();
    start_hi2 = clock_hi();
    while (start_hi != start_hi2) {
        start_hi = start_hi2;
        start_lo = clock_lo();
        start_hi2 = clock_hi();
    }
    start_hi = start_hi2;

    fft16();

    end_hi = clock_hi();
    end_lo = clock_lo();
    end_hi2 = clock_hi();
    while (end_hi != end_hi2) {
        end_hi = end_hi2;
        end_lo = clock_lo();
        end_hi2 = clock_hi();
    }
    end_hi = end_hi2;

    instr_end_hi = instr_hi();
    instr_end_lo = instr_lo();
    instr_end_hi2 = instr_hi();
    while (instr_end_hi != instr_end_hi2) {
        instr_end_hi = instr_end_hi2;
        instr_end_lo = instr_lo();
        instr_end_hi2 = instr_hi();
    }
    instr_end_hi = instr_end_hi2;

    cycles_lo = end_lo - start_lo;
    borrow = end_lo < start_lo;
    cycles_hi = end_hi - start_hi - borrow;

    instr_count_lo = instr_end_lo - instr_start_lo;
    borrow = instr_end_lo < instr_start_lo;
    instr_count_hi = instr_end_hi - instr_start_hi - borrow;

    errors = verify();
    gfx_results(cycles_hi, cycles_lo, instr_count_hi, instr_count_lo);
    console_results(cycles_hi, cycles_lo, instr_count_hi, instr_count_lo);
    wait_key();

    return errors;
}
