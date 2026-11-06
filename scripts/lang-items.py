"""kayak's lang items (docs/research/coopt-adoption.md §6, step 1; C367):
one table, lang/items.sexp, of what the compiler, the runtimes and the
standard library name across their boundaries, after rustc's lang items
(a closed list the compiler knows; definitions the libraries supply; each
defined once, and its absence an error where it is needed).

The table mirrors today's vocabularies, each from where it is defined:
- `primitive`: a runtime primitive, its name and arity (`max` -1 if
  variadic). Fixpt's from its `PRIMITIVES` (`fixpt-runtime/src/prim.rs`,
  the first of a name, as `runtime_primitive` finds it); kayak's own those
  tier 1 has that Fixpt has not.
- `immediate`: a value of Fixpt's that is no object, from `value.rs`.
- `syscall`: a host operation of kayak-rt's registry (`syscall.rs`), its
  type as the registry has it.
- `runtime-fn`: a KFX name whose Rust is kayak-rt's function of its
  mangled name; `emitted`: one the emitter writes itself.
- `emitter-call`: a function of kayak-rt's the emitter's Rust calls.

And checks each copy against it:
- tier 1's primitives (`cells/cprims.kfx`'s `pdef`s): each a primitive's,
  of its arity;
- the immediates in `cells/layout.kfx`, `fx26/vheap.kfx` and kayak-jit;
- each `define-syscall` a syscall's, and one name declared alike wherever
  it is;
- the KFX names typed in `kfx/check-defs.kfx` (`primitives`, `prelude`):
  each a `runtime-fn` (and kayak-rt has its function) or `emitted`;
- the emitter's literals: no kayak-rt function called there that is not
  an `emitter-call`.
Where Fixpt's own table names one primitive twice, it is said, not failed:
that is Fixpt's to fix (kayak takes its first, as Fixpt does).

usage: python3 -I scripts/lang-items.py [--check | --update]

--update writes lang/items.sexp from today's sources (needs the clone);
--check (the default) fails if the table is not what --update would
write, or if a copy disagrees with it.
"""

import glob
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
TABLE = os.path.join(ROOT, "lang", "items.sexp")


def clone():
    return os.environ.get("KAYAK_FIXPT_CLONE", os.path.join(ROOT, "..", "kayak-refs", "fixpt"))


def read(path):
    with open(os.path.join(ROOT, path) if not os.path.isabs(path) else path, encoding="utf-8") as f:
        return f.read()


# -------------------------------------------------------------- sources


def fixpt_primitives(problems):
    """Fixpt's primitives, the first of each name, in order: (name, min, max)."""
    src = read(os.path.join(clone(), "crates/fixpt-runtime/src/prim.rs"))
    body = src[src.index("prims! {"):]
    seen, out = {}, []
    for m in re.finditer(r'^\s*"([^"]+)",\s*(\d+),\s*(?:Some\((\d+)\)|None)\s*,', body, re.M):
        name, lo, hi = m.group(1), int(m.group(2)), int(m.group(3)) if m.group(3) else -1
        if name in seen:
            if seen[name] != (lo, hi):
                problems.append(
                    f"note: Fixpt's PRIMITIVES names {name} twice, {seen[name]} and {(lo, hi)}; "
                    "the first is found (Fixpt's to fix)"
                )
            continue
        seen[name] = (lo, hi)
        out.append((name, lo, hi))
    return out


def fixpt_immediates():
    """Fixpt's immediates (`value.rs`): (name, value), and the sub-tags."""
    src = read(os.path.join(clone(), "crates/fixpt-heap/src/value.rs"))
    sub = {m.group(1): int(m.group(2)) for m in re.finditer(r"^const IMM_([A-Z0-9_]+): u64 = (\d+);", src, re.M)}
    shift = int(re.search(r"^const IMM_SHIFT: u32 = (\d+);", src, re.M).group(1))
    tag = int(re.search(r"^pub const TAG_IMMEDIATE: u64 = (\d+);", src, re.M).group(1)) if re.search(
        r"^pub const TAG_IMMEDIATE: u64 = (\d+);", src, re.M) else 3
    out = []
    for m in re.finditer(r"pub const ([A-Z_]+): Value = Value::imm\(IMM_([A-Z0-9_]+), 0\);", src):
        out.append((m.group(1).lower(), (sub[m.group(2)] << shift) | tag))
    for name in ("CHAR", "F32"):
        out.append((f"sub-{name.lower()}", sub[name]))
    return out


def tier1_pdefs():
    return [(m.group(1), int(m.group(2)), int(m.group(3)))
            for m in re.finditer(r'\(pdef "([^"]+)" (-?\d+) (-?\d+) ', read("cells/cprims.kfx"))]


def registry_syscalls():
    src = read("crates/kayak-rt/src/syscall.rs")
    return [(m.group(1), m.group(2)) for m in re.finditer(r'host: "([^"]+)",\s*canonical: "([^"]*)"', src)]


