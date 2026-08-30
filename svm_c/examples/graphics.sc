include "lib/graphics.sc";

u16 main() {
    gfx_set_palette(0, 9, 14, 15);
    clear(0);
    gfx_set_color(1);
    line(0, 0, 319, 199);
    line(319, 0, 0, 199);
    gfx_set_color(2);
    rect(20, 20, 100, 60);
    gfx_set_color(3);
    circle(160, 100, 50);
    return 0;
}
