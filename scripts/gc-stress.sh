#!/bin/sh
# The collector's stress mode (C511; TODO §54, step 4): collections where
# no threshold would make them, each at a different allocation, so that a
# root missed, or a word not followed, shows. `KAYAK_GC_STRESS=n`: a
# collection at every nth allocation; freed pages poisoned.
set -eu
# Timed whole (`scripts/timed`): `gc-stress`, `-in-check` in the check.
[ -z "${TIMED_GC_STRESS-}" ] && TIMED_GC_STRESS=1 exec "$(dirname "$0")/timed" "gc-stress${KAYAK_TIMED_IN-}" "$0" "$@"
cd "$(dirname "$0")/.."
# kayak-rt's own tests, every 7th allocation, their threads each
# collecting its own pages at once (the long loops left out: quadratic).
KAYAK_GC_STRESS=7 cargo test --release --quiet -p kayak-rt --lib -- --skip a_long_ > /dev/null
echo "gc-stress: kayak-rt's tests, a collection every 7 allocations"
# And at every allocation, what is freed poisoned at once, the tests'
# threads ending as the others collect, each one's pages then another's
# (C513): a word to what moved, or was freed, read soon after.
KAYAK_GC_STRESS=1 KAYAK_GC_POISON=1 cargo test --release --quiet -p kayak-rt --lib -- --skip a_long_ > /dev/null
echo "gc-stress: kayak-rt's tests, a collection at every allocation, freed memory poisoned"
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
# The compiler making itself, every 300007th: boot/src/kfx/ again.
cargo build --release --quiet -p kfx
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
KAYAK_GC_STRESS=300007 ./target/release/kfx kfx/kfx.unit "$tmp"
diff -r "$tmp" boot/src/kfx
echo "gc-stress: the compiler makes boot/src/kfx/ again, a collection every 300007 allocations"
