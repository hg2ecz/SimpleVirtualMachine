fn sum(u16* data, u16 count) -> u16 {
    u16 i = 0;
    u16 result = 0;
    while (i < count) {
        result = result + data[i];
        i = i + 1;
    }
    return result;
}

fn main() -> u16 {
    u16 values[4];
    values[0] = 10;
    values[1] = 20;
    values[2] = 30;
    values[3] = 40;
    return sum(&values[0], 4);
}
