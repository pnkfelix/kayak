"""kc's steps (docs/research/kc.md): each program of tests/kc/stepN/,
compiled by each of kc's back ends that takes it, run, and its value
compared with the one it states (`;; => V`), which must be Fixpt's (the
pinned Fixpt's `eval`, on the fx-cells-rsinterp machine).

usage: python3 -I scripts/kc-steps.py

  native  every program: `kc build` (C419), its objects assembled and
          linked by kc with the system's cc against crates/kayak-kcrt, run;
          and `kc run` (C451), the same code in process, its output and
          status the executable's;
  rust    the KFX programs (`.kfx`): `kc --rust`, each a module of one
          crate (target/kc-rust), built once and run.

The programs are swept by WORKERS at once (C551), each worker a database
of its own (target/kc-steps/db-K.db), each program built by the first
worker free, the slowest begun first; beside them, the edits (`cutoff`,
`expansions`), the convention and the Rust crate, each its own database or
none. Fixpt's answer to a program that reads nothing of the world is
asked once at the pin (scripts/fixptcache.py, target/fixpt-cache/).

And early cutoff, as kc's database does it (C399): a program compiled,
then edited to text that means the same (spaces, a comment) runs again
only what reads the text (`checked`, `l0`), not the back end after it,
whose input, `l0`'s value, is as it was; edited to another value, the back
end runs too.

And warm (C463): after the sweep, each program built again, nothing
changed, with the database its worker left: no query body may run. And
early cutoff of the kinds of item kc has grown since (C463): a datatype,
a module's value, a global a closure captures, a prompt's body; a core
form's keyword and a literal never looked up as a global's name.

And a file's macros (C445): each form's expansion a query of its own, an
edit expands again only the forms that name what it changed.

And kc's own convention (C451): a recursion past the stack's limit fails
as kc says it, and a loop past a slice of fuel runs on, built and in
process alike.

And kc's collector (C453): each program in process again with a heap of a
few words, so that nearly every allocation collects, its output and status
the executable's; and a program whose garbage passes a built program's
heap many times over.
"""

import concurrent.futures
import os
import queue
import shutil
import subprocess
import sys
import threading

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import fixptcache  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
TESTS = os.path.join(ROOT, "tests", "kc")
WORK = os.path.join(ROOT, "target", "kc-steps")
KC = os.path.join(ROOT, "target", "release", "kc")
RT = os.path.join(ROOT, "target", "release", "libkayak_kcrt.a")
LIBS = ["-lgcc_s", "-lutil", "-lrt", "-lpthread", "-lm", "-ldl", "-lc"]


def fixpt():
    rev = open(os.path.join(ROOT, "FIXPT_REV")).read().strip()
    p = os.path.join(ROOT, "target", "fixpt", "fixpt-" + rev)
    return p if os.path.exists(p) else None


def stated(path):
    for line in open(path):
        if line.startswith(";; => "):
            return line[len(";; => "):].strip()
    sys.exit("kc-steps: %s states no value" % path)


def programs():
    out = []
    for step in sorted(os.listdir(TESTS)):
        d = os.path.join(TESTS, step)
        if os.path.isdir(d):
            for f in sorted(os.listdir(d)):
                if f.endswith(".fx") or f.endswith(".kfx"):
                    out.append(os.path.join(d, f))
    return out


def run(args, **kw):
    # Its standard input empty, at its end (C546), as Fixpt's tests have
    # it: a program that reads (`read-byte`) takes nothing of the check's.
    kw.setdefault("stdin", subprocess.DEVNULL)
    return subprocess.run(args, capture_output=True, text=True, **kw)


def bodies_ran(stderr):
    """The queries `kc --stats` says ran."""
    for line in stderr.splitlines():
        if "query bodies ran:" in line:
            return sorted(line.split("query bodies ran:")[1].split())
    return None


TWO = """(define inc (subr pure (int) int) (lambda (n) %s))
(define dbl (subr pure (int) int) (lambda (n) (* n 2)))
(dbl (inc 3))
"""


