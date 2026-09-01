#!/bin/sh
set -eu

if [ "$#" -ne 1 ]; then
    echo "usage: $0 program.cl" >&2
    exit 2
fi

SOURCE=$1
SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
CLITE_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)
WORKSPACE=$(CDPATH= cd -- "$CLITE_DIR/.." && pwd)
TARGET_DIR=${CARGO_TARGET_DIR:-"$WORKSPACE/target"}
CLITE=${SVM_CLITE:-"$TARGET_DIR/debug/svm-clite"}
ASM=${SVM_ASM:-"$TARGET_DIR/debug/svm-asm"}

if [ ! -f "$SOURCE" ]; then
    echo "missing source: $SOURCE" >&2
    exit 1
fi
if [ ! -x "$CLITE" ] || [ ! -x "$ASM" ]; then
    echo "build svm-clite and svm-asm first" >&2
    exit 1
fi

OUT=$(mktemp -d "${TMPDIR:-/tmp}/svm-clite-codegen.XXXXXX")
trap 'rm -rf "$OUT"' EXIT HUP INT TERM

targets='register stack accumulator memreg loadstore regmem memory2memory belt tta'
printf '%-16s %10s %10s\n' target asm_lines bin_bytes
for target in $targets; do
    asm="$OUT/$target.asm"
    bin="$OUT/$target.bin"
    "$CLITE" --target "$target" "$SOURCE" "$asm" >/dev/null
    "$ASM" "$target" "$asm" "$bin" >/dev/null
    lines=$(awk 'BEGIN { n=0 } /^[[:space:]]*[A-Za-z0-9_.]+([[:space:]]|$)/ { n++ } END { print n }' "$asm")
    bytes=$(wc -c < "$bin" | tr -d ' ')
    printf '%-16s %10s %10s\n' "$target" "$lines" "$bytes"
done
