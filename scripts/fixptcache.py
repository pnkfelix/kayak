"""What Fixpt says of a file, asked once (C132): what it says depends on the
Fixpt pinned (FIXPT_REV), the arguments and the files' texts alone.

Two stores. tests/fixpt-answers/ is kayak's, in git, so that the checks
need no Fixpt to run (Fixpt's programs are kept in tests/fx26/programs/,
scripts/fx26-tests.py): a file for each question a script asked, NAME.HASH.json,
HASH of the arguments and the texts (not the pin, not the machine), and in
it each distinct answer Fixpt gave, with the machines and pins that gave it
(`by`, each `PLATFORM@PIN`, C223): most questions are answered alike
everywhere, one answer with several keys; where machines differ (libm's
last bit, code a program makes run or not), an answer each. Under it,
target/fixpt-cache/ keeps every run of every pin on this machine (the
feedback loop's time, 2026-10-12), so that asking again costs nothing.

An answer is this machine's at the pin only if its key says so: then it is
used as it is; else, with a clone at the pin, Fixpt is asked, and this
machine's key added to the answer it gave, or a new answer kept (git says
which). With no clone, what is kept stands in: this machine's at another
pin, else one kept before keys were, else any. Each question asked is
noted (target/fixpt-answers-used/), so that after a full check
`fx26-tests.py --prune` drops this machine's keys that nothing asked for,
and its keys at other pins: an answer no machine holds is dropped, and a
file with none. Another machine's keys are its own to drop. Keys
`unknown@PIN` are the answers kept before keys were (C223), the home
machine's (HOME_PLATFORM) as its stamp said: its prune drops them.

A program that reads the world (the clock, files) is not to be run through
this. Imported by the scripts that run Fixpt, with the scripts' directory
put on sys.path first (they run under `python3 -I`)."""

import atexit
import hashlib
import json
import os
import platform
import re
import subprocess
import sys
import threading
from concurrent.futures import ThreadPoolExecutor

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CACHE = os.path.join(ROOT, "target", "fixpt-cache")
ANSWERS = os.path.join(ROOT, "tests", "fixpt-answers")
USED = os.path.join(ROOT, "target", "fixpt-answers-used")
REV = open(os.path.join(ROOT, "FIXPT_REV")).read().strip()
# Fixpt's FX-26 test programs, kept (scripts/fx26-tests.py).
PROGRAMS = os.path.join(ROOT, "tests", "fx26", "programs")


def binary():
    """The Fixpt of the pin (scripts/fixpt.sh), or None when there is no
    clone at the pin: then the answers kept are all there is."""
    r = subprocess.run([os.path.join(ROOT, "scripts", "fixpt.sh")], capture_output=True, text=True)
    return r.stdout.strip() if r.returncode == 0 else None


# Kayak's own FX-26 programs (C229), answered by Fixpt as its own are: for
# what Fixpt's corpus does not exercise of kayak's machines.
KAYAK_PROGRAMS = os.path.join(ROOT, "tests", "fx26-kayak")


def kayak_programs():
    """Kayak's own programs: each as kayak/DIR/NAME.fx, and its path."""
    out = []
    for d, _, fs in os.walk(KAYAK_PROGRAMS):
        for f in fs:
            if f.endswith(".fx"):
                p = os.path.join(d, f)
                out.append(("kayak/" + os.path.relpath(p, KAYAK_PROGRAMS), p))
    return sorted(out)


def programs():
    """Fixpt's programs kept: each as DIR/NAME.fx, and its path."""
    out = []
    for d, _, fs in os.walk(PROGRAMS):
        for f in fs:
            if f.endswith(".fx"):
                p = os.path.join(d, f)
                out.append((os.path.relpath(p, PROGRAMS), p))
    return sorted(out)

# The machine whose answers kayak's are compared with, named: Fixpt's
# default on arm64 (register code on native frames), which on x86_64 is
# not its default (C134): there the default's fixed stacks overflow where
# native frames do not.
ANSWERING = ["--fx26-machine", "fx-regs-rsjit-nativecc"]

# This machine, as Rust's `std::env::consts` names it: `ARCH-OS`.
PLATFORM = "%s-%s" % ({"arm64": "aarch64", "amd64": "x86_64"}.get(platform.machine().lower(), platform.machine().lower()),
                      {"darwin": "macos"}.get(sys.platform, sys.platform))

# Where Fixpt's answers were first kept (C134): NAME.out is its answer on
# arm64 macOS; on another machine, where Fixpt answers otherwise (libm's
# last bit, code a program makes run or not), NAME.PLATFORM.out beside it.
HOME_PLATFORM = "aarch64-macos"


