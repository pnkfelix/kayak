"""Fixpt's FX-26 programs that kfx accepts, run both ways (PLAN.md, K1).

usage: python3 -I scripts/run-fixpt-programs.py [--list]

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

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
RUNNER = os.path.join(ROOT, "target", "fixpt-programs")
LIMIT = 30


def run(args, **kw):
    return subprocess.run(args, capture_output=True, text=True, **kw)


def forms(text):
    """The top-level forms' heads and sizes: (head, items), where items is
    what each binds, for definitions; a string's head is `"`."""
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
            out.append(("atom", 1))
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
    if head in ("define", "define*"):
        return (head, 1)
    return ("expression", 1)


def fixpt_values(binary, path, text):
    """The values Fixpt gives the program's top-level expressions, or why
    there are none."""
    try:
        r = run([binary, "--dialect", "fx26", "--step-limit", "none", "eval", path], timeout=LIMIT)
    except subprocess.TimeoutExpired:
        return None, "Fixpt runs past %ds" % LIMIT
    if r.returncode != 0:
        return None, "Fixpt fails: " + r.stderr.strip()[:200]
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
    return rust == "#<procedure>" and re.match(r"#<.*(closure|procedure).*>$", fixpt) is not None


def main():
    listing = sys.argv[1:] == ["--list"]
    fixpt = run([os.path.join(ROOT, "scripts", "fixpt.sh")]).stdout.strip()
    programs_dir = os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(fixpt))),
                                "crates", "fixpt-fx26", "tests", "programs")
    if run(["cargo", "build", "--release", "--quiet", "-p", "kfx"], cwd=ROOT).returncode != 0:
        sys.exit("run-fixpt-programs: kfx does not build")
    kfx = os.path.join(ROOT, "target", "release", "kfx")

    shutil.rmtree(RUNNER, ignore_errors=True)
    os.makedirs(os.path.join(RUNNER, "src"))
    accepted = []
    for d, _, fs in sorted(os.walk(programs_dir)):
        # Fixpt's diverge/ programs are meant to run for ever.
        if os.path.relpath(d, programs_dir).startswith("diverge"):
            continue
        for f in sorted(fs):
            if not f.endswith(".fx"):
                continue
            path = os.path.join(d, f)
            r = run([kfx, "--toplevel", path], timeout=LIMIT)
            if r.returncode != 0:
                continue
            name = "p%d" % len(accepted)
            with open(os.path.join(RUNNER, "src", name + ".rs"), "w") as g:
                g.write(r.stdout)
            accepted.append((name, os.path.relpath(path, programs_dir), path))

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
    for name, rel, path in accepted:
        with open(path) as g:
            text = g.read()
        expected, why = fixpt_values(fixpt, path, text)
        if expected is None:
            skipped.append((rel, why))
            continue
        try:
            r = run([runner, name], timeout=LIMIT)
        except subprocess.TimeoutExpired:
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
    sys.exit(1 if differ else 0)


main()