def declared_syscalls():
    """Each `(define-syscall NAME "HOST" TYPE)` in KFX: (file, host, type text)."""
    out = []
    for path in sorted(glob.glob(os.path.join(ROOT, "*", "*.kfx"))):
        rel = os.path.relpath(path, ROOT)
        s = read(path)
        for m in re.finditer(r'\(define-(?:optional-)?syscall [^\s()]+ "([^"]+)"\s*', s):
            ty, _ = sexp_at(s, m.end())
            out.append((rel, m.group(1), " ".join(ty.split())))
    return out


def sexp_at(s, i):
    """The s-expression text from `i`, and where it ends."""
    while s[i].isspace():
        i += 1
    if s[i] != "(":
        j = i
        while j < len(s) and not s[j].isspace() and s[j] not in "()":
            j += 1
        return s[i:j], j
    depth, j = 0, i
    while True:
        c = s[j]
        if c == '"':
            j += 1
            while s[j] != '"':
                j += 2 if s[j] == "\\" else 1
        elif c == "(":
            depth += 1
        elif c == ")":
            depth -= 1
            if depth == 0:
                return s[i:j + 1], j + 1
        j += 1


def string_constant(src, name):
    """The text of `(define NAME string "…")`, unescaped."""
    m = re.search(r'\(define ' + re.escape(name) + r' string\s*"', src)
    i, out = m.end(), []
    while src[i] != '"':
        if src[i] == "\\":
            out.append(src[i + 1])
            i += 2
        else:
            out.append(src[i])
            i += 1
    return "".join(out)


def typed_names():
    """The KFX names `check-defs.kfx` types: its `primitives` and `prelude`."""
    src = read("kfx/check-defs.kfx")
    names = []
    for const in ("primitives", "prelude"):
        text, i = string_constant(src, const), 0
        while True:
            while i < len(text) and text[i].isspace():
                i += 1
            if i >= len(text):
                break
            entry, i = sexp_at(text, i)
            names.append(entry[1:].split(None, 1)[0])
    return names


RUST_KEYWORDS = set("""as async await box break const continue do dyn else enum extern false final fn for gen
if impl in let loop macro match mod move mut override priv pub ref return static struct trait true try type
typeof unsafe unsized use virtual where while yield abstract become""".split())
MANGLED = {"-": "_", "?": "_p", "!": "_x", "+": "_plus", "*": "_star", "/": "_slash", "<": "_lt",
           ">": "_gt", "=": "_eq", "%": "_pct", ":": "_colon"}


def mangle(n):
    """kfx/names.kfx's `mangle`."""
    if n == "-":
        return "_minus"
    if n in ("self", "super", "crate", "Self"):
        return n + "_"
    if n in RUST_KEYWORDS:
        return "r#" + n
    out, i = [], 0
    while i < len(n):
        if n[i] == "-" and i + 1 < len(n) and n[i + 1] == ">":
            out.append("_to_")
            i += 2
            continue
        if n[i] == "-" and i + 1 == len(n):
            out.append("_minus")
        else:
            out.append(MANGLED.get(n[i], n[i]))
        i += 1
    return "".join(out)


def runtime_fns():
    out = set()
    for p in glob.glob(os.path.join(ROOT, "crates/kayak-rt/src/*.rs")):
        out.update(m.group(1) for m in re.finditer(r"pub (?:const )?fn ([a-z_][a-z0-9_]*)", read(p)))
    return out


def emitter_called(rt):
    """kayak-rt's functions the emitter's Rust literals call."""
    out = set()
    for p in ("kfx/emit.kfx", "kfx/emit-tops.kfx", "kfx/incr.kfx", "kfx/compile.kfx"):
        for m in re.finditer(r'"((?:[^"\\]|\\.)*)"', read(p)):
            for n in re.finditer(r"\b([a-z_][a-z0-9_]*)(?:::<[^>(]*>)?\(", m.group(1)):
                if n.group(1) in rt:
                    out.add(n.group(1))
    return out


# ---------------------------------------------------------------- table


def table_text(problems):
    fx = fixpt_primitives(problems)
    fx_names = {n for n, _, _ in fx}
    rt = runtime_fns()
    lines = [
        ";; kayak's lang items (scripts/lang-items.py, C367): what the compiler,",
        ";; the runtimes and the standard library name across their boundaries.",
        ";; Written by `scripts/lang-items.py --update`; checked by `--check`.",
        ";; Each row: (KIND NAME ... SOURCE), SOURCE `fixpt` (the pin's) or `kayak`.",
        "",
        ";; Runtime primitives: name, least and most arguments (-1: any).",
    ]
    lines += [f'(primitive "{n}" {lo} {hi} fixpt)' for n, lo, hi in fx]
    lines += [f'(primitive "{n}" {lo} {hi} kayak)' for n, lo, hi in tier1_pdefs() if n not in fx_names]
    lines += ["", ";; Immediates: Fixpt's values that are no object, and the sub-tags."]
    lines += [f"(immediate {n} {v} fixpt)" for n, v in fixpt_immediates()]
    lines += ["", ";; Host operations (kayak-rt's syscall registry), their canonical types."]
    lines += [f'(syscall "{h}" "{t}" kayak)' for h, t in registry_syscalls()]
    lines += ["", ";; KFX's typed names: a kayak-rt function of the mangled name, or the emitter's own."]
    for n in typed_names():
        lines.append(f'(runtime-fn "{n}" kayak)' if mangle(n) in rt else f'(emitted "{n}" kayak)')
    lines += ["", ";; kayak-rt's functions the emitter's Rust calls."]
    lines += [f'(emitter-call "{n}" kayak)' for n in sorted(emitter_called(rt))]
    return "\n".join(lines) + "\n"


