"""What has changed upstream in the Fixpt sources kayak's files were
ported from (C121): each such file says so in a line of its own,

    fixpt-source: PATH@REV

in whatever comment its language has (`;;`, `//`, `#`), PATH a file of
Fixpt's and REV the Fixpt commit it was ported at. For each, the commits
between REV and the pin (FIXPT_REV) that touched PATH. Moving the pin is
done when each is carried over, or decided against (a note in
CLAUDES_CHOICES.md), and the line's REV moved to the pin.

Files Fixpt's are copied whole (scripts/built-in.py) say nothing: they
are the pin's by construction.

usage: python3 -I scripts/fixpt-drift.py [--check]

--check: exits 1 if any ported file is behind the pin (check.sh runs it,
when there is a clone).
"""

import os
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
MARK = re.compile(r"fixpt-source: (\S+)@([0-9a-f]{7,40})\b")


def git(clone, *args):
    return subprocess.run(["git", "-C", clone] + list(args), capture_output=True, text=True)


def marks():
    """Each ported file's sources: (file, line, path, rev)."""
    files = subprocess.run(["git", "-C", ROOT, "ls-files"], capture_output=True, text=True).stdout.split()
    out = []
    for f in files:
        if f == "scripts/fixpt-drift.py" or not f.endswith((".kfx", ".fx", ".rs", ".py", ".sh", ".md")):
            continue
        try:
            text = open(os.path.join(ROOT, f), encoding="utf-8").read()
        except (UnicodeDecodeError, FileNotFoundError):
            continue
        for n, line in enumerate(text.splitlines(), 1):
            for m in MARK.finditer(line):
                out.append((f, n, m.group(1), m.group(2)))
    return out


def main():
    check = sys.argv[1:] == ["--check"]
    if sys.argv[1:] not in ([], ["--check"]):
        sys.exit("usage: fixpt-drift.py [--check]")
    clone = os.environ.get("KAYAK_FIXPT_CLONE", os.path.join(ROOT, "..", "kayak-refs", "fixpt"))
    pin = open(os.path.join(ROOT, "FIXPT_REV")).read().strip()
    behind = 0
    found = marks()
    for f, n, path, rev in found:
        if git(clone, "cat-file", "-e", rev + "^{commit}").returncode != 0:
            print("%s:%d: %s@%s: no such Fixpt commit" % (f, n, path, rev))
            behind += 1
            continue
        if git(clone, "merge-base", "--is-ancestor", rev, pin).returncode != 0:
            print("%s:%d: %s@%s: not before the pin %s" % (f, n, path, rev, pin[:7]))
            behind += 1
            continue
        if git(clone, "cat-file", "-e", "%s:%s" % (pin, path)).returncode != 0:
            print("%s:%d: %s: gone at the pin %s (moved or removed)" % (f, n, path, pin[:7]))
            behind += 1
            continue
        log = git(clone, "log", "--oneline", "%s..%s" % (rev, pin), "--", path).stdout.strip()
        if log:
            behind += 1
            print("%s:%d: %s, ported at %s, changed since:" % (f, n, path, rev[:7]))
            for l in log.splitlines():
                print("    " + l)
    if behind:
        print("fixpt-drift: %d of %d ported source(s) behind the pin %s" % (behind, len(found), pin[:7]))
        if check:
            sys.exit(1)
    else:
        print("fixpt-drift: %d ported source(s), each at the pin %s" % (len(found), pin[:7]))


if __name__ == "__main__":
    main()
