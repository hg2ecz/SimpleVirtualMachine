fn main() -> u16 {
    u16 i = 0;
    u16 sum = 0;
    while (i < 10) {
        i = i + 1;
        if (i == 3) {
            continue;
        }
        if (i == 8) {
            break;
        }
        sum = sum + i;
    }
    return sum;
}
