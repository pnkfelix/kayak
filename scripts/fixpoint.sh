#!/bin/sh
# The bootstrap's checks (PLAN.md, section 1):
#   K2: boot/'s compiler, run on kfx/kfx.unit, makes boot/src/kfx/ again;
#   K1: Fixpt, running the compiler as FX-26 (stage 0), makes it too.
set -eu
cd "$(dirname "$0")/.."
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

cargo build --release --quiet -p kfx
mkdir "$tmp/stage1" # kfx makes no directories (C50)
./target/release/kfx kfx/kfx.unit "$tmp/stage1"
diff -r "$tmp/stage1" boot/src/kfx
echo "fixpoint: boot/'s kfx makes boot/src/kfx/ again"

# Stage 0 runs the Fixpt of FIXPT_REV (scripts/fixpt.sh), and is skipped,
# saying so, only when there is no clone at all.
status=0
scripts/fixpt.sh > /dev/null || status=$?
case $status in
0)
    python3 -I scripts/stage0.py kfx/kfx.unit "$tmp/stage0"
    diff -r "$tmp/stage0" boot/src/kfx
    echo "fixpoint: Fixpt $(cut -c1-7 FIXPT_REV), running the compiler, makes it too"
    ;;
3) echo "fixpoint: stage 0 skipped: no Fixpt clone" ;;
*) exit "$status" ;;
esac
