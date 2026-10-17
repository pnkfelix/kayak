"""What Fixpt says of a file, asked once (C132): what it says depends on the
Fixpt pinned (FIXPT_REV), the arguments and the files' texts alone.

Two stores. tests/fixpt-answers/ is kayak's, in git: each answer a scripts
asked for, under a hash of the arguments and the texts (not the pin), as
Fixpt at tests/fixpt-answers/REV gave it, so that the checks need no Fixpt
to run: Fixpt's programs are kept in tests/fx26/programs/
(scripts/fx26-tests.py), and these are their answers. Under it,
target/fixpt-cache/ keeps every run of every pin (the feedback loop's time,
2026-10-12), so that going back to an old pin costs nothing.

An answer kept is used as it is, unless there is a clone at the pin and the
answers are another pin's: then Fixpt is asked again, and the answer kept
rewritten where it differs (git says which). Each answer asked for is noted
(target/fixpt-answers-used/), so that after a full check `fx26-tests.py
--prune` drops those no longer asked for and stamps the pin.

A program that reads the world (the clock, files) is not to be run through
this. Imported by the scripts that run Fixpt, with the scripts' directory
put on sys.path first (they run under `python3 -I`)."""

import atexit
import hashlib
import json
import os
import subprocess
import threading
from concurrent.futures import ThreadPoolExecutor

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CACHE = os.path.join(ROOT, "target", "fixpt-cache")
ANSWERS = os.path.join(ROOT, "tests", "fixpt-answers")
USED = os.path.join(ROOT, "target", "fixpt-answers-used")
REV = open(os.path.join(ROOT, "FIXPT_REV")).read().strip()
STAMP = os.path.join(ANSWERS, "REV")
# Fixpt's FX-26 test programs, kept (scripts/fx26-tests.py).
PROGRAMS = os.path.join(ROOT, "tests", "fx26", "programs")


def binary():
    """The Fixpt of the pin (scripts/fixpt.sh), or None when there is no
    clone at the pin: then the answers kept are all there is."""
    r = subprocess.run([os.path.join(ROOT, "scripts", "fixpt.sh")], capture_output=True, text=True)
    return r.stdout.strip() if r.returncode == 0 else None


def programs():
    """Fixpt's programs kept: each as DIR/NAME.fx, and its path."""
    out = []
    for d, _, fs in os.walk(PROGRAMS):
        for f in fs:
            if f.endswith(".fx"):
                p = os.path.join(d, f)
                out.append((os.path.relpath(p, PROGRAMS), p))
    return sorted(out)


class Ran:
    def __init__(self, returncode, stdout, stderr):
        self.returncode, self.stdout, self.stderr = returncode, stdout, stderr


_lock = threading.Lock()
_used = set()
_kept = None


def _stamp():
    return open(STAMP).read().strip() if os.path.exists(STAMP) else None


def _index():
    """The answers kept, by hash: each file is NAME.HASH.json, NAME only
    to be read by people."""
    global _kept
    if _kept is None:
        _kept = {}
        if os.path.isdir(ANSWERS):
            for f in os.listdir(ANSWERS):
                if f.endswith(".json"):
                    _kept[f.rsplit(".", 2)[1]] = os.path.join(ANSWERS, f)
    return _kept


def _note_used():
    if _used:
        os.makedirs(USED, exist_ok=True)
        with open(os.path.join(USED, "%d.txt" % os.getpid()), "a") as g:
            g.write("".join(k + "\n" for k in sorted(_used)))


atexit.register(_note_used)


class Unanswered(Exception):
    pass


def _read(path):
    """An answer kept; a run that took too long is kept as one, and read
    back as it ended, by raising."""
    with open(path) as g:
        d = json.load(g)
    if d["code"] == "timeout":
        raise subprocess.TimeoutExpired(d["args"], d["out"][0])
    return Ran(d["code"], "\n".join(d["out"]), "\n".join(d["err"]))


def _write(path, args, ran):
    d = {"args": args, "code": ran.returncode, "out": ran.stdout.split("\n"), "err": ran.stderr.split("\n")}
    text = json.dumps(d, indent=0, ensure_ascii=False) + "\n"
    if os.path.exists(path) and open(path).read() == text:
        return
    os.makedirs(ANSWERS, exist_ok=True)
    tmp = path + ".%d.%d" % (os.getpid(), threading.get_ident())
    with open(tmp, "w") as g:
        g.write(text)
    os.replace(tmp, path)