# This machine's architecture, as Rust's `target_arch` names it.
ARCH = PLATFORM.split("-")[0]


def machine_of(paths):
    """The machine a program is of, as Rust's `target_arch` names it, if it
    declares a native convention (`define-convention`), whose `target`
    says which: Fixpt refuses another machine's (C135), so the program is
    compiled, run and checked only on its own. None for any machine."""
    for p in paths:
        with open(p) as f:
            m = re.search(r"\(define-convention\s+\S+\s*\(target\s+([a-z0-9_]+)\)", f.read())
        if m:
            return {"arm64": "aarch64"}.get(m.group(1), m.group(1))
    return None


def answer_path(directory, name):
    """Where program `name`'s answer on this machine is kept: its own file
    if this machine has one, else the shared one."""
    mine = os.path.join(directory, "%s.%s.out" % (name, PLATFORM))
    return mine if os.path.exists(mine) else os.path.join(directory, name + ".out")


class Ran:
    def __init__(self, returncode, stdout, stderr):
        self.returncode, self.stdout, self.stderr = returncode, stdout, stderr


_lock = threading.Lock()
_used = set()
_kept = None
KEY = re.compile(r"[0-9a-f]{20}")
# This machine at the pin, as an answer's `by` names it.
ME = "%s@%s" % (PLATFORM, REV)


def _index():
    """The questions kept, by hash: each file NAME.HASH.json, NAME only to
    be read by people."""
    global _kept
    if _kept is None:
        _kept = {}
        if os.path.isdir(ANSWERS):
            for f in os.listdir(ANSWERS):
                if f.endswith(".json"):
                    parts = f[:-len(".json")].split(".")
                    if KEY.fullmatch(parts[-1]):
                        _kept[parts[-1]] = os.path.join(ANSWERS, f)
    return _kept


def _note_used():
    if _used:
        os.makedirs(USED, exist_ok=True)
        with open(os.path.join(USED, "%d.txt" % os.getpid()), "a") as g:
            g.write("".join(k + "\n" for k in sorted(_used)))


atexit.register(_note_used)


class Unanswered(Exception):
    pass


# Where kayak is, as an answer kept writes it: Fixpt's messages name its
# files by their paths, which are not the same in every checkout (C134).
HERE = "<kayak>"
# And Fixpt's programs, the clone's or kayak's copy of them, the same files
# (fx26-tests.py): as `<programs>`, read back as kayak's copy.
THERE = "<programs>"
COPIES = [HERE + "/../kayak-refs/fixpt/crates/fixpt-fx26/tests/programs", HERE + "/tests/fx26/programs"]


# And a temporary directory a script made for the run (corpus.py's), on
# macOS or Linux: `<tmp>`.
TEMPORARY = re.compile(r"(/private)?(/var/folders/[A-Za-z0-9_+-]+/[A-Za-z0-9_+-]+/T|/tmp)/tmp[a-z0-9_]{8}")


def _placeless(text):
    text = TEMPORARY.sub("<tmp>", text.replace(ROOT, HERE))
    for c in COPIES:
        text = text.replace(c, THERE)
    return text


def _load(path):
    """A question kept: `{"args": …, "answers": [{"by": [KEY …], "code",
    "out", "err"} …]}`, as written (placeless)."""
    with open(path) as g:
        return json.load(g)


def _store(path, d):
    """A question written, each answer's keys sorted, one to a line; a
    question no machine holds an answer to, removed."""
    d["answers"] = [a for a in d["answers"] if a["by"]]
    if not d["answers"]:
        if os.path.exists(path):
            os.remove(path)
        return
    for a in d["answers"]:
        a["by"] = sorted(set(a["by"]))
    text = json.dumps(d, indent=0, ensure_ascii=False) + "\n"
    if os.path.exists(path) and open(path).read() == text:
        return
    os.makedirs(ANSWERS, exist_ok=True)
    tmp = path + ".%d.%d" % (os.getpid(), threading.get_ident())
    with open(tmp, "w") as g:
        g.write(text)
    os.replace(tmp, path)


def _answer(ran):
    """A run as an answer is kept: placeless, its lines apart."""
    return {"code": ran.returncode, "out": _placeless(ran.stdout).split("\n"), "err": _placeless(ran.stderr).split("\n")}


