# ISA ár–érték revízió – Register v3 / MemReg hot-logika

Ez a revízió nem csökkenti a logikai képességeket. Az `AND`, `OR`, `XOR`, `NOT` minden érintett architektúrán megmarad. A változás a rövid/hot opcode-helyek jobb kiosztása.

## Register ISA v3

- `B0..BF`: compact `AND Rd,Rs` `R0..R3` között.
- `XOR Rd,Rs`: továbbra is teljes értékű általános `E8 pp` utasítás.
- `D0..D7`: a hardveres `SUBI Rd,imm16` megmarad.

A korábbi ötlet, hogy `SUBI` legyen `ADDI -imm16` assembler-alias, numerikusan helyes, de flag-szinten nem ekvivalens: a `SUBI` carry bitje no-borrow jelentésű, míg az `ADDI` carry az összeadás túlcsordulását jelzi. Mivel a `C` flag assemblyből megfigyelhető, ezt az egyszerűsítést nem alkalmazzuk.

Az `AND` kapja a compact logikai helyet, mert a 32 bites integer- és soft-float könyvtárakban a maszkolás (`0x8000`, `0x7fff`, exponent/mantissa maszkok) gyakoribb az XOR-nál. A C Register backend konstans maszkolásnál `MOVI R1,imm16` + compact `AND R0,R1` sorozatot használ; ehhez nem szükséges külön `ANDI` hardveropcode.

## MemReg hot kódolás

- `E0..EF`: hot `AND f,W`.
- `F0..FF`: hot `AND f,F`.
- `XOR f,W/F`: normál ALU-kódolással továbbra is elérhető.

A hot file tartomány továbbra is `0x00..0x0F`; a compiler scratch `0x0E..0x0F`, ezért a gyakori maszkolás közvetlenül profitál a rövid formából.

## Kompatibilitás

Ez bináris ISA-változás a Register `B0..BF` és a MemReg `E0..FF` tartományában. A `D0..D7` Register `SUBI` kódolás változatlan marad. A forrás-assembly logikai képességkészlete változatlan: az AND és XOR mnemonic egyaránt megmarad.

## Load/Store carry-helyességi korrekció

A Load/Store `SUBI` korábban `ADDI -imm` alias volt. Ez numerikusan helyes, de a `C` flag jelentését nem őrzi meg. A hosszú-immediate major `9`, function `3` korábban szabad volt, ezért most `SUBI16 Rd,imm16` dekódot kap. Ez nem új datapath: ugyanazt a kivonót használja, mint a regiszter-regiszter `SUB`, csak helyes flag-szemantikát biztosít.
