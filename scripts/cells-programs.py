"""Tier 1 (C122): kayak-cells, Fixpt's cellular machine in KFX, run on the
words Fixpt's own compiler makes, its answers compared with Fixpt's.

usage: python3 -I scripts/cells-programs.py [--bless] [--why]

--why: the reasons programs are not run, counted.

kfx compiles cells/cells.unit; its Rust is kept in
crates/kayak-cells/src/cells (--bless rewrites it; else it must be what
kfx makes). Then, for each of Fixpt's FX-26 test programs, at the pinned
Fixpt: `fixpt --fx26-machine fx-cells-rsinterp compile` lists its words,
kayak-cells runs them, and `fixpt ... eval` says what the answer is. Each
program agrees (the same value, or the same failure), differs (the check
fails), or is not run: its listing is not exact (a literal cut short,
...), it needs what tier 1 has not yet (C122), it leaves no value (a
program of definitions), or Fixpt refuses it. The programs that agree are
kept in tests/cells-passing (--bless rewrites it); one that agreed and
no longer does fails the check. Each but Fixpt's benchmarks is run again
with a collection at every safepoint (`--gc-stress`), to the same end;
and each that runs tasks (processes, `par`, `race`, `pselect`), again
with a time slice of two polls (`--slice 2`, as Fixpt's `FIXPT_SLICE`,
C125), preempted at nearly every poll; with no step limit, as Fixpt's
limit is each form's and tier 1 runs the program as one word. (A slice of
one poll leaves a task none of its own after its grace: Fixpt's run of
`monitors/one-message.fx` so does not end.)
"""

import os
import re
import shutil
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import fixptcache  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
GENERATED = os.path.join(ROOT, "crates", "kayak-cells", "src", "cells")
PASSING = os.path.join(ROOT, "tests", "cells-passing")
MACHINE = ["--fx26-machine", "fx-cells-rsinterp"]
# Fixpt's benchmarks run past FX-26's default step limit: run without one.
UNLIMITED = ("bench/",)


def run(args, **kw):
    return subprocess.run(args, capture_output=True, text=True, **kw)


def generated():
    """kfx's Rust of cells/cells.unit: the files' texts, by name."""
    with tempfile.TemporaryDirectory() as out:
        r = run([os.path.join(ROOT, "target", "release", "kfx"), os.path.join(ROOT, "cells", "cells.unit"), out])
        if r.returncode != 0:
            sys.exit("cells-programs: kfx refuses cells/: " + r.stderr.strip()[-400:])
        return {f: open(os.path.join(out, f), encoding="utf-8").read() for f in sorted(os.listdir(out))}


def programs(clone):
    base = os.path.join(clone, "crates", "fixpt-fx26", "tests", "programs")
    out = []
    for d in sorted(os.listdir(base)):
        full = os.path.join(base, d)
        if os.path.isdir(full):
            for f in sorted(os.listdir(full)):
                if f.endswith(".fx"):
                    out.append((d + "/" + f, os.path.join(full, f)))
    return out


def failure(text):
    """What a failed run says, past Fixpt's and kayak's framing."""
    for l in text.splitlines():
        if "evaluation failed: " in l:
            m = l.split("evaluation failed: ", 1)[1].strip()
            return m[len("cellular word: "):] if m.startswith("cellular word: ") else m
    return None


DEFINING = ("(define", "(define*", "(define-rec", "(define-type", "(define-datatype", "(define-effect",
            "(define-generative", "(define-convention", "(import", "(module-parameters")


def last_defines(path):
    """Whether a program's last top-level form defines (it leaves no value)."""
    text = open(path, encoding="utf-8").read()
    depth, start, last, i = 0, None, "", 0
    while i < len(text):
        c = text[i]
        if c == ";":
            while i < len(text) and text[i] != "\n":
                i += 1
            continue
        if c == '"':
            i += 1
            while i < len(text) and text[i] != '"':
                i += 2 if text[i] == "\\" else 1
            if depth == 0:
                last = '"'
        elif text.startswith("#\\", i):
            i += 3
            continue
        elif c == "(":
            if depth == 0:
                start = i
            depth += 1
        elif c == ")":
            depth -= 1
            if depth == 0:
                last = text[start:i + 1]
        elif depth == 0 and not c.isspace():
            last = c
        i += 1
    return last.startswith(DEFINING)


