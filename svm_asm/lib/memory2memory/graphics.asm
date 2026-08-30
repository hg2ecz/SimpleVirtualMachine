; memory2memory graphics support for the common 320x200x2-bpp video device.
; Pixel values are slots 0..3 mapped by palette MMIO 0xFF0C..0xFF0F.
; The complete portable geometry implementation (putpixel/line/rect/fillrect/circle)
; is in svm_c/lib/graphics.sc. The Register ISA assembly library is the hand-written
; assembly reference implementation for packed-pixel primitives.
;
; This file reserves scratch byte 0x00E8 for the current drawing slot.
; Direct VRAM VLOAD/VSTORE instructions remain available on this ISA.
