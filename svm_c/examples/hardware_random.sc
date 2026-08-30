include "hrandom.sc";

u16 main() {
    u16 a;
    u16 b;

    // Optional deterministic seed. Omit this call to use the device's reset seed.
    hrand_seed(0x1234);
    a = hrand();
    b = hrand_range(100);
    return a ^ b;
}
