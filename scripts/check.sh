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
# The slow ones also through `scripts/timed`, by name, their history kept
# (`tstep NAME COMMAND…`): run here beside others, each name ends
# `-in-check`, which a script that times itself reads too.
KAYAK_TIMED_IN=-in-check
export KAYAK_TIMED_IN
now() { date +%s.%N; }
began=$(now)
step() {
    t=$(now)
    "$@" || return $?
    printf 'time: %6.1f s  %s\n' "$(echo "$(now) - $t" | bc)" "$*" >&2
}
tstep() {
    tstep_name=$1
    shift
    step scripts/timed "$tstep_name$KAYAK_TIMED_IN" "$@"
}
# With --all, every job run, whatever changed: a job of `scripts/memo.py`
# (C552) is otherwise run only when the inputs it declares have changed
# since it last passed.
quick=
case "${1-}" in
--quick) quick=--quick ;;
--all) KAYAK_CHECK_ALL=1; export KAYAK_CHECK_ALL ;;
"") ;;
*) echo "usage: scripts/check.sh [--quick | --all]" >&2; exit 2 ;;
esac
step cargo fmt --all --check
# Which of Fixpt's answers kept this check asks for (C132): none yet.
step python3 -I scripts/fx26-tests.py --reset
# The workspace built first, so that no stage builds it under another.
cargo build --release --quiet --workspace
logs=$(mktemp -d)
# Every stage after the workspace's build runs as a job beside the others
# (C551): each its log, said when it is waited for, in the order begun; the
# check as long as its longest, not as their sum. None writes what another
# reads (each its own target/ directory, Fixpt's answers kept atomically,
# what each asks noted by process); those that do, in order in one job.
names=; pids=
job() {
    job_name=$1
    shift
    "$@" > "$logs/$job_name" 2>&1 &
    names="$names $job_name"
    pids="$pids $!"
}
# Should a stage fail first, every job and everything they started stopped.
killed() { for c in $(pgrep -P "$1" 2>/dev/null); do killed "$c"; done; kill "$1" 2>/dev/null || true; }
trap 'for p in $pids; do killed "$p"; done; rm -rf "$logs"' EXIT
# The bootstrap's fixed point (stage 0, Fixpt's runs) and the workspace's
# tests under nextest (C276: the incremental compiles among them, chains at
# once), begun first, as they take longest.
job fixpoint tstep fixpoint${quick:+-quick} scripts/fixpoint.sh $quick
# The second self-hosting (C366): the compiler on tier 1 makes itself; not
# with --quick, for its 100 s.
if [ -z "$quick" ]; then
    # Run only when what it reads changed (C552): the compiler's sources,
    # what it must make, and the two binaries, kayak-rt and the heap in them.
    job selfhost python3 -I scripts/memo.py selfhost-cells --path kfx --path boot/src/kfx \
        --path scripts/selfhost-cells.sh --bin kayak-fx26 --bin kayak-cells \
        -- scripts/timed selfhost-cells-in-check scripts/selfhost-cells.sh
    # The emitter's async mode (C371): the compiler built async makes itself.
    job asyncs tstep async-check scripts/async-check.sh
    # The collector's stress mode (C511): the corpus and the compiler, a
    # collection where no threshold would make one.
    job stress step scripts/gc-stress.sh
fi
rm -rf target/kayak-checks
job tests tstep workspace-tests${quick:+-quick} env ${quick:+KAYAK_CHECK_QUICK=1} cargo nextest run --release --workspace --hide-progress-bar --status-level fail
# kc's steps (C399): each back end against Fixpt's answers; then kc's
# assembler (C450), each object kc-steps' builds made, as `as` makes it.
kc_steps() {
    tstep kc-steps python3 -I scripts/kc-steps.py
    step python3 -I scripts/kc-objects.py
}
job kc-steps kc_steps
job io-programs step python3 -I scripts/io-programs.py
job clippy step cargo clippy --release --workspace --all-targets --quiet -- -D warnings
job corpus step python3 -I scripts/corpus.py
job engine-programs tstep engine-programs python3 -I scripts/engine-programs.py
# kc's expansion, a form at a time, against the engine's, a program at a
# time (C445): each of Fixpt's programs that defines a macro.
job kc-expanded step python3 -I scripts/kc-expanded.py
# The models (C400): Quint's simulator, the fixed holding, the unfixed not.
job models step python3 -I scripts/models.py
# The REPL's sessions; C103's REPL, kept to compare with C111's: the same
# sessions, the same bytes, and its clippy (the one target/rust-repl).
repl() {
    step python3 -I scripts/repl-sessions.py
    step python3 -I scripts/repl-sessions.py --rust-repl
    step cargo clippy --release -p kayak-fx26 --features rust-repl --target-dir target/rust-repl --quiet -- -D warnings
}
job repl repl
# The corpus again with `par`'s shared branches at once (C102, C195): the
# one run of kayak-sync's `unsafe`.
job unsafe-par step cargo nextest run --release -p kfx-corpus --features kayak-rt/unsafe-par --target-dir target/unsafe-par --hide-progress-bar --status-level fail
# Fixpt's own programs (tests/fx26/programs) and its answers to them
# (tests/fixpt-answers), kept in kayak (C132): with a clone at the pin, each
# is checked to be the pin's, and an answer of another pin's asked again.
clone=
if scripts/fixpt.sh > /dev/null 2>&1; then
    clone=1
    job built-in step python3 -I scripts/built-in.py --check
    # Fixpt's macro expander, made KFX (C429), the pin's.
    job macros-port step python3 -I scripts/macros-port.py --check
    # What kayak ported from Fixpt, each source at the pin (C121).
    job fixpt-drift step python3 -I scripts/fixpt-drift.py --check
    job fx26-tests step python3 -I scripts/fx26-tests.py --check
    # KFX is a sublanguage of FX-26: each of kayak's KFX units (the
    # compiler, the engine, tier 1) Fixpt's checkers accept too (C254).
    job fixpt-accepts step python3 -I scripts/fixpt-accepts.py
    # The lang items (C367): one table of what compiler, runtimes and
    # standard library name across their boundaries, and every copy of it
    # checked against it.
    job lang-items step python3 -I scripts/lang-items.py --check
fi
# The compiler's identity, its sources' (C387): its memo files its own.
job identity step python3 -I scripts/compiler-identity.py --check
# What kayak took from Coopt (C368): forks, kayak's own to change (the
# user, C369), so what changed upstream since is said, not required.
if [ -d "${KAYAK_COOPT_CLONE:-../kayak-refs/coopt}/.git" ]; then
    job coopt-drift step python3 -I scripts/coopt-drift.py
fi
# Those that kfx accepts, run both ways; those it refuses, refused.
fixpt_programs() {
    tstep fixpt-programs python3 -I scripts/run-fixpt-programs.py
    step python3 -I scripts/refused-fixpt-programs.py
}
job fixpt-programs fixpt_programs
# Each job waited for, in the order begun, its log said.
set -- $pids
for name in $names; do
    status=0
    wait "$1" || status=$?
    shift
    cat "$logs/$name"
    [ $status = 0 ] || { echo "check: $name failed" >&2; exit 1; }
done
pids=
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
