#!/bin/sh
# The collector's stress mode (C511; TODO §54, step 4): collections where
# no threshold would make them, each at a different allocation, so that a
# root missed, or a word not followed, shows. `KAYAK_GC_STRESS=n`: a
# collection at every nth allocation; freed pages poisoned.
#
# usage: scripts/gc-stress.sh [rt | corpus | kc | compiler]...
#
# Its parts (C555), all if none is named: kayak-rt's own tests; the
# corpus; kc's code in the heap; the compiler making itself. The check
# runs each a job of its own, kc's every time and the others when what
# they read changed (scripts/memo.py).
set -eu
parts=${*:-rt corpus kc compiler}
for part in $parts; do
    case $part in rt | corpus | kc | compiler) ;; *) echo "usage: scripts/gc-stress.sh [rt | corpus | kc | compiler]..." >&2; exit 2 ;; esac
done
# Timed whole (`scripts/timed`): `gc-stress`, `-in-check` in the check; a
# part alone, `gc-stress-PART`.
name=gc-stress
[ $# = 1 ] && name=gc-stress-$1
[ -z "${TIMED_GC_STRESS-}" ] && TIMED_GC_STRESS=1 exec "$(dirname "$0")/timed" "$name${KAYAK_TIMED_IN-}" "$0" "$@"
cd "$(dirname "$0")/.."
wants() { case " $parts " in *" $1 "*) return 0 ;; esac; return 1; }
if wants rt; then
# kayak-rt's own tests, every 7th allocation, their threads each
# collecting its own pages at once (the long loops left out: quadratic).
KAYAK_GC_STRESS=7 cargo test --release --quiet -p kayak-rt -p kayak-heap --lib -- --skip a_long_ > /dev/null
echo "gc-stress: kayak-rt's tests, a collection every 7 allocations"
# And at every allocation, what is freed poisoned at once, the tests'
# threads ending as the others collect, each one's pages then another's
# (C513): a word to what moved, or was freed, read soon after.
KAYAK_GC_STRESS=1 KAYAK_GC_POISON=1 cargo test --release --quiet -p kayak-rt -p kayak-heap --lib -- --skip a_long_ > /dev/null
echo "gc-stress: kayak-rt's tests, a collection at every allocation, freed memory poisoned"
fi
if wants corpus; then
# The corpus, every 97th; the two programs that run longest (400 s each
# so), every 9973rd.
big=made-generated-x86_64,made-frames-x86_64,made-generated-aarch64,made-frames-aarch64
for run in "97 SKIP" "9973 ONLY"; do
    set -- $run
    env KAYAK_GC_STRESS="$1" "KAYAK_CORPUS_$2=$big" cargo nextest run --release -p kfx-corpus \
        --hide-progress-bar --status-level fail -E 'test(every_program_gives_fixpts_answer)' > /dev/null 2>&1 \
        || { echo "gc-stress: the corpus, every $1 ($2 the longest), differs or fails" >&2; exit 1; }
done
echo "gc-stress: the corpus gives Fixpt's answers, a collection every 97 allocations (its longest two, 9973)"
# Its programs of processes and `par`, small, at every allocation, their
# freed pages poisoned: each thread collecting as the others run (C512).
threads=processes,tasks,monitors,promises,frozen-shared,responsive,process-load-place
env KAYAK_GC_STRESS=1 KAYAK_GC_POISON=1 KAYAK_CORPUS_ONLY=$threads cargo nextest run --release -p kfx-corpus \
    --hide-progress-bar --status-level fail -E 'test(every_program_gives_fixpts_answer)' > /dev/null 2>&1 \
    || { echo "gc-stress: the corpus's programs of threads, at every allocation, differ or fail" >&2; exit 1; }
echo "gc-stress: its programs of processes and par, a collection at every allocation"
fi
if wants kc; then
# kc's code in process, its objects in kayak-rt's heap (C538): its
# programs of it (tests/kc/heap), a collection every 31st allocation, freed
# pages poisoned, each collection verified (the barrier's slots among it).
# And each built (C541), its heap the shared heap too, its roots its own.
cargo build --release --quiet -p kayak-kc -p kayak-kcrt
# Each program beside the others (C555), each its own database.
kcdb=$(mktemp -d)
heap_program() {
    f=$1
    db=$2
    want=$(sed -n '1s/^;; => //p' "$f")
    # Compiled first, unstressed (C544: a program of processes compiles the
    # scheduler too, which stressed would take kc minutes); then run
    # stressed, its database warm, no query body run.
    ./target/release/kc --db "$db/db" run "$f" >/dev/null 2>&1
    # A program of processes (its name `tasks…`), in process, every 997th:
    # kc reads its database, the scheduler's items in it, under the stress
    # too, every 31st taking it minutes. Built, every 31st as the others.
    every=31
    case "$(basename "$f")" in tasks*) every=997 ;; esac
    got=$(KAYAK_GC_STRESS=$every KAYAK_GC_POISON=1 KAYAK_GC_VERIFY=1 ./target/release/kc --db "$db/db" run "$f" 2>&1 | tail -1)
    [ "$got" = "$want" ] || { echo "gc-stress: kc's $f in process: $got, not $want" >&2; return 1; }
    ./target/release/kc --db "$db/db" --rt target/release/libkayak_kcrt.a build "$f" "$db/exe" >/dev/null 2>&1 \
        || { echo "gc-stress: kc's $f: kc build fails" >&2; return 1; }
    got=$(KAYAK_GC_STRESS=31 KAYAK_GC_POISON=1 KAYAK_GC_VERIFY=1 "$db/exe" 2>&1 | tail -1)
    [ "$got" = "$want" ] || { echo "gc-stress: kc's $f built: $got, not $want" >&2; return 1; }
}
pids=
k=0
for f in tests/kc/heap/*.fx; do
    k=$((k + 1))
    mkdir "$kcdb/$k"
    heap_program "$f" "$kcdb/$k" &
    pids="$pids $!"
done
failed=0
for p in $pids; do wait "$p" || failed=1; done
[ $failed = 0 ] || { rm -rf "$kcdb"; exit 1; }
rm -rf "$kcdb"
echo "gc-stress: kc's code in kayak-rt's heap, in process and built, a collection every 31 allocations, verified"
fi
if wants compiler; then
# The compiler making itself, every 300007th: boot/src/kfx/ again.
cargo build --release --quiet -p kfx
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
KAYAK_GC_STRESS=300007 ./target/release/kfx kfx/kfx.unit "$tmp"
diff -r "$tmp" boot/src/kfx
echo "gc-stress: the compiler makes boot/src/kfx/ again, a collection every 300007 allocations"
fi
