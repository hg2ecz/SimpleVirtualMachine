u16 counter = 3;
u8 buffer[16];

fn main() -> u16 {
    buffer[0] = 10;
    counter = counter + buffer[0];
    return counter;
}
