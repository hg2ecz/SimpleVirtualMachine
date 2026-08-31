// 320x200, 2-bpp graphics helpers for the common SVM video device.
// Pixel values are palette slots 0..3. Palette registers map those slots
// to master-palette indices 0..15.
//
// Geometry primitives take an explicit color. This keeps calls self-contained
// and exercises the general SVM-C parameter ABI (functions may have >4 scalar
// parameters). Convenience *_current wrappers use gfx_current_color.

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

void putpixel(u16 x, u16 y, u16 color) {
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

void putpixel_current(u16 x, u16 y) {
    putpixel(x, y, gfx_current_color);
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

void hline(u16 x0, u16 x1, u16 y, u16 color) {
    u16 t;
    if (y >= 200) return;
    if (x0 > x1) { t = x0; x0 = x1; x1 = t; }
    if (x0 >= 320) return;
    if (x1 >= 320) x1 = 319;
    while (x0 <= x1) {
        putpixel(x0, y, color);
        x0 = x0 + 1;
    }
}

void vline(u16 x, u16 y0, u16 y1, u16 color) {
    u16 t;
    if (x >= 320) return;
    if (y0 > y1) { t = y0; y0 = y1; y1 = t; }
    if (y0 >= 200) return;
    if (y1 >= 200) y1 = 199;
    while (y0 <= y1) {
        putpixel(x, y0, color);
        y0 = y0 + 1;
    }
}

// Integer DDA line for screen coordinates. Endpoints are expected in
// 0..319 / 0..199. The largest product is 319*199, which fits in u16.
void line(u16 x1, u16 x2, u16 y1, u16 y2, u16 color) {
    u16 x0;
    u16 y0;
    u16 xe;
    u16 ye;
    u16 dx;
    u16 dy;
    u16 sx;
    u16 sy;
    u16 i;
    u16 t;
    u16 x;
    u16 y;
    x0 = x1;
    xe = x2;
    y0 = y1;
    ye = y2;
    if (xe >= x0) { dx = xe - x0; sx = 1; }
    else { dx = x0 - xe; sx = 0; }
    if (ye >= y0) { dy = ye - y0; sy = 1; }
    else { dy = y0 - ye; sy = 0; }

    i = 0;
    x = x0;
    y = y0;
    if (dx >= dy) {
        if (dx == 0) { putpixel(x0, y0, color); return; }
        while (i <= dx) {
            t = (i * dy) / dx;
            if (sy) y = y0 + t; else y = y0 - t;
            putpixel(x, y, color);
            if (i == dx) return;
            if (sx) x = x + 1; else x = x - 1;
            i = i + 1;
        }
    } else {
        while (i <= dy) {
            t = (i * dx) / dy;
            if (sx) x = x0 + t; else x = x0 - t;
            putpixel(x, y, color);
            if (i == dy) return;
            if (sy) y = y + 1; else y = y - 1;
            i = i + 1;
        }
    }
}

void line_current(u16 x1, u16 x2, u16 y1, u16 y2) {
    line(x1, x2, y1, y2, gfx_current_color);
}

void rect(u16 x, u16 y, u16 w, u16 h, u16 color) {
    if (w == 0 || h == 0) return;
    hline(x, x + w - 1, y, color);
    if (h > 1) hline(x, x + w - 1, y + h - 1, color);
    if (h > 2) {
        vline(x, y + 1, y + h - 2, color);
        if (w > 1) vline(x + w - 1, y + 1, y + h - 2, color);
    }
}

void fillrect(u16 x, u16 y, u16 w, u16 h, u16 color) {
    u16 yy;
    if (w == 0 || h == 0) return;
    yy = 0;
    while (yy < h) {
        hline(x, x + w - 1, y + yy, color);
        yy = yy + 1;
    }
}

// Integer octant scan. The full circle is expected to fit on screen.
void circle(u16 cx, u16 cy, u16 r, u16 color) {
    u16 x;
    u16 y;
    u16 rr;
    u16 d;
    x = r;
    y = 0;
    rr = r * r;
    while (y <= r) {
        d = x * x + y * y;
        while (d > rr) {
            x = x - 1;
            d = x * x + y * y;
        }
        putpixel(cx + x, cy + y, color); putpixel(cx + y, cy + x, color);
        putpixel(cx - y, cy + x, color); putpixel(cx - x, cy + y, color);
        putpixel(cx - x, cy - y, color); putpixel(cx - y, cy - x, color);
        putpixel(cx + y, cy - x, color); putpixel(cx + x, cy - y, color);
        y = y + 1;
    }
}

void fillcircle(u16 cx, u16 cy, u16 r, u16 color) {
    u16 x;
    u16 y;
    u16 rr;
    u16 d;
    x = r;
    y = 0;
    rr = r * r;
    while (y <= r) {
        d = x * x + y * y;
        while (d > rr) {
            x = x - 1;
            d = x * x + y * y;
        }
        hline(cx - x, cx + x, cy + y, color);
        if (y != 0) hline(cx - x, cx + x, cy - y, color);
        if (x != y) {
            hline(cx - y, cx + y, cy + x, color);
            if (x != 0) hline(cx - y, cx + y, cy - x, color);
        }
        y = y + 1;
    }
}

void rect_current(u16 x, u16 y, u16 w, u16 h) {
    rect(x, y, w, h, gfx_current_color);
}

void fillrect_current(u16 x, u16 y, u16 w, u16 h) {
    fillrect(x, y, w, h, gfx_current_color);
}

void circle_current(u16 cx, u16 cy, u16 r) {
    circle(cx, cy, r, gfx_current_color);
}

void fillcircle_current(u16 cx, u16 cy, u16 r) {
    fillcircle(cx, cy, r, gfx_current_color);
}

void triangle(u16 x1, u16 y1, u16 x2, u16 y2, u16 x3, u16 y3, u16 color) {
    line(x1, x2, y1, y2, color);
    line(x2, x3, y2, y3, color);
    line(x3, x1, y3, y1, color);
}

// Filled triangle by scanline edge interpolation. Coordinates are expected
// to be on-screen; hline() performs final clipping.
void filltriangle(u16 x1, u16 y1, u16 x2, u16 y2, u16 x3, u16 y3, u16 color) {
    u16 tx; u16 ty;
    u16 y; u16 xa; u16 xb;
    u16 dy13; u16 dy12; u16 dy23;
    u16 dx13; u16 dx12; u16 dx23;
    u16 left; u16 right;

    if (y1 > y2) { tx=x1; x1=x2; x2=tx; ty=y1; y1=y2; y2=ty; }
    if (y2 > y3) { tx=x2; x2=x3; x3=tx; ty=y2; y2=y3; y3=ty; }
    if (y1 > y2) { tx=x1; x1=x2; x2=tx; ty=y1; y1=y2; y2=ty; }

    if (y1 == y3) {
        left=x1; right=x1;
        if (x2 < left) left=x2; if (x3 < left) left=x3;
        if (x2 > right) right=x2; if (x3 > right) right=x3;
        hline(left,right,y1,color);
        return;
    }

    dy13 = y3 - y1;
    if (x3 >= x1) dx13 = x3 - x1; else dx13 = x1 - x3;
    dy12 = y2 - y1;
    if (x2 >= x1) dx12 = x2 - x1; else dx12 = x1 - x2;
    dy23 = y3 - y2;
    if (x3 >= x2) dx23 = x3 - x2; else dx23 = x2 - x3;

    y = y1;
    while (y <= y3) {
        if (x3 >= x1) xa = x1 + ((y - y1) * dx13) / dy13;
        else xa = x1 - ((y - y1) * dx13) / dy13;

        if (y <= y2 && dy12 != 0) {
            if (x2 >= x1) xb = x1 + ((y - y1) * dx12) / dy12;
            else xb = x1 - ((y - y1) * dx12) / dy12;
        } else if (dy23 != 0) {
            if (x3 >= x2) xb = x2 + ((y - y2) * dx23) / dy23;
            else xb = x2 - ((y - y2) * dx23) / dy23;
        } else xb = x2;

        if (xa <= xb) hline(xa,xb,y,color); else hline(xb,xa,y,color);
        if (y == y3) return;
        y = y + 1;
    }
}

void triangle_current(u16 x1, u16 y1, u16 x2, u16 y2, u16 x3, u16 y3) {
    triangle(x1,y1,x2,y2,x3,y3,gfx_current_color);
}

void filltriangle_current(u16 x1, u16 y1, u16 x2, u16 y2, u16 x3, u16 y3) {
    filltriangle(x1,y1,x2,y2,x3,y3,gfx_current_color);
}
