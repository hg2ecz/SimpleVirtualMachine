# FFT256 és FFT4096 SVM-C példák

A projekt két nagyobb radix-2 DIT FFT-méretet tartalmaz három numerikus
reprezentációval:

| Méret | egész/fixpontos | félpontosság | egyszeres pontosság |
| --- | --- | --- | --- |
| 256 | `fft256_u16.sc` | `fft256_f16.sc` | `fft256_f32.sc` |
| 4096 | `fft4096_u16.sc` | `fft4096_f16.sc` | `fft4096_f32.sc` |

Az `u16` változat signed Q15 értékeket tárol `u16` bitekben és a közös
`MULQ15` integer/DSP primitívet használja. Az `f16` és `f32` változat tisztán
szoftveres IEEE-754 könyvtárra épül; egyik CPU sem kap floating-point hardvert.

Mind a hat példa azonos algoritmikai szerkezetet használ:

- in-place radix-2 decimation-in-time FFT;
- bit-reversal permutáció;
- minden stage után 1/2 skálázás;
- stage-enként egy előre számolt komplex egységgyök;
- nincs teljes twiddle-tábla: a következő twiddle iteratív komplex szorzással
  készül;
- azonos, első felében `+1`, második felében `-1` valós négyszögjel-bemenet;
- a DC komponens pontosan nulla, ezt használja az egyszerű önellenőrzés.

## Bemeneti jel

Mind a hat példa ugyanazt a komplex négyszögjelet transzformálja:

```c
for (i = 0; i < SIZE/2; i++) xy[i] = +1;
for (i = SIZE/2; i < SIZE; i++) xy[i] = -1;
```

A képzetes rész minden mintánál nulla. Az `f16` és `f32` változatban a `+1.0` és `-1.0` pontosan reprezentálható. A Q15/`u16` változat a szimmetrikus `+0x7FFF` és `0x8001` (`-0x7FFF`) párt használja, mert a Q15 pozitív `+1.0` értéke nem ábrázolható pontosan. Így a DC komponens mindhárom reprezentációban pontosan nulla.

A radix-2 butterfly minden fokozatban 1/2-del skáláz, ezért a kimenet `FFT(x)/SIZE`, nem a skálázatlan FFT. Ez a Q15 túlcsordulás elkerülése és a három numerikus változat közvetlen összehasonlíthatósága miatt szándékos.

A transzformáció in-place történik: a munkaterület kezdetben a fenti `xy`, a futás végén ugyanott található az FFT-kimenet (`xy_out` szerepe). Ez különösen a `SIZE=4096`, `f32` esetben szükséges, mert külön komplex bemeneti és kimeneti tömb együtt már nem férne el a 64 KiB CPU-címtérben.

## Memória

A fordító lokális/globális objektumai szándékosan kis statikus területet
használnak. A nagy FFT-k ezért explicit CPU RAM címekre dolgoznak.

### FFT256

- `u16` / `f16` real: `0x8000..0x81FF`
- `u16` / `f16` imag: `0x8200..0x83FF`
- `f32` real: `0x8000..0x83FF`
- `f32` imag: `0x8400..0x87FF`

### FFT4096

- `u16` / `f16` real: `0x8000..0x9FFF`
- `u16` / `f16` imag: `0xA000..0xBFFF`
- `f32` real: `0x8000..0xBFFF`
- `f32` imag: `0xC000..0xFFFF`

A `4096 × complex f32` adatterület pontosan 32 KiB. Ez a CPU felső 32 KiB
RAM-ját teljesen kitölti, de nem ütközik a külön VRAM-mal. A példát ezért úgy
kell használni, hogy a programkód és a compiler által kiosztott statikus adatok
az alsó címtartományban maradjanak, ahogy a jelenlegi SVM-C modellben történik.

## Numerikus jelentés

- `u16`: Q15 signed bitminta, stage-skálázással;
- `f16`: binary16 bitminta `u16`-ban, `f16.sc` könyvtárral;
- `f32`: binary32 bitminta `u32` objektumban, cím-alapú `f32.sc` API-val.

Az `f16/f32` példák célja nem hardveres FPU demonstrálása, hanem annak
megmutatása, milyen költséggel építhető lebegőpontos FFT ugyanarra a 16 bites
integer ISA-ra tisztán szoftverből.

## Kimeneti táblázat és futási számlálók

A példák az FFT után az első hat komplex bin értékét írják ki printf-szerű, négy tizedesjegyes formában:

```text
bin        real             imag           absval
  0           0.0000           0.0000           0.0000
  1           ...              ...              ...
...
```

A teljes `printf` nincs beépítve az SVM-C részhalmazba; a példák az `fft_report.sc` kis formázókönyvtárat használják. Az `absval` a négy tizedesre konvertált real/imag komponensekből számított euklideszi magnitúdó.

A táblázat után két 32 bites számláló jelenik meg:

```text
instruction_count
<FFT körüli utasításszám>
time_cycles
<FFT körüli VM ciklusszám>
```

A `time_cycles` a közös determinisztikus SVM cycle model ideje. A platform nem rögzít fizikai órajelet, ezért másodperc helyett VM-ciklust írunk ki. A számlálók mintavétele néhány vendégutasításnyi kis mérési overheadet tartalmaz; FFT256/FFT4096 összehasonlításnál ez elhanyagolható, de nem hardveres benchmark-trigger.

## Soft-float backend smoke teszt

A `svm_c/examples/softfloat_smoke_f16.sc` kis célzott teszt a `f16_add`,
`f16_mul` és `f16_div` útvonalra. Új backend/runtime hibánál ezt érdemes előbb
lefuttatni, mert nem tartalmaz FFT-t vagy riportformázást.

### Szamlalok kiirasa

A 32 bites utasitasszam- es ciklusszam-decimalis formatazasa 16 bites hosszu osztassal tortenik. A formatazas az FFT meresi intervalluman kivul fut, ezert nem befolyasolja a kozolt `instruction_count` es `time_cycles` erteket.

## Binary32 diagnosztika

Az `svm_c/examples/softfloat_smoke_f32.sc` bitpontosan ellenorzi a binary32
wide-object ABI es az `f32_add/sub/mul/div`, `f32_from_u16/to_u16` utvonalat.
Ha az FFT32 hibas, ezt kell eloszor mind a kilenc targeten lefuttatni; igy
elvalaszthato a soft-float alapmuvelet hibaja az FFT/butterfly vagy riport hibajatol.