def cutoff():
    """What runs again after edits: the wrongs found."""
    p = os.path.join(WORK, "cutoff.fx")
    db = os.path.join(WORK, "cutoff.db")
    s = os.path.join(WORK, "cutoff.s")
    if os.path.exists(db):
        os.remove(db)
    wrong = []
    # The forms are inputs, set by the driver from one reading of the text
    # (C414): no query reads the whole program but the checker. The whole
    # program's assembly is not kept in the database's files (C417): the
    # file it was written to is, its fingerprint the one the database has.
    for text, want in [("42\n", ["asm", "asm-item", "checked", "l0-item"]),
                       # The same datum: the forms' inputs the same, all else spared.
                       ("  42   ; the same value, written otherwise\n", ["checked"]),
                       ("42\n", ["checked"]),
                       ("43\n", ["asm", "asm-item", "checked", "l0-item"]),
                       ("43\n", []),
                       # Two procedures and a call: one body edited, the other's
                       # form, item and code spared; the edited one's interface the
                       # same, so no other item again (C407, C414).
                       (TWO % "(+ n 1)", None),
                       (TWO % "(+ n 2)", ["asm", "asm-item", "checked", "interface", "l0-item"])]:
        open(p, "w").write(text)
        r = run([KC, "--db", db, "--stats", "--asm", p, s])
        got = bodies_ran(r.stderr)
        if want is not None and got != want:
            wrong.append("cutoff: after %r, ran %s, not %s" % (text, got, want))
    # A core form's keyword and a literal, never a global's name (C463): a
    # program of them, cold, runs what `42` does, no `form` or `interface`
    # of `lambda`, `if`, `#t` ...
    for text in ["(let ((x 1)) (if (and #t #f) ((lambda ((y int)) y) x) (begin #u 2)))\n"]:
        if os.path.exists(db):
            os.remove(db)
        open(p, "w").write(text)
        got = bodies_ran(run([KC, "--db", db, "--stats", "--asm", p, s]).stderr)
        if got != ["asm", "asm-item", "checked", "l0-item"]:
            wrong.append("cutoff: %r, cold, ran %s, not as `42`" % (text, got))
    # The kinds of item since (C463), each: cold; the same; another
    # procedure's body edited, it alone; then the kind's own edit.
    one = ["asm", "asm-item", "checked", "interface", "l0-item"]
    for kind, tmpl, edit, want in GROWN:
        if os.path.exists(db):
            os.remove(db)
        for args, w in [(("a", "(* n 2)"), None), (("a", "(* n 2)"), []), (("a", "(* 2 n)"), one), (("b", "(* 2 n)"), want)]:
            open(p, "w").write(tmpl % (edit[args[0]], args[1]))
            r = run([KC, "--db", db, "--stats", "--asm", p, s])
            got = bodies_ran(r.stderr)
            if r.returncode != 0:
                wrong.append("cutoff: %s: kc --asm: %s" % (kind, r.stderr.strip()[-300:]))
                break
            if w is not None and got != w:
                wrong.append("cutoff: %s, edit %s %s: ran %s, not %s" % (kind, args[0], args[1], got, w))
    # A load (C471): what a program loads an input too, its forms spliced in
    # as the program's; an edit of the loaded file, the program's own text as
    # it was, runs what an edit of one item does. A file of no parameters'
    # value; a file of a place's procedure (its maker one item, C471).
    for kind, prog, name, loaded, edits in LOADS:
        if os.path.exists(db):
            os.remove(db)
        open(p, "w").write(prog)
        for k, w in [(0, None), (0, []), (1, one)]:
            open(os.path.join(WORK, name), "w").write(loaded % edits[k])
            r = run([KC, "--db", db, "--stats", "--asm", p, s])
            got = bodies_ran(r.stderr)
            if r.returncode != 0:
                wrong.append("cutoff: %s: kc --asm: %s" % (kind, r.stderr.strip()[-300:]))
                break
            if w is not None and got != w:
                wrong.append("cutoff: %s, edit %s: ran %s, not %s" % (kind, edits[k], got, w))
    return wrong


