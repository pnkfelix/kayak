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

kc runs with one database for the whole sweep (target/kc-steps.db), so a
second sweep with nothing changed runs no query body: its count is said.

And early cutoff, as kc's database does it (C399): a program compiled,
then edited to text that means the same (spaces, a comment) runs again
only what reads the text (`checked`, `l0`), not the back end after it,
whose input, `l0`'s value, is as it was; edited to another value, the back
end runs too.

And warm (C463): after the sweep, each program built again, nothing
changed, with the database the sweep left: no query body may run. And
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

import os
import subprocess
import sys

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
    return wrong


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


def main():
    if sys.argv[1:]:
        sys.exit(__doc__)
    os.makedirs(WORK, exist_ok=True)
    db = os.path.join(ROOT, "target", "kc-steps.db")
    fx = fixpt()
    wrong, ran, in_process, collected = [], 0, 0, 0
    ps = programs()
    for p in ps:
        name = os.path.relpath(p, TESTS)
        want = stated(p)
        if fx:
            r = run([fx, "--fx26-machine", "fx-cells-rsinterp", "eval", p], cwd=os.path.dirname(p))
            lines = [l for l in (r.stdout + r.stderr).splitlines() if l.strip() and not l.startswith(";")]
            got = lines[-1].split(" : ")[0] if lines else "(none)"
            if got.startswith("! evaluation failed: "):
                got = "! " + got[len("! evaluation failed: "):]
            if got != want:
                wrong.append("%s: states %s, Fixpt gives %s" % (name, want, got))
        exe = os.path.join(WORK, name.replace("/", "--"))
        # `kc build` (C419): its objects assembled and linked by kc.
        r = run([KC, "--db", db, "--rt", RT, "--stats", "build", p, exe])
        ran += int(r.stderr.split("kc: ")[-1].split()[0]) if "query bodies ran" in r.stderr else 0
        if r.returncode != 0:
            wrong.append("%s: kc build: %s" % (name, r.stderr.strip()))
            continue
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
        if (q.stdout, q.stderr, q.returncode) != (r.stdout, r.stderr, r.returncode):
            wrong.append("%s: kc run gives %r %r %d, the executable %r %r %d"
                         % (name, q.stdout[-300:], q.stderr[-300:], q.returncode, r.stdout[-300:], r.stderr[-300:], r.returncode))
        else:
            in_process += 1
        # And with a heap of a few words (C453): the collector called at
        # nearly every allocation, the program's answer its executable's.
        q = run([KC, "--db", db, "run", "--heap", str(TINY_HEAP), p])
        if (q.stdout, q.stderr, q.returncode) != (r.stdout, r.stderr, r.returncode):
            wrong.append("%s: kc run --heap %d gives %r %r %d, the executable %r %r %d"
                         % (name, TINY_HEAP, q.stdout[-300:], q.stderr[-300:], q.returncode, r.stdout[-300:], r.stderr[-300:], r.returncode))
        else:
            collected += 1
    # Warm (C463): each built again, nothing changed: no query body runs.
    warm = 0
    for p in ps:
        name = os.path.relpath(p, TESTS)
        exe = os.path.join(WORK, name.replace("/", "--"))
        r = run([KC, "--db", db, "--rt", RT, "--stats", "build", p, exe])
        got = bodies_ran(r.stderr)
        if r.returncode == 0 and got == []:
            warm += 1
        elif r.returncode == 0:
            wrong.append("%s: built again, nothing changed, ran %s" % (name, " ".join(got or ["(no stats)"])))
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
        "        Ok(v) => println!(\"{}\\t{}\", name, v.as_str()),\n"
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
    wrong += cutoff()
    wrong += expansions()
    wrong += convention()
    for w in wrong:
        print("kc-steps: " + w)
    print("kc-steps: %d programs, %d by native code (%d of them in process as their executables, %d with a heap of %d words, %d built again warm, no body run), %d by Rust; %d wrong; %d query bodies ran%s"
          % (len(ps), len(ps), in_process, collected, TINY_HEAP, warm, len(kfx), len(wrong), ran, "" if fx else "; Fixpt not here, its answers unchecked"))
    sys.exit(1 if wrong else 0)


if __name__ == "__main__":
    main()
