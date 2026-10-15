"""kayak's checker against Fixpt's, form by form (TODO §30, PLAN §9): each
of Fixpt's FX-26 programs given to `fixpt check` and to `kayak-fx26
--check`, which writes the same lines (`define NAME : TYPE ! EFFECT`, an
expression's `TYPE ! EFFECT`), and the two compared. A reference must
accept what Fixpt accepts, refuse what it refuses, and say the same
types and effects.

Each program is counted once:
  same       both accept, every line the same;
  alike      both accept, the lines the same once each effect's atoms and
             each `(globals ...)`'s names are sorted (Fixpt sorts them);
  differ     both accept, a line differs even so;
  refused    Fixpt accepts, kayak refuses: what kayak's checker lacks;
  ACCEPTED   Fixpt refuses, kayak accepts: a K1 breach, always reported;
  both       both refuse (their messages not compared yet).
Fixpt's runs are cached (fixptcache.py).

usage: python3 -I scripts/compare-checkers.py [--list KIND] [--dir DIR] [--why] [--summary]
  --list KIND  the programs of KIND, with the first line that differs or
               kayak's refusal
  --dir DIR    only the programs under Fixpt's tests/programs/DIR
  --why        kayak's refusals of what Fixpt accepts, counted by message
  --summary    the last line alone (scripts/check.sh)
Exits 1 if kayak accepts a program Fixpt refuses.
"""

import collections
import os
import re
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import fixptcache  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ENGINE = os.path.join(ROOT, "target", "release", "kayak-fx26")
KINDS = ["same", "alike", "differ", "refused", "ACCEPTED", "both"]


def fixpt_binary():
    r = subprocess.run([os.path.join(ROOT, "scripts", "fixpt.sh")], capture_output=True, text=True)
    if r.returncode != 0:
        sys.exit("compare-checkers: no Fixpt clone (scripts/fixpt.sh)")
    return r.stdout.strip()


def fixpt_says(fixpt, path):
    """Fixpt's lines, or None if it refuses (or fails, or takes too long)."""
    try:
        r = fixptcache.run(fixpt, ["--dialect", "fx26", "check", path], [path], timeout=60)
    except subprocess.TimeoutExpired:
        return None
    lines = r.stdout.splitlines()
    if r.returncode != 0 or any(l.startswith("! ") for l in lines):
        return None
    return [l for l in lines if l and not l.startswith(";")]


def kayak_says(path):
    """kayak's lines, or its refusal (a string)."""
    try:
        r = subprocess.run([ENGINE, "--check", path], capture_output=True, text=True, timeout=60,
                           stdin=subprocess.DEVNULL)
    except subprocess.TimeoutExpired:
        return "takes more than 60 s"
    if r.returncode != 0:
        said = [l for l in r.stderr.splitlines() if l.startswith("kfx:") or l.startswith("kayak-fx26:")]
        return said[0] if said else (r.stderr.strip().splitlines() or ["fails"])[-1]
    return [l for l in r.stdout.splitlines() if l]


def sorted_effects(line):
    """`line` with every `(maxeff ...)`'s atoms and `(globals ...)`'s names
    sorted, as Fixpt writes them."""

    def atoms(text):
        out, depth, start = [], 0, None
        for i, c in enumerate(text):
            if c == "(":
                if depth == 0:
                    start = i
                depth += 1
            elif c == ")":
                depth -= 1
                if depth == 0:
                    out.append(text[start:i + 1])
            elif c == " " and depth == 0:
                if start is not None and start < i and not text[start:i].startswith("("):
                    out.append(text[start:i])
                start = i + 1
            elif depth == 0 and start is None:
                start = i
        if start is not None and start < len(text) and not text[start:].startswith("(") and text[start:].strip():
            out.append(text[start:].strip())
        return out

    def fix(text):
        i = 0
        out = []
        while i < len(text):
            for head in ("(maxeff ", "(globals "):
                if text.startswith(head, i):
                    depth, j = 0, i
                    while True:
                        if text[j] == "(":
                            depth += 1
                        elif text[j] == ")":
                            depth -= 1
                            if depth == 0:
                                break
                        j += 1
                    inner = fix(text[i + len(head):j])
                    out.append(head + " ".join(sorted(atoms(inner))) + ")")
                    i = j + 1
                    break
            else:
                out.append(text[i])
                i += 1
        return "".join(out)

    return fix(line)


