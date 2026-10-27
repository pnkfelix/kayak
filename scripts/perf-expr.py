"""`perf report --sort srcline` of a run with `KAYAK_JITDUMP=expr`, its
lines read back as FX-26 expressions (C247).

With `KAYAK_JITDUMP=expr`, kayak-rt names in the jitdump's debug-info
records, for each source file, a side file `/tmp/jit-PID.NAME.expr`:
its first N lines the source's own, then a line

    ;;; kayak expr (C247): the N lines above are PATH's; line N + 2 + P below, ...

then line N + 2 + P for each character position P, `LINE:COLUMN  TEXT`.
A record's line is N + 2 + the position where an expression's code
begins, so a line past N + 1 is a position: this script reads each back
as `PATH:LINE:COLUMN` and the expression's text. With `--lines`, the
expressions of each source line summed, as plain `KAYAK_JITDUMP=1`'s
report would give.

usage: python3 -I scripts/perf-expr.py PERF_DATA [--lines] [-n COUNT]

PERF_DATA is `perf inject --jit`'s output, e.g.

    KAYAK_JITDUMP=expr perf record -k 1 kayak-cells prog.cells --compiled
    perf inject --jit -i perf.data -o perf.jit.data
    python3 -I scripts/perf-expr.py perf.jit.data
"""

import os
import re
import subprocess
import sys

HEADER = re.compile(r";;; kayak expr \(C247\): the (\d+) lines above are (.*)'s; line")
ROW = re.compile(r"^\s*([\d.]+)%\s+(.*?)\s*$")

sides = {}


def side(path):
    """A side file's N, its source's path, and its lines; None if it is not one."""
    if path not in sides:
        sides[path] = None
        if path.endswith(".expr"):
            try:
                with open(path, encoding="utf-8") as f:
                    lines = f.read().split("\n")
            except OSError:
                return None
            for l in lines:
                m = HEADER.match(l)
                if m:
                    sides[path] = (int(m.group(1)), short(m.group(2)), lines)
                    break
    return sides[path]


def short(path):
    """`path` relative to here, if it is under here."""
    rel = os.path.relpath(path)
    return path if rel.startswith("..") else rel


def decoded(srcline):
    """(place, text, (source, line)) of a srcline `PATH:N`; else (srcline, "", None)."""
    path, _, n = srcline.rpartition(":")
    if not n.isdigit():
        return srcline, "", None
    n = int(n)
    s = side(path)
    if s is None:
        return srcline, "", (path, n)
    count, source, lines = s
    if n <= count:
        return "%s:%d" % (source, n), lines[n - 1].strip(), (source, n)
    if n == count + 1 or n > len(lines):
        return srcline, "", None
    at, _, text = lines[n - 1].partition("  ")
    line = int(at.split(":")[0]) if at else 0
    return "%s:%s" % (source, at), text, (source, line)


def main(argv):
    if not argv or argv[0] in ("-h", "--help"):
        print(__doc__)
        return 0 if argv else 2
    data, by_line, count = argv[0], "--lines" in argv, 25
    if "-n" in argv:
        count = int(argv[argv.index("-n") + 1])
    report = subprocess.run(
        ["perf", "report", "-i", data, "--stdio", "--sort", "srcline", "--full-source-path"],
        capture_output=True, text=True)
    if report.returncode != 0:
        sys.stderr.write(report.stderr)
        return 1
    rows = []
    for l in report.stdout.splitlines():
        m = ROW.match(l)
        if m and not l.lstrip().startswith("#"):
            rows.append((float(m.group(1)), m.group(2)))
    if by_line:
        sums = {}
        for pct, srcline in rows:
            key = decoded(srcline)[2] or (srcline, 0)
            sums[key] = sums.get(key, 0.0) + pct
        # Each source's own lines, from its side file's first N.
        own = {v[1]: v[2][:v[0]] for v in sides.values() if v}
        for (source, line), pct in sorted(sums.items(), key=lambda kv: -kv[1])[:count]:
            text = own[source][line - 1].strip() if source in own and 0 < line <= len(own[source]) else ""
            print("%7.2f%%  %s  %s" % (pct, "%s:%d" % (source, line) if line else source, text[:70]))
    else:
        for pct, srcline in rows[:count]:
            place, text, _ = decoded(srcline)
            print("%7.2f%%  %s  %s" % (pct, place, text))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
