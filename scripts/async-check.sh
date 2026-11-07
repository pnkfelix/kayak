#!/bin/sh
# The emitter's async mode (C371, `kfx --async`): the compiler compiled with
# every procedure an `async fn`, built, must make boot/src/kfx/ in plain
# mode, as the synchronous compiler does, and make its own async Rust again
# with --async. About 3 minutes (rustc's build of async code is 86 s); in the
# full check, beside the rest, its memory capped where systemd can.
set -eu
cd "$(dirname "$0")/.."
cargo build --release --quiet -p kfx
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
mkdir "$tmp/async" "$tmp/plain" "$tmp/again"
./target/release/kfx --async kfx/kfx.unit "$tmp/async"
# A crate of its own, beside boot/: boot's manifest and main, the async Rust.
mkdir -p "$tmp/crate/src"
cp -r "$tmp/async" "$tmp/crate/src/kfx"
cp boot/src/main.rs "$tmp/crate/src/main.rs"
root=$(pwd)
sed -e "s|path = \"\.\./crates/|path = \"$root/crates/|" \
    -e 's/^version.workspace = true/version = "0.0.0"/' \
    -e 's/^edition.workspace = true/edition = "2024"/' \
    -e 's/^license.workspace = true//' \
    -e 's/^workspace = true//' \
    -e 's/^\[lints\]//' \
    boot/Cargo.toml > "$tmp/crate/Cargo.toml"
printf '\n[workspace]\n\n[profile.release]\ncodegen-units = 16\n' >> "$tmp/crate/Cargo.toml"
cap=
if systemd-run --user --scope -q -p MemoryMax=1G true 2>/dev/null; then
    cap="systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0"
fi
CARGO_TARGET_DIR="$root/target/async-check" cargo build --release --quiet --manifest-path "$tmp/crate/Cargo.toml"
bin="$root/target/async-check/release/kfx"
$cap "$bin" kfx/kfx.unit "$tmp/plain"
diff -r "$tmp/plain" boot/src/kfx
$cap "$bin" --async kfx/kfx.unit "$tmp/again"
diff -r "$tmp/again" "$tmp/async"
echo "async-check: the compiler built async makes boot/src/kfx/ again, and its own async Rust"
# And the corpus in the async mode (C373, C376): each program, every
# procedure async, gives Fixpt's answer.
python3 -I scripts/async-corpus.py
# And the I/O programs (C381): each, every procedure async, its I/O the
# runner's, does as the synchronous build does.
python3 -I scripts/async-io-programs.py
