"""Fixpt's FX-26 programs that kfx accepts, run both ways (PLAN.md, K1).

usage: python3 -I scripts/run-fixpt-programs.py [--list] [--bless]

Each program in Fixpt's crates/fixpt-fx26/tests/programs that kfx accepts
is compiled with `kfx --toplevel`, whose `kfx_toplevel` writes the values
of its top-level expressions as `fixpt eval` does. All are built into one
runner, in target/fixpt-programs, and each is run there and by Fixpt,
with a time limit; the values are compared, line by line. A procedure is
written as its machine made it, by Fixpt, so any `#<...>` naming a
closure matches any procedure. Exits 1 if any program's values differ.
"""

import os
import re
import shutil
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import fixptcache  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
RUNNER = os.path.join(ROOT, "target", "fixpt-programs")
LIMIT = 30


def run(args, **kw):
    return subprocess.run(args, capture_output=True, text=True, **kw)


def forms(text):
    """The top-level forms' heads and sizes: (head, items), where items is
    what each binds, for definitions; a string's head is `"`, and a name's
    or a number's, an expression's."""
    out, i, n = [], 0, len(text)
    while i < n:
        c = text[i]
        if c in " \t\r\n":
            i += 1
        elif c == ";":
            while i < n and text[i] != "\n":
                i += 1
        elif c == "#" and text.startswith("#|", i):
            i = text.index("|#", i) + 2
        elif c == '"':
            i = skip_string(text, i)
            out.append(('"', 1))
        elif c == "(":
            j = skip_list(text, i)
            out.append(head_of(text[i:j]))
            i = j
        else:
            j = i
            while j < n and text[j] not in " \t\r\n()":
                j += 1
            out.append(("expression", 1))
            i = j
    return out


def skip_string(text, i):
    i += 1
    while text[i] != '"':
        i += 2 if text[i] == "\\" else 1
    return i + 1


def skip_list(text, i):
    depth = 0
    while True:
        c = text[i]
        if c == '"':
            i = skip_string(text, i)
            continue
        if c == ";":
            while text[i] != "\n":
                i += 1
            continue
        if c == "#" and text[i + 1] == "\\":
            i += 3
            continue
        if c == "(":
            depth += 1
        elif c == ")":
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1


def items(form):
    """A list's items, each a string of its text."""
    out, i = [], 1
    while i < len(form) - 1:
        c = form[i]
        if c in " \t\r\n":
            i += 1
        elif c == ";":
            while form[i] != "\n":
                i += 1
        elif c == '"':
            j = skip_string(form, i)
            out.append(form[i:j])
            i = j
        elif c == "(":
            j = skip_list(form, i)
            out.append(form[i:j])
            i = j
        elif c == "#" and form[i + 1] == "\\":
            out.append(form[i : i + 3])
            i += 3
        else:
            j = i
            while j < len(form) and form[j] not in " \t\r\n()":
                j += 1
            out.append(form[i:j])
            i = j
    return out


def head_of(form):
    xs = items(form)
    head = xs[0] if xs else ""
    if head in ("define-effect", "define-type"):
        return (head, 0)
    if head == "define-rec":
        return (head, len(xs) - 1)
    if head == "define-datatype":
        return (head, len(xs) - 2)
    if head == "define-generative":
        return (head, 2)
    if head in ("define", "define*"):
        return (head, 1)
    return ("expression", 1)


VALUES = {}


def fixpt_values(binary, path, text):
    """The values Fixpt gives the program's top-level expressions, or why
    there are none; each program's once, and cached (fixptcache.py)."""
    if path not in VALUES:
        VALUES[path] = fixpt_values_run(binary, path, text)
    return VALUES[path]


def prefetch(binary, paths):
    """Fixpt's values for each of `paths`, many at once."""
    def one(path):
        with open(path) as g:
            return path, fixpt_values_run(binary, path, g.read())
    for path, v in fixptcache.parallel(one, [p for p in paths if p not in VALUES]):
        VALUES[path] = v


def fixpt_values_run(binary, path, text):
    try:
        r = fixptcache.run(binary, ["--dialect", "fx26", "--step-limit", "none", "eval", path], [path], timeout=LIMIT)
    except subprocess.TimeoutExpired:
        return None, "Fixpt runs past %ds" % LIMIT
    if r.returncode != 0:
        return None, "REFUSED: " + r.stderr.strip()[:200]
    lines = [l for l in r.stdout.splitlines() if not l.startswith(";")]
    values = []
    for head, count in forms(text):
        taken, lines = lines[:count], lines[count:]
        if head in ("expression", '"'):
            for l in taken:
                values.append(l.rsplit(" ! ", 1)[0].rsplit(" : ", 1)[0])
    return values, None


def same(fixpt, rust):
    if fixpt == rust:
        return True
    return rust in ("#<procedure>", "#<closure>") and re.match(r"#<.*(closure|procedure).*>$", fixpt) is not None


ENGINE_PASSING = os.path.join(ROOT, "tests", "engine-fixpt-passing")


