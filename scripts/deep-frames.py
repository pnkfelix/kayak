#!/usr/bin/env python3
"""The frames' census, read (`KAYAK_GC_FRAMES`; crates/kayak-rt/src/obj/frames.rs).

Usage: scripts/deep-frames.py CENSUS [CENSUS ...] [--top N]

Each call site met in 32 frames or more of a collection's stack, named by
the procedure its return address is in (the executable's symbols, by
`nm`), and summed by procedure: in how many collections, its most frames,
its most pages pinned in one collection, and the pages it pinned in all
(what its frames cost the collector, page-collections). Ranked by the
last.
"""

import argparse
import bisect
import collections
import subprocess
import sys


def segments(path):
    """The executable's loadable segments: (file offset, vaddr, size)."""
    out = subprocess.run(["readelf", "-lW", path], capture_output=True, text=True, check=True).stdout
    segs = []
    for line in out.splitlines():
        p = line.split()
        if p and p[0] == "LOAD":
            segs.append((int(p[1], 16), int(p[2], 16), int(p[4], 16)))
    return segs


def symbols(path):
    out = subprocess.run(["nm", "-C", "--defined-only", path], capture_output=True, text=True, check=True).stdout
    syms = []
    for line in out.splitlines():
        p = line.split(None, 2)
        if len(p) == 3 and p[1] in "tTwW":
            syms.append((int(p[0], 16), p[2]))
    syms.sort()
    return syms


class Exe:
    cache = {}

    def __init__(self, path):
        self.segs = segments(path)
        self.syms = symbols(path)
        self.addrs = [a for a, _ in self.syms]

    @classmethod
    def of(cls, path):
        if path not in cls.cache:
            try:
                cls.cache[path] = cls(path)
            except (subprocess.CalledProcessError, FileNotFoundError):
                cls.cache[path] = None
        return cls.cache[path]

    def name(self, off):
        """The procedure at file offset `off`."""
        for fo, va, size in self.segs:
            if fo <= off < fo + size:
                v = off - fo + va
                i = bisect.bisect_right(self.addrs, v) - 1
                return self.syms[i][1] if i >= 0 else "?"
        return "?"


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("census", nargs="+")
    ap.add_argument("--top", type=int, default=25)
    a = ap.parse_args()
    per = collections.defaultdict(lambda: [0, 0, 0, 0])  # collections, frames max, pages max, pages sum
    cols = collections.Counter()
    pinned = collections.Counter()
    for path in a.census:
        maps = {}
        name = {}
        for line in open(path):
            p = line.split()
            if not p:
                continue
            if p[0] == "process":
                maps[p[1]] = []
                cur = p[1]
            elif p[0] == "map":
                lo, hi = (int(x, 16) for x in p[1].split("-"))
                maps[cur].append((lo, hi, int(p[3], 16), p[6] if len(p) > 6 else ""))
            elif p[0] == "collection":
                cur = p[1]
                cols[p[2]] += 1
                pinned[p[2]] += int(p[4])
            elif p[0] == "site":
                ret = int(p[1], 16)
                key = (cur, ret)
                if key not in name:
                    n = "<unmapped>"
                    for lo, hi, off, exe in maps.get(cur, []):
                        if lo <= ret < hi:
                            e = Exe.of(exe)
                            n = e.name(ret - lo + off) if e else exe
                            break
                    name[key] = n
                r = per[name[key]]
                frames, pages = int(p[3]), int(p[7])
                r[0] += 1
                r[1] = max(r[1], frames)
                r[2] = max(r[2], pages)
                r[3] += pages
    print("collections: " + ", ".join(f"{k} {v} (pages pinned in all {pinned[k]})" for k, v in sorted(cols.items())))
    print(f"{'colls':>6} {'frames':>7} {'pages':>6} {'all':>8}  procedure (pages: most in one collection; all: summed)")
    for n, (c, f, pm, ps) in sorted(per.items(), key=lambda x: -x[1][3])[: a.top]:
        print(f"{c:>6} {f:>7} {pm:>6} {ps:>8}  {n[:150]}")


if __name__ == "__main__":
    sys.exit(main())
