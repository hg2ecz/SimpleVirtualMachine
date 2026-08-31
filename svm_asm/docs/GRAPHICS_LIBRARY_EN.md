# Assembly graphics library

Every assembly target provides a native implementation for the common 320x200, 2-bpp SVM framebuffer:

`svm_asm/lib/<arch>/graphics.asm`

The whole file may be included safely: every routine is a `.proc/.endproc` block and procedure-GC keeps only reachable procedures.

## Basic primitives

Every target exports `gfx_set_color`, `gfx_set_palette`, `putpixel`, `clear`, `hline`, and `vline`. Their fast-call ABI follows the natural data-movement model of the ISA; see the header of the target's `graphics.asm` and its Assembly Programming Manual.

## Higher-level geometry

Every target also exports:

- `line` - arbitrary integer-coordinate line;
- `rect` - rectangle outline;
- `fillrect` - filled rectangle;
- `circle` - circle outline;
- `fillcircle` - filled circle.

Calls that need five or more logical parameters use the same 16-bit **graphics parameter block** on every target instead of forcing an artificial register ABI:

| name | address | meaning |
|---|---:|---|
| `GFX_X0` | `0x00C0` | x / x0 / cx |
| `GFX_Y0` | `0x00C2` | y / y0 / cy |
| `GFX_X1` | `0x00C4` | x1 |
| `GFX_Y1` | `0x00C6` | y1 |
| `GFX_W` | `0x00C8` | width |
| `GFX_H` | `0x00CA` | height |
| `GFX_R` | `0x00CC` | radius |
| `GFX_COLOR` | `0x00CE` | palette slot 0..3 |

`0x00B0..0x00BE` is internal virtual-register scratch on targets whose high-level geometry lowering is memory based; `0x00D0..0x00E7` is geometry-algorithm scratch and `0x00E8..0x00FA` is used by the basic primitives/current-colour state. Therefore including and using `graphics.asm` reserves `0x00B0..0x00FA` for the graphics library.

`line(x1,x2,y1,y2,color)` maps x1/x2/y1/y2 to `GFX_X0/GFX_X1/GFX_Y0/GFX_Y1` and reads `GFX_COLOR`. `rect` and `fillrect` read `GFX_X0/GFX_Y0/GFX_W/GFX_H/GFX_COLOR`. `circle` and `fillcircle` read `GFX_X0/GFX_Y0/GFX_R/GFX_COLOR`.

See `svm_asm/examples/<arch>/graphics_library.asm` for target-specific stores into the common parameter block.

The assembly `line` routine uses integer DDA. `circle` and `fillcircle` use an integer octant scan without floating point or square root. Assembly circle calls require the complete circle to fit inside the framebuffer; the C library performs coordinate checks.

Procedure-GC keeps only the dependency chain that is actually used. For example, `fillrect` retains `hline` and `putpixel` but does not retain `line` or circle code.
