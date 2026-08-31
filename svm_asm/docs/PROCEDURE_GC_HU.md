# Eljárásszintű kódeltávolítás

Az `svm-asm` a standard könyvtárak teljes include-olhatóságát úgy támogatja, hogy a
nem használt eljárásokat még a cél-ISA assemblerének futása előtt eltávolítja.

## Feldolgozási sorrend

1. `.include` fájlok kifejtése, beleértve a target saját `lib/<arch>/` könyvtárát.
2. `.equ` szimbolikus konstansok kifejtése.
3. `.proc` blokkok feltérképezése és elérhetőségi gráf építése.
4. El nem érhető eljárások eltávolítása.
5. A megmaradt forrás átadása a cél-ISA assemblerének.

## Forrásforma

```asm
.proc routine_name
    ; belső címkék és utasítások
    RET
.endproc
```

A `.proc` blokkok nem ágyazhatók egymásba. Az `.entry` és `.keep` csak eljáráson
kívül használható.

## Elérhetőség

Az elemzés konzervatív: ha egy élő forrásrészben egy deklarált eljárás neve
azonosító tokenként előfordul, az eljárás megtartandó. Emiatt a közvetlen hívások,
tail jumpok és címként átadott eljárások egyaránt biztonságosan működnek.

Az algoritmus a gyökerektől szélességi bejárással követi a hivatkozásokat, így a
közvetett könyvtári függőségek tranzitívan bekerülnek.

## Kapcsolat a C backenddel

A C backendnek nem kell külön könyvtári dead-code eliminációt végeznie. Elég, ha a
runtime helper nevekre közönséges assembly-hivatkozásokat generál. A végső assembler
passz csak a ténylegesen elért helper-eljárásokat tartja meg. Ugyanez a mechanizmus
később függvénypointerekkel és generált ugrótáblákkal is használható, amennyiben a
cél eljárásszimbóluma megjelenik a generált assemblyben.

## Könyvtári konvenció

Az `svm_asm/lib/<arch>/` könyvtárban minden hívható rutin `.proc/.endproc` blokk. A `platform.asm` csak `.equ` konstansokat tartalmaz, ezért ott nincs eljárásblokk. Belső ciklus- és elágazási címkét mindig a tulajdonos `.proc` blokkon belül kell tartani; két eljárás ne osszon meg közös belső `done`/`loop` címkét. Ez elkerüli a véletlen keresztfüggőségeket és egyértelművé teszi az elérhetőségi gráfot.

Belépési pont nélküli, közvetlenül is összeállítható könyvtári példa `.keep` direktívával jelölheti a demonstráció miatt megtartandó rutinokat. Normál programban erre nincs szükség: a hívás vagy más szimbolikus hivatkozás automatikusan élővé teszi a cél eljárást.
