# Memory-register assembly programozási kézikönyv


> A jelenlegi videómodellben a normál rendszerterület és a videómemória két külön 16 bites címtér. A normál memóriautasítás nem lép át a videótérbe. A hiteles kiosztás: `../../../docs/PLATFORM_HU.md`.

A Memory-register CPU PIC ihletésű, költségoptimalizált 16 bites architektúra, a történelmi bankolt memória korlátai nélkül. Az aritmetika központja a W regiszter és a 0. lapos file operandus; a teljes 64 KiB címtérhez két FSR tartozik.

A javasolt felosztás: `0x0000..0x00EF` gyors változók, `0x00F0..0x00FF` fordítói/scratch terület, a programkód pedig `0x0100`-tól.

## Destination flag modell

```asm
MOV16 0x10,W
ADD   0x12,W     ; W = W + file[0x12]
ADD   0x14,F     ; file[0x14] = file[0x14] + W
```

A legelső 16 file-címnél a gyakori műveletek egyetlen opcode-bájtra rövidülnek.

## Indirekt címzés

```asm
FSR0I source
FSR1I destination
LDB0+             ; W = *FSR0++
STB1+             ; *FSR1++ = W
```

Átfedő hátramásolásnál a pointerek a másolandó tartomány vége után indulnak:

```asm
LDB0-
STB1-
```

## Zero page és teljes memória

A direct file címzés csak `0x0000..0x00FF` területet ér el; más címhez FSR-indirekt elérés használatos. Nincs bankregiszter vagy lapváltási állapot.

## Timer / interrupt gyors referencia

A közös platform 32 bites virtuális órát, egy 16 bites timert és timer/VSYNC/billentyűzet IRQ-forrásokat biztosít a `0xFF12..0xFF1F` tartományban. A vektort és forrásmaszkot tiltott interrupt mellett célszerű beállítani; a kezelt forrást az `IRQ_ACK` (`0xFF14`) regiszteren kell nyugtázni, majd `IRET`-tel visszatérni. A normatív MMIO-szemantika: `../../../docs/PLATFORM_HU.md`.


## Utasításkódolás és végrehajtási idő

A hex opcode, utasításhossz és ciklusidő táblázatai: [INSTRUCTION_REFERENCE_HU.md](INSTRUCTION_REFERENCE_HU.md).
