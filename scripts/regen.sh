#!/bin/sh
# Regenerates boot/src/kfx/ from kfx/ (the files kfx/kfx.unit lists) through
# the bootstrap: build the compiler there is, run it on the source, and
# repeat until the output is the input (the fixed point, PLAN.md section 1).
# Only the output of a run that succeeded replaces the seed; on any failure
# the seed is put back as it was, and the script says which generation
# failed.
#
# usage: scripts/regen.sh [--all] [--check]
#
# With --all, then also the Rust kfx makes of the cellular machine and the
# engine, kept in crates/kayak-cells/src/cells and crates/kayak-fx26/src/fx26
# (the first checked by the `generated` test of crates/kayak-checks/tests/
# cells.rs, the second what `engine-programs.py --bless` writes,
# without running their programs), and those crates built.
#
# With --check, nothing is written: boot/'s compiler is built and run once,
# and its output compared with boot/src/kfx/ (and, with --all, the crates'
# with theirs), saying each file that differs; exit 1 if any does, 0 at the
# fixed point. (scripts/fixpoint.sh checks boot/ too, and more: a recorded
# compile's replay, and Fixpt's stage 0.)
set -eu
cd "$(dirname "$0")/.."
all=no
check=no
for arg in "$@"; do
    case "$arg" in
        --all) all=yes ;;
        --check) check=yes ;;
        *) echo "usage: scripts/regen.sh [--all] [--check]" >&2; exit 2 ;;
    esac
done
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
# Each step timed, to standard error: `time: N s  STEP`.
now() { date +%s.%N; }
step() {
    t=$(now)
    "$@" || return $?
    printf 'time: %6.1f s  %s\n' "$(echo "$(now) - $t" | bc)" "$*" >&2
}
# Whether the directory kfx made, $1, is the kept $2: if not, the files that
# differ said, prefixed `regen: `.
same() {
    diff -r -q "$1" "$2" > "$tmp/diff" && return 0
    sed "s|$tmp/[a-z]*|(made)|g; s|^|regen: |" "$tmp/diff"
    return 1
}
# A unit's Rust, made by the compiler just built: replacing a crate's
# generated directory whole (so that a file no longer made goes), or, with
# --check, compared with it.
differs=no
crate() {
    rm -rf "$tmp/crate"
    mkdir "$tmp/crate"
    step ./target/release/kfx "$1" "$tmp/crate" || { echo "regen: kfx failed on $1; $2 is as it was" >&2; exit 1; }
    if same "$tmp/crate" "$2"; then
        echo "regen: $2 is kfx's"
    elif [ $check = yes ]; then
        echo "regen: $2 is not kfx's"
        differs=yes
    else
        rm -rf "$2"
        cp -R "$tmp/crate" "$2"
        echo "regen: $2 rewritten"
    fi
}
crates() {
    crate cells/cells.unit crates/kayak-cells/src/cells
    crate fx26/fx26.unit crates/kayak-fx26/src/fx26
    if [ $check = no ]; then step cargo build --release --quiet -p kayak-cells -p kayak-fx26; fi
}
if [ $check = yes ]; then
    cargo build --release --quiet -p kfx
    mkdir "$tmp/next" # kfx makes no directories (C50)
    ./target/release/kfx kfx/kfx.unit "$tmp/next"
    if same "$tmp/next" boot/src/kfx; then
        echo "regen: boot/src/kfx/ is at the fixed point"
    else
        echo "regen: boot/src/kfx/ is not at the fixed point (scripts/regen.sh makes it)"
        differs=yes
    fi
    if [ $all = yes ]; then crates; fi
    [ $differs = no ]
    exit $?
fi
cp -R boot/src/kfx "$tmp/seed"
install() {
    rm -rf boot/src/kfx
    cp -R "$1" boot/src/kfx
}
restore() {
    install "$tmp/seed"
    echo "regen: generation $1 failed; boot/src/kfx/ is as it was" >&2
    exit 1
}
generation=0
while [ $generation -lt 4 ]; do
    generation=$((generation + 1))
    step cargo build --release --quiet -p kfx || restore $generation
    rm -rf "$tmp/next"
    mkdir "$tmp/next" # kfx makes no directories (C50)
    # The memos a compile keeps (C54): scripts/fixpoint.sh compiles without
    # them, and the incremental tests (crates/kayak-checks) check them
    # against compiles without.
    # A memo is one compiler's: their file is named by the compiler's own
    # generated code, so that one compiler never takes another's.
    compiler=$(cat boot/src/kfx/*.rs | shasum | cut -c1-16)
    find target -maxdepth 1 -name 'kfx-memos-*' ! -name "kfx-memos-$compiler" -delete
    step ./target/release/kfx --cache "target/kfx-memos-$compiler" kfx/kfx.unit "$tmp/next" || restore $generation
    if diff -r -q "$tmp/next" boot/src/kfx > /dev/null; then
        echo "regen: the fixed point, after $((generation - 1)) new generation(s)"
        if [ $all = yes ]; then crates; fi
        exit 0
    fi
    install "$tmp/next"
done
install "$tmp/seed"
echo "regen: no fixed point after $generation generations; boot/src/kfx/ is as it was" >&2
exit 1
