// CRC-8/ATM: poly 0x07, init 0x00.
fn crc8(u8* data, u16 count) -> u8 {
    u8 crc = 0;
    u16 i = 0;
    while (i < count) {
        crc = crc ^ data[i];
        u16 bit = 0;
        while (bit < 8) {
            if ((crc & 0x80) != 0) {
                crc = (crc << 1) ^ 0x07;
            } else {
                crc = crc << 1;
            }
            bit = bit + 1;
        }
        i = i + 1;
    }
    return crc;
}
