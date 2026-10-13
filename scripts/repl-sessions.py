"""The REPL's sessions (docs/research/repl.md): each tests/repl/NAME.keys,
keys at virtual times, run by `kayak-fx26 --keys`, its bytes as a 24 by
80 terminal is sent them NAME.out; and each NAME.in, piped (plain mode),
its output NAME.out. --bless rewrites the .out files.

usage: python3 -I scripts/repl-sessions.py [--bless]
"""

import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DIR = os.path.join(ROOT, "tests", "repl")


def main():
    bless = sys.argv[1:] == ["--bless"]
    if sys.argv[1:] not in ([], ["--bless"]):
        sys.exit("usage: repl-sessions.py [--bless]")
    r = subprocess.run(["cargo", "build", "--release", "--quiet", "-p", "kayak-fx26"], cwd=ROOT)
    if r.returncode != 0:
        sys.exit("repl-sessions: the engine does not build")
    engine = os.path.join(ROOT, "target", "release", "kayak-fx26")
    names = sorted(f for f in os.listdir(DIR) if f.endswith(".keys") or f.endswith(".in"))
    failed = []
    for f in names:
        path = os.path.join(DIR, f)
        if f.endswith(".keys"):
            r = subprocess.run([engine, "--keys", path], capture_output=True, timeout=120, cwd=ROOT)
        else:
            with open(path, "rb") as given:
                r = subprocess.run([engine, "repl"], stdin=given, capture_output=True, timeout=120, cwd=ROOT)
        out = os.path.join(DIR, os.path.splitext(f)[0] + ".out")
        if r.returncode != 0:
            failed.append(f"{f}: status {r.returncode}: {r.stderr.decode(errors='replace')[-300:]}")
            continue
        if bless:
            with open(out, "wb") as g:
                g.write(r.stdout)
        elif not os.path.exists(out) or open(out, "rb").read() != r.stdout:
            failed.append(f"{f}: not as {os.path.basename(out)}")
    for line in failed:
        print("repl-sessions: " + line)
    if failed:
        sys.exit(1)
    print("repl-sessions: %d sessions %s" % (len(names), "blessed" if bless else "as before"))


main()
