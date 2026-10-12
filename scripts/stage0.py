"""Stage 0 (PLAN.md, section 1): Fixpt runs the compiler, kfx/, on a program.

usage: python3 -I scripts/stage0.py FILE.kfx > OUT.rs
       python3 -I scripts/stage0.py PROGRAM.unit DIR
       and the same after --record LOG or --replay LOG (C52)

The compiler's driver is KFX (kfx/main.kfx, C50), its I/O FX-26's (C49):
so Fixpt runs the whole compiler, the prelude first, with `fixpt run`, given
the same arguments the binary kfx is, and reads and writes the same files.
DIR is made first: FX-26 makes no directories.
"""

import os
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def fixpt():
    # The binary built from the pinned commit, FIXPT_REV (scripts/fixpt.sh).
    run = subprocess.run([os.path.join(ROOT, "scripts", "fixpt.sh")], capture_output=True, text=True)
    if run.returncode != 0:
        sys.stderr.write(run.stderr)
        sys.exit(run.returncode)
    return run.stdout.strip()


def manifest(path):
    """A manifest's files, in order."""
    files = []
    with open(path) as f:
        for line in f:
            line = line.strip()
            if line and not line.startswith(";"):
                files.append(os.path.join(os.path.dirname(path), line))
    return files


def compiler():
    """The prelude, then the compiler: what Fixpt runs (kfx/prelude.fx)."""
    return [os.path.join(ROOT, "kfx", "prelude.fx")] + manifest(os.path.join(ROOT, "kfx", "kfx.unit"))


def main():
    args = sys.argv[1:]
    if not args:
        sys.exit("usage: stage0.py [--record LOG | --replay LOG] (FILE.kfx | PROGRAM.unit DIR | --toplevel FILE.kfx)")
    if len(args) >= 2 and args[-2].endswith(".unit"):
        os.makedirs(args[-1], exist_ok=True)
    binary = fixpt()
    # One file, the compiler's forms one after another, as kfx/kfx.unit
    # gives them to FX-26.
    with tempfile.TemporaryDirectory() as tmp:
        whole = os.path.join(tmp, "kfx.fx")
        with open(whole, "w") as f:
            for path in compiler():
                with open(path) as g:
                    f.write(g.read() + "\n")
        result = subprocess.run([binary, "--dialect", "fx26", "run", whole, "--"] + args, capture_output=True, text=True)
    # Fixpt 17ac0c5 writes its notes on how it runs a procedure ("; not in
    # the native convention yet, ...") on the standard output, among the
    # program's own: left out here. No line of generated Rust starts `; `.
    sys.stdout.write("".join(l for l in result.stdout.splitlines(True) if not l.startswith("; ")))
    if result.returncode != 0:
        sys.stderr.write(result.stderr)
        sys.exit("stage0: the compiler, run by Fixpt, ends with status %d" % result.returncode)


main()
