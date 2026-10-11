#!/bin/sh
# The bootstrap's checks (PLAN.md, section 1):
#   K2: boot/'s compiler, run on kfx/kfx.kfx, makes boot/src/kfx.rs again;
#   K1: Fixpt, running kfx/kfx.kfx as FX-26 (stage 0), makes it too.
set -eu
cd "$(dirname "$0")/.."
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

cargo build --release --quiet -p kfx
./target/release/kfx kfx/kfx.kfx > "$tmp/stage1.rs"
cmp "$tmp/stage1.rs" boot/src/kfx.rs
echo "fixpoint: boot/'s kfx makes boot/src/kfx.rs again"

# Stage 0 runs the Fixpt of FIXPT_REV (scripts/fixpt.sh), and is skipped,
# saying so, only when there is no clone at all.
status=0
scripts/fixpt.sh > /dev/null || status=$?
case $status in
0)
    python3 -I scripts/stage0.py kfx/kfx.kfx > "$tmp/stage0.rs"
    cmp "$tmp/stage0.rs" boot/src/kfx.rs
    echo "fixpoint: Fixpt $(cut -c1-7 FIXPT_REV), running kfx/kfx.kfx, makes it too"
    ;;
3) echo "fixpoint: stage 0 skipped: no Fixpt clone" ;;
*) exit "$status" ;;
esac
