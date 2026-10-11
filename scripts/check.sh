#!/bin/sh
# Every check, before a commit: formatting of the hand-written Rust, clippy,
# the tests (the corpus's answers and the tidy rules among them), the corpus's
# golden files against the compiler and Fixpt, and the bootstrap's fixed point.
set -eu
cd "$(dirname "$0")/.."
cargo fmt --all --check
cargo clippy --release --workspace --all-targets --quiet -- -D warnings
cargo test --release --workspace --quiet
python3 -I scripts/corpus.py
scripts/fixpoint.sh
echo "check: all passed"
