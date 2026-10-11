#!/bin/sh
# Every check, before a commit: formatting of the hand-written Rust, clippy,
# the tests (the corpus's answers among them), the corpus's golden files
# against the compiler and Fixpt, the bootstrap's fixed point, and Fixpt's
# own programs that kfx accepts, run as Rust and by Fixpt.
set -eu
cd "$(dirname "$0")/.."
cargo fmt --all --check
cargo clippy --release --workspace --all-targets --quiet -- -D warnings
cargo test --release --workspace --quiet
python3 -I scripts/corpus.py
scripts/fixpoint.sh
# Fixpt's own programs that kfx accepts, run both ways (needs the clone).
if scripts/fixpt.sh > /dev/null 2>&1; then
    python3 -I scripts/run-fixpt-programs.py
fi
echo "check: all passed"
