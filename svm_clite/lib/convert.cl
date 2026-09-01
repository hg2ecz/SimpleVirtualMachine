fn hex_digit(u8 v) -> u8 {
    v = v & 15;
    if (v < 10) { return 48 + v; }
    return 55 + v;
}

// dst must have room for 5 bytes: 4 hex digits and NUL.
fn u16_to_hex(u8* dst, u16 value) {
    dst[0] = hex_digit(value >> 12);
    dst[1] = hex_digit(value >> 8);
    dst[2] = hex_digit(value >> 4);
    dst[3] = hex_digit(value);
    dst[4] = 0;
}
