; Stack ISA graphics include marker for the common 320x200x2-bpp video device.
; Pixel values are palette slots 0..3 mapped by MMIO 0xFF0C..0xFF0F.
; The portable complete drawing API (putpixel/line/rect/fillrect/circle) is
; svm_c/lib/graphics.sc and compiles to the Stack target. For hand-written
; assembly, use the native VLOAD8/VSTORE8 words and the packing rule documented
; in svm_asm/docs/GRAPHICS_LIBRARY_HU.md.
; Scratch byte 0x00E8 is reserved by the graphics convention.
