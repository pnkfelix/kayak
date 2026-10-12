"""Fixpt's runs, cached (the feedback loop's time, 2026-10-12): what Fixpt
says of a file depends on the Fixpt pinned (FIXPT_REV), the arguments and
the file's text alone, so each run is kept under a hash of those, in
target/fixpt-cache/. A program that reads the world (the clock, files)
is not to be run through this.

Imported by the scripts that run Fixpt, with the scripts' directory put
on sys.path first (they run under `python3 -I`)."""

import hashlib
import json
import os
import subprocess
from concurrent.futures import ThreadPoolExecutor

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CACHE = os.path.join(ROOT, "target", "fixpt-cache")
REV = open(os.path.join(ROOT, "FIXPT_REV")).read().strip()


class Ran:
    def __init__(self, returncode, stdout, stderr):
        self.returncode, self.stdout, self.stderr = returncode, stdout, stderr


def run(binary, args, files, timeout=None):
    """`binary` given `args`, which name `files` (read for the key): its
    status and output, from the cache if this Fixpt has said them."""
    h = hashlib.sha256()
    h.update(REV.encode())
    for a in args:
        h.update(b"\0" + (os.path.basename(a) if a in files else a).encode())
    for f in files:
        with open(f, "rb") as g:
            h.update(b"\1" + g.read())
    path = os.path.join(CACHE, h.hexdigest() + ".json")
    if os.path.exists(path):
        with open(path) as g:
            d = json.load(g)
        return Ran(d["code"], d["out"], d["err"])
    r = subprocess.run([binary] + args, capture_output=True, text=True, timeout=timeout)
    os.makedirs(CACHE, exist_ok=True)
    tmp = path + ".%d" % os.getpid()
    with open(tmp, "w") as g:
        json.dump({"code": r.returncode, "out": r.stdout, "err": r.stderr}, g)
    os.replace(tmp, path)
    return Ran(r.returncode, r.stdout, r.stderr)


def parallel(f, xs):
    """`f` of each of `xs`, as many at once as there are processors; in order."""
    with ThreadPoolExecutor(max_workers=os.cpu_count() or 4) as pool:
        return list(pool.map(f, xs))
