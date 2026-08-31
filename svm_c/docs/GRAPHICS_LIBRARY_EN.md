# SVM-C 2-bpp graphics helper library

Include `lib/graphics.sc`. The common framebuffer is 320x200 at 2 bpp in a separate 16 KiB VRAM.

The primary geometry API takes an explicit palette-slot colour. `line` uses integer DDA for in-screen endpoints; `circle`/`fillcircle` use an integer octant scan and require the full circle to fit in the framebuffer:

- `putpixel(x,y,color)` and `getpixel(x,y)`;
- `clear(color)`;
- `hline(x0,x1,y,color)`, `vline(x,y0,y1,color)`;
- `line(x1,x2,y1,y2,color)`;
- `rect(x,y,w,h,color)`, `fillrect(x,y,w,h,color)`;
- `circle(cx,cy,r,color)`, `fillcircle(cx,cy,r,color)`.

`gfx_set_color`/`gfx_get_color` and the `*_current` wrappers remain convenient when stateful drawing is desirable.

SVM-C no longer has a four-parameter function limit. Scalar parameters have statically allocated callee slots. Register/Accumulator/MemReg-family backends first stage all evaluated arguments on the runtime stack and then populate callee slots before `CALL`; the Stack backend naturally uses its stack ABI. This supports the five-parameter line/rectangle API and future six- or seven-parameter primitives without another ABI change. The ABI remains non-recursive and non-reentrant because parameters and locals use static storage.

## Triangles

The C graphics library also provides:

```c
triangle(x1, y1, x2, y2, x3, y3, color);
filltriangle(x1, y1, x2, y2, x3, y3, color);
```

The filled variant uses scanline filling. The portable algorithm is canonical in C rather than duplicated manually for all nine assembly ISAs.