# Each: the program, the file it loads (written beside it), that file's
# text with a slot, and the slot's text before and after the edit.
LOADS = [
    ("a loaded file's value", """(define m (load-module "cutoff-loaded.fx"))
(define dbl (subr pure (int) int) (lambda (n) (* n 2)))
(+ (with m (get)) (dbl 3))
""", "cutoff-loaded.fx", """(define v int %s)
(define get (subr pure () int) (lambda () (* v 2)))
""", ["5", "6"]),
    ("a file of a place's procedure", """(define f (subr (maxeff (alloc @w) (read @globals) spin) (int) int)
  (lambda (a) (letrena q (let ((x ((proj (load-module "cutoff-place.fx") q)))) (with x (bump a))))))
(define dbl (subr pure (int) int) (lambda (n) (* n 2)))
(+ (f 3) (dbl 3))
""", "cutoff-place.fx", """(module-parameters ((p place)))
(define n (ref int p) (new 0))
(define bump (subr (maxeff (read p) (write p)) (int) int)
  (lambda (k) (begin (set n %s) (get n))))
""", ["(+ k (get n))", "(+ (get n) k)"]),
]


# Each: its program, a slot for the kind's own text (`a`, then `b`) and one
# for another procedure's body; what the kind's edit runs. A datatype's
# variants changed: a `tagcase` of it lowered again, its code the same,
# cut off; the datatype's own code made again. A module's value, a
# captured global's: that item alone. A prompt's body made an abort: that
# item, and `abort-current-continuation` looked up as a global's name
# (unlike a keyword, a program may define one).
GROWN = [
    ("a datatype", """(define-datatype shape (circle int) (square %s))
(define area (subr pure (shape) int) (lambda (s) (tagcase s (circle (r) (* 3 (* r r))) (else x 0))))
(define dbl (subr pure (int) int) (lambda (n) %s))
(+ (area (circle 2)) (dbl 3))
""", {"a": "int", "b": "int int"}, ["asm", "asm-item", "checked", "interface", "l0-item", "l0-item"]),
    ("a module's value", """(define m (module (define v int %s) (define get (subr pure () int) (lambda () (* v 2)))))
(define dbl (subr pure (int) int) (lambda (n) %s))
(+ (with m (get)) (dbl 3))
""", {"a": "5", "b": "6"}, ["asm", "asm-item", "checked", "interface", "l0-item"]),
    ("a captured global", """(define k int %s)
(define mk (subr (read (globals k)) () (subr pure (int) int)) (lambda () (let ((j k)) (lambda ((x int)) (* x j)))))
(define dbl (subr pure (int) int) (lambda (n) %s))
(+ ((mk) 2) (dbl 3))
""", {"a": "5", "b": "6"}, ["asm", "asm-item", "checked", "interface", "l0-item"]),
    ("a prompt's body", """(define t (prompt-tag int int (maxeff spin (read (globals t))) @p) (make-continuation-prompt-tag))
(define f (subr (maxeff spin (read (globals t))) (int) int) (lambda (n) (prompt t %s (lambda ((v int)) (+ v 1)))))
(define dbl (subr pure (int) int) (lambda (n) %s))
(+ (f 3) (dbl 3))
""", {"a": "(+ n 1)", "b": "(abort-current-continuation t n)"},
     ["asm", "asm-item", "checked", "form", "interface", "interface", "l0-item"]),
]


MACROS = """(define-syntax inc
  (macro (_ expr) expr)
  (syntax-rules ((_ e) %s)))
(define double (subr pure (int) int) (lambda (n) %s))
(define-syntax inc2
  (macro (_ expr) expr)
  (syntax-rules ((_ e) (inc (inc e)))))
(define f (subr pure (int) int) (lambda (n) (inc2 n)))
(+ (f 3) (- (double 1) 2))
"""


