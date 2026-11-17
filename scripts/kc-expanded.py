"""kc's expansion, a form at a time, against the engine's, a program at a
time (C445).

kc expands each top-level form of a file that defines a macro as a query
of its own (kc/expand.kfx; docs/research/kc.md, step 15), in an instance
of the expander given none of the forms before it but the definitions of
the macros it names, to a fixed point, the shapes of the globals it names,
and how many forms of each hash were begun before it: so that what a form
makes depends on what it names alone. The engine expands the program as
one, as Fixpt's parser does (`fx26/main.kfx`'s `expanded`).

For each of the pinned Fixpt's programs that defines a macro, kc's
`--expanded-located` and the engine's must write the same: each node and
where it is, or the same refusal.

usage: python3 -I scripts/kc-expanded.py [-v] [PROGRAM ...]

The programs known to differ, and why, are KNOWN; any other that differs
fails the check, and a KNOWN one that agrees is said, to be taken out.
"""

import concurrent.futures
import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ENGINE = os.path.join(ROOT, "target", "release", "kayak-fx26")
KC = os.path.join(ROOT, "target", "release", "kc")
PROGRAMS = os.path.join(ROOT, "..", "kayak-refs", "fixpt", "crates", "fixpt-fx26", "tests", "programs")

MADE = "a later form names a macro a use makes, or a `hide` holds, not a `define-syntax` written at the top level: kc refuses, not yet"
SHAPED = "a later form names a global a use of a macro defines, its shape known (a module's): kc refuses, not yet"
# Programs whose expansion by kc differs from the engine's, each with why.
KNOWN = {
    "macros/defn-defines-macro-ellipsis.fx": MADE,
    "macros/defn-defines-macro-hygiene.fx": MADE,
    "macros/defn-defines-macro-said.fx": MADE,
    "macros/defn-defines-macro.fx": MADE,
    "macros/defn-heads-scoped.fx": MADE,
    "macros/top-level-hide.fx": MADE,
    "procedural/contract-defn-macro.fx": MADE,
    "procedural/defines-macro.fx": MADE,
    "cost/diamond-found-last.fx": SHAPED,
    "macros/desc-subr-named.fx": SHAPED,
}


def programs():
    out = []
    for d, _, fs in os.walk(PROGRAMS):
        for f in sorted(fs):
            if f.endswith(".fx"):
                p = os.path.join(d, f)
                with open(p, encoding="utf-8", errors="replace") as h:
                    if "define-syntax" in h.read():
                        out.append(p)
    return sorted(out)


def run(argv):
    try:
        r = subprocess.run(argv, capture_output=True, text=True, timeout=300)
    except subprocess.TimeoutExpired:
        return "timeout"
    if r.returncode != 0:
        return "status %d: %s" % (r.returncode, (r.stderr.strip().splitlines() or [""])[-1])
    return r.stdout


def compare(path):
    whole = run([ENGINE, "--expanded-located", path])
    alone = run([KC, "--expanded-located", path])
    if whole == alone:
        return path, None
    w, a = whole.splitlines(), alone.splitlines()
    for i in range(max(len(w), len(a))):
        x = w[i] if i < len(w) else "(none)"
        y = a[i] if i < len(a) else "(none)"
        if x != y:
            k = next((j for j in range(min(len(x), len(y))) if x[j] != y[j]), min(len(x), len(y)))
            return path, "line %d, at %d:\n  engine: %s\n  kc:     %s" % (i + 1, k, x[max(0, k - 150):k + 250], y[max(0, k - 150):k + 250])
    return path, "differ"


def main():
    verbose = "-v" in sys.argv[1:]
    given = [a for a in sys.argv[1:] if a != "-v"]
    ps = given or programs()
    with concurrent.futures.ThreadPoolExecutor(max_workers=os.cpu_count() or 4) as ex:
        results = list(ex.map(compare, ps))
    bad, known_agree, known = [], [], 0
    for p, why in results:
        rel = os.path.relpath(p, PROGRAMS)
        if why is None:
            if rel in KNOWN:
                known_agree.append(rel)
        elif rel in KNOWN:
            known += 1
            if verbose:
                print("known: %s: %s" % (rel, KNOWN[rel]))
        else:
            bad.append((rel, why))
    for rel, why in bad:
        print("DIFFERS: %s: %s" % (rel, why))
    for rel in known_agree:
        print("agrees now, take it out of KNOWN: %s" % rel)
    print("kc-expanded: %d programs, %d agree, %d known to differ, %d differ"
          % (len(results), sum(1 for _, w in results if w is None), known, len(bad)))
    sys.exit(1 if bad or known_agree else 0)


if __name__ == "__main__":
    main()
