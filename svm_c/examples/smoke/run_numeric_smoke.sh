#!/bin/sh
set -eu

# Complete numeric regression driver.  Each layer is compiled separately so
# even the least code-dense target does not need one giant monolithic image.
# Usage:
#   sh svm_c/examples/smoke/run_numeric_smoke.sh [target/release]

BIN_DIR=${1:-target/release}
CC="$BIN_DIR/svm-c"
RT="$BIN_DIR/svm-rt"
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
SMOKE="$ROOT/svm_c/examples/smoke"
LIB="$ROOT/svm_c/lib"
OUT=${TMPDIR:-/tmp}/svm-numeric-smoke-$$
mkdir -p "$OUT"
trap 'rm -rf "$OUT"' EXIT HUP INT TERM

cases='smoke_scalar_int smoke_wide_int smoke_q15 smoke_f16 smoke_f32 smoke_arithmetic'
targets='register stack accumulator memreg loadstore regmem memory2memory belt tta'
exts='svm svs sva svf svl svr svc svb svt'

set -- $exts
for target in $targets; do
    ext=$1
    shift
    echo "== $target ($ext) =="
    for test in $cases; do
        out="$OUT/$test.$ext"
        printf '  %-22s ' "$test"
        "$CC" --target "$target" -O2 -I "$LIB" "$SMOKE/$test.sc" "$out"
        result=$($RT "$out")
        last=$(printf '%s\n' "$result" | tail -n 1)
        if [ "$last" != "OK" ]; then
            printf 'FAIL\n%s\n' "$result"
            exit 1
        fi
        echo OK
    done

done

echo "ALL TARGETS / ALL NUMERIC LAYERS OK"
