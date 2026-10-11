"""Stage 0 (PLAN.md, section 1): Fixpt runs the compiler, kfx/, on a program.

usage: python3 -I scripts/stage0.py FILE.kfx > OUT.rs
       python3 -I scripts/stage0.py PROGRAM.unit DIR

FX-26 has no file I/O, so the program's text goes in as string literals
after the compiler's own forms (the prelude, then the files kfx/kfx.unit
lists), and the Rust comes out as the last form's value, which
`fixpt eval --raw` prints exactly. A program of several files comes out as
one string, `bundle`'s: a line `@@kfx-file NAME` before each file.
"""

import os
import re
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


def fx_string(text):
    return '"' + text.replace("\\", "\\\\").replace('"', '\\"') + '"'


def manifest(path):
    """A manifest's files, in order: (name, path) each."""
    files = []
    with open(path) as f:
        for line in f:
            line = line.strip()
            if line and not line.startswith(";"):
                p = os.path.join(os.path.dirname(path), line)
                files.append((os.path.splitext(os.path.basename(p))[0], p))
    return files


def read(path):
    with open(path) as f:
        return f.read()


def run(call):
    # The prelude, then the compiler: what Fixpt runs (kfx/prelude.fx).
    text = read(os.path.join(ROOT, "kfx", "prelude.fx")) + "\n"
    for _, path in manifest(os.path.join(ROOT, "kfx", "kfx.unit")):
        text += read(path) + "\n"
    binary = fixpt()
    with tempfile.TemporaryDirectory() as tmp:
        wrapped = os.path.join(tmp, "stage0.fx")
        with open(wrapped, "w") as f:
            f.write(text + call + "\n")
        result = subprocess.run(
            [binary, "--dialect", "fx26", "--step-limit", "none", "eval", "--raw", wrapped],
            capture_output=True,
            text=True,
        )
    if result.returncode != 0:
        sys.stderr.write(result.stderr)
        sys.exit("stage0: fixpt failed")
    return result.stdout


def main():
    args = sys.argv[1:]
    if len(args) == 1:
        sys.stdout.write(run("(compile " + fx_string(read(args[0])) + ")"))
    elif len(args) == 2:
        sources = " ".join(
            "(source " + fx_string(name) + " " + fx_string(read(path)) + ")" for name, path in manifest(args[0])
        )
        out = run("(bundle (compile-units (list " + sources + ")))")
        os.makedirs(args[1], exist_ok=True)
        # A file begins at a line that begins `@@kfx-file `, and at no other.
        parts = re.split(r"(?m)^@@kfx-file ", out)
        if parts[0] != "":
            sys.exit("stage0: the bundle does not begin with a file")
        for part in parts[1:]:
            name, text = part.split("\n", 1)
            with open(os.path.join(args[1], name), "w") as f:
                f.write(text)
    else:
        sys.exit("usage: stage0.py FILE.kfx | stage0.py PROGRAM.unit DIR")


main()
