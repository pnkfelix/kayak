#!/bin/sh
# Regenerates boot/src/kfx/ from kfx/ (the files kfx/kfx.unit lists) through
# the bootstrap: build the compiler there is, run it on the source, and
# repeat until the output is the input (the fixed point, PLAN.md section 1).
# Only the output of a run that succeeded replaces the seed; on any failure
# the seed is put back as it was, and the script says which generation
# failed.
set -eu
cd "$(dirname "$0")/.."
tmp=$(mktemp -d)
cp -R boot/src/kfx "$tmp/seed"
install() {
    rm -rf boot/src/kfx
    cp -R "$1" boot/src/kfx
}
restore() {
    install "$tmp/seed"
    echo "regen: generation $1 failed; boot/src/kfx/ is as it was" >&2
    rm -rf "$tmp"
    exit 1
}
generation=0
while [ $generation -lt 4 ]; do
    generation=$((generation + 1))
    cargo build --release --quiet -p kfx || restore $generation
    rm -rf "$tmp/next"
    mkdir "$tmp/next" # kfx makes no directories (C50)
    # The memos a compile keeps (C54): scripts/fixpoint.sh compiles without
    # them, and scripts/incremental.py checks them against compiles without.
    ./target/release/kfx --cache target/kfx-memos kfx/kfx.unit "$tmp/next" || restore $generation
    if diff -r -q "$tmp/next" boot/src/kfx > /dev/null; then
        echo "regen: the fixed point, after $((generation - 1)) new generation(s)"
        rm -rf "$tmp"
        exit 0
    fi
    install "$tmp/next"
done
install "$tmp/seed"
rm -rf "$tmp"
echo "regen: no fixed point after $generation generations; boot/src/kfx/ is as it was" >&2
exit 1
