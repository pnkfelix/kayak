"""The live REPL's responsiveness while an entry computes (C99): on a
terminal of its own (a pty), it defines a factorial, enters (fact 30000),
whose steps are slow ones on bignums, and then, while that runs, types a
form a key at a time, measuring how long each takes to be drawn; enters
one, and types and enters another, measuring how long their values take.
By the clock, so not part of scripts/check.sh.

usage: python3 -I scripts/repl-latency.py
"""

import os
import pty
import re
import select
import subprocess
import sys
import time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ESCAPES = re.compile(rb"\x1b\[[0-9;?]*[A-Za-z]")


def main():
    r = subprocess.run(["cargo", "build", "--release", "--quiet", "-p", "kayak-fx26"], cwd=ROOT)
    if r.returncode != 0:
        sys.exit(1)
    engine = os.path.join(ROOT, "target", "release", "kayak-fx26")
    pid, fd = pty.fork()
    if pid == 0:
        os.environ["HOME"] = "/nonexistent"  # no history kept
        os.execv(engine, [engine])
    seen = b""

    def until(pattern, limit=60.0):
        """Milliseconds until the screen's text so far shows `pattern`."""
        nonlocal seen
        began = time.time()
        while not re.search(pattern, ESCAPES.sub(b"", seen)):
            ready, _, _ = select.select([fd], [], [], limit)
            if not ready:
                sys.exit(f"repl-latency: no {pattern!r} after {limit} s")
            seen += os.read(fd, 65536)
        return (time.time() - began) * 1000

    def type_(text):
        nonlocal seen
        seen = b""
        os.write(fd, text.encode())

    try:
        until(rb"fx26> ")
        type_("(define fact (letrec ((f (subr pure (int) int)"
              " (lambda (n) (if (< n 1) 1 (* n (f (- n 1))))))) f))\r")
        until(rb"\[1\] fact")
        type_("(fact 30000)\r")
        until(rb"\[2\]")
        time.sleep(0.5)
        typed, waits = "", []
        for c in "(+ 1 2)":
            typed += c
            type_(c)
            waits.append(until(re.escape(("fx26> " + typed).encode())))
        print("repl-latency: each key drawn in %.0f ms at most (%s)"
              % (max(waits), " ".join("%.0f" % w for w in waits)))
        type_("\r")
        print("repl-latency: (+ 1 2) entered, its value in %.0f ms" % until(rb"\[3\] 3"))
        type_("(fact 1000)")
        print("repl-latency: (fact 1000) typed, its early value in %.0f ms" % until(rb"= 4023"))
        type_("\r")
        print("repl-latency: (fact 1000) entered, its value in %.0f ms" % until(rb"\[4\] 4023"))
        print("repl-latency: (fact 30000) still running:", b"running" in seen)
    finally:
        os.kill(pid, 9)


if __name__ == "__main__":
    main()