def table_rows(text):
    rows = []
    for line in text.splitlines():
        line = line.strip()
        if not line or line.startswith(";"):
            continue
        rows.append(re.findall(r'"((?:[^"\\]|\\.)*)"|([^\s()"]+)', line[1:-1]))
    return [[a or b for a, b in r] for r in rows]


# --------------------------------------------------------------- checks


def check_copies(rows, problems):
    prims = {r[1]: (int(r[2]), int(r[3])) for r in rows if r[0] == "primitive"}
    for name, lo, hi in tier1_pdefs():
        if name not in prims:
            problems.append(f"cells/cprims.kfx: primitive {name} is no lang item")
        elif prims[name] != (lo, hi):
            problems.append(f"cells/cprims.kfx: {name} takes {lo}..{hi}, the table {prims[name]}")

    imm = {r[1]: int(r[2]) for r in rows if r[0] == "immediate"}
    copies = [
        ("cells/layout.kfx", r"\(define v-([a-z-]+) i64 (\d+)\)",
         {"false": "false", "true": "true", "null": "null", "unit-word": "unit", "eof": "eof",
          "unspecified": "unspecified", "default": "default", "unbound": "unbound"}),
        ("cells/layout.kfx", r"\(define (sub-[a-z0-9]+) i64 (\d+)\)", {"sub-char": "sub-char", "sub-f32": "sub-f32"}),
        ("fx26/vheap.kfx", r"\(define w-([a-z]+) i64 (\d+)\)",
         {"unit": "unit", "nil": "null", "true": "true", "false": "false", "unbound": "unbound"}),
        ("crates/kayak-jit/src/lib.rs", r"pub const ([A-Z_]+): u64 = (\d+);",
         {"FALSE": "false", "TRUE": "true", "NULL": "null", "UNIT": "unit", "CHAR_SUBTAG": "sub-char"}),
    ]
    for path, pat, names in copies:
        for m in re.finditer(pat, read(path)):
            item = names.get(m.group(1))
            if item and imm.get(item) != int(m.group(2)):
                problems.append(f"{path}: {m.group(1)} is {m.group(2)}, the immediate {item} {imm.get(item)}")

    hosts = {r[1] for r in rows if r[0] == "syscall"}
    first = {}
    for path, host, ty in declared_syscalls():
        if host not in hosts:
            problems.append(f"{path}: syscall {host} is no lang item (not in kayak-rt's registry)")
        if host in first and first[host][1] != ty:
            problems.append(f"{path}: syscall {host} declared `{ty}`, {first[host][0]} `{first[host][1]}`")
        first.setdefault(host, (path, ty))

    rt = runtime_fns()
    for r in rows:
        if r[0] == "runtime-fn" and mangle(r[1]) not in rt:
            problems.append(f"lang item runtime-fn {r[1]}: kayak-rt has no `{mangle(r[1])}`")
        if r[0] == "emitter-call" and r[1] not in rt:
            problems.append(f"lang item emitter-call {r[1]}: kayak-rt has no `{r[1]}`")


def main():
    mode = sys.argv[1] if len(sys.argv) > 1 else "--check"
    if mode not in ("--check", "--update"):
        print(__doc__.strip().split("usage: ")[1].splitlines()[0], file=sys.stderr)
        return 2
    notes = []
    want = table_text(notes)
    if mode == "--update":
        os.makedirs(os.path.dirname(TABLE), exist_ok=True)
        with open(TABLE, "w", encoding="utf-8") as f:
            f.write(want)
    problems = []
    have = read(TABLE) if os.path.exists(TABLE) else ""
    if have != want:
        problems.append("lang/items.sexp is not what its sources make: scripts/lang-items.py --update, and say why")
    check_copies(table_rows(have or want), problems)
    for n in notes:
        print(f"lang-items: {n}")
    for p in problems:
        print(f"lang-items: {p}", file=sys.stderr)
    rows = table_rows(have or want)
    kinds = {}
    for r in rows:
        kinds[r[0]] = kinds.get(r[0], 0) + 1
    print("lang-items: " + ", ".join(f"{v} {k}" for k, v in kinds.items()) + ("" if problems else "; every copy agrees"))
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