def one(binary, cells, name, path):
    limit = ["--step-limit", "none"] if name.startswith(UNLIMITED) else []
    ev = fixptcache.run(binary, MACHINE + limit + ["eval", path], [path])
    want_fail = failure(ev.stdout + "\n" + ev.stderr)
    if ev.returncode != 0 and want_fail is None:
        return "fixpt-refuses", ""
    lines = [l for l in ev.stdout.splitlines() if l.strip() and not l.startswith(";")]
    if want_fail is None and (not lines or last_defines(path)):
        return "no-value", ""
    want = None if want_fail is not None else lines[-1].rsplit(" ! ", 1)[0].rsplit(" : ", 1)[0]
    co = fixptcache.run(binary, MACHINE + ["compile", "--full-literals", path], [path])
    text = co.stdout
    if co.returncode != 0 and "; the compilers disagree" in co.stdout:
        # Fixpt 1925bca's `--full-literals` compares its two compilers'
        # listings as text, and one labels literals the other makes twice
        # (flonums): they disagree only so, when the plain listing says both
        # made the same. Then the first, the FX-26 compiler's (C123).
        plain = fixptcache.run(binary, MACHINE + ["compile", path], [path])
        if plain.returncode == 0 and "; both compilers made this" in plain.stdout:
            text = co.stdout.split("; the Rust compiler:", 1)[0]
        else:
            return "fixpt-refuses", ""
    elif co.returncode != 0:
        return "fixpt-refuses", ""
    with tempfile.NamedTemporaryFile("w", suffix=".cells", delete=False) as f:
        f.write(text)
        listing = f.name
    try:
        got = run([cells, listing] + (["--step-limit", "none"] if limit else []), timeout=600)
    finally:
        os.unlink(listing)
    if "the listing: " in got.stderr:
        return "refused", got.stderr.split("the listing: ", 1)[1].strip().splitlines()[0]
    got_fail = failure(got.stderr)
    if got_fail is not None and "(C122)" in got_fail:
        return "not-yet", got_fail
    if got_fail == "the word left nothing":
        return "no-value", ""
    if got.returncode != 0 and got_fail is None:
        return "differ", "kayak-cells failed: " + got.stderr.strip()[-200:]
    if want_fail is not None or got_fail is not None:
        if want_fail != got_fail:
            return "differ", "fixpt: %r, kayak: %r" % (want_fail, got_fail)
        return stressed(cells, name, path, text, limit, "", got_fail)
    have = got.stdout.strip()
    if have != want:
        return "differ", "fixpt: %r, kayak: %r" % (want, have)
    return stressed(cells, name, path, text, limit, have, None)


TASKS = re.compile(r"\((spawn|spawn-monitored|par|race|pselect|send|receive|icell-get|icell-put!)\b")


def stressed(cells, name, path, text, limit, have, fail):
    """The same answer with a collection at every safepoint (C122, Coopt's
    stress policy), and, for a program that runs tasks, preempted at every
    poll (C125); Fixpt's benchmarks, long, are not run so."""
    if name.startswith(UNLIMITED):
        return "agree", ""
    ways = [["--gc-stress"]]
    if TASKS.search(open(path, encoding="utf-8").read()):
        ways.append(["--slice", "2", "--step-limit", "none"])
    with tempfile.NamedTemporaryFile("w", suffix=".cells", delete=False) as f:
        f.write(text)
        listing = f.name
    try:
        for way in ways:
            again = run([cells, listing] + way, timeout=600)
            if failure(again.stderr) != fail or (fail is None and again.stdout.strip() != have):
                return "differ", "under %s: %r %r" % (" ".join(way), again.stdout.strip(), failure(again.stderr))
    finally:
        os.unlink(listing)
    return "agree", ""


def main():
    bless = "--bless" in sys.argv[1:]
    made = generated()
    kept = {f: open(os.path.join(GENERATED, f), encoding="utf-8").read() for f in os.listdir(GENERATED)} if os.path.isdir(GENERATED) else {}
    if made != kept:
        if not bless:
            sys.exit("cells-programs: kfx makes other Rust of cells/ (--bless, if it should)")
        shutil.rmtree(GENERATED, ignore_errors=True)
        os.makedirs(GENERATED)
        for f, text in made.items():
            with open(os.path.join(GENERATED, f), "w", encoding="utf-8") as g:
                g.write(text)
    r = run(["cargo", "build", "--release", "--quiet", "-p", "kayak-cells"], cwd=ROOT)
    if r.returncode != 0:
        sys.exit("cells-programs: kayak-cells does not build:\n" + r.stderr[-2000:])
    cells = os.path.join(ROOT, "target", "release", "kayak-cells")
    fp = run([os.path.join(ROOT, "scripts", "fixpt.sh")])
    if fp.returncode != 0:
        print("cells-programs: kayak-cells built; no Fixpt clone at the pin, so no programs run")
        return
    binary = fp.stdout.strip()
    clone = os.path.dirname(os.path.dirname(os.path.dirname(binary)))
    ps = programs(clone)
    ran = fixptcache.parallel(lambda p: one(binary, cells, p[0], p[1]), ps)
    by = {}
    wrong = []
    for (name, _), (kind, why) in zip(ps, ran):
        by.setdefault(kind, []).append(name)
        if kind == "differ":
            wrong.append("%s: %s" % (name, why))
    agree = sorted(by.get("agree", []))
    before = open(PASSING).read().split() if os.path.exists(PASSING) else []
    lost = [n for n in before if n not in agree]
    if bless:
        with open(PASSING, "w") as f:
            f.write("\n".join(agree) + "\n")
    if "--why" in sys.argv[1:]:
        reasons = {}
        for (name, _), (kind, why) in zip(ps, ran):
            if kind in ("refused", "not-yet"):
                reasons[(kind, why)] = reasons.get((kind, why), 0) + 1
        for (kind, why), n in sorted(reasons.items(), key=lambda x: -x[1]):
            print("%5d  %s: %s" % (n, kind, why[:150]))
    for w in wrong:
        print("DIFFER  " + w)
    if lost and not bless:
        for n in lost:
            print("NO LONGER AGREES  " + n)
    print("cells-programs: %d of Fixpt's programs agree on tier 1, %d differ; not run: %d listings not exact, %d need what tier 1 has not yet, %d leave no value, %d Fixpt refuses"
          % (len(agree), len(wrong), len(by.get("refused", [])), len(by.get("not-yet", [])), len(by.get("no-value", [])), len(by.get("fixpt-refuses", []))))
    if wrong or (lost and not bless):
        sys.exit(1)


if __name__ == "__main__":
    main()
