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
# Each step timed, to standard error: `time: N s  STEP`, and the whole.
now() { date +%s.%N; }
began=$(now)
step() {
    t=$(now)
    "$@" || return $?
    printf 'time: %6.1f s  %s\n' "$(echo "$(now) - $t" | bc)" "$*" >&2
}
quick=
case "${1-}" in
--quick) quick=--quick ;;
"") ;;
*) echo "usage: scripts/check.sh [--quick]" >&2; exit 2 ;;
esac
step cargo fmt --all --check
# Which of Fixpt's answers kept this check asks for (C132): none yet.
step python3 -I scripts/fx26-tests.py --reset
# The bootstrap's fixed point (stage 0, Fixpt's runs) and the workspace's
# tests under nextest (C276: the incremental compiles among them, chains at
# once) run beside the rest, started first, as they take longest and need
# nothing the rest does; their output kept and said when they end. The
# workspace built first, so that no stage builds it under another.
cargo build --release --quiet --workspace
logs=$(mktemp -d)
fixpoint=; tests=; selfhost=; asyncs=; stress=
# Should a stage fail first, the five and everything they started stopped.
killed() { for c in $(pgrep -P "$1" 2>/dev/null); do killed "$c"; done; kill "$1" 2>/dev/null || true; }
trap 'for p in $fixpoint $tests $selfhost $asyncs $stress; do killed "$p"; done; rm -rf "$logs"' EXIT
step scripts/fixpoint.sh $quick > "$logs/fixpoint" 2>&1 & fixpoint=$!
# The second self-hosting (C366): the compiler on tier 1 makes itself; not
# with --quick, for its 100 s.
if [ -z "$quick" ]; then
    step scripts/selfhost-cells.sh > "$logs/selfhost" 2>&1 & selfhost=$!
    # The emitter's async mode (C371): the compiler built async makes itself.
    step scripts/async-check.sh > "$logs/asyncs" 2>&1 & asyncs=$!
    # The collector's stress mode (C511): the corpus and the compiler, a
    # collection where no threshold would make one.
    step scripts/gc-stress.sh > "$logs/stress" 2>&1 & stress=$!
fi
rm -rf target/kayak-checks
step env ${quick:+KAYAK_CHECK_QUICK=1} cargo nextest run --release --workspace --hide-progress-bar --status-level fail > "$logs/tests" 2>&1 & tests=$!
step python3 -I scripts/io-programs.py
step cargo clippy --release --workspace --all-targets --quiet -- -D warnings
step python3 -I scripts/corpus.py
step python3 -I scripts/engine-programs.py
# kc's steps (C399): each back end against Fixpt's answers.
step python3 -I scripts/kc-steps.py
# kc's assembler (C450): each object kc-steps' builds made, as `as` makes it.
step python3 -I scripts/kc-objects.py
# kc's expansion, a form at a time, against the engine's, a program at a
# time (C445): each of Fixpt's programs that defines a macro.
step python3 -I scripts/kc-expanded.py
# The models (C400): Quint's simulator, the fixed holding, the unfixed not.
step python3 -I scripts/models.py
step python3 -I scripts/repl-sessions.py
# C103's REPL, kept to compare with C111's: the same sessions, the same bytes.
step python3 -I scripts/repl-sessions.py --rust-repl
step cargo clippy --release -p kayak-fx26 --features rust-repl --target-dir target/rust-repl --quiet -- -D warnings
# The corpus again with `par`'s shared branches at once (C102, C195): the
# one run of kayak-sync's `unsafe`.
step cargo nextest run --release -p kfx-corpus --features kayak-rt/unsafe-par --target-dir target/unsafe-par --hide-progress-bar --status-level fail
# Fixpt's own programs (tests/fx26/programs) and its answers to them
# (tests/fixpt-answers), kept in kayak (C132): with a clone at the pin, each
# is checked to be the pin's, and an answer of another pin's asked again.
clone=
if scripts/fixpt.sh > /dev/null 2>&1; then
    clone=1
    step python3 -I scripts/built-in.py --check
    # Fixpt's macro expander, made KFX (C429), the pin's.
    step python3 -I scripts/macros-port.py --check
    # What kayak ported from Fixpt, each source at the pin (C121).
    step python3 -I scripts/fixpt-drift.py --check
    step python3 -I scripts/fx26-tests.py --check
    # KFX is a sublanguage of FX-26: each of kayak's KFX units (the
    # compiler, the engine, tier 1) Fixpt's checkers accept too (C254).
    step python3 -I scripts/fixpt-accepts.py
    # The lang items (C367): one table of what compiler, runtimes and
    # standard library name across their boundaries, and every copy of it
    # checked against it.
    step python3 -I scripts/lang-items.py --check
fi
# The compiler's identity, its sources' (C387): its memo files its own.
step python3 -I scripts/compiler-identity.py --check
# What kayak took from Coopt (C368): forks, kayak's own to change (the
# user, C369), so what changed upstream since is said, not required.
if [ -d "${KAYAK_COOPT_CLONE:-../kayak-refs/coopt}/.git" ]; then
    step python3 -I scripts/coopt-drift.py
fi
# Those that kfx accepts, run both ways.
step python3 -I scripts/run-fixpt-programs.py
step python3 -I scripts/refused-fixpt-programs.py
# The five beside the rest, in order.
for job in fixpoint tests selfhost asyncs stress; do
    eval pid=\$$job
    [ -n "$pid" ] || continue
    status=0
    wait "$pid" || status=$?
    cat "$logs/$job"
    [ $status = 0 ] || { echo "check: $job failed" >&2; exit 1; }
done
fixpoint=; tests=; selfhost=; asyncs=; stress=
# Tier 1, Fixpt's cellular machine in KFX, on Fixpt's own words (C122) and
# on kayak's own (C133): what the tests found (crates/kayak-checks/tests/
# cells.rs), counted.
step cargo run --release --quiet -p kayak-checks --bin checks-summary -- cells
# The checkers compared, form by form (TODO §30, tests/checkers.rs): fails
# only if kayak's accepts a program Fixpt's refuses; the rest is the
# measure of §30.
step cargo run --release --quiet -p kayak-checks --bin checks-summary -- checkers --summary
# kayak's reps against Fixpt's (`fixpt check --reps`, tests/reps.rs, C352):
# fails if a program on tests/reps-passing is no longer Fixpt's.
step cargo run --release --quiet -p kayak-checks --bin checks-summary -- reps
# Every answer kept was asked for in this check: the rest dropped, and the
# answers stamped as the pin's.
if [ -n "$clone" ]; then
    step python3 -I scripts/fx26-tests.py --prune
fi
printf 'time: %6.1f s  the whole check\n' "$(echo "$(now) - $began" | bc)" >&2
echo "check: all passed${quick:+ (quick: no stage 0)}"
