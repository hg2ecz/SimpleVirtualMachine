// Bidirectional VT100/RS-232 console example. Press q or Q to exit.
void vt100_clear() {
    putc(27); putc(91); putc(50); putc(74);  // ESC [ 2 J
    putc(27); putc(91); putc(72);             // ESC [ H
}

u16 main() {
    u8 ch;

    vt100_clear();
    puts("SVM VT100 console");
    puts("Characters are echoed; q exits.");

    while (1) {
        ch = getc();
        if (ch == 113 || ch == 81) {
            break;
        }
        putc(ch);
    }

    puts("console closed");
    return 0;
}
