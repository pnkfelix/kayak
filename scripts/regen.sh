#!/bin/sh
# Regenerates boot/src/kfx.rs from kfx/kfx.kfx through the bootstrap: build
# the compiler there is, run it on the source, and repeat until the output
# is the input (the fixed point, PLAN.md section 1). Only the output of a run
# that succeeded replaces the seed; on any failure the seed is put back as
# it was, and the script says which generation failed.
set -eu
cd "$(dirname "$0")/.."
tmp=$(mktemp -d)
cp boot/src/kfx.rs "$tmp/seed.rs"
restore() {
    cp "$tmp/seed.rs" boot/src/kfx.rs
    echo "regen: generation $1 failed; boot/src/kfx.rs is as it was" >&2
    rm -rf "$tmp"
    exit 1
}
generation=0
while [ $generation -lt 4 ]; do
    generation=$((generation + 1))
    cargo build --release --quiet -p kfx || restore $generation
    ./target/release/kfx kfx/kfx.kfx > "$tmp/next.rs" || restore $generation
    if cmp -s "$tmp/next.rs" boot/src/kfx.rs; then
        echo "regen: the fixed point, after $((generation - 1)) new generation(s)"
        rm -rf "$tmp"
        exit 0
    fi
    cp "$tmp/next.rs" boot/src/kfx.rs
done
cp "$tmp/seed.rs" boot/src/kfx.rs
rm -rf "$tmp"
echo "regen: no fixed point after $generation generations; boot/src/kfx.rs is as it was" >&2
exit 1
