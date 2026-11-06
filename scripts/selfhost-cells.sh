#!/bin/sh
# The second self-hosting, on tier 1 (C366; docs/direction.md, step 1): the
# compiler, as FX-26 (its prelude first, as Fixpt's stage 0 reads it),
# compiled to cellular words by kayak-fx26 and run by kayak-cells with its
# words compiled, compiles itself, and must make boot/src/kfx/ exactly.
# About 100 s and 8 GB; in the full check, beside the rest. Its memory is
# capped where systemd can (a run that grows without bound stopped, not the
# machine starved).
set -eu
cd "$(dirname "$0")/.."
cargo build --release --quiet -p kayak-fx26 -p kayak-cells
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
files=$(grep -v '^;' kfx/kfx.unit | grep -v '^$' | sed 's|^|kfx/|')
# shellcheck disable=SC2086
./target/release/kayak-fx26 --cells-run kfx/prelude.fx $files > "$tmp/kfx.cells"
mkdir "$tmp/out"
cap=
if systemd-run --user --scope -q -p MemoryMax=1G true 2>/dev/null; then
    cap="systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0"
fi
start=$(python3 -c 'import time; print(time.time())')
$cap ./target/release/kayak-cells "$tmp/kfx.cells" --main --compiled --step-limit none -- kfx/kfx.unit "$tmp/out"
took=$(python3 -c "import time; print('%.0f' % (time.time() - $start))")
diff -r "$tmp/out" boot/src/kfx
echo "selfhost-cells: the compiler on tier 1 makes boot/src/kfx/ again, in $took s"
