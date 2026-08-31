include "lib/graphics.sc";

u16 main() {
    gfx_set_palette(0, 9, 12, 15);
    clear(0);

    line(10, 309, 10, 189, 1);
    rect(20, 20, 100, 60, 2);
    fillrect(140, 25, 70, 45, 1);
    circle(80, 135, 35, 3);
    fillcircle(220, 135, 30, 2);

    return 0;
}
