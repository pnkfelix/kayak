#!/bin/sh
# Every check, before a commit: formatting of the hand-written Rust, clippy,
# the programs that do I/O, run both ways (C49), the tests (the corpus's answers among them), the corpus's golden files
# against the compiler and Fixpt, the bootstrap's fixed point, and Fixpt's
# own programs that kfx accepts, run as Rust and by Fixpt, and those Fixpt
# refuses, refused by kfx too (C81).
#
# With --quick (TODO §28), for each round of work: Fixpt's runs of the
# compiler (stage 0, minutes) left out, and fewer incremental edits; the
# full check before a commit that touches the compiler or the language.
set -eu
cd "$(dirname "$0")/.."
quick=
case "${1-}" in
--quick) quick=--quick ;;
"") ;;
*) echo "usage: scripts/check.sh [--quick]" >&2; exit 2 ;;
esac
cargo fmt --all --check
# Which of Fixpt's answers kept this check asks for (C132): none yet.
python3 -I scripts/fx26-tests.py --reset
python3 -I scripts/io-programs.py
cargo clippy --release --workspace --all-targets --quiet -- -D warnings
cargo test --release --workspace --quiet
python3 -I scripts/corpus.py
python3 -I scripts/engine-programs.py
python3 -I scripts/repl-sessions.py
# C103's REPL, kept to compare with C111's: the same sessions, the same bytes.
python3 -I scripts/repl-sessions.py --rust-repl
cargo clippy --release -p kayak-fx26 --features rust-repl --target-dir target/rust-repl --quiet -- -D warnings
scripts/fixpoint.sh $quick
python3 -I scripts/incremental.py $quick
# Fixpt's own programs (tests/fx26/programs) and its answers to them
# (tests/fixpt-answers), kept in kayak (C132): with a clone at the pin, each
# is checked to be the pin's, and an answer of another pin's asked again.
clone=
if scripts/fixpt.sh > /dev/null 2>&1; then
    clone=1
    python3 -I scripts/built-in.py --check
    # What kayak ported from Fixpt, each source at the pin (C121).
    python3 -I scripts/fixpt-drift.py --check
    python3 -I scripts/fx26-tests.py --check
fi
# Those that kfx accepts, run both ways.
python3 -I scripts/run-fixpt-programs.py
# Tier 1, Fixpt's cellular machine in KFX, on Fixpt's own words (C122).
python3 -I scripts/cells-programs.py
python3 -I scripts/refused-fixpt-programs.py
# The checkers compared, form by form (TODO §30): fails only if kayak's
# accepts a program Fixpt's refuses; the rest is the measure of §30.
python3 -I scripts/compare-checkers.py --summary
# Every answer kept was asked for in this check: the rest dropped, and the
# answers stamped as the pin's.
if [ -n "$clone" ]; then
    python3 -I scripts/fx26-tests.py --prune
fi
echo "check: all passed${quick:+ (quick: no stage 0)}"
