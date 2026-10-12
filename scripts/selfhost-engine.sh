#!/bin/sh
# The second self-hosting (PLAN.md section 6.5, step 13; C66): the compiler,
# run on kayak's own FX-26 engine (kayak-fx26 --run), compiles itself, and
# must make boot/src/kfx/ exactly. Some five minutes: not in check.sh.
set -eu
cd "$(dirname "$0")/.."
cargo build --release --quiet -p kayak-fx26
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
files=$(grep -v '^;' kfx/kfx.unit | grep -v '^$' | sed 's|^|kfx/|')
# shellcheck disable=SC2086
./target/release/kayak-fx26 --run $files -- kfx/kfx.unit "$tmp"
diff -r "$tmp" boot/src/kfx
echo "selfhost-engine: the compiler on the engine makes boot/src/kfx/ again"
