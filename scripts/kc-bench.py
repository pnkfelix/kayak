"""kc against Fixpt (docs/research/kc.md; the user, 2026-11-13: "start
benchmarking it relative to Fixpt, especially when it comes to
responsiveness to incremental changes to code ... make such evaluations
part of our workflow now", and: "make sure that we cover edits that
introduce errors and then undo them, as well as introducing an error
followed by fixing forward ... I want to check that we are leveraging
incremental compilation cache in smart way, where we don't throw things
away too safely").

usage: python3 -I scripts/kc-bench.py [--sizes N,...] [--runs R] [--record | --rows FILE]
       python3 -I scripts/kc-bench.py --tag FILE COMMIT

A program of N procedures is made (each a small arithmetic procedure, the
last expression calling a few), then put through a session of edits, each
timed end to end as a person at the keyboard meets it, best of R:

  kc     `kc --db DB --stats build` (C419: each procedure an object of its
         own, assembled once; linked only if an object is new), then the
         program run: the edit's answer, or its error;
  serve  the same build asked of one `kc --stats serve` (C415), which keeps
         its database in memory for the whole session;
  Fixpt  `fixpt eval FILE` on its fastest machine: the same answer, or
         error (its own caches, its `.fasl`s, as it keeps them).

and for kc, which query bodies ran, against what the edit needs at least
(its `need`): a body run beyond that is work the database threw away. The
session, each step from the one before:

  cold          the program, no database
  same          run again, nothing changed
  comment       a comment added: the same forms
  body          one procedure's body edited, its interface the same
  error         that body made a type error
  undo          the error undone: the text of `body` again
  error2        the error again
  fix-forward   fixed another way: a body neither `body`'s nor the error's
  add           a procedure added at the end, and called

Each row: the step; kc's ms, its build (compile, assemble, link), then
serve's, then the run's; Fixpt's ms; kc's bodies run, and those beyond the need
(serve's are the same, or said apart). With --record, each size's rows are
appended to bench/kc.tsv with the commit, so that the numbers are kept
step by step (they gate nothing: a speed's change is for discussion).

As a step's workflow has it (C463; the user: "make gathering those
timings part of the default workflow"): scripts/check-snapshot.sh runs
this after each check that passes, `--rows FILE`, the rows kept apart, their
commit not yet made; and when the next check begins, the commit that
was made of that snapshot known (after any amend), `--tag FILE COMMIT`
appends them to bench/kc.tsv, so that each step's rows go in with the
step after it.
"""

import os
import shutil
import subprocess
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import fixptcache  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
WORK = os.path.join(ROOT, "target", "kc-bench")
KC = os.path.join(ROOT, "target", "release", "kc")
RT = os.path.join(ROOT, "target", "release", "libkayak_kcrt.a")
LIBS = ["-lgcc_s", "-lutil", "-lrt", "-lpthread", "-lm", "-ldl", "-lc"]
RECORD = os.path.join(ROOT, "bench", "kc.tsv")


def fixpt():
    rev = open(os.path.join(ROOT, "FIXPT_REV")).read().strip()
    p = os.path.join(ROOT, "target", "fixpt", "fixpt-" + rev)
    return p if os.path.exists(p) else None


def proc(k, body=None):
    b = body or "(if (< n %d) (* n 2) (- n %d))" % (k, k)
    return "(define p%d (subr pure (int) int) (lambda (n) %s))\n" % (k, b)


