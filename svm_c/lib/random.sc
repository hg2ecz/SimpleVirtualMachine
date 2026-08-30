// Small ANSI-C-like pseudo-random number generator.
// rand() returns 0..32767, matching the common minimum ANSI RAND_MAX range.
// The generator is a 16-bit LCG; arithmetic intentionally wraps modulo 65536.

u16 svm_rand_state = 1;

void srand(u16 seed) {
    if (seed == 0) { svm_rand_state = 1; }
    else { svm_rand_state = seed; }
}

u16 rand() {
    svm_rand_state = svm_rand_state * 25173 + 13849;
    return svm_rand_state >> 1;
}

u16 rand_max() {
    return 32767;
}

u16 rand_range(u16 limit) {
    if (limit == 0) { return 0; }
    return rand() % limit;
}
