"""The corpus (TODO.md, section 8): tests/kfx/NAME.kfx, each a KFX program
whose `main` gives its answer as a string.

usage: python3 -I scripts/corpus.py [--bless]

A program is a file, NAME.kfx, or several, NAME.unit, a manifest of its
files in order (kept in NAME/). For each, two things are kept beside it,
and checked:
  NAME.rs   the Rust kfx makes of it, or NAME/*.rs and NAME/mod.rs for a
            program of several files (golden: a change to the compiler
            shows in every program it changes);
  NAME.out  its answer, as Fixpt gives it running the program as FX-26,
            the prelude first (stage 0's Fixpt, FIXPT_REV).
crates/kfx-corpus builds every NAME.rs, and its test requires each one's
answer to be NAME.out: K1, program by program. --bless rewrites the files
instead of checking them. With no Fixpt clone the answers are not checked,
and the script says so.

And two directories of programs kfx must refuse:
  tests/kfx-refused   FX-26 refuses each too: K1's other half, that KFX
                      accepts nothing FX-26 refuses;
  tests/kfx-stricter  FX-26 accepts each: where KFX is stricter, on
                      purpose, each program saying why.
"""

import os
import re
import shutil
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import fixptcache  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CORPUS = os.path.join(ROOT, "tests", "kfx")
LIB = os.path.join(ROOT, "crates", "kfx-corpus", "src", "lib.rs")


def run(args, **kw):
    return subprocess.run(args, capture_output=True, text=True, **kw)


def fixpt():
    r = run([os.path.join(ROOT, "scripts", "fixpt.sh")])
    if r.returncode == 3:
        return None
    if r.returncode != 0:
        sys.stderr.write(r.stderr)
        sys.exit(r.returncode)
    return r.stdout.strip()


def kfx():
    r = run(["cargo", "build", "--release", "--quiet", "-p", "kfx"], cwd=ROOT)
    if r.returncode != 0:
        sys.stderr.write(r.stderr)
        sys.exit("corpus: kfx does not build")
    return os.path.join(ROOT, "target", "release", "kfx")


def manifest(path):
    """A manifest's files, in order."""
    with open(path) as f:
        lines = [l.strip() for l in f]
    return [os.path.join(os.path.dirname(path), l) for l in lines if l and not l.startswith(";")]


def files_of(path):
    return manifest(path) if path.endswith(".unit") else [path]


def stage0(path, then):
    """The prelude, the program's files, and `then`, as one file's text."""
    with open(os.path.join(ROOT, "kfx", "prelude.fx")) as f:
        text = f.read()
    for p in files_of(path):
        with open(p) as f:
            text += "\n" + f.read()
    return text + then


def own_loads(path):
    """The files a program loads of its own (not Fixpt's `fx26:` ones), and
    theirs, as paths relative to its directory (C118)."""
    base = os.path.dirname(path)
    found, todo = [], list(files_of(path))
    while todo:
        p = todo.pop()
        with open(p) as f:
            for m in re.finditer(r'\((?:load-module|load-input) "([^"]+)"\)', f.read()):
                rel = os.path.normpath(os.path.join(os.path.relpath(os.path.dirname(p), base), m.group(1)))
                if not m.group(1).startswith("fx26:") and rel not in found:
                    found.append(rel)
                    todo.append(os.path.join(base, rel))
    return found


def fixpt_on(binary, text, args, path=None, keep=False):
    """Fixpt on `text` as one file, cached (fixptcache.py); beside it, the
    files program `path` loads of its own."""
    with tempfile.TemporaryDirectory() as d:
        tmp = os.path.join(d, "stage0.fx")
        with open(tmp, "w") as f:
            f.write(text)
        copies = []
        for rel in own_loads(path) if path is not None else []:
            os.makedirs(os.path.dirname(os.path.join(d, rel)), exist_ok=True)
            shutil.copyfile(os.path.join(os.path.dirname(path), rel), os.path.join(d, rel))
            copies.append(os.path.join(d, rel))
        # An answer is kept as NAME.out, not in tests/fixpt-answers; a
        # refusal there (C132).
        name = os.path.splitext(os.path.relpath(path, ROOT))[0].replace(os.sep, "--") if path else None
        return fixptcache.run(binary, ["--dialect", "fx26"] + args + [tmp], [tmp] + copies, name=name, keep=keep)


def answer(binary, path):
    r = fixpt_on(binary, stage0(path, "\n(main)\n"), ["--step-limit", "none", "eval", "--raw"], path)
    if r.returncode != 0:
        return None, r.stderr
    return r.stdout, None


def refused(compiler, interpreter, kind, wrong):
    directory = os.path.join(ROOT, "tests", kind)
    names = sorted(f[:-4] for f in os.listdir(directory) if f.endswith(".kfx"))

    def both(n):
        src = os.path.join(directory, n + ".kfx")
        mine = run([compiler, src])
        theirs = fixpt_on(interpreter, stage0(src, ""), ["eval"], src, keep=True)
        return mine, theirs

    ran = fixptcache.parallel(both, names)
    for n, (r, theirs) in zip(names, ran):
        src = os.path.join(directory, n + ".kfx")
        if r.returncode == 0:
            wrong.append("%s/%s: kfx accepts it" % (kind, n))
        elif not any(l.startswith("kfx: " + src + ":") for l in r.stderr.splitlines()):
            # Every refusal says where: the file, and the line (TODO.md section 7).
            wrong.append("%s/%s: kfx's message says not where: %s" % (kind, n, r.stderr.strip()[-200:]))
        if theirs is not None:
            accepted = theirs.returncode == 0
            if accepted != (kind == "kfx-stricter"):
                wrong.append("%s/%s: Fixpt %s it" % (kind, n, "accepts" if accepted else "refuses"))
    return len(names)


