"""A check's job run only when its inputs change (C552; the user: "Let's go
with 'Run it only when its inputs change'"): the job's key is a hash of
what it reads, as declared; a pass is kept under its key
(target/check-memo/NAME/KEY.log, its output), and a job whose key has
passed is not run again: its kept output is said instead, marked. A job
that fails keeps nothing.

usage: python3 -I scripts/memo.py NAME [--path P]... [--except P]...
           [--bin B]... [--crate C]... [--keeps D]... -- COMMAND...

  --path P   a file or directory of the tree the job reads (every file
             under it, by name and contents)
  --except P a path under a --path the job does not read, left out
  --bin B    a binary of target/release/ the job runs (its contents: what
             it was built of, its crates and theirs, all in it)
  --crate C  a package of the workspace the job builds or tests: its
             sources and those of every package of this tree it depends
             on, by `cargo metadata`, and Cargo.lock (C554)
  --keeps D  a directory the job writes what a later stage reads
             (target/kayak-checks, the summaries'; target/fixpt-answers-used,
             the prune's): what a pass added or changed there kept beside
             its output, and put back when the job is not run (C554)

The key also has the job's name, its command, and this script. With
KAYAK_CHECK_ALL=1 in the environment (`scripts/check.sh --all`) every job
runs, and a pass is kept as ever; one that fails where a pass under the
same key was kept says so: its declared inputs are not all it reads.

What a job reads must be all declared: one left out is a change it does
not see. Declare more rather than less (a directory, not its file)."""

import hashlib
import json
import os
import shutil
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
STORE = os.path.join(ROOT, "target", "check-memo")


def hashed_path(h, rel, excepts=()):
    """Every file under `rel`, by its name and contents, in order, but
    those under `excepts`; a path not there, said so."""
    full = os.path.join(ROOT, rel)
    if os.path.isfile(full):
        files = [full]
    elif os.path.isdir(full):
        files = []
        for d, ds, fs in os.walk(full):
            ds.sort()
            files += [os.path.join(d, f) for f in sorted(fs)]
    else:
        h.update(b"\0missing\0" + rel.encode())
        return
    for f in files:
        r = os.path.relpath(f, ROOT)
        if any(r == e or r.startswith(e.rstrip("/") + "/") for e in excepts):
            continue
        h.update(b"\0file\0" + r.encode() + b"\0")
        with open(f, "rb") as g:
            for block in iter(lambda: g.read(1 << 20), b""):
                h.update(block)


def crate_dirs(names):
    """The source directories of packages `names` and of every package of
    this tree they depend on, normal, build or dev, transitively."""
    meta = json.loads(subprocess.run(["cargo", "metadata", "--format-version", "1", "--offline"],
                                     cwd=ROOT, capture_output=True, text=True, check=True).stdout)
    by_id = {p["id"]: p for p in meta["packages"]}
    by_name = {p["name"]: p for p in meta["packages"] if p["source"] is None}
    deps = {n["id"]: [d["pkg"] for d in n["deps"]] for n in meta["resolve"]["nodes"]}
    todo = [by_name[n]["id"] for n in names]
    seen = set()
    while todo:
        i = todo.pop()
        if i in seen or by_id[i]["source"] is not None:
            continue
        seen.add(i)
        todo += deps.get(i, [])
    return sorted(os.path.relpath(os.path.dirname(by_id[i]["manifest_path"]), ROOT) for i in seen)


def listing(d):
    """Each file under `d`, by its path, and its size and time."""
    out = {}
    for base, ds, fs in os.walk(os.path.join(ROOT, d)):
        for f in fs:
            full = os.path.join(base, f)
            st = os.stat(full)
            out[os.path.relpath(full, ROOT)] = (st.st_size, st.st_mtime_ns)
    return out


def main():
    args = sys.argv[1:]
    if not args or "--" not in args:
        sys.exit(__doc__)
    name, rest = args[0], args[1:]
    cut = rest.index("--")
    opts, command = rest[:cut], rest[cut + 1:]
    paths, excepts, bins, crates, keeps = [], [], [], [], []
    lists = {"--path": paths, "--except": excepts, "--bin": bins, "--crate": crates, "--keeps": keeps}
    while opts:
        flag, value = opts[0], opts[1] if len(opts) > 1 else None
        if flag not in lists or not value:
            sys.exit(__doc__)
        lists[flag].append(value)
        opts = opts[2:]
    if not command:
        sys.exit(__doc__)
    h = hashlib.sha256()
    h.update(b"memo 1\0" + name.encode() + b"\0" + b"\0".join(c.encode() for c in command))
    hashed_path(h, os.path.relpath(os.path.abspath(__file__), ROOT))
    for p in sorted(paths):
        hashed_path(h, p, excepts)
    for b in sorted(bins):
        hashed_path(h, os.path.join("target", "release", b))
    if crates:
        for d in crate_dirs(crates):
            hashed_path(h, d)
        hashed_path(h, "Cargo.lock")
    for k in sorted(keeps):
        h.update(b"\0keeps\0" + k.encode())
    key = h.hexdigest()[:24]
    kept = os.path.join(STORE, name, key + ".log")
    files = os.path.join(STORE, name, key + ".keeps")
    everything = os.environ.get("KAYAK_CHECK_ALL") == "1"
    if os.path.exists(kept) and not everything:
        # What the pass wrote for later stages, put back.
        if os.path.isdir(files):
            for base, ds, fs in os.walk(files):
                for f in fs:
                    src = os.path.join(base, f)
                    dst = os.path.join(ROOT, os.path.relpath(src, files))
                    os.makedirs(os.path.dirname(dst), exist_ok=True)
                    shutil.copy2(src, dst)
        sys.stdout.write(open(kept).read())
        print("memo: %s not run, its inputs as when it passed (%s)" % (name, key))
        return 0
    before = {}
    for k in keeps:
        before.update(listing(k))
    r = subprocess.run(command, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    sys.stdout.write(r.stdout)
    if r.returncode != 0:
        if os.path.exists(kept):
            print("memo: %s failed where a pass under the same key (%s) was kept: "
                  "its declared inputs are not all it reads" % (name, key))
        return r.returncode
    os.makedirs(os.path.dirname(kept), exist_ok=True)
    shutil.rmtree(files, ignore_errors=True)
    for k in keeps:
        for rel, stamp in listing(k).items():
            if before.get(rel) != stamp:
                dst = os.path.join(files, rel)
                os.makedirs(os.path.dirname(dst), exist_ok=True)
                shutil.copy2(os.path.join(ROOT, rel), dst)
    tmp = kept + ".%d" % os.getpid()
    with open(tmp, "w") as g:
        g.write(r.stdout)
    os.replace(tmp, kept)
    print("memo: %s passed, kept (%s)" % (name, key))
    return 0


if __name__ == "__main__":
    sys.exit(main())