def expansions():
    """What a file's macros expand again after edits (C445): each form a
    query of its own, reading the macros it names, and the shapes of the
    globals it names, alone: the wrongs found. `(expansion, macros-for)`,
    how many of each ran."""
    p = os.path.join(WORK, "expansions.fx")
    db = os.path.join(WORK, "expansions.db")
    if os.path.exists(db):
        os.remove(db)
    wrong = []
    body = "(* n 2)"
    for text, want, why in [(MACROS % ("(+ e 1)", body), (5, 5), "cold"),
                            (MACROS % ("(+ e 1)", body), (0, 0), "the same"),
                            # A procedure's body: its form alone; what names it,
                            # its entry the same, spared.
                            (MACROS % ("(+ e 1)", "(* 2 n)"), (1, 1), "a body edited"),
                            # A macro: it, and what names it, to a fixed point
                            # (`inc2`'s definition, `f`), and no other.
                            (MACROS % ("(+ 1 e)", "(* 2 n)"), (3, 3), "a macro edited"),
                            # A form put first: it alone expanded; the scopes'
                            # queries again, their values the same.
                            ("(define unused int 9)\n" + MACROS % ("(+ 1 e)", "(* 2 n)"), (1, 3), "a form put first")]:
        open(p, "w").write(text)
        r = run([KC, "--db", db, "--stats", "--check", p])
        got = bodies_ran(r.stderr) or []
        n = (got.count("expansion"), got.count("macros-for"))
        if r.returncode != 0:
            wrong.append("expansions: %s: kc --check: %s" % (why, r.stderr.strip()))
        elif n != want:
            wrong.append("expansions: %s: ran %d expansions and %d macros-for, not %d and %d" % (why, n[0], n[1], want[0], want[1]))
    # A procedural macro (C446): its transformer's answers queries' values,
    # kept in the database; a use's expansion again only when its form is
    # edited. Each form is expanded once (C449): a request is asked as the
    # expansion goes, of the `phase1` query, not answered between runs of
    # it (C446's restart, which ran the `define-syntax`'s and `f`'s twice).
    # The transformer compiled by kc (C447), its phase-1 program a file of
    # the database: cold, its 9 definitions' code (the prelude's 6, the
    # transformer, kc's reader and `%phase1-apply`, C448); a use's operand
    # edited, none: the use is given to the program built, a run alone.
    if os.path.exists(db):
        os.remove(db)
    for text, want, why in [(PROCEDURAL % "n", (4, 4, 9), "procedural, cold"),
                            (PROCEDURAL % "n", (0, 0, 0), "procedural, the same"),
                            (PROCEDURAL % "(+ n 1)", (1, 1, 0), "procedural, a use's operand edited")]:
        open(p, "w").write(text)
        r = run([KC, "--db", db, "--stats", "--check", p])
        got = bodies_ran(r.stderr) or []
        n = (got.count("expansion"), got.count("macros-for"), got.count("asm-item"))
        if r.returncode != 0:
            wrong.append("expansions: %s: kc --check: %s" % (why, r.stderr.strip()))
        elif n != want:
            wrong.append("expansions: %s: ran %d expansions, %d macros-for and %d asm-items, not %d, %d and %d" % ((why,) + n + want))
    return wrong


PROCEDURAL = """(define-syntax twice
  (er-macro-transformer
    (lambda (form rename compare) `(,(rename '+) ,(car (cdr form)) ,(car (cdr form))))))
(define f (subr pure (int) int) (lambda (n) (twice %s)))
(define g (subr pure (int) int) (lambda (n) (* n 2)))
(+ (f 3) (g 1))
"""


# The heap `kc run --heap` is given (C453): its words.
TINY_HEAP = 64