def _ran(args, a):
    """An answer kept, as a run; a run that took too long is kept as one,
    and read back as it ended, by raising."""
    def here(lines):
        return "\n".join(lines).replace(THERE, PROGRAMS).replace(HERE, ROOT)
    if a["code"] == "timeout":
        raise subprocess.TimeoutExpired(args, a["out"][0])
    return Ran(a["code"], here(a["out"]), here(a["err"]))


def _same(a, b):
    return a["code"] == b["code"] and a["out"] == b["out"] and a["err"] == b["err"]


def _standing_in(answers):
    """With no clone: this machine's answer at another pin, else one kept
    before keys were, else any."""
    for prefix in (PLATFORM + "@", "unknown@"):
        for a in answers:
            if any(k.startswith(prefix) for k in a["by"]):
                return a
    return answers[0] if answers else None


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
    shown = [os.path.basename(a) if a in files else a for a in args]
    with _lock:
        _used.add(key)
        path = _index().get(key)
        kept = _load(path) if path else None
    if kept is not None:
        mine = [a for a in kept["answers"] if ME in a["by"]]
        if mine:
            return _ran(args, mine[0])
        if binary is None and _standing_in(kept["answers"]) is not None:
            return _ran(args, _standing_in(kept["answers"]))
    if binary is None:
        raise Unanswered("fixptcache: no answer of Fixpt's kept for %s (a clone at the pin makes it)" % " ".join(shown))
    try:
        ran = _asked(binary, args, old, timeout)
    except subprocess.TimeoutExpired:
        ran = Ran("timeout", str(timeout), "")
    if name is None and files and os.path.abspath(files[0]).startswith(PROGRAMS + os.sep):
        name = os.path.splitext(os.path.relpath(os.path.abspath(files[0]), PROGRAMS))[0].replace(os.sep, "--")
    stem = name or (os.path.splitext(os.path.basename(files[0]))[0] if files else "run")
    new = _answer(ran)
    with _lock:
        path = _index().get(key) or os.path.join(ANSWERS, "%s.%s.json" % (stem, key))
        d = _load(path) if os.path.exists(path) else {"args": shown, "answers": []}
        # This machine's key moves to the answer it gave now.
        for a in d["answers"]:
            a["by"] = [k for k in a["by"] if k != ME]
        same = [a for a in d["answers"] if _same(a, new)]
        if same:
            same[0]["by"].append(ME)
        else:
            new["by"] = [ME]
            d["answers"].append(new)
        _store(path, d)
        _index()[key] = path
    return _ran(args, new)


def prune():
    """This machine's keys that nothing asked for since `reset_used`
    dropped, and its keys at other pins, and, on the home machine, every
    `unknown@` key; an
    answer no machine holds is dropped then, and a question with none:
    how many questions were. Another machine's keys are its own to drop."""
    used = set()
    if os.path.isdir(USED):
        for f in os.listdir(USED):
            with open(os.path.join(USED, f)) as g:
                used.update(line.strip() for line in g if line.strip())
    dropped = 0
    for key, path in list(_index().items()):
        d = _load(path)
        for a in d["answers"]:
            a["by"] = [k for k in a["by"]
                       if not ((k.startswith("unknown@") and PLATFORM == HOME_PLATFORM)
                               or (k.startswith(PLATFORM + "@") and (k != ME or key not in used)))]
        _store(path, d)
        if not os.path.exists(path):
            dropped += 1
    return dropped


def reset_used():
    if os.path.isdir(USED):
        for f in os.listdir(USED):
            os.remove(os.path.join(USED, f))


def blessing():
    """Whether a run is to bless (`--bless`, or `--bless-drop`, which also
    lets it drop programs from a passing list)."""
    return "--bless" in sys.argv[1:] or "--bless-drop" in sys.argv[1:]


def drop_refused(lost, tool):
    """A bless's passing list would lose `lost` (C235): unless the run was
    given `--bless-drop`, each named, and True, so that the caller writes
    nothing and fails; a program that stopped agreeing is then a decision,
    not a side effect. With it, each named as dropped, and False."""
    if not lost:
        return False
    if "--bless-drop" in sys.argv[1:]:
        for n in lost:
            print("%s: DROPPED  %s" % (tool, n))
        return False
    for n in lost:
        print("%s: WOULD DROP  %s" % (tool, n))
    print("%s: --bless would drop %d program(s) from its passing list; nothing written. "
          "Fix them, or bless with --bless-drop to drop them." % (tool, len(lost)))
    return True


def parallel(f, xs):
    """`f` of each of `xs`, as many at once as there are processors; in order."""
    with ThreadPoolExecutor(max_workers=os.cpu_count() or 4) as pool:
        return list(pool.map(f, xs))
