#!/bin/sh
# The bootstrap's checks (PLAN.md, section 1):
#   K2: boot/'s compiler, run on kfx/kfx.unit, makes boot/src/kfx/ again;
#   K1: Fixpt, running the compiler as FX-26 (stage 0), makes it too;
#   and the first run, recorded (C52), replays: as Rust, and on Fixpt.
# With --quick, K2 and the replay as Rust alone: Fixpt's runs are minutes
# (TODO §28), kept for the full check.
set -eu
quick=0
case "${1-}" in
--quick) quick=1 ;;
"") ;;
*) echo "usage: scripts/fixpoint.sh [--quick]" >&2; exit 2 ;;
esac
cd "$(dirname "$0")/.."
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

cargo build --release --quiet -p kfx
mkdir "$tmp/stage1" # kfx makes no directories (C50)
start=$(python3 -c 'import time; print(time.time())')
./target/release/kfx --record "$tmp/log" kfx/kfx.unit "$tmp/stage1"
took=$(python3 -c "import time; print('%.2f' % (time.time() - $start))")
diff -r "$tmp/stage1" boot/src/kfx
# The time is said so that a slower compiler shows (C84: one was 3x slower
# for several commits, unnoticed); 0.8 s, recording, on the machine of 2026-10-12.
echo "fixpoint: boot/'s kfx makes boot/src/kfx/ again, in $took s"
./target/release/kfx --replay "$tmp/log" kfx/kfx.unit "$tmp/stage1"

# Stage 0 runs the Fixpt of FIXPT_REV (scripts/fixpt.sh), and is skipped,
# saying so, only when there is no clone at all.
if [ $quick = 1 ]; then
    echo "fixpoint: stage 0 skipped (--quick)"
    exit 0
fi
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
