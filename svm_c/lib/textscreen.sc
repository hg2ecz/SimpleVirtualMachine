// 40x25 character-screen helpers for the common 320x200 framebuffer text device.
// TEXT_X=0xFF02, TEXT_Y=0xFF03, TEXT_FG=0xFF04, TEXT_BG=0xFF05, TEXT_CHAR=0xFF06.
// This is distinct from the VT100 console at 0xFF20.

u16 text_width() { return 40; }
u16 text_height() { return 25; }

void text_set_colors(u16 fg, u16 bg) {
    store8(0xFF04, fg & 3);
    store8(0xFF05, bg & 3);
}

void text_goto(u16 x, u16 y) {
    if (x >= 40) x = 39;
    if (y >= 25) y = 24;
    store8(0xFF02, x);
    store8(0xFF03, y);
}

u16 text_x() { return load8(0xFF02); }
u16 text_y() { return load8(0xFF03); }

void text_home() {
    store8(0xFF02, 0);
    store8(0xFF03, 0);
}

void text_cr() { store8(0xFF02, 0); }

void text_cursor_left() {
    u16 x;
    x = load8(0xFF02);
    if (x != 0) store8(0xFF02, x - 1);
}

void text_cursor_right() {
    u16 x;
    x = load8(0xFF02);
    if (x < 39) store8(0xFF02, x + 1);
}

void text_cursor_up() {
    u16 y;
    y = load8(0xFF03);
    if (y != 0) store8(0xFF03, y - 1);
}

void text_cursor_down() {
    u16 y;
    y = load8(0xFF03);
    if (y < 24) store8(0xFF03, y + 1);
}

void text_lf() { text_cursor_down(); }

void text_newline() {
    text_cr();
    text_lf();
}

// Draw one character at the current cell and advance with 40-column wrapping.
// At the bottom-right corner the cursor remains at (39,24); no scrolling is performed.
void text_putc(u16 ch) {
    u16 x;
    u16 y;
    if (ch == 13) { text_cr(); return; }
    if (ch == 10) { text_newline(); return; }
    if (ch == 8) { text_cursor_left(); return; }
    store8(0xFF06, ch);
    x = load8(0xFF02);
    y = load8(0xFF03);
    if (x < 39) store8(0xFF02, x + 1);
    else if (y < 24) {
        store8(0xFF02, 0);
        store8(0xFF03, y + 1);
    }
}

// Clear the framebuffer to the current text background colour.
// Direct VRAM filling is much cheaper than drawing 1000 space glyphs.
void text_clear() {
    u16 bg;
    u16 packed;
    u16 a;
    bg = load8(0xFF05) & 3;
    packed = bg * 0x55;
    a = 0;
    while (a < 16000) {
        vstore8(a, packed);
        a = a + 1;
    }
    text_home();
}

// Clear the current 8-pixel-high character row and return to column zero.
void text_clear_line() {
    u16 bg;
    u16 packed;
    u16 y;
    u16 row;
    u16 a;
    bg = load8(0xFF05) & 3;
    packed = bg * 0x55;
    y = load8(0xFF03);
    row = 0;
    a = y * 640;
    while (row < 8) {
        u16 n;
        n = 0;
        while (n < 80) {
            vstore8(a + n, packed);
            n = n + 1;
        }
        a = a + 80;
        row = row + 1;
    }
    text_cr();
}

// Erase from the current column through column 39 using the current background.
// Cursor position is preserved.
void text_clear_eol() {
    u16 sx;
    u16 sy;
    u16 x;
    sx = load8(0xFF02);
    sy = load8(0xFF03);
    x = sx;
    while (x < 40) {
        store8(0xFF02, x);
        store8(0xFF03, sy);
        store8(0xFF06, 32);
        x = x + 1;
    }
    store8(0xFF02, sx);
    store8(0xFF03, sy);
}
