"""Fixpt's FX-26 test programs, kept in kayak (C132): tests/fx26/programs/
is Fixpt's crates/fixpt-fx26/tests/programs/ at the pinned commit
(FIXPT_REV), every file of it, so that the scripts that check kayak against
those programs read them here, and need no clone of Fixpt to run them. What
Fixpt says of each (its answers) is kept beside them, in
tests/fixpt-answers/ (scripts/fixptcache.py).

usage: python3 -I scripts/fx26-tests.py [--check]
       python3 -I scripts/fx26-tests.py --reset | --prune

With no option, writes tests/fx26/programs/ from the clone. --check exits 1
if it is not the clone's (check.sh runs it, when there is a clone).
--reset forgets which answers were asked for (check.sh runs it first);
--prune drops the answers no script asked for since, and
says the answers are the pin's (check.sh runs it after a full check, so
that moving the pin and running the check is all it takes).
"""

import os
import shutil
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import fixptcache  # noqa: E402

OUT = os.path.join(ROOT, "tests", "fx26", "programs")
FROM = "crates/fixpt-fx26/tests/programs"


def clone():
    r = subprocess.run([os.path.join(ROOT, "scripts", "fixpt.sh")], capture_output=True, text=True)
    if r.returncode != 0:
        sys.stderr.write(r.stderr)
        sys.exit("fx26-tests: no Fixpt clone at the pin (scripts/fixpt.sh)")
    return os.path.dirname(os.path.dirname(os.path.dirname(r.stdout.strip())))


def theirs(c):
    """The clone's files under FROM, at the pin (its index, not its
    untracked files): each path relative to FROM, and its bytes."""
    rev = open(os.path.join(ROOT, "FIXPT_REV")).read().strip()
    names = subprocess.run(["git", "-C", c, "ls-tree", "-r", "--name-only", rev, FROM + "/"],
                           capture_output=True, text=True, check=True).stdout.split("\n")
    files = {}
    for n in names:
        if n:
            files[os.path.relpath(n, FROM)] = subprocess.run(["git", "-C", c, "show", rev + ":" + n],
                                                             capture_output=True, check=True).stdout
    return files


def ours():
    files = {}
    for d, _, fs in os.walk(OUT):
        for f in fs:
            p = os.path.join(d, f)
            with open(p, "rb") as g:
                files[os.path.relpath(p, OUT)] = g.read()
    return files


def main():
    args = sys.argv[1:]
    if args == ["--reset"]:
        fixptcache.reset_used()
        return
    if args == ["--prune"]:
        dropped = fixptcache.prune()
        print("fx26-tests: Fixpt's answers are the pin's; %d no longer asked for, dropped" % dropped)
        return
    if args not in ([], ["--check"]):
        sys.exit(__doc__)
    want = theirs(clone())
    if args == ["--check"]:
        have = ours()
        if have != want:
            gone = sorted(set(have) - set(want))
            new = sorted(set(want) - set(have))
            changed = sorted(n for n in set(want) & set(have) if want[n] != have[n])
            for what, ns in (("not Fixpt's", gone), ("missing", new), ("changed", changed)):
                for n in ns[:10]:
                    print("fx26-tests: %s: %s" % (what, n))
            sys.exit("fx26-tests: tests/fx26/programs/ is not the pinned Fixpt's (python3 -I scripts/fx26-tests.py)")
        print("fx26-tests: %d files, the pinned Fixpt's" % len(want))
        return
    shutil.rmtree(OUT, ignore_errors=True)
    for n, data in want.items():
        p = os.path.join(OUT, n)
        os.makedirs(os.path.dirname(p), exist_ok=True)
        with open(p, "wb") as g:
            g.write(data)
    print("fx26-tests: %d files written, the pinned Fixpt's" % len(want))


if __name__ == "__main__":
    main()
