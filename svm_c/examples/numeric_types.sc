include "numeric.sc";

u16 main() {
    bool ok;
    i8 small;
    i16 native_signed;
    u32 a;
    u32 b;
    u32 sum;
    u64 product;
    u32 fa;
    u32 fb;
    u32 fc;
    u16 h;

    ok = 1;
    small = 0x7f;
    native_signed = 0 - 123;

    u32_from_u16(&a, 60000);
    u32_from_u16(&b, 1000);
    u32_add(&sum, &a, &b);
    u32_mul_u64(&product, &a, &b);

    h = f16_add(f16_from_u16(3), f16_from_u16(4));

    f32_from_u16(&fa, 3);
    f32_from_u16(&fb, 4);
    f32_add(&fc, &fa, &fb);

    if (ok && small == 0x7f && i16_lt(native_signed, 0)) {
        return f16_to_u16(h) + f32_to_u16(&fc);
    }
    return 0;
}
