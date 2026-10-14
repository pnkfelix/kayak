"""The REPL's sessions (docs/research/repl.md): each tests/repl/NAME.keys,
keys at virtual times, run by `kayak-fx26 --keys`, its bytes as a 24 by
80 terminal is sent them NAME.out; and each NAME.in, piped (plain mode),
its output NAME.out. --bless rewrites the .out files.

With --rust-repl, the REPL is C103's, kayak-fx26 built with its
`rust-repl` feature (in target/rust-repl/), to compare the two (C111): the
sessions are the same files, and each must give the same bytes.

usage: python3 -I scripts/repl-sessions.py [--bless] [--rust-repl]
"""

import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DIR = os.path.join(ROOT, "tests", "repl")


def main():
    flags = sys.argv[1:]
    bless = "--bless" in flags
    rust = "--rust-repl" in flags
    if any(f not in ("--bless", "--rust-repl") for f in flags):
        sys.exit("usage: repl-sessions.py [--bless] [--rust-repl]")
    target = os.path.join(ROOT, "target", "rust-repl") if rust else os.path.join(ROOT, "target")
    build = ["cargo", "build", "--release", "--quiet", "-p", "kayak-fx26", "--target-dir", target]
    r = subprocess.run(build + (["--features", "rust-repl"] if rust else []), cwd=ROOT)
    if r.returncode != 0:
        sys.exit("repl-sessions: the engine does not build")
    engine = os.path.join(target, "release", "kayak-fx26")
    names = sorted(f for f in os.listdir(DIR) if f.endswith(".keys") or f.endswith(".in"))
    failed = []
    for f in names:
        path = os.path.join(DIR, f)
        if f.endswith(".keys"):
            r = subprocess.run([engine, "--keys", path], capture_output=True, timeout=120, cwd=ROOT, stdin=subprocess.DEVNULL)
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
    print("repl-sessions: %d sessions %s%s" % (len(names), "blessed" if bless else "as before", ", C103's REPL" if rust else ""))


main()
