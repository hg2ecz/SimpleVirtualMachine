# Accumulator assembly programming manual


> Current video model: system memory and video memory are separate 16-bit spaces. Ordinary memory instructions never cross into video memory. See `../../../docs/PLATFORM.md` for the authoritative map and the architecture-specific video instruction forms.

## Programming model

The architecture is intentionally minimal:

- `A`: accumulator; arithmetic results and function return values.
- `X`: index/address and secondary arithmetic operand.
- `Y`: low-cost second address/destination pointer; not a general ALU register.
- `SP`: hardware stack pointer for calls and temporary saves.
- `PC`: program counter.
- flags: `Z`, `N`, `C`.

Most arithmetic is `A op X`. This avoids a register file while keeping binary expressions cheap.

## Basic program

```asm
.load 0
.entry start

start:
    LDAI 72
    STA8 0xFF20
    HALT
```

## Expressions

A useful compiler/manual idiom is:

```asm
; compute left + right
; left is already in A
    PUSHA
; compute right into A
    TAX
    POPA
    ADDX
```

## Memory

Static data and MMIO are cheapest with the 3-byte absolute forms:

```asm
    LDA16 0x6000
    INC
    STA16 0x6000
```

Dynamic addresses use X:

```asm
    LDXI 0x8000
    LDAI 0x00FF
    STA8 [X]
```

Linear traversal uses the one-byte post-increment forms:

```asm
copy:
    LDA8 [X+]
```

Post-increment and the matching high-value pre-decrement `[-X]`/`[-Y]` forms support forward and backward linear traversal. General complex indexed addressing remains deliberately omitted.

## Calls

`CALL` and `RET` use the hardware stack. `A` is the return value. SVM-C uses statically allocated parameter/local slots, so it does not require a frame pointer.

## Machine I/O

The accumulator runtime uses the same 64 KiB machine profile as the other CPUs: 320x200x2 bpp video in one separate 16 KiB data-only VRAM, four colour slots selected from a fixed 16-colour master palette, an internal CPU-invisible character ROM, and MMIO beginning at `0xFF00`.


## X/Y memory-copy model
`X` is the source/index pointer and `Y` is the low-cost destination pointer. `Y` is intentionally not a general ALU register. Forward copies use `LDA8 [X+]` / `STA8 [Y+]`; backward copies use `LDA8 [-X]` / `STA8 [-Y]`. Word forms step by two bytes.


## Automatic short branches

The assembler automatically encodes local JMP/CALL/conditional branches as 2-byte signed PC-relative instructions when the target is within range; otherwise it keeps the 3-byte absolute form. No special source mnemonic is required. Current accumulator executables use `SVA\x06`.

## Fast zero page

Use `LDA8Z/LDA16Z/STA8Z/STA16Z` for two-byte accesses to `0x00..0xFF`. SVM-C selects these for fast-page variables automatically.

## Timer / interrupt quick reference

The shared machine provides a 32-bit virtual clock, one 16-bit timer, and timer/VSYNC/keyboard IRQ sources at `0xFF12..0xFF1F`. Configure the vector and source mask while interrupts are disabled, acknowledge handled bits through `IRQ_ACK` (`0xFF14`), then return with `IRET`. See the project-level `../../../docs/PLATFORM.md` for the normative MMIO semantics.
