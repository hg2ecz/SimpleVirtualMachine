# C-first algoritmuskönyvtár és az assembly szerepe

A projekt alkalmazási algoritmusainak kanonikus implementációja az SVM-C `svm_c/lib/` könyvtárában található. Memória-, string-, CRC-, bitkezelő-, konverziós, ring-buffer és hasonló algoritmusokat nem tartunk fenn kilenc, kézzel szinkronizálandó ISA-specifikus assembly változatban.

Az assembly könyvtár feladata elsősorban:

- MMIO és hardverközeli primitívek;
- target-specifikus ABI helper;
- olyan hot-path optimalizáció, amely mérés alapján indokolt;
- a gépi architektúra és az assembler demonstrációja.

A `lib/register/algorithms_demo.asm` egy kis include-olható referencia arra, hogyan lehet kézi helper rutint hozzáadni. Nem teljes párhuzamos standard library.

Ha egy C könyvtári rutin később mérhető szűk keresztmetszet, célszerű először C-ben megtartani a referencia szemantikát, majd az érintett target backendjében intrinsic/assembly helper gyorsítást adni ugyanahhoz a művelethez.
