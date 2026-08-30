# Karakteres képernyő segédfüggvények

A `svm_c/lib/textscreen.sc` a közös 320x200x2bpp videóeszköz 8x8 pixeles belső karakter-ROM-jára épülő **40x25 karakteres framebuffer-képernyőt** kezeli. Ez nem azonos a `0xFF20` címen elérhető VT100/RS-232 konzollal.

## MMIO

- `0xFF02` - karakteroszlop (`0..39`)
- `0xFF03` - karaktersor (`0..24`)
- `0xFF04` - előtérszín slot (`0..3`)
- `0xFF05` - háttérszín slot (`0..3`)
- `0xFF06` - karakter kirajzolása az aktuális cellába

A színslotok a `0xFF0C..0xFF0F` palettaregisztereken keresztül a 16 elemű master palettára mutatnak.

## Include

```c
include "lib/textscreen.sc";
```

## API

- `text_width()` - 40;
- `text_height()` - 25;
- `text_goto(x,y)` - kurzorpozíció, a tartományon kívüli értékeket a képernyő szélére korlátozza.
- `text_home()` - `(0,0)` pozíció.
- `text_cr()` - sor eleje, az aktuális sor megtartásával.
- `text_lf()` - egy sorral lejjebb, legfelj a 24. sorig.
- `text_newline()` - sor eleje + egy sorral lejjebb.
- `text_cursor_left/right/up/down()` - határolt kurzormozgatás.
- `text_x()`, `text_y()` - aktuális pozíció lekérdezése.
- `text_set_colors(fg,bg)` - előtér/háttér 0..3 színslot.
- `text_putc(ch)` - karakter kiírás és automatikus kurzor-előrelépés. CR/LF/backspace vezérlést is kezel. Nincs automatikus scroll.
- `text_clear()` - teljes framebuffer törlése az aktuális háttérszínre és `home`.
- `text_clear_line()` - aktuális karakteres sor törlése és sor elejére állás.
- `text_clear_eol()` - törlés az aktuális oszloptól a sor végéig, a kurzor megtartásával.

A `text_clear()` közvetlen VRAM-töltést használ, ezért lényegesen olcsóbb, mint 1000 szóköz-karakter kirajzolása.

## Példa

Lásd: `svm_c/examples/textscreen.sc`.
