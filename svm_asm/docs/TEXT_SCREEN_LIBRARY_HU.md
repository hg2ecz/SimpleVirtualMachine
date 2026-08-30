# Karakteres képernyő assembly segédkönyvtár

Minden ISA-hoz tartozik include-olható fájl:

```text
svm_asm/lib/<arch>/textscreen.asm
```

A könyvtár a 40x25 karakteres framebuffer-szöveges réteget kezeli. Ez külön perifériaút a VT100 konzoltól.

Közös entry pontok:

- `text_goto` - karakterpozíció beállítása;
- `text_set_colors` - 0..3 előtér/háttér slot;
- `text_home` - bal felső sarok;
- `text_cr` - aktuális sor eleje;
- `text_putc` - egy karakter kirajzolása az aktuális cellába;
- `text_clear` - a 40x25 karakteres felület törlése szóközökkel, majd `home`.

A konkrét operandus-ABI az adott `textscreen.asm` fejlécében található. Az assembly `text_putc` szándékosan csak az aktuális cellába rajzol; automatikus kurzor-kezelést a hívó vagy saját wrapper végezhet. A C könyvtár ennél magasabb szintű kurzorkezelést is ad.

MMIO: `TEXT_X=0xFF02`, `TEXT_Y=0xFF03`, `TEXT_FG=0xFF04`, `TEXT_BG=0xFF05`, `TEXT_CHAR=0xFF06`.
