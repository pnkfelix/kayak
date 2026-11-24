#!/bin/sh
# What is run before a full check (scripts/check-snapshot.sh) after a
# change to kayak-rt or the emitter, so that what would fail it fails
# sooner and alone: formatting and the compiler's identity, the
# collector's stress (scripts/gc-stress.sh), clippy of the workspace and
# of kayak-rt with each feature the check does not build so, the builds
# the check runs in target dirs of their own, and Fixpt's programs. Each
# step timed by name (`scripts/timed`), the whole too (`pre-check`); every
# step run, and the script exits 1 if any failed, so that a failed run is
# never recorded as a time.
#
# usage: scripts/pre-check.sh
set -u
[ -z "${TIMED_PRE_CHECK-}" ] && TIMED_PRE_CHECK=1 exec "$(dirname "$0")/timed" pre-check "$0" "$@"
cd "$(dirname "$0")/.."
failed=
# A step timed by its name, NAME; noted if it fails. A clippy or a build
# may find its work done (nothing changed since): one under 5 s is not
# recorded (`--at-least`).
run() {
    name=$1
    shift
    scripts/timed --at-least 5 "$name" "$@" || failed="$failed $name"
}
cargo fmt --all || failed="$failed fmt"
python3 -I scripts/compiler-identity.py | tail -1 || failed="$failed identity"
scripts/gc-stress.sh || failed="$failed gc-stress"
run clippy-workspace cargo clippy --release --workspace --all-targets --quiet -- -D warnings
run clippy-heap-stats cargo clippy --release -p kayak-rt --all-targets --features heap-stats --quiet -- -D warnings
run clippy-unsafe-par cargo clippy --release -p kayak-rt --all-targets --features unsafe-par --quiet -- -D warnings
run build-rust-repl cargo build --release --quiet -p kayak-fx26 --features rust-repl --target-dir target/rust-repl
run build-unsafe-par cargo build --release --quiet -p kfx-corpus --tests --features kayak-rt/unsafe-par --target-dir target/unsafe-par
run fixpt-programs sh -c 'python3 -I scripts/run-fixpt-programs.py > target/fixpt-programs.log 2>&1; s=$?; tail -3 target/fixpt-programs.log; exit $s'
if [ -n "$failed" ]; then
    echo "pre-check: failed:$failed" >&2
    exit 1
fi
echo "pre-check: all passed"
