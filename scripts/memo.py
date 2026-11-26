"""A check's job run only when its inputs change (C552; the user: "Let's go
with 'Run it only when its inputs change'"): the job's key is a hash of
what it reads, as declared; a pass is kept under its key
(target/check-memo/NAME/KEY.log, its output), and a job whose key has
passed is not run again: its kept output is said instead, marked. A job
that fails keeps nothing.

usage: python3 -I scripts/memo.py NAME [--path P]... [--bin B]... -- COMMAND...

  --path P   a file or directory of the tree the job reads (every file
             under it, by name and contents)
  --bin B    a binary of target/release/ the job runs (its contents: what
             it was built of, its crates and theirs, all in it)

The key also has the job's name, its command, and this script. With
KAYAK_CHECK_ALL=1 in the environment (`scripts/check.sh --all`) every job
runs, and a pass is kept as ever; one that fails where a pass under the
same key was kept says so: its declared inputs are not all it reads.

What a job reads must be all declared: one left out is a change it does
not see. Declare more rather than less (a directory, not its file)."""

import hashlib
import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
STORE = os.path.join(ROOT, "target", "check-memo")


def hashed_path(h, rel):
    """Every file under `rel`, by its name and contents, in order; a path
    not there, said so."""
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
        h.update(b"\0file\0" + os.path.relpath(f, ROOT).encode() + b"\0")
        with open(f, "rb") as g:
            for block in iter(lambda: g.read(1 << 20), b""):
                h.update(block)


def main():
    args = sys.argv[1:]
    if not args or "--" not in args:
        sys.exit(__doc__)
    name, rest = args[0], args[1:]
    cut = rest.index("--")
    opts, command = rest[:cut], rest[cut + 1:]
    paths, bins = [], []
    while opts:
        flag, value = opts[0], opts[1] if len(opts) > 1 else None
        if flag == "--path" and value:
            paths.append(value)
        elif flag == "--bin" and value:
            bins.append(value)
        else:
            sys.exit(__doc__)
        opts = opts[2:]
    if not command:
        sys.exit(__doc__)
    h = hashlib.sha256()
    h.update(b"memo 1\0" + name.encode() + b"\0" + b"\0".join(c.encode() for c in command))
    hashed_path(h, os.path.relpath(os.path.abspath(__file__), ROOT))
    for p in sorted(paths):
        hashed_path(h, p)
    for b in sorted(bins):
        hashed_path(h, os.path.join("target", "release", b))
    key = h.hexdigest()[:24]
    kept = os.path.join(STORE, name, key + ".log")
    everything = os.environ.get("KAYAK_CHECK_ALL") == "1"
    if os.path.exists(kept) and not everything:
        sys.stdout.write(open(kept).read())
        print("memo: %s not run, its inputs as when it passed (%s)" % (name, key))
        return 0
    r = subprocess.run(command, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    sys.stdout.write(r.stdout)
    if r.returncode != 0:
        if os.path.exists(kept):
            print("memo: %s failed where a pass under the same key (%s) was kept: "
                  "its declared inputs are not all it reads" % (name, key))
        return r.returncode
    os.makedirs(os.path.dirname(kept), exist_ok=True)
    tmp = kept + ".%d" % os.getpid()
    with open(tmp, "w") as g:
        g.write(r.stdout)
    os.replace(tmp, kept)
    print("memo: %s passed, kept (%s)" % (name, key))
    return 0


if __name__ == "__main__":
    sys.exit(main())
