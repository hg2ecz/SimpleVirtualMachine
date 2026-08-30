/* Belt16 C target smoke example.
   The source is ordinary SVM-C; select Belt16 with --target belt. */

u16 add_then_scale(u16 a, u16 b) {
    return (a + b) * 3;
}

u16 main() {
    u16 x;
    x = add_then_scale(10, 20);
    store16(0x6000, x);
    return x;
}
