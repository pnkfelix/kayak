"""usage: python3 -I scripts/engine-bench.py [--runs N]

Times kayak's engine (kayak-fx26, built --release) on Fixpt's benchmark
programs (tests/fx26/programs/bench): best of N runs (default 3), in ms,
the whole process. A measure for the engine's heap moving to Fixpt's
layout (C318, docs/research/one-heap.md): run before and after a change.
The compiler run on the engine (C63, C64), the largest program there is
for it, is to be a row once the engine has `rep-nodes` (TODO).
"""

import os
import subprocess
import sys
import time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ENGINE = os.path.join(ROOT, "target", "release", "kayak-fx26")
BENCH = os.path.join(ROOT, "tests", "fx26", "programs", "bench")


def best(args, runs):
    times, out = [], None
    for _ in range(runs):
        t = time.perf_counter()
        r = subprocess.run(args, capture_output=True, text=True, cwd=ROOT)
        times.append((time.perf_counter() - t) * 1000)
        out = r
    return min(times), out


def main():
    runs = 3
    if "--runs" in sys.argv:
        runs = int(sys.argv[sys.argv.index("--runs") + 1])
    r = subprocess.run(["cargo", "build", "--release", "--quiet", "-p", "kayak-fx26"], cwd=ROOT)
    if r.returncode != 0:
        sys.exit("engine-bench: the engine does not build")
    rows = []
    for name in sorted(os.listdir(BENCH)):
        if name.endswith(".fx"):
            ms, r = best([ENGINE, os.path.join(BENCH, name)], runs)
            rows.append((name, ms, r.returncode))
    for name, ms, code in rows:
        print(f"{name:<28} {ms:>9.0f} ms" + ("" if code == 0 else f"  (status {code})"))


main()
