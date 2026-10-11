"""Stage 0 (PLAN.md, section 1): Fixpt runs kfx/kfx.kfx on a KFX file.

usage: python3 -I scripts/stage0.py FILE.kfx > OUT.rs

FX-26 has no file I/O, so the program's text goes in as a string literal,
`(compile "...")` after the compiler's own forms, and the Rust comes out as
that form's value, which `fixpt eval --raw` prints exactly.
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


def fx_string(text):
    return '"' + text.replace("\\", "\\\\").replace('"', '\\"') + '"'


def main():
    [path] = sys.argv[1:]
    # The prelude, then the compiler: what Fixpt runs (kfx/prelude.fx).
    compiler = ""
    for name in ("prelude.fx", "kfx.kfx"):
        with open(os.path.join(ROOT, "kfx", name)) as f:
            compiler += f.read() + "\n"
    with open(path) as f:
        program = f.read()
    binary = fixpt()
    with tempfile.TemporaryDirectory() as tmp:
        wrapped = os.path.join(tmp, "stage0.fx")
        with open(wrapped, "w") as f:
            f.write(compiler + "\n(compile " + fx_string(program) + ")\n")
        run = subprocess.run(
            [binary, "--dialect", "fx26", "--step-limit", "none", "eval", "--raw", wrapped],
            capture_output=True,
            text=True,
        )
    if run.returncode != 0:
        sys.stderr.write(run.stderr)
        sys.exit("stage0: fixpt failed")
    sys.stdout.write(run.stdout)


main()
