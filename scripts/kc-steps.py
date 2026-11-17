"""kc's steps (docs/research/kc.md): each program of tests/kc/stepN/,
compiled by each of kc's back ends that takes it, run, and its value
compared with the one it states (`;; => V`), which must be Fixpt's (the
pinned Fixpt's `eval`, on the fx-cells-rsinterp machine).

usage: python3 -I scripts/kc-steps.py

  native  every program: `kc build` (C419), its objects assembled and
          linked by kc with the system's cc against crates/kayak-kcrt, run;
  rust    the KFX programs (`.kfx`): `kc --rust`, each a module of one
          crate (target/kc-rust), built once and run.

kc runs with one database for the whole sweep (target/kc-steps.db), so a
second sweep with nothing changed runs no query body: its count is said.

And early cutoff, as kc's database does it (C399): a program compiled,
then edited to text that means the same (spaces, a comment) runs again
only what reads the text (`checked`, `l0`), not the back end after it,
whose input, `l0`'s value, is as it was; edited to another value, the back
end runs too.

And a file's macros (C445): each form's expansion a query of its own, an
edit expands again only the forms that name what it changed.
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


def ran(stderr):
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
        got = ran(r.stderr)
        if want is not None and got != want:
            wrong.append("cutoff: after %r, ran %s, not %s" % (text, got, want))
    return wrong


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
        got = ran(r.stderr) or []
        n = (got.count("expansion"), got.count("macros-for"))
        if r.returncode != 0:
            wrong.append("expansions: %s: kc --check: %s" % (why, r.stderr.strip()))
        elif n != want:
            wrong.append("expansions: %s: ran %d expansions and %d macros-for, not %d and %d" % (why, n[0], n[1], want[0], want[1]))
    return wrong


def main():
    if sys.argv[1:]:
        sys.exit(__doc__)
    os.makedirs(WORK, exist_ok=True)
    db = os.path.join(ROOT, "target", "kc-steps.db")
    fx = fixpt()
    wrong, ran = [], 0
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
    for w in wrong:
        print("kc-steps: " + w)
    print("kc-steps: %d programs, %d by native code, %d by Rust; %d wrong; %d query bodies ran%s"
          % (len(ps), len(ps), len(kfx), len(wrong), ran, "" if fx else "; Fixpt not here, its answers unchecked"))
    sys.exit(1 if wrong else 0)


if __name__ == "__main__":
    main()