def engine_runs(accepted, fixpt, listing, bless):
    """FX-26's engine (C58) on the same programs: K6. A wrong value fails;
    a program it cannot run yet is counted; one in tests/engine-fixpt-passing
    that no longer agrees fails."""
    engine = os.path.join(ROOT, "target", "release", "kayak-fx26")
    if run(["cargo", "build", "--release", "--quiet", "-p", "kayak-fx26"], cwd=ROOT).returncode != 0:
        sys.exit("run-fixpt-programs: the engine does not build")
    agree, differ, cannot = [], [], []
    ran = dict(zip([p for _, _, p in accepted], fixptcache.parallel(lambda p: timed(engine, p), [p for _, _, p in accepted])))
    for name, rel, path in accepted:
        with open(path) as g:
            text = g.read()
        expected, why = fixpt_values(fixpt, path, text)
        r = ran[path]
        if r is None:
            cannot.append((rel, "runs past %ds" % LIMIT))
            continue
        if expected is None and why.startswith("REFUSED: ! evaluation failed"):
            if r.returncode != 0:
                agree.append(rel)
            else:
                differ.append((rel, "Fixpt fails when run, and the engine does not"))
            continue
        if expected is None:
            continue
        if r.returncode != 0:
            said = [l for l in r.stderr.splitlines() if "engine" in l or "kfx:" in l]
            cannot.append((rel, (said or r.stderr.strip().splitlines()[-1:] or ["?"])[0][:160]))
            continue
        got = r.stdout.splitlines()
        if len(got) == len(expected) and all(same(e, g) for e, g in zip(expected, got)):
            agree.append(rel)
        else:
            differ.append((rel, "Fixpt %r, the engine %r" % (expected[:6], got[:6])))
    if listing:
        for rel, why in cannot:
            print("engine cannot yet %s: %s" % (rel, why))
    for rel, why in differ:
        print("ENGINE DIFFERS  %s: %s" % (rel, why))
    was = set(open(ENGINE_PASSING).read().split()) if os.path.exists(ENGINE_PASSING) else set()
    lost = sorted(was - set(agree))
    if bless:
        with open(ENGINE_PASSING, "w") as g:
            g.write("".join(r + "\n" for r in agree))
        lost = []
    for rel in lost:
        print("ENGINE NO LONGER AGREES  %s" % rel)
    print("run-fixpt-programs: the engine: %d agree, %d differ, %d not run yet" % (len(agree), len(differ), len(cannot)))
    return bool(differ or lost)


def timed(binary, *args):
    """A run, or None if it runs past the limit."""
    try:
        return run([binary] + list(args), timeout=LIMIT)
    except subprocess.TimeoutExpired:
        return None


def beyond_runs(refused, fixpt, listing):
    """The engine on Fixpt's programs that kfx refuses: FX-26 beyond KFX,
    what the engine is for (C58, C60). Counted, not yet failed: a program
    the engine's checker refuses is the checker's to grow."""
    engine = os.path.join(ROOT, "target", "release", "kayak-fx26")
    agree, differ, checker, cannot = [], [], [], []
    ran = dict(zip([p for _, _, p in refused], fixptcache.parallel(lambda p: timed(engine, p), [p for _, _, p in refused])))
    for _, rel, path in refused:
        with open(path) as g:
            text = g.read()
        expected, why = fixpt_values(fixpt, path, text)
        if expected is None and not why.startswith("REFUSED: ! evaluation failed"):
            continue
        r = ran[path]
        if r is None:
            cannot.append((rel, "runs past %ds" % LIMIT))
            continue
        if expected is None:
            (agree if r.returncode != 0 else differ).append(rel if r.returncode != 0 else (rel, "Fixpt fails when run, the engine not"))
            continue
        if r.returncode != 0:
            said = [l for l in r.stderr.splitlines() if "engine" in l or "kfx:" in l]
            line = (said or r.stderr.strip().splitlines()[-1:] or ["?"])[0]
            (cannot if "the engine" in line else checker).append((rel, line[:160]))
            continue
        got = r.stdout.splitlines()
        if len(got) == len(expected) and all(same(e, g) for e, g in zip(expected, got)):
            agree.append(rel)
        else:
            differ.append((rel, "Fixpt %r, the engine %r" % (expected[:6], got[:6])))
    if listing:
        for rel in agree:
            print("beyond agree %s" % rel)
        for rel, why in checker:
            print("beyond, the checker refuses %s: %s" % (rel, why))
        for rel, why in cannot:
            print("beyond, the engine cannot yet %s: %s" % (rel, why))
    for rel, why in differ:
        print("BEYOND, THE ENGINE DIFFERS  %s: %s" % (rel, why))
    print("run-fixpt-programs: beyond KFX, the engine: %d agree, %d differ, %d its checker refuses, %d not run yet"
          % (len(agree), len(differ), len(checker), len(cannot)))