def program(n, edits, extra=""):
    """N procedures, `edits` replacing some bodies, then an expression."""
    out = []
    for k in range(n):
        out.append(proc(k, edits.get(k)))
    out.append(extra)
    out.append("(+ (p%d 5) (+ (p%d 7) (p0 3)))\n" % (n - 1, n // 2))
    return "".join(out)


def session(n):
    """The edits: (name, text, need), `need` the query bodies the edit needs
    run at least, as `kc --stats` names them, or None where any will do."""
    m = n // 2
    base = program(n, {})
    body = program(n, {m: "(+ n 1)"})
    error = program(n, {m: "(+ n #t)"})
    fixed = program(n, {m: "(* n 3)"})
    added = program(n, {m: "(* n 3)"}, "(define extra (subr pure (int) int) (lambda (n) (- n 1)))\n").replace(
        "(p0 3)))\n", "(extra 3)))\n")
    # The edited form's own queries, and the whole program's: no other
    # form's.
    # (`objects`, C419: what a build is assembled of, by fingerprint.)
    one = ["asm-item", "checked", "interface", "l0-item", "objects"]
    return [
        ("cold", base, None),
        ("same", base, []),
        ("comment", "; an edit that means nothing\n" + base, ["checked"]),
        ("body", body, one),
        # A refusal is `checked`'s value (C415): the run ends as any other,
        # its work kept; undone, the edited form's interface and item are made
        # again (the engine keeps a query's last value, not each before it),
        # and found as they were, so nothing after them runs.
        ("error", error, ["checked"]),
        ("undo", body, ["checked", "interface", "l0-item"]),
        ("error2", error, ["checked"]),
        ("fix-forward", fixed, one),
        ("add", added, None),
    ]


def run(args, **kw):
    return subprocess.run(args, capture_output=True, text=True, **kw)


def ran(stderr):
    for line in stderr.splitlines():
        if "query bodies ran:" in line:
            return sorted(line.split("query bodies ran:")[1].split())
    return []


def beyond(got, need):
    """The bodies run beyond `need`, as a multiset."""
    left = list(need)
    out = []
    for g in got:
        if g in left:
            left.remove(g)
        else:
            out.append(g)
    return out


def said(text, marks):
    """A failure's message: the first line with the first of `marks` that
    one has."""
    for m in marks:
        for line in text.splitlines():
            if m in line and "query bodies ran" not in line:
                return "! " + line.split(m, 1)[1].strip()[:50]
    return "! ?"


def counted(names):
    """`form×50 keys`: each name, and how many times, if more than once."""
    out = []
    for n in sorted(set(names)):
        k = names.count(n)
        out.append(n if k == 1 else "%s×%d" % (n, k))
    return " ".join(out)


def kc_step(src, db, path, exe):
    """`kc build` (C419: assembling and linking only what is new), then the
    run, in seconds; its bodies; its answer."""
    t0 = time.perf_counter()
    r = run([KC, "--db", db, "--rt", RT, "--stats", "build", path, exe])
    t1 = time.perf_counter()
    bodies = ran(r.stderr)
    if r.returncode != 0:
        return t1 - t0, 0.0, bodies, said(r.stderr, [".fx:", "kc: "])
    r3 = run([exe])
    return t1 - t0, time.perf_counter() - t1, bodies, (r3.stdout.strip().splitlines() or ["?"])[-1]


class Serve:
    """One `kc --stats serve`, asked a line at a time."""

    def __init__(self):
        self.p = subprocess.Popen([KC, "--rt", RT, "--stats", "serve"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)

    def ask(self, line):
        t0 = time.perf_counter()
        self.p.stdin.write(line + "\n")
        self.p.stdin.flush()
        answer = self.p.stdout.readline()
        t = time.perf_counter() - t0
        head, _, names = answer.rstrip("\n").partition("\t")
        return t, sorted(names.split()), head

    def close(self):
        self.p.stdin.write("quit\n")
        self.p.stdin.close()
        self.p.wait()


def fixpt_step(fx, path):
    t0 = time.perf_counter()
    r = run([fx, "eval", path], cwd=os.path.dirname(path))
    # Fixpt's warnings no part of the answer (C558).
    err = fixptcache.unwarned(r.stderr)
    lines = [l for l in (r.stdout + err).splitlines() if l.strip() and not l.startswith(";")]
    if r.returncode != 0:
        return time.perf_counter() - t0, said(r.stdout + r.stderr, [".fx:"])
    got = (lines[-1] if lines else "?").split(" : ")[0]
    return time.perf_counter() - t0, got[:60]


def bench(n, runs, fx):
    d = os.path.join(WORK, "n%d" % n)
    os.makedirs(d, exist_ok=True)
    rows = []
    best = {}
    for rnd in range(runs):
        db = os.path.join(d, "kc.db")
        # Cold: no database, its files in DB.d too (kayak-db 5, C559: its
        # heads there, so that removing DB alone left it warm).
        if os.path.exists(db):
            os.remove(db)
        shutil.rmtree(db + ".d", ignore_errors=True)
        server = Serve()
        for i, (name, text, need) in enumerate(session(n)):
            # One file for kc, one for Fixpt, so that neither's caches are
            # the other's.
            kp, fp = os.path.join(d, "prog.fx"), os.path.join(d, "fixpt", "prog.fx")
            os.makedirs(os.path.dirname(fp), exist_ok=True)
            open(kp, "w").write(text)
            open(fp, "w").write(text)
            kc_t, link_t, bodies, kgot = kc_step(text, db, kp, os.path.join(d, "prog"))
            kt = kc_t + link_t
            sp = os.path.join(d, "serve", "prog.fx")
            os.makedirs(os.path.dirname(sp), exist_ok=True)
            open(sp, "w").write(text)
            sv_t, sv_bodies, sv_head = server.ask("build %s %s" % (sp, sp[:-3]))
            if sorted(sv_bodies) != sorted(bodies):
                kgot = kgot + " (serve ran %s)" % counted(sv_bodies)
            ft, fgot = fixpt_step(fx, fp) if fx else (float("nan"), "-")
            over = beyond(bodies, need) if need is not None else []
            prev = best.get(i)
            if prev is None or kt < prev[1] + prev[2]:
                best[i] = (name, kc_t, link_t, ft if prev is None else min(ft, prev[3]), len(bodies), over, kgot, fgot, need,
                           sv_t if prev is None else min(sv_t, prev[9]))
            else:
                best[i] = prev[:3] + (min(ft, prev[3]),) + prev[4:9] + (min(sv_t, prev[9]),)
        server.close()
    for i in sorted(best):
        rows.append(best[i])
    return rows


def main():
    sizes, runs, record, rows = [100, 1000], 3, False, None
    args = sys.argv[1:]
    if args[:1] == ["--tag"] and len(args) == 3:
        tagged(args[1], args[2])
        return
    while args:
        a = args.pop(0)
        if a == "--sizes":
            sizes = [int(x) for x in args.pop(0).split(",")]
        elif a == "--runs":
            runs = int(args.pop(0))
        elif a == "--record":
            record = True
        elif a == "--rows" and args:
            rows = args.pop(0)
        else:
            sys.exit(__doc__)
    fx = fixpt()
    commit = run(["git", "rev-parse", "--short", "HEAD"], cwd=ROOT).stdout.strip()
    out = []
    for n in sizes:
        print("kc-bench: %d procedures, best of %d (ms)" % (n, runs))
        print("  %-12s %8s %8s %8s %8s %7s  %-24s %s" % ("step", "kc", "serve", "+run", "Fixpt", "bodies", "beyond the need", "answers (kc / Fixpt)"))
        for name, kc_t, link_t, ft, nb, over, kgot, fgot, need, sv_t in bench(n, runs, fx):
            extra = "-" if need is None else (counted(over) or "none")
            print("  %-12s %8.1f %8.1f %8.1f %8.1f %7d  %-24s %s / %s" % (name, kc_t * 1000, sv_t * 1000, link_t * 1000,
                                                                       ft * 1000, nb, extra, kgot, fgot))
            out.append("\t".join([commit, str(n), name, "%.1f" % (kc_t * 1000), "%.1f" % (sv_t * 1000), "%.1f" % (link_t * 1000), "%.1f" % (ft * 1000), str(nb), extra]))
    if record:
        recorded(out)
    if rows:
        # Their commit not made yet: `-`, which `--tag` names.
        with open(rows, "w") as f:
            f.write("".join("-" + r[len(commit):] + "\n" for r in out))


def recorded(out):
    new = not os.path.exists(RECORD)
    with open(RECORD, "a") as f:
        if new:
            f.write("commit\tprocedures\tstep\tkc_ms\tserve_ms\tlink_run_ms\tfixpt_ms\tbodies\tbeyond_need\n")
        f.write("\n".join(out) + "\n")


def tagged(path, commit):
    """`--rows`' rows, each its commit's now that it is made, recorded."""
    out = [commit + l[1:] for l in open(path).read().splitlines() if l.startswith("-\t")]
    if out:
        recorded(out)


if __name__ == "__main__":
    main()
