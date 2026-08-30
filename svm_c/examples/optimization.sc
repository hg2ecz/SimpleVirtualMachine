// Compile with -O0, -O1, -O2 and -Os, then compare --emit asm output.
u16 main() {
    u16 a = 2 + 3;
    u16 b = a + 1;
    u16 unused = 7;

    unused = 9;
    b = b * 2;
    b = b + 0;

    if (1) {
        b += 1;
    }

    if (0) {
        b += 100;
    }

    return b;
}