def lib(names):
    out = [
        "//! Generated by `scripts/corpus.py`: every program in `tests/kfx`, as the",
        "//! Rust `kfx` makes of it. Do not edit; run `scripts/corpus.py --bless`.",
        "",
    ]
    for n in names:
        out.append("#[allow(warnings, clippy::all)]")
        out.append("#[rustfmt::skip]")
        if os.path.exists(os.path.join(CORPUS, n + ".unit")):
            out.append('#[path = "../../../tests/kfx/%s/mod.rs"]' % n)
        else:
            out.append('#[path = "../../../tests/kfx/%s.rs"]' % n)
        out.append("pub mod %s;" % n.replace("-", "_"))
        out.append("")
    out.append("/// A program's `main`, which gives its answer.")
    out.append("pub type Main = fn() -> kayak_rt::Str;")
    out.append("")
    # A `main` that may suspend (C45) is an `async fn`: run to its end.
    suspending = [n for n in names if is_async_main(n)]
    for n in suspending:
        m = n.replace("-", "_")
        out.append("fn %s_main() -> kayak_rt::Str {" % m)
        out.append("    kayak_rt::suspend::block(%s::main()).unwrap_or_else(|u| kayak_rt::unreachable_unwind(u))" % m)
        out.append("}")
        out.append("")
    out.append("/// Each program's name, and its `main`.")
    out.append("pub const PROGRAMS: &[(&str, Main)] = &[")
    for n in names:
        m = n.replace("-", "_")
        out.append('    ("%s", %s),' % (n, m + "_main" if n in suspending else m + "::main"))
    out.append("];")
    return "\n".join(out) + "\n"


def is_async_main(n):
    if os.path.exists(os.path.join(CORPUS, n + ".unit")):
        d = os.path.join(CORPUS, n)
        paths = [os.path.join(d, f) for f in sorted(os.listdir(d)) if f.endswith(".rs")]
    else:
        paths = [os.path.join(CORPUS, n + ".rs")]
    for path in paths:
        with open(path) as f:
            if "pub async fn main()" in f.read():
                return True
    return False


def main():
    bless = sys.argv[1:] == ["--bless"]
    if sys.argv[1:] not in ([], ["--bless"]):
        sys.exit("usage: corpus.py [--bless]")
    compiler, interpreter = kfx(), fixpt()
    if interpreter is None:
        print("corpus: no Fixpt clone, so answers are not checked")
    names = sorted(os.path.splitext(f)[0] for f in os.listdir(CORPUS)
                   if f.endswith(".kfx") or f.endswith(".unit"))
    wrong = []

    def keep(path, text):
        old = open(path).read() if os.path.exists(path) else None
        if old == text:
            return
        if bless:
            with open(path, "w") as f:
                f.write(text)
        else:
            wrong.append(os.path.relpath(path, ROOT) + (" is missing" if old is None else " differs"))

    def made(n):
        """kfx's Rust of program `n` (its files' texts, for a unit), and
        Fixpt's answer."""
        unit = os.path.join(CORPUS, n + ".unit")
        src = unit if os.path.exists(unit) else os.path.join(CORPUS, n + ".kfx")
        with tempfile.TemporaryDirectory() as out:
            r = run([compiler, src, out] if src == unit else [compiler, src])
            files = {f: open(os.path.join(out, f)).read() for f in sorted(os.listdir(out))} if src == unit else None
        theirs = answer(interpreter, src) if interpreter is not None else None
        return r, files, theirs

    for n, (r, files, theirs) in zip(names, fixptcache.parallel(made, names)):
        unit = os.path.join(CORPUS, n + ".unit")
        src = unit if os.path.exists(unit) else os.path.join(CORPUS, n + ".kfx")
        if r.returncode != 0:
            said = [l for l in r.stderr.splitlines() if l.startswith("kfx:")]
            wrong.append("%s: kfx refuses it: %s" % (n, (said or r.stderr.strip().splitlines()[-1:])[0]))
            continue
        if files is not None:
            for f, text in files.items():
                keep(os.path.join(CORPUS, n, f), text)
        else:
            keep(os.path.join(CORPUS, n + ".rs"), r.stdout)
        if theirs is not None:
            got, err = theirs
            if err is not None:
                wrong.append("%s: Fixpt refuses it: %s" % (n, err.strip()))
            else:
                keep(os.path.join(CORPUS, n + ".out"), got)
    keep(LIB, lib(names))
    counts = [refused(compiler, interpreter, kind, wrong) for kind in ("kfx-refused", "kfx-stricter")]
    for w in wrong:
        print("corpus: " + w)
    if wrong:
        sys.exit(1)
    print("corpus: %d programs %s; %d refused by both, %d by KFX alone"
          % (len(names), "blessed" if bless else "agree", counts[0], counts[1]))


main()
