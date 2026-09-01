fn first(u8* data) -> u8 {
    return data[0];
}

fn main() -> u16 {
    u8 bytes[8];
    bytes[0] = 65;
    bytes[1] = 66;
    return first(&bytes[0]);
}
