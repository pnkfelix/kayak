"""FX-26's engine (docs/research/engine.md, C58) on the corpus: K6.

usage: python3 -I scripts/engine-programs.py [--bless]

The engine's Rust is made from fx26/fx26.unit by kfx, and kept in
crates/kayak-fx26/src/fx26 (golden: --bless rewrites it). Built, it runs
each corpus program (tests/kfx/NAME.kfx or NAME.unit), then its `main`,
and its answer is compared with NAME.out, Fixpt's. Programs the engine
cannot run yet are counted, not failed; those it ran rightly are kept in
tests/engine-passing (--bless rewrites it), and one of those that no
longer does fails the script.
"""

import os
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import fixptcache  # noqa: E402

ROOT =os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CORPUS = os.path.join(ROOT, "tests", "kfx")
GENERATED = os.path.join(ROOT, "crates", "kayak-fx26", "src", "fx26")
PASSING = os.path.join(ROOT, "tests", "engine-passing")


def run(args, **kw):
    return subprocess.run(args, capture_output=True, **kw)


def manifest(path):
    files = []
    with open(path) as f:
        for line in f:
            line = line.strip()
            if line and not line.startswith(";"):
                files.append(os.path.join(os.path.dirname(path), line))
    return files


def read(path):
    with open(path, "rb") as f:
        return f.read()


def main():
    bless = "--bless" in sys.argv[1:]
    if run(["cargo", "build", "--release", "--quiet", "-p", "kfx"], cwd=ROOT).returncode != 0:
        sys.exit("engine-programs: kfx does not build")
    with tempfile.TemporaryDirectory() as out:
        r = run([os.path.join(ROOT, "target", "release", "kfx"), os.path.join(ROOT, "fx26", "fx26.unit"), out])
        if r.returncode != 0:
            sys.stderr.write(r.stderr.decode())
            sys.exit("engine-programs: kfx refuses the engine")
        made = {f: read(os.path.join(out, f)) for f in os.listdir(out)}
    kept = {f: read(os.path.join(GENERATED, f)) for f in os.listdir(GENERATED)} if os.path.isdir(GENERATED) else {}
    if made != kept:
        if not bless:
            sys.exit("engine-programs: kfx makes other Rust of the engine (--bless, if it should)")
        os.makedirs(GENERATED, exist_ok=True)
        for f in kept:
            if f not in made:
                os.remove(os.path.join(GENERATED, f))
        for f, text in made.items():
            with open(os.path.join(GENERATED, f), "wb") as g:
                g.write(text)
    if run(["cargo", "build", "--release", "--quiet", "-p", "kayak-fx26"], cwd=ROOT).returncode != 0:
        sys.exit("engine-programs: the engine does not build")
    engine = os.path.join(ROOT, "target", "release", "kayak-fx26")
    names = sorted(n.rsplit(".", 1)[0] for n in os.listdir(CORPUS) if n.endswith(".kfx") or n.endswith(".unit"))
    agree, differ, cannot, elsewhere = [], [], [], set()
    for name in names:
        unit = os.path.join(CORPUS, name + ".unit")
        files = manifest(unit) if os.path.exists(unit) else [os.path.join(CORPUS, name + ".kfx")]
        if fixptcache.machine_of(files) not in (None, fixptcache.ARCH):
            # Another machine's (C135): what it was there, kept.
            elsewhere.add(name)
            continue
        expected = fixptcache.answer_path(CORPUS, name)
        if not os.path.exists(expected):
            continue
        try:
            r = run([engine, "--main"] + files, timeout=60)
        except subprocess.TimeoutExpired:
            cannot.append((name, "a minute passed"))
            continue
        if r.returncode != 0:
            said = [l for l in r.stderr.decode(errors="replace").splitlines() if "kfx:" in l or "kayak-fx26:" in l or "engine" in l]
            cannot.append((name, (said or r.stderr.decode(errors="replace").strip().splitlines()[-1:] or ["?"])[0]))
        elif r.stdout == read(expected).rstrip(b"\n") or r.stdout == read(expected):
            agree.append(name)
        else:
            differ.append((name, r.stdout[:120]))
    for name, why in cannot:
        print(f"engine-programs: {name}: cannot yet: {why[:150]}")
    for name, got in differ:
        print(f"engine-programs: {name}: DIFFERS: {got!r}")
    was = set(open(PASSING).read().split()) if os.path.exists(PASSING) else set()
    lost = sorted(was - set(agree) - elsewhere)
    if bless:
        with open(PASSING, "w") as f:
            f.write("".join(n + "\n" for n in sorted(set(agree) | (was & elsewhere))))
    print(f"engine-programs: {len(agree)} of {len(names)} corpus programs agree with Fixpt; {len(differ)} differ, {len(cannot)} not run yet")
    if differ or (lost and not bless):
        sys.exit("engine-programs: " + ("differ: " + ", ".join(n for n, _ in differ) if differ else "no longer agree: " + ", ".join(lost)))


main()
