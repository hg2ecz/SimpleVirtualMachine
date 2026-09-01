// One source program intended to compile unchanged for every SVM architecture.
fn checksum(u8* data, u16 count) -> u16 {
    u16 i = 0;
    u16 sum = 0;

    while (i < count) {
        sum = sum + data[i];
        i = i + 1;
    }
    return sum;
}

fn main() -> u16 {
    u8 data[4];
    data[0] = 1;
    data[1] = 2;
    data[2] = 3;
    data[3] = 4;
    return checksum(&data[0], 4);
}
