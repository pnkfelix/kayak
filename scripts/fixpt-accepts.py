"""Whether Fixpt accepts kayak's KFX (C254): KFX is a sublanguage of
FX-26, so each KFX program kayak accepts, Fixpt's checkers must accept too
(the user, 2026-10-27: "surely any KFX program accepted by Kayak should
likewise be accepted by Fixpt?"). For each unit (`kfx/`, the compiler;
`fx26/`, the engine; `cells/`, tier 1), the prelude Fixpt runs KFX with
(`kfx/prelude.fx`) and the unit's files, in order, as one program, given
to `fixpt check` (`--syscalls` for a runtime's own); a refusal said at its
file and line.

usage: python3 -I scripts/fixpt-accepts.py [UNIT ...]

Exits 1 if Fixpt refuses any, 3 if there is no Fixpt clone (as
`scripts/fixpt.sh`).
"""

import os
import re
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
UNITS = ["kfx/kfx.unit", "fx26/fx26.unit", "cells/cells.unit"]


def fixpt():
    run = subprocess.run([os.path.join(ROOT, "scripts", "fixpt.sh")], capture_output=True, text=True)
    if run.returncode != 0:
        sys.stderr.write(run.stderr)
        sys.exit(run.returncode)
    return run.stdout.strip()


def unit_files(unit):
    """A unit's files, in order, and whether it is a runtime's own."""
    files, syscalls = [], False
    base = os.path.dirname(os.path.join(ROOT, unit))
    for line in open(os.path.join(ROOT, unit)):
        line = line.strip()
        if not line or line.startswith(";"):
            continue
        if line == "--syscalls":
            syscalls = True
            continue
        files.append(os.path.normpath(os.path.join(base, line)))
    return files, syscalls


def check(binary, unit):
    files, syscalls = unit_files(unit)
    parts = [os.path.join(ROOT, "kfx", "prelude.fx")] + files
    starts, text, line = [], "", 1
    for p in parts:
        body = open(p, encoding="utf-8").read()
        if not body.endswith("\n"):
            body += "\n"
        starts.append((line, p))
        text += body
        line += body.count("\n")
    with tempfile.NamedTemporaryFile("w", suffix=".fx", delete=False) as f:
        f.write(text)
        whole = f.name
    try:
        run = subprocess.run([binary] + (["--syscalls"] if syscalls else []) + ["check", whole],
                             capture_output=True, text=True, timeout=1800)
    finally:
        os.unlink(whole)
    if run.returncode == 0:
        return None
    said = run.stdout + run.stderr
    m = re.search(re.escape(whole) + r":(\d+):(\d+): (.*)", said)
    if not m:
        return said.strip().splitlines()[-1] if said.strip() else "refused, saying nothing"
    at = int(m.group(1))
    first, path = max((s for s in starts if s[0] <= at), key=lambda s: s[0])
    return "%s:%d:%s: %s" % (os.path.relpath(path, ROOT), at - first + 1, m.group(2), m.group(3))


def main():
    units = sys.argv[1:] or UNITS
    binary = fixpt()
    refused = 0
    for unit in units:
        why = check(binary, unit)
        if why is None:
            print("fixpt-accepts: %s: Fixpt accepts it" % unit)
        else:
            refused += 1
            print("fixpt-accepts: %s: Fixpt refuses it: %s" % (unit, why))
    sys.exit(1 if refused else 0)


if __name__ == "__main__":
    main()
