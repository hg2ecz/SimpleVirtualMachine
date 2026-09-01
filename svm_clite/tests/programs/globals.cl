u16 counter = 2;
u8 data[4];

fn main() -> u16 {
    data[0] = 1;
    data[1] = 2;
    return counter + data[0] + data[1];
}