CONVENTION = {
    # Past the stack's limit (C451): a failure, as kc says it, by both.
    "deep": ("(define down (subr (maxeff spin (read (globals down))) (int) int) (lambda (n) (if (zero? n) 0 (+ 1 (down (- n 1))))))\n"
             "(down 1000)\n(down 100000000)\n",
             "1000\n", "evaluation failed: stack overflow\n", 1),
    # Fuel spent and given again by `poll`, a slice at a time (2^20 calls).
    # Garbage past the heap many times over (C453): 20000 rounds of a list
    # of 1000, an array and a closure, some 330 MB made, a list a global
    # keeps and a closure another keeps across every collection.
    "collected": ("(define build (subr (maxeff spin (read (globals build))) (int (listof int)) (listof int))\n"
                  "  (lambda (n acc) (if (zero? n) acc (build (- n 1) (cons n acc)))))\n"
                  "(define total (subr (maxeff spin (read (globals total))) ((listof int) int) int)\n"
                  "  (lambda (l acc) (if (null? l) acc (total (cdr l) (+ acc (car l))))))\n"
                  "(define keep (listof int) (build 100 nil))\n"
                  "(define adder (subr (maxeff) (int) (subr (maxeff) (int) int)) (lambda (k) (lambda (x) (+ x k))))\n"
                  "(define add7 (subr (maxeff) (int) int) (adder 7))\n"
                  "(define churn (subr (maxeff spin (read (globals churn build total adder))) (int int) int)\n"
                  "  (lambda (k acc)\n"
                  "    (if (zero? k) acc\n"
                  "        (let* ((a (make-array 50 k)) (f (adder k)) (l (build 1000 nil)))\n"
                  "          (churn (- k 1) (+ acc (+ (total l 0) (f (array-ref a 49)))))))))\n"
                  "(churn 20000 0)\n(total keep 0)\n(add7 1)\n",
                  "10410020000\n5050\n8\n", "", 0),
    "fuel": ("(define loop (subr (maxeff spin (read (globals loop))) (int int) int) (lambda (n acc) (if (zero? n) acc (loop (- n 1) (+ acc 1)))))\n"
             "(loop 3000000 0)\n",
             "3000000\n", "", 0),
}


# A task's read or write parked, not the thread blocked (C547): a ticker
# task sleeping 100 ms wakes in time (1) though another task reads a byte
# that comes 0.8 s late, or writes 1 MB to a pipe not read for 0.8 s.
# Fixpt's block the thread (it gives 0); kc's is the async form, the
# user's choice, so these are checked here, not against Fixpt.
PARKING = {
    "read": ('(define-type byte-result (sumof (byte u8) (eof unit) (failed io-error)))\n(define* code (subr pure (byte-result) int)\n  (lambda (b) (tagcase b (byte k (u8->int k)) (eof u -1) (failed e -2))))\n(define start i64 (clock))\n(define got (icell int @g) (make-icell))\n(define woke (icell i64 @w) (make-icell))\n(define reader (addr int @p) (spawn (box int r) (icell-put! got (code (read-byte)))))\n(define ticker (addr int @p) (spawn (box int r) (begin (sleep-until (i64+ start (int->i64 100000000))) (icell-put! woke (i64- (clock) start)))))\n(let* ((w (icell-get woke)) (b (icell-get got)))\n  (list b (if (i64< w (int->i64 400000000)) 1 0)))\n', "(90 1)"),
    "write": ('(define* done? (subr pure ((sumof (done unit) (failed io-error))) int)\n  (lambda (r) (tagcase r (done u 1) (failed e 0))))\n(define* doubled (subr spin (string int) string) (lambda (s n) (if (= n 0) s (doubled (string-append s s) (- n 1)))))\n(define big string (doubled "0123456789abcdef" 16))\n(define start i64 (clock))\n(define wrote (icell int @g) (make-icell))\n(define woke (icell i64 @w) (make-icell))\n(define writer (addr int @p) (spawn (box int r) (icell-put! wrote (done? (write-err big)))))\n(define ticker (addr int @p) (spawn (box int r) (begin (sleep-until (i64+ start (int->i64 100000000))) (icell-put! woke (i64- (clock) start)))))\n(let* ((w (icell-get woke)) (b (icell-get wrote)))\n  (list b (if (i64< w (int->i64 400000000)) 1 0) (string-length big)))\n', "(1 1 1048576)"),
}


