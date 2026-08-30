// 320x200, 2-bpp graphics helpers for the common SVM video device.
// Pixel values are palette slots 0..3. Palette registers map those slots
// to master-palette indices 0..15.

u16 gfx_current_color = 3;

void gfx_set_color(u16 color) {
    gfx_current_color = color & 3;
}

u16 gfx_get_color() {
    return gfx_current_color & 3;
}

void gfx_set_palette(u16 p0, u16 p1, u16 p2, u16 p3) {
    store8(0xFF0C, p0 & 15);
    store8(0xFF0D, p1 & 15);
    store8(0xFF0E, p2 & 15);
    store8(0xFF0F, p3 & 15);
}

// Default palette: black, dark gray, light gray, white.
void gfx_default_palette() {
    gfx_set_palette(0, 8, 7, 15);
}

void putpixelc(u16 x, u16 y, u16 color) {
    u16 addr;
    u16 shift;
    u16 mask;
    u16 oldv;
    u16 nv;
    if (x >= 320 || y >= 200) return;
    addr = y * 80 + (x >> 2);
    shift = 6 - ((x & 3) << 1);
    mask = 3 << shift;
    oldv = vload8(addr);
    nv = (oldv & (0x00FF ^ mask)) | ((color & 3) << shift);
    vstore8(addr, nv);
}

void putpixel(u16 x, u16 y) {
    putpixelc(x, y, gfx_current_color);
}

u16 getpixel(u16 x, u16 y) {
    u16 addr;
    u16 shift;
    if (x >= 320 || y >= 200) return 0;
    addr = y * 80 + (x >> 2);
    shift = 6 - ((x & 3) << 1);
    return (vload8(addr) >> shift) & 3;
}

void clear(u16 color) {
    u16 addr;
    u16 packed;
    packed = (color & 3) * 0x55;
    addr = 0;
    while (addr < 16000) {
        vstore8(addr, packed);
        addr = addr + 1;
    }
}

void hline(u16 x0, u16 x1, u16 y) {
    u16 t;
    if (y >= 200) return;
    if (x0 > x1) { t = x0; x0 = x1; x1 = t; }
    if (x0 >= 320) return;
    if (x1 >= 320) x1 = 319;
    while (x0 <= x1) {
        putpixel(x0, y);
        x0 = x0 + 1;
    }
}

void vline(u16 x, u16 y0, u16 y1) {
    u16 t;
    if (x >= 320) return;
    if (y0 > y1) { t = y0; y0 = y1; y1 = t; }
    if (y0 >= 200) return;
    if (y1 >= 200) y1 = 199;
    while (y0 <= y1) {
        putpixel(x, y0);
        y0 = y0 + 1;
    }
}

// Bresenham line. Current colour is selected with gfx_set_color().
void line(i16 x0, i16 y0, i16 x1, i16 y1) {
    i16 dx;
    i16 sx;
    i16 dy;
    i16 sy;
    i16 err;
    i16 e2;
    dx = x1 - x0;
    if (dx < 0) dx = 0 - dx;
    if (x0 < x1) sx = 1; else sx = -1;
    dy = y1 - y0;
    if (dy < 0) dy = 0 - dy;
    dy = 0 - dy;
    if (y0 < y1) sy = 1; else sy = -1;
    err = dx + dy;
    while (1) {
        if (x0 >= 0 && y0 >= 0 && x0 < 320 && y0 < 200) putpixel(x0, y0);
        if (x0 == x1 && y0 == y1) return;
        e2 = err + err;
        if (e2 >= dy) { err = err + dy; x0 = x0 + sx; }
        if (e2 <= dx) { err = err + dx; y0 = y0 + sy; }
    }
}

void rect(u16 x, u16 y, u16 w, u16 h) {
    if (w == 0 || h == 0) return;
    hline(x, x + w - 1, y);
    if (h > 1) hline(x, x + w - 1, y + h - 1);
    if (h > 2) {
        vline(x, y + 1, y + h - 2);
        if (w > 1) vline(x + w - 1, y + 1, y + h - 2);
    }
}

void fillrect(u16 x, u16 y, u16 w, u16 h) {
    u16 yy;
    if (w == 0 || h == 0) return;
    yy = 0;
    while (yy < h) {
        hline(x, x + w - 1, y + yy);
        yy = yy + 1;
    }
}

// Midpoint circle, current colour.
void circle(i16 cx, i16 cy, i16 r) {
    i16 x;
    i16 y;
    i16 d;
    if (r < 0) return;
    x = r;
    y = 0;
    d = 1 - r;
    while (x >= y) {
        putpixel(cx + x, cy + y); putpixel(cx + y, cy + x);
        putpixel(cx - y, cy + x); putpixel(cx - x, cy + y);
        putpixel(cx - x, cy - y); putpixel(cx - y, cy - x);
        putpixel(cx + y, cy - x); putpixel(cx + x, cy - y);
        y = y + 1;
        if (d < 0) d = d + (y << 1) + 1;
        else { x = x - 1; d = d + ((y - x) << 1) + 1; }
    }
}
