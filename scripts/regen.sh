#!/bin/sh
# Regenerates boot/src/kfx/ from kfx/ (the files kfx/kfx.unit lists) through
# the bootstrap: build the compiler there is, run it on the source, and
# repeat until the output is the input (the fixed point, PLAN.md section 1).
# Only the output of a run that succeeded replaces the seed; on any failure
# the seed is put back as it was, and the script says which generation
# failed.
#
# usage: scripts/regen.sh [--all]
#
# With --all, then also the Rust kfx makes of the cellular machine and the
# engine, kept in crates/kayak-cells/src/cells and crates/kayak-fx26/src/fx26
# (what `cells-programs.py --bless` and `engine-programs.py --bless` write,
# without running their programs), and those crates built.
set -eu
cd "$(dirname "$0")/.."
all=no
case "${1-}" in
    "") ;;
    --all) all=yes ;;
    *) echo "usage: scripts/regen.sh [--all]" >&2; exit 2 ;;
esac
# A unit's Rust, made by the compiler just built, replacing a crate's
# generated directory whole (so that a file no longer made goes).
crate() {
    rm -rf "$tmp/crate"
    mkdir "$tmp/crate"
    ./target/release/kfx "$1" "$tmp/crate" || { echo "regen: kfx failed on $1; $2 is as it was" >&2; rm -rf "$tmp"; exit 1; }
    if diff -r -q "$tmp/crate" "$2" > /dev/null; then
        echo "regen: $2 unchanged"
    else
        rm -rf "$2"
        cp -R "$tmp/crate" "$2"
        echo "regen: $2 rewritten"
    fi
}
crates() {
    crate cells/cells.unit crates/kayak-cells/src/cells
    crate fx26/fx26.unit crates/kayak-fx26/src/fx26
    cargo build --release --quiet -p kayak-cells -p kayak-fx26 || { rm -rf "$tmp"; exit 1; }
}
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
    # A memo is one compiler's: their file is named by the compiler's own
    # generated code, so that one compiler never takes another's.
    compiler=$(cat boot/src/kfx/*.rs | shasum | cut -c1-16)
    find target -maxdepth 1 -name 'kfx-memos-*' ! -name "kfx-memos-$compiler" -delete
    ./target/release/kfx --cache "target/kfx-memos-$compiler" kfx/kfx.unit "$tmp/next" || restore $generation
    if diff -r -q "$tmp/next" boot/src/kfx > /dev/null; then
        echo "regen: the fixed point, after $((generation - 1)) new generation(s)"
        if [ $all = yes ]; then crates; fi
        rm -rf "$tmp"
        exit 0
    fi
    install "$tmp/next"
done
install "$tmp/seed"
rm -rf "$tmp"
echo "regen: no fixed point after $generation generations; boot/src/kfx/ is as it was" >&2
exit 1
