#!/bin/sh
# Every check, before a commit: formatting of the hand-written Rust, clippy,
# the programs that do I/O, run both ways (C49), the tests (the corpus's answers among them), the corpus's golden files
# against the compiler and Fixpt, the bootstrap's fixed point, and Fixpt's
# own programs that kfx accepts, run as Rust and by Fixpt, and those Fixpt
# refuses, refused by kfx too (C81).
set -eu
cd "$(dirname "$0")/.."
cargo fmt --all --check
python3 -I scripts/io-programs.py
cargo clippy --release --workspace --all-targets --quiet -- -D warnings
cargo test --release --workspace --quiet
python3 -I scripts/corpus.py
python3 -I scripts/engine-programs.py
scripts/fixpoint.sh
python3 -I scripts/incremental.py
# Fixpt's own programs that kfx accepts, run both ways (needs the clone).
if scripts/fixpt.sh > /dev/null 2>&1; then
    python3 -I scripts/run-fixpt-programs.py
    python3 -I scripts/refused-fixpt-programs.py
fi
echo "check: all passed"