def _asked(binary, args, old, timeout):
    """Fixpt asked: from target/fixpt-cache/ if this pin has said it (its
    key the pin's hash, then the same as the answer's)."""
    path = os.path.join(CACHE, old.hexdigest() + ".json")
    if os.path.exists(path):
        with open(path) as g:
            d = json.load(g)
        return Ran(d["code"], d["out"], d["err"])
    r = subprocess.run([binary] + args, capture_output=True, text=True, timeout=timeout, stdin=subprocess.DEVNULL)
    os.makedirs(CACHE, exist_ok=True)
    tmp = path + ".%d.%d" % (os.getpid(), threading.get_ident())
    with open(tmp, "w") as g:
        json.dump({"code": r.returncode, "out": r.stdout, "err": r.stderr}, g)
    os.replace(tmp, path)
    return Ran(r.returncode, r.stdout, r.stderr)


def run(binary, args, files, timeout=None, name=None, keep=True):
    """`binary` given `args`, which name `files` (read for the key): its
    status and output, as kept, or asked of Fixpt. `binary` is None when
    there is no clone at the pin: then only what is kept, and Unanswered
    raised if nothing is. `name`, for people reading tests/fixpt-answers/: the
    first file's, by default. A run past `timeout` raises, as
    subprocess.run does. With `keep` false, nothing is kept in tests/:
    for a script that keeps Fixpt's answers its own way (corpus.py's
    NAME.out)."""
    h = hashlib.sha256()
    old = hashlib.sha256()
    old.update(REV.encode())
    for a in args:
        part = b"\0" + (os.path.basename(a) if a in files else a).encode()
        h.update(part)
        old.update(part)
    for f in files:
        with open(f, "rb") as g:
            part = b"\1" + g.read()
        h.update(part)
        old.update(part)
    key = h.hexdigest()[:20]
    if not keep:
        if binary is None:
            raise Unanswered("fixptcache: Fixpt asked with no clone at the pin")
        ran = _asked(binary, args, old, timeout)
        return ran
    with _lock:
        _used.add(key)
        kept = _index().get(key)
    if kept is not None and (binary is None or _stamp() == REV):
        return _read(kept)
    shown = [os.path.basename(a) if a in files else a for a in args]
    if binary is None:
        raise Unanswered("fixptcache: no answer of Fixpt's kept for %s (a clone at the pin makes it)" % " ".join(shown))
    try:
        ran = _asked(binary, args, old, timeout)
    except subprocess.TimeoutExpired:
        ran = Ran("timeout", str(timeout), "")
    if name is None and files and os.path.abspath(files[0]).startswith(PROGRAMS + os.sep):
        name = os.path.splitext(os.path.relpath(os.path.abspath(files[0]), PROGRAMS))[0].replace(os.sep, "--")
    stem = name or (os.path.splitext(os.path.basename(files[0]))[0] if files else "run")
    path = kept or os.path.join(ANSWERS, "%s.%s.json" % (stem, key))
    _write(path, shown, ran)
    with _lock:
        _index()[key] = path
    return _read(path)


def prune():
    """Drop the answers kept that nothing asked for since `reset_used`,
    and stamp the rest as the pin's: how many were dropped."""
    used = set()
    if os.path.isdir(USED):
        for f in os.listdir(USED):
            with open(os.path.join(USED, f)) as g:
                used.update(line.strip() for line in g if line.strip())
    dropped = 0
    for key, path in list(_index().items()):
        if key not in used:
            os.remove(path)
            dropped += 1
    os.makedirs(ANSWERS, exist_ok=True)
    with open(STAMP, "w") as g:
        g.write(REV + "\n")
    return dropped


def reset_used():
    if os.path.isdir(USED):
        for f in os.listdir(USED):
            os.remove(os.path.join(USED, f))


def parallel(f, xs):
    """`f` of each of `xs`, as many at once as there are processors; in order."""
    with ThreadPoolExecutor(max_workers=os.cpu_count() or 4) as pool:
        return list(pool.map(f, xs))
