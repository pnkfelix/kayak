"""kayak's models (docs/research/models.md), checked by Quint's simulator:
each model's fixed instance must keep its invariant over many random runs,
and its unfixed instance, the engine as it was before the bugs the model
stands for were fixed, must be found wanting (so that the model is shown
to see them).

usage: python3 -I scripts/models.py

Quint is pinned in tools/quint, installed there (`npm ci`) if it is not;
if that fails, the models are not checked, and the script says so. Exhaustive checking (`quint verify`)
is not run here: it needs a Java of 17 or more for Apalache.
"""

import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
QUINT = os.path.join(ROOT, "tools", "quint")

# (file, invariant, instance that must hold, instance that must fail)
MODELS = [
    ("models/db.qnt", "sound", "fixed", "unfixed"),
]


def simulate(path, main, invariant):
    r = subprocess.run(
        ["npx", "quint", "run", os.path.join(ROOT, path), "--main=" + main, "--init=init", "--step=step",
         "--invariant=" + invariant, "--max-steps=30", "--max-samples=10000", "--seed=20261112"],
        cwd=QUINT, capture_output=True, text=True)
    return r.returncode == 0, (r.stdout + r.stderr).strip().splitlines()[-1:]


def main():
    if not os.path.isdir(os.path.join(QUINT, "node_modules", "@informalsystems", "quint")):
        # The lockfile's version, from npm's cache where it can (a snapshot
        # of the tree has no node_modules).
        subprocess.run(["npm", "ci", "--prefer-offline", "--no-fund", "--no-audit"], cwd=QUINT, capture_output=True)
    if not os.path.isdir(os.path.join(QUINT, "node_modules", "@informalsystems", "quint")):
        print("models: no Quint in tools/quint (npm ci there failed), so the models are not checked")
        return
    wrong = []
    for path, inv, good, bad in MODELS:
        ok, said = simulate(path, good, inv)
        if not ok:
            wrong.append("%s: %s breaks %s: %s" % (path, good, inv, said))
        ok, said = simulate(path, bad, inv)
        if ok:
            wrong.append("%s: %s keeps %s, which it should not: the model no longer sees the bug" % (path, bad, inv))
    for w in wrong:
        print("models: " + w)
    print("models: %d model(s), %d wrong" % (len(MODELS), len(wrong)))
    sys.exit(1 if wrong else 0)


main()
