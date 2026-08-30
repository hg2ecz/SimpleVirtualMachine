// Hardware-assisted random generator interface.
//
// The common SVM platform exposes a 16-bit xorshift PRNG peripheral at MMIO
// 0xFF26..0xFF2A. It is deliberately a platform device rather than an ISA
// instruction, so all nine CPUs see exactly the same random source.
//
// This is not a cryptographic random source.

u16 hrand() {
    return load16(0xFF26);
}

u16 hrand_max() {
    return 65535;
}

void hrand_seed(u16 seed) {
    store16(0xFF29, seed);
}

u16 hrand_range(u16 limit) {
    u16 x;
    if (limit == 0) { return 0; }
    x = hrand();
    return x % limit;
}
