# Register-Memory assembly programozási kézikönyv

A gép erőssége, hogy a második ALU operandus közvetlenül jöhet memóriából vagy immediate-ből:

```asm
ADD R0, [R1+4]
AND R0, 0x7FFF
CMP R0, 10
```

Ezért külön `ANDI` vagy `ADDI` hardvercsalád nem szükséges; ezek forrás-szintű aliasok. A memória source címregiszterét ALU művelet közben nem lehet auto-incrementálni. Explicit load/store és VRAM műveletek továbbra is rendelkezésre állnak.
