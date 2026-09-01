fn main() -> u16 {
    u16 i = 0;
    u16 sum = 0;

    while (i < 5) {
        if (i != 2) {
            sum = sum + i;
        }
        i = i + 1;
    }

    return sum;
}