def compare(fixpt, path):
    f = fixpt_says(fixpt, path)
    k = kayak_says(path)
    if f is None:
        return ("both", "") if isinstance(k, str) else ("ACCEPTED", "kayak accepts what Fixpt refuses")
    if isinstance(k, str):
        return "refused", k
    if f == k:
        return "same", ""
    if [sorted_effects(l) for l in f] == [sorted_effects(l) for l in k]:
        return "alike", ""
    for i in range(max(len(f), len(k))):
        a = f[i] if i < len(f) else "(nothing)"
        b = k[i] if i < len(k) else "(nothing)"
        if sorted_effects(a) != sorted_effects(b):
            return "differ", "Fixpt: %s\n      kayak: %s" % (a, b)
    return "differ", "the lines differ"


def why(message):
    """A refusal's kind: its place and particulars dropped."""
    m = re.sub(r"^(kfx|kayak-fx26): ", "", message)
    m = re.sub(r"^\S+:\d+: ", "", m)
    m = re.sub(r"^in [^:]+: ", "", m)
    m = re.sub(r"`[^`]*`", "`…`", m)
    m = re.sub(r"named \S+", "named X", m)
    m = re.sub(r": .*", "", m)
    return m[:90]


def main():
    args = sys.argv[1:]
    listing, only, whys, summary = None, None, False, False
    while args:
        if args[0] == "--list" and len(args) > 1 and args[1] in KINDS:
            listing, args = args[1], args[2:]
        elif args[0] == "--dir" and len(args) > 1:
            only, args = args[1], args[2:]
        elif args[0] == "--why":
            whys, args = True, args[1:]
        elif args[0] == "--summary":
            summary, args = True, args[1:]
        else:
            sys.exit(__doc__)
    r = subprocess.run(["cargo", "build", "--release", "--quiet", "-p", "kayak-fx26"], cwd=ROOT)
    if r.returncode != 0:
        sys.exit("compare-checkers: the engine does not build")
    fixpt = fixpt_binary()
    programs = os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(fixpt))),
                            "crates", "fixpt-fx26", "tests", "programs")
    files = sorted(os.path.join(d, f) for d, _, fs in os.walk(programs) for f in fs if f.endswith(".fx"))
    if only:
        files = [p for p in files if os.path.relpath(p, programs).startswith(only.rstrip("/") + "/")]
    results = fixptcache.parallel(lambda p: (p, compare(fixpt, p)), files)
    counts = collections.Counter(kind for _, (kind, _) in results)
    by_dir = collections.defaultdict(collections.Counter)
    for p, (kind, _) in results:
        by_dir[os.path.relpath(p, programs).split(os.sep)[0]][kind] += 1
    if listing:
        for p, (kind, detail) in results:
            if kind == listing:
                print(os.path.relpath(p, programs) + (":\n      " + detail if detail else ""))
    if whys:
        c = collections.Counter(why(d) for _, (kind, d) in results if kind == "refused")
        for m, n in c.most_common():
            print("%4d  %s" % (n, m))
    if not listing and not whys and not summary:
        print("%-16s" % "directory" + "".join("%9s" % k for k in KINDS))
        for d in sorted(by_dir):
            print("%-16s" % d + "".join("%9s" % (by_dir[d][k] or "") for k in KINDS))
    accepted_by_fixpt = sum(counts[k] for k in ("same", "alike", "differ", "refused"))
    print("compare-checkers: of %d programs Fixpt accepts, kayak's checker says the same of %d, alike of %d, "
          "differs on %d, refuses %d; of %d Fixpt refuses, kayak refuses %d, accepts %d"
          % (accepted_by_fixpt, counts["same"], counts["alike"], counts["differ"], counts["refused"],
             counts["both"] + counts["ACCEPTED"], counts["both"], counts["ACCEPTED"]))
    if counts["ACCEPTED"]:
        for p, (kind, _) in results:
            if kind == "ACCEPTED":
                print("compare-checkers: K1: kayak accepts " + os.path.relpath(p, programs))
        sys.exit(1)


main()
