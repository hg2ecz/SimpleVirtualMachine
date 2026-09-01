#!/bin/sh
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
CLITE_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)
WORKSPACE=$(CDPATH= cd -- "$CLITE_DIR/.." && pwd)
TARGET_DIR=${CARGO_TARGET_DIR:-"$WORKSPACE/target"}
CLITE=${SVM_CLITE:-"$TARGET_DIR/debug/svm-clite"}
ASM=${SVM_ASM:-"$TARGET_DIR/debug/svm-asm"}
PROGRAMS="$CLITE_DIR/tests/programs"

if [ ! -x "$CLITE" ]; then
    echo "missing executable: $CLITE" >&2
    echo "run: cargo build -p svm-clite" >&2
    exit 1
fi
if [ ! -x "$ASM" ]; then
    echo "missing executable: $ASM" >&2
    echo "run: cargo build -p svm-asm" >&2
    exit 1
fi


# Architecture guard: every target must lower CLIR directly.
DISPATCH="$CLITE_DIR/src/backend/mod.rs"
if [ -e "$CLITE_DIR/src/backend/canonical.rs" ]; then
    echo "obsolete canonical backend still exists" >&2
    exit 1
fi
if grep -q 'canonical::' "$DISPATCH"; then
    echo "dispatch still references canonical lowering" >&2
    exit 1
fi
for native in register stack accumulator memreg loadstore regmem memory2memory belt tta; do
    if ! grep -q "${native}::lower(clir)" "$DISPATCH"; then
        echo "target does not own a direct CLIR lowerer: $native" >&2
        exit 1
    fi
done

OUT=$(mktemp -d "${TMPDIR:-/tmp}/svm-clite-9targets.XXXXXX")
trap 'rm -rf "$OUT"' EXIT HUP INT TERM

targets='register stack accumulator memreg loadstore regmem memory2memory belt tta'
programs='arithmetic while array_pointer function control memory bool globals'

count=0
for target in $targets; do
    for program in $programs; do
        src="$PROGRAMS/$program.cl"
        asm="$OUT/$target-$program.asm"
        bin="$OUT/$target-$program.bin"
        "$CLITE" --target "$target" "$src" "$asm" >/dev/null
        "$ASM" "$target" "$asm" "$bin" >/dev/null
        test -s "$bin"
        if [ "$target" = stack ]; then
            if grep -q '0x00D0' "$asm"; then
                echo "stack backend still contains virtual-register temp slots: $asm" >&2
                exit 1
            fi
        fi
        if [ "$target" = accumulator ]; then
            if grep -Eq '(^|[[:space:],])R[0-7]([,[:space:]\]]|$)' "$asm"; then
                echo "accumulator backend still contains canonical registers: $asm" >&2
                exit 1
            fi
        fi
        if [ "$target" = memreg ]; then
            if grep -Eq '(^|[[:space:],])R[0-7]([,[:space:]\]]|$)' "$asm"; then
                echo "memreg backend still contains canonical registers: $asm" >&2
                exit 1
            fi
        fi
        count=$((count + 1))
    done

    asm="$OUT/$target-include.asm"
    bin="$OUT/$target-include.bin"
    "$CLITE" --target "$target" "$PROGRAMS/include/main.cl" "$asm" >/dev/null
    "$ASM" "$target" "$asm" "$bin" >/dev/null
    test -s "$bin"
    count=$((count + 1))

done

echo "ok: $count C-Lite -> ASM -> binary integration cases"