def parking():
    import time
    wrong = []
    for name, (text, want) in PARKING.items():
        src = os.path.join(WORK, "parking-%s.fx" % name)
        open(src, "w").write(text)
        exe = src[:-3]
        r = run([KC, "--rt", RT, "build", src, exe])
        if r.returncode != 0:
            wrong.append("parking %s: kc build: %s" % (name, r.stderr.strip()))
            continue
        for how, args in (("built", [exe]), ("kc run", [KC, "run", src])):
            p = subprocess.Popen(args, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            time.sleep(0.8)
            out, err = p.communicate(b"Z" if name == "read" else b"", timeout=120)
            got = (out.decode().strip().splitlines() or [""])[-1]
            if got != want or (name == "write" and len(err) != 1048576):
                wrong.append("parking %s: %s gives %r and %d bytes of error, not %r"
                             % (name, how, got, len(err), want))
    return wrong


def convention():
    """kc's own convention (C451), in a built program and in process: a
    stack's limit checked at each entry, and fuel."""
    wrong = []
    for name, (text, out, err, status) in CONVENTION.items():
        src = os.path.join(WORK, "convention-%s.kfx" % name)
        open(src, "w").write(text)
        exe = src[:-4]
        r = run([KC, "--rt", RT, "build", src, exe])
        if r.returncode != 0:
            wrong.append("convention %s: kc build: %s" % (name, r.stderr.strip()))
            continue
        for how, args in (("built", [exe]), ("kc run", [KC, "run", src])):
            r = run(args)
            if (r.stdout, r.stderr, r.returncode) != (out, err, status):
                wrong.append("convention %s: %s gives %r %r %d, not %r %r %d"
                             % (name, how, r.stdout, r.stderr, r.returncode, out, err, status))
    return wrong


# How many programs are swept at once (C551), each worker with a database
# of its own: a program's cold build compiles what it shares with the others
# (the runtime's routines, the prelude's items) once in each.
WORKERS = 16

# Programs that read the world (a file by a path from where they are, the
# clock, the standard input): Fixpt asked each time, in their directory.
WORLDLY = ("loads", "tasks", "io")


def fixpt_gives(fx, p):
    """What Fixpt's `eval` says `p` is, as kc-steps reads its answer."""
    args = ["--fx26-machine", "fx-cells-rsinterp", "eval", p]
    if os.path.relpath(p, TESTS).split(os.sep)[0] in WORLDLY:
        r = run([fx] + args, cwd=os.path.dirname(p))
    else:
        r = fixptcache.run(fx, args, [p], keep=False)
    lines = [l for l in (r.stdout + r.stderr).splitlines() if l.strip() and not l.startswith(";")]
    got = lines[-1].split(" : ")[0] if lines else "(none)"
    if got.startswith("! evaluation failed: "):
        got = "! " + got[len("! evaluation failed: "):]
    return got


def fresh(db):
    """Database `db` removed, its head and its files (`DB.d`), so that a
    worker begins cold."""
    if os.path.exists(db):
        os.remove(db)
    shutil.rmtree(db + ".d", ignore_errors=True)


def swept(p, db):
    """`p` built, run, run in process and with a tiny heap, against what it
    states: the wrongs, the query bodies ran, and whether it ran in process
    and collected as its executable."""
    name = os.path.relpath(p, TESTS)
    want = stated(p)
    wrong, ran = [], 0
    exe = os.path.join(WORK, name.replace("/", "--"))
    # `kc build` (C419): its objects assembled and linked by kc.
    r = run([KC, "--db", db, "--rt", RT, "--stats", "build", p, exe])
    ran += int(r.stderr.split("kc: ")[-1].split()[0]) if "query bodies ran" in r.stderr else 0
    if r.returncode != 0:
        return ["%s: kc build: %s" % (name, r.stderr.strip())], ran, False, False
    r = run([exe])
    # Each top-level expression's value on a line; the last is stated.
    got = (r.stdout.strip().splitlines() or [""])[-1]
    if r.returncode != 0:
        # A failure, as `fixpt eval` says one: `! MESSAGE`.
        got = "! " + r.stderr.strip().split("evaluation failed: ")[-1]
    if got != want:
        wrong.append("%s: native gives %s, not %s" % (name, got, want))
    # `kc run` (C451): the same code in process, sealed in the code area
    # and called, its runtime KFX's: what it writes, and its status, the
    # executable's own.
    q = run([KC, "--db", db, "run", p])
    in_process = (q.stdout, q.stderr, q.returncode) == (r.stdout, r.stderr, r.returncode)
    if not in_process:
        wrong.append("%s: kc run gives %r %r %d, the executable %r %r %d"
                     % (name, q.stdout[-300:], q.stderr[-300:], q.returncode, r.stdout[-300:], r.stderr[-300:], r.returncode))
    # And with a heap of a few words (C453): the collector called at
    # nearly every allocation, the program's answer its executable's.
    q = run([KC, "--db", db, "run", "--heap", str(TINY_HEAP), p])
    collected = (q.stdout, q.stderr, q.returncode) == (r.stdout, r.stderr, r.returncode)
    if not collected:
        wrong.append("%s: kc run --heap %d gives %r %r %d, the executable %r %r %d"
                     % (name, TINY_HEAP, q.stdout[-300:], q.stderr[-300:], q.returncode, r.stdout[-300:], r.stderr[-300:], r.returncode))
    return wrong, ran, in_process, collected


def warmed(p, db):
    """`p` built again, nothing changed, with the database that built it: a
    wrong if any query body ran (C463)."""
    name = os.path.relpath(p, TESTS)
    exe = os.path.join(WORK, name.replace("/", "--"))
    r = run([KC, "--db", db, "--rt", RT, "--stats", "build", p, exe])
    got = bodies_ran(r.stderr)
    if r.returncode == 0 and got == []:
        return None
    if r.returncode == 0:
        return "%s: built again, nothing changed, ran %s" % (name, " ".join(got or ["(no stats)"]))
    return "%s: built again: %s" % (name, r.stderr.strip())


def slowest_first(ps):
    """The programs, those that take longest (a load of a large file, the
    scheduler's) begun first, so that no worker is left with one at the end."""
    def rank(p):
        top = os.path.relpath(p, TESTS).split(os.sep)[0]
        return (0 if top == "loads" else 1 if top == "tasks" else 2, p)
    return sorted(ps, key=rank)


def sweep(ps):
    """Every program swept (`swept`) and built again warm (`warmed`) by
    WORKERS, each its own database, cold at first: the wrongs, the query
    bodies ran, how many ran in process and collected, how many warm."""
    todo = queue.Queue()
    for p in slowest_first(ps):
        todo.put(p)
    lock = threading.Lock()
    totals = {"wrong": [], "ran": 0, "in_process": 0, "collected": 0, "warm": 0}

    def worker(k):
        db = os.path.join(WORK, "db-%d.db" % k)
        fresh(db)
        mine = []
        while True:
            try:
                p = todo.get_nowait()
            except queue.Empty:
                break
            wrong, ran, in_process, collected = swept(p, db)
            with lock:
                totals["wrong"] += wrong
                totals["ran"] += ran
                totals["in_process"] += in_process
                totals["collected"] += collected
            # Built: to be built again warm.
            if not (wrong and ": kc build: " in wrong[0]):
                mine.append(p)
        # Warm (C463): each built again, nothing changed: no query body runs.
        for p in mine:
            w = warmed(p, db)
            with lock:
                if w is None:
                    totals["warm"] += 1
                else:
                    totals["wrong"].append(w)

    threads = [threading.Thread(target=worker, args=(k,)) for k in range(WORKERS)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()
    return totals


def rust(ps):
    """The KFX programs by `kc --rust`, each a module of one crate
    (target/kc-rust), built once and run: the wrongs, the query bodies ran."""
    db = os.path.join(WORK, "rust.db")
    fresh(db)
    wrong, ran = [], 0
    kfx = [p for p in ps if p.endswith(".kfx")]
    crate = os.path.join(ROOT, "target", "kc-rust")
    os.makedirs(os.path.join(crate, "src"), exist_ok=True)
    mods, calls = [], []
    for i, p in enumerate(kfx):
        out = os.path.join(crate, "src", "p%d.rs" % i)
        r = run([KC, "--db", db, "--stats", "--rust", p, out])
        ran += int(r.stderr.split("kc: ")[-1].split()[0]) if "query bodies ran" in r.stderr else 0
        if r.returncode != 0:
            wrong.append("%s: kc --rust: %s" % (os.path.relpath(p, TESTS), r.stderr.strip()))
            continue
        mods.append("mod p%d;" % i)
        calls.append('    shown(%r, std::panic::catch_unwind(p%d::main));' % (os.path.relpath(p, TESTS), i))
    open(os.path.join(crate, "Cargo.toml"), "w").write(
        '[package]\nname = "kc-rust"\nversion = "0.1.0"\nedition = "2024"\n\n[workspace]\n\n'
        '[dependencies]\nkayak-rt = { path = "%s" }\n' % os.path.join(ROOT, "crates", "kayak-rt"))
    open(os.path.join(crate, "src", "main.rs"), "w").write(
        "\n".join(mods) + "\n\n"
        "/// A program's value, or its failure as `fixpt eval` says one: `! MESSAGE`.\n"
        "fn shown(name: &str, r: std::thread::Result<kayak_rt::Str>) {\n"
        "    match r {\n"
        "        Ok(v) => println!(\"{}\\t{}\", name, v.text()),\n"
        "        Err(e) => {\n"
        "            let m = e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();\n"
        "            println!(\"{}\\t! {}\", name, m)\n"
        "        }\n"
        "    }\n"
        "}\n\n"
        "fn main() {\n    std::panic::set_hook(Box::new(|_| {}));\n" + "\n".join(calls).replace("'", '"') + "\n}\n")
    r = run(["cargo", "run", "--release", "-q"], cwd=crate,
            env=dict(os.environ, CARGO_TARGET_DIR=os.path.join(ROOT, "target", "kc-rust-target")))
    if r.returncode != 0:
        wrong.append("rust: the crate does not build or run: %s" % r.stderr.strip()[-2000:])
    else:
        for line in r.stdout.splitlines():
            name, got = line.split("\t")
            want = stated(os.path.join(TESTS, name))
            if got != want:
                wrong.append("%s: rust gives %s, not %s" % (name, got, want))
    return wrong, ran


def main():
    if sys.argv[1:]:
        sys.exit(__doc__)
    os.makedirs(WORK, exist_ok=True)
    fx = fixpt()
    ps = programs()
    with concurrent.futures.ThreadPoolExecutor(max_workers=32) as ex:
        # Beside the sweep: Fixpt's answers, the edits, the convention, the
        # crate, none sharing a database with another.
        said = [ex.submit(fixpt_gives, fx, p) for p in ps] if fx else []
        edits = [ex.submit(f) for f in (cutoff, expansions, convention)]
        crate = ex.submit(rust, ps)
        totals = sweep(ps)
        wrong = totals["wrong"]
        for p, f in zip(ps, said):
            got, want = f.result(), stated(p)
            if got != want:
                wrong.append("%s: states %s, Fixpt gives %s" % (os.path.relpath(p, TESTS), want, got))
        rust_wrong, rust_ran = crate.result()
        wrong += rust_wrong
        for f in edits:
            wrong += f.result()
    # Last, alone: what parks waits on the clock, a machine left to it.
    wrong += parking()
    kfx = [p for p in ps if p.endswith(".kfx")]
    for w in wrong:
        print("kc-steps: " + w)
    print("kc-steps: %d programs, %d by native code (%d of them in process as their executables, %d with a heap of %d words, %d built again warm, no body run), %d by Rust; %d wrong; %d query bodies ran%s"
          % (len(ps), len(ps), totals["in_process"], totals["collected"], TINY_HEAP, totals["warm"], len(kfx), len(wrong),
             totals["ran"] + rust_ran, "" if fx else "; Fixpt not here, its answers unchecked"))
    sys.exit(1 if wrong else 0)


if __name__ == "__main__":
    main()
