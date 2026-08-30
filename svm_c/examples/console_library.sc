include "../lib/console.sc";

u16 main() {
    puts("console helpers");
    puts("decimal:");
    putu16(12345);
    newline();
    puts("hex:");
    puthex16(0xBEEF);
    newline();
    return 0;
}