def main():
    listing = "--list" in sys.argv[1:]
    bless = "--bless" in sys.argv[1:]
    fixpt = run([os.path.join(ROOT, "scripts", "fixpt.sh")]).stdout.strip()
    programs_dir = os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(fixpt))),
                                "crates", "fixpt-fx26", "tests", "programs")
    if run(["cargo", "build", "--release", "--quiet", "-p", "kfx"], cwd=ROOT).returncode != 0:
        sys.exit("run-fixpt-programs: kfx does not build")
    kfx = os.path.join(ROOT, "target", "release", "kfx")

    shutil.rmtree(RUNNER, ignore_errors=True)
    os.makedirs(os.path.join(RUNNER, "src"))
    accepted, refused = [], []
    paths = []
    for d, _, fs in sorted(os.walk(programs_dir)):
        # Fixpt's diverge/ programs are meant to run for ever.
        if os.path.relpath(d, programs_dir).startswith("diverge"):
            continue
        paths.extend(os.path.join(d, f) for f in sorted(fs) if f.endswith(".fx"))
    compiled = fixptcache.parallel(lambda path: run([kfx, "--toplevel", path], timeout=LIMIT), paths)
    for path, r in zip(paths, compiled):
        if r.returncode != 0:
            refused.append(("", os.path.relpath(path, programs_dir), path))
            continue
        name = "p%d" % len(accepted)
        with open(os.path.join(RUNNER, "src", name + ".rs"), "w") as g:
            g.write(r.stdout)
        accepted.append((name, os.path.relpath(path, programs_dir), path))
    prefetch(fixpt, paths)

    with open(os.path.join(RUNNER, "Cargo.toml"), "w") as g:
        g.write('[package]\nname = "fixpt-programs"\nversion = "0.1.0"\nedition = "2024"\n\n'
                '[dependencies]\nkayak-rt = { path = "%s" }\n\n[workspace]\n'
                % os.path.join(ROOT, "crates", "kayak-rt"))
    with open(os.path.join(RUNNER, "src", "main.rs"), "w") as g:
        for name, rel, _ in accepted:
            g.write("#[allow(warnings, clippy::all)]\nmod %s; // %s\n" % (name, rel))
        g.write("\nfn main() {\n    let which = std::env::args().nth(1).unwrap();\n    let f: fn() -> kayak_rt::Str = match which.as_str() {\n")
        for name, _, _ in accepted:
            g.write('        "%s" => %s::kfx_toplevel,\n' % (name, name))
        g.write('        _ => panic!("no program {which}"),\n    };\n'
                '    print!("{}", kayak_rt::with_big_stack(f));\n}\n')
    built = run(["cargo", "build", "--release", "--quiet"], cwd=RUNNER)
    if built.returncode != 0:
        sys.stderr.write(built.stderr[-4000:])
        sys.exit("run-fixpt-programs: the programs' Rust does not build")
    runner = os.path.join(RUNNER, "target", "release", "fixpt-programs")

    agree, differ, skipped = 0, [], []
    rust_ran = dict(zip([n for n, _, _ in accepted], fixptcache.parallel(lambda n: timed(runner, n), [n for n, _, _ in accepted])))
    for name, rel, path in accepted:
        with open(path) as g:
            text = g.read()
        expected, why = fixpt_values(fixpt, path, text)
        if expected is None and why.startswith("REFUSED: ! evaluation failed"):
            # A run-time error, Fixpt's: the Rust must fail too.
            r = rust_ran[name]
            if r is None:
                differ.append((rel, "Rust runs past %ds" % LIMIT))
                continue
            if r.returncode != 0:
                agree += 1
                if listing:
                    print("agree   %s (both fail when run)" % rel)
            else:
                differ.append((rel, "Fixpt fails when run, and Rust does not: " + why))
            continue
        if expected is None and why.startswith("REFUSED"):
            # kfx accepted what Fixpt refuses: K1's other half, broken.
            differ.append((rel, "kfx accepts it, and Fixpt " + why))
            continue
        if expected is None:
            skipped.append((rel, why))
            continue
        r = rust_ran[name]
        if r is None:
            differ.append((rel, "Rust runs past %ds" % LIMIT))
            continue
        if r.returncode != 0:
            differ.append((rel, "Rust fails: " + r.stderr.strip()[-200:]))
            continue
        got = r.stdout.splitlines()
        if len(got) == len(expected) and all(same(e, g) for e, g in zip(expected, got)):
            agree += 1
            if listing:
                print("agree   %s" % rel)
        else:
            differ.append((rel, "Fixpt %r, Rust %r" % (expected, got)))
    for rel, why in skipped:
        print("skipped %s: %s" % (rel, why))
    for rel, why in differ:
        print("DIFFER  %s: %s" % (rel, why))
    print("run-fixpt-programs: %d of Fixpt's programs kfx accepts; %d agree, %d differ, %d skipped"
          % (len(accepted), agree, len(differ), len(skipped)))
    engine_failed = engine_runs(accepted, fixpt, listing, bless)
    beyond_runs(refused, fixpt, listing)
    sys.exit(1 if differ or engine_failed else 0)


main()
