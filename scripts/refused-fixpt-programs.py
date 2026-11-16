"""K1's other half on Fixpt's own programs: kfx refuses every FX-26 program
in Fixpt's crates/fixpt-fx26/tests/programs that Fixpt's checker refuses
(C81). Exits 1, naming them, if kfx accepts any.

usage: python3 -I scripts/refused-fixpt-programs.py
"""

import os
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import fixptcache  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def main():
    fixpt = fixptcache.binary()
    progs = fixptcache.PROGRAMS
    files = [p for _, p in fixptcache.programs()]
    kfx = os.path.join(ROOT, "target", "release", "kfx")

    def refused_by_fixpt(p):
        """Fixpt's checker refuses `p`: it fails, or says an error (a line
        begun `! `), which `fixpt check` does with status 0."""
        r = fixptcache.run(fixpt, ["--dialect", "fx26", "check", p], [p], timeout=60)
        return r.returncode != 0 or any(l.startswith("! ") for l in r.stdout.splitlines() + r.stderr.splitlines())

    engine = os.path.join(ROOT, "target", "release", "kayak-fx26")

    def accepted_wrongly(p):
        if not refused_by_fixpt(p):
            return None
        k = subprocess.run([kfx, p], capture_output=True, text=True, timeout=60)
        if k.returncode == 0:
            return os.path.relpath(p, progs)
        # A program that defines a macro, which kfx refuses as KFX: the
        # engine's expander and checker refuse it too (C431), checked and
        # compiled, not run (`--cells`).
        with open(p) as f:
            if "define-syntax" in f.read():
                e = subprocess.run([engine, "--cells", p], capture_output=True, text=True, timeout=60,
                                   cwd=os.path.dirname(p))
                if e.returncode == 0:
                    return os.path.relpath(p, progs) + " (the engine)"
        return None

    wrong = [x for x in fixptcache.parallel(accepted_wrongly, files) if x]
    refused = sum(1 for p in files if refused_by_fixpt(p))
    if wrong:
        for w in wrong:
            print("refused-fixpt-programs: KFX ACCEPTS WHAT FX-26 REFUSES  " + w)
        sys.exit(1)
    print("refused-fixpt-programs: of Fixpt's %d programs its checker refuses, kfx refuses every one" % refused)


if __name__ == "__main__":
    main()
