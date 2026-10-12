"""The incremental compile checked against a compile from scratch (C54).

usage: python3 -I scripts/incremental.py [--all]

A copy of the compiler's sources (kfx/, and lib/ beside it) is edited, an
edit at a time, and after each the copy is compiled twice: with the memos
kept from the compiles before (`kfx --cache FILE`), and with none. The two
must be the same, file for file. Each edit keeps the program one kfx
accepts; together they move forms (an insertion before them), change a
form's text and its facts, add definitions, a datatype (the context), and
take them away again; and across files, a form whose text is the same but
whose type is found from a definition that changes. With --all, every edit in every file; by default, a
few files, for scripts/check.sh.
"""

import os
import shutil
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
KFX = os.path.join(ROOT, "target", "release", "kfx")

EDITS = [
    ("a comment first, moving every form after it", lambda s: '"An edit, first."\n' + s),
    ("a definition last", lambda s: s + '\n(define incr-probe (subr (maxeff (read @globals) spin) () i64) (lambda () (i64+ 1 2)))\n'),
    ("its body changed, and its facts", lambda s: s.replace("(i64+ 1 2)", "(i64* 1 (i64+ 2 3))")),
    ("a datatype last, in the context", lambda s: s + '\n(define-datatype incr-shape (incr-round i64) (incr-square i64 i64))\n'),
    ("the comment first taken away", lambda s: s.replace('"An edit, first."\n', "", 1)),
    ("the definitions taken away", lambda s: s.split("\n(define incr-probe")[0] + "\n"),
]


def run(args):
    r = subprocess.run(args, capture_output=True, text=True)
    if r.returncode != 0:
        sys.stderr.write(r.stderr)
        sys.exit("incremental: kfx fails: " + " ".join(args))


def files(d):
    return {f: open(os.path.join(d, f)).read() for f in sorted(os.listdir(d))}


def main():
    every = "--all" in sys.argv[1:]
    r = subprocess.run(["cargo", "build", "--release", "--quiet", "-p", "kfx"], cwd=ROOT)
    if r.returncode != 0:
        sys.exit("incremental: kfx does not build")
    with tempfile.TemporaryDirectory() as tmp:
        shutil.copytree(os.path.join(ROOT, "kfx"), os.path.join(tmp, "kfx"))
        shutil.copytree(os.path.join(ROOT, "lib"), os.path.join(tmp, "lib"))
        unit = os.path.join(tmp, "kfx", "kfx.unit")
        cache = os.path.join(tmp, "cache")
        names = [l.strip() for l in open(unit) if l.strip() and not l.startswith(";")]
        chosen = names if every else [names[0], names[len(names) // 2], names[-1]]
        out = os.path.join(tmp, "out")
        count = 0

        def compare(what):
            nonlocal count
            for d in ("warm", "cold"):
                shutil.rmtree(os.path.join(tmp, d), ignore_errors=True)
                os.makedirs(os.path.join(tmp, d))
            run([KFX, "--cache", cache, unit, os.path.join(tmp, "warm")])
            run([KFX, unit, os.path.join(tmp, "cold")])
            warm, cold = files(os.path.join(tmp, "warm")), files(os.path.join(tmp, "cold"))
            if warm != cold:
                differ = [f for f in cold if warm.get(f) != cold[f]]
                sys.exit(f"incremental: after {what}, the compile with memos differs in {', '.join(differ)}")
            count += 1

        compare("nothing")
        for name in chosen:
            path = os.path.join(tmp, "kfx", name)
            for what, edit in EDITS:
                with open(path) as f:
                    text = f.read()
                with open(path, "w") as f:
                    f.write(edit(text))
                compare(f"{name}: {what}")
        # Across files: a later form's text the same, what it reads changed.
        first, last = os.path.join(tmp, "kfx", names[0]), os.path.join(tmp, "kfx", names[-1])
        steps = [
            (first, "a definition, read later", lambda s: s + "\n(define incr-given (subr (maxeff (read @globals) spin) () i64) (lambda () 7))\n"),
            (last, "a definition whose type is found from it", lambda s: s + "\n(define incr-use (lambda () (incr-given)))\n"),
            (first, "the first's type changed", lambda s: s.replace("() i64) (lambda () 7))", "() i32) (lambda () 7))")),
            (first, "its body changed, its type the same", lambda s: s.replace("(lambda () 7))", "(lambda () 8))")),
        ]
        for path, what, edit in steps:
            with open(path) as f:
                text = f.read()
            with open(path, "w") as f:
                f.write(edit(text))
            compare(what)
        print(f"incremental: {count} compiles with memos, each as one from scratch")


main()
