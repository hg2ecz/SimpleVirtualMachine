fn main() -> u16 {
    vstore8(0xff00, 65);
    return vload8(0xff00);
}
