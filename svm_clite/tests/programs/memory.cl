fn main() -> u16 {
    store8(0x1000, 65);
    store16(0x1002, 0x1234);
    vstore8(0xff00, 66);
    vstore16(0xff02, 0xabcd);
    load8(0x1000);
    vload8(0xff00);
    return load16(0x1002) + vload16(0xff02);
}
