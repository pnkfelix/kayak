#!/bin/sh
# The bootstrap's checks (PLAN.md, section 1):
#   K2: boot/'s compiler, run on kfx/kfx.unit, makes boot/src/kfx/ again;
#   K1: Fixpt, running the compiler as FX-26 (stage 0), makes it too;
#   and the first run, recorded (C52), replays: as Rust, and on Fixpt.
set -eu
cd "$(dirname "$0")/.."
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

cargo build --release --quiet -p kfx
mkdir "$tmp/stage1" # kfx makes no directories (C50)
./target/release/kfx --record "$tmp/log" kfx/kfx.unit "$tmp/stage1"
diff -r "$tmp/stage1" boot/src/kfx
echo "fixpoint: boot/'s kfx makes boot/src/kfx/ again"
./target/release/kfx --replay "$tmp/log" kfx/kfx.unit "$tmp/stage1"

# Stage 0 runs the Fixpt of FIXPT_REV (scripts/fixpt.sh), and is skipped,
# saying so, only when there is no clone at all.
status=0
scripts/fixpt.sh > /dev/null || status=$?
case $status in
0)
    # Fixpt's three runs at once: each its own directory.
    mkdir "$tmp/warm" "$tmp/warm0"
    ./target/release/kfx --cache "$tmp/memos" kfx/kfx.unit "$tmp/warm"
    cp "$tmp/memos" "$tmp/memos.rust"
    python3 -I scripts/stage0.py kfx/kfx.unit "$tmp/stage0" > "$tmp/s0.log" 2>&1 & a=$!
    python3 -I scripts/stage0.py --replay "$tmp/log" kfx/kfx.unit "$tmp/stage1" > "$tmp/s1.log" 2>&1 & b=$!
    python3 -I scripts/stage0.py --cache "$tmp/memos" kfx/kfx.unit "$tmp/warm0" > "$tmp/s2.log" 2>&1 & c=$!
    wait $a || { cat "$tmp/s0.log"; exit 1; }
    diff -r "$tmp/stage0" boot/src/kfx
    echo "fixpoint: Fixpt $(cut -c1-7 FIXPT_REV), running the compiler, makes it too"
    wait $b || { cat "$tmp/s1.log"; exit 1; }
    echo "fixpoint: the compile recorded replays, as Rust and on Fixpt"
    # Memos (C54): Fixpt, given the Rust's, takes each and gives them back.
    wait $c || { cat "$tmp/s2.log"; exit 1; }
    diff -r "$tmp/warm0" boot/src/kfx
    cmp "$tmp/memos" "$tmp/memos.rust"
    echo "fixpoint: Fixpt takes the Rust's memos, and keeps the same"
    ;;
3) echo "fixpoint: stage 0 skipped: no Fixpt clone" ;;
*) exit "$status" ;;
esac
