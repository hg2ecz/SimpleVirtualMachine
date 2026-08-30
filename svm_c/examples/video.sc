// 320x200, 2-bpp VRAM demo.
// Each VRAM byte contains four pixels: bits 7..6 are the leftmost pixel.
// The four pixel values select programmable palette slots 0..3.

void set_palette() {
    // black, bright blue, bright cyan, white from the fixed 16-colour palette
    store8(0xFF0C, 0);
    store8(0xFF0D, 9);
    store8(0xFF0E, 11);
    store8(0xFF0F, 15);
}

u16 main() {
    u16 y;
    u16 xb;
    u16 offset;
    u8 packed;

    set_palette();

    for (y = 0; y < 200; y++) {
        for (xb = 0; xb < 80; xb++) {
            if (xb < 20) {
                packed = 0x00;
            } else if (xb < 40) {
                packed = 0x55;
            } else if (xb < 60) {
                packed = 0xAA;
            } else {
                packed = 0xFF;
            }
            offset = y * 80 + xb;
            vstore8(offset, packed);
        }
    }

    return 0;
}
