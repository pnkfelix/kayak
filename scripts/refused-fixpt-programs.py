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
    fixpt = subprocess.run([os.path.join(ROOT, "scripts", "fixpt.sh")], capture_output=True, text=True).stdout.strip()
    progs = os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(fixpt))), "crates", "fixpt-fx26", "tests", "programs")
    files = sorted(os.path.join(d, f) for d, _, fs in os.walk(progs) for f in fs if f.endswith(".fx"))
    kfx = os.path.join(ROOT, "target", "release", "kfx")

    def accepted_wrongly(p):
        if fixptcache.run(fixpt, ["--dialect", "fx26", "check", p], [p], timeout=60).returncode == 0:
            return None
        k = subprocess.run([kfx, p], capture_output=True, text=True, timeout=60)
        return os.path.relpath(p, progs) if k.returncode == 0 else None

    wrong = [x for x in fixptcache.parallel(accepted_wrongly, files) if x]
    refused = sum(1 for p in files if fixptcache.run(fixpt, ["--dialect", "fx26", "check", p], [p]).returncode != 0)
    if wrong:
        for w in wrong:
            print("refused-fixpt-programs: KFX ACCEPTS WHAT FX-26 REFUSES  " + w)
        sys.exit(1)
    print("refused-fixpt-programs: of Fixpt's %d programs its checker refuses, kfx refuses every one" % refused)


if __name__ == "__main__":
    main()
