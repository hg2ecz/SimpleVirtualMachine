include "../lib/textscreen.sc";

int main() {
    text_set_colors(3, 0);
    text_clear();
    text_goto(5, 3);
    text_putc(72);
    text_putc(101);
    text_putc(108);
    text_putc(108);
    text_putc(111);
    text_newline();
    text_putc(83);
    text_putc(86);
    text_putc(77);
    return 0;
}
