// CRC/checksum helpers for byte streams in SVM memory.

u16 checksum8(u16 data, u16 count) {
    u16 sum;
    sum = 0;
    while (count != 0) {
        sum = (sum + load8(data)) & 0x00ff;
        data = data + 1;
        count = count - 1;
    }
    return sum;
}

u16 checksum16(u16 data, u16 count) {
    u16 sum;
    sum = 0;
    while (count != 0) {
        sum = sum + load8(data);
        data = data + 1;
        count = count - 1;
    }
    return sum;
}

u16 crc8_update(u16 crc, u16 byte) {
    u16 i;
    crc = (crc ^ byte) & 0x00ff;
    i = 0;
    while (i < 8) {
        if (crc & 0x80) crc = ((crc << 1) ^ 0x07) & 0x00ff;
        else crc = (crc << 1) & 0x00ff;
        i = i + 1;
    }
    return crc;
}

// CRC-8/ATM (poly 0x07, init 0x00).
u16 crc8(u16 data, u16 count) {
    u16 crc;
    crc = 0;
    while (count != 0) {
        crc = crc8_update(crc, load8(data));
        data = data + 1;
        count = count - 1;
    }
    return crc;
}

u16 crc16_ccitt_update(u16 crc, u16 byte) {
    u16 i;
    crc = crc ^ ((byte & 0x00ff) << 8);
    i = 0;
    while (i < 8) {
        if (crc & 0x8000) crc = (crc << 1) ^ 0x1021;
        else crc = crc << 1;
        i = i + 1;
    }
    return crc;
}

// CRC-16/CCITT-FALSE (poly 0x1021, init 0xffff, no reflection, xorout 0).
u16 crc16_ccitt(u16 data, u16 count) {
    u16 crc;
    crc = 0xffff;
    while (count != 0) {
        crc = crc16_ccitt_update(crc, load8(data));
        data = data + 1;
        count = count - 1;
    }
    return crc;
}
