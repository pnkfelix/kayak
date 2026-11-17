"""Fixpt's macro expander written in FX-26 (`crates/fixpt-fx26/src/macros*.fx`),
made KFX files of kayak's engine (C429): kayak's expander is Fixpt's, its
text, not a port of it by hand, so that the pin's moving moves it.

usage: python3 -I scripts/macros-port.py [--check]

Each of Fixpt's files is a module of its own, loaded by the others
(`(define m ((proj (load-module "fx26:F.fx") rs re rm rc rp)))`, then
`(import m name ...)` and `(with m name)`); kayak's unit is one namespace.
The expander's files (`macros*.fx`) are taken whole; of the files they
load that are not the expander's (Fixpt's parser, its types, its reader's
types), only the definitions the expander reaches, by name, transitively.
Each kept form is as Fixpt writes it but for:

- the loads, module aliases and imports, which go (the files are in one
  unit, in the order the loads need), and aliases of a definition,
  `(define-type t (select m t))`, `(define x (with m x))`, which are the
  definition itself;
- each name: the definition it resolves to, in its file or through its
  imports, written as that definition's global name, its own unless
  another file, or kayak's unit, defines it too, then `mx-FILE-NAME`.
  Every occurrence of a name in a file is the same definition's, a local
  of that name too, so the renaming is consistent; quoted data, a
  `case`'s data, strings and comments are left as they are;
- `(with m name)`, `(select m name)`: the definition's global name;
- a module's region parameters, by position: `@ms @me @mm @mc @mp`;
- what KFX has not of FX-26 (keywords, `datum-int?`, `eq?`): the shim's
  (fx26/macros-shim.kfx, written by hand); a local KFX may not name so
  (`fail`): `mx-fail`.

--check: exits 1 if the files are not what the pinned Fixpt's make.
"""

import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, "fx26", "macros")
CLONE = os.environ.get("KAYAK_FIXPT_CLONE", os.path.join(ROOT, "..", "kayak-refs", "fixpt"))
SRC = "crates/fixpt-fx26/src"
# The expander's files are those loaded from these, taken whole if their
# names say they are the expander's.
ROOTS = ["macros-imports.fx", "macros-shapes.fx"]
WHOLE = re.compile(r"^macros.*\.fx$")
REGIONS = ["@ms", "@me", "@mm", "@mc", "@mp"]
# What an instance's making, and its expanding, may do: in its place, and
# fail to its prompt there (C437).
MX_EFFECT = "(maxeff (read @globals) spin (read mxp) (write mxp) (alloc mxp) (goto mxp))"
# What making an instance does: its state made, which fails to no prompt.
MX_MAKES = "(maxeff (read @globals) spin (read mxp) (write mxp) (alloc mxp))"
# The place's name, which no local of the expander's has (Rust's
# parameters are one namespace).
PLACE = "mxp"
# The asker's effect (C449), written where the shim and the driver say
# `mxe`, a file for each host: the engine's and the REPL's, which run
# `--phase1`, a process (`(io process)`, its command line a constant, so
# that a process holding the instance holds nothing of `@io`); kc's, its
# `phase1` query, of the database's effect `qe` (lib/db.kfx), its atoms
# written out, as a load sees the standard environment alone. Two files
# of one text but for the asker's effect: KFX's Rust writes no module type
# of an effect parameter (C24, C170), which one file would need.
ASKS = {"expander.fx": "(read @globals) (io process)",
        "expander-kc.fx": "(read @globals) nondet (read @q) (write @q) (alloc @q) (io process) (io fs-read) (io fs-write)"
                          " (io stderr) (io exit) (alloc @io) (read @io) (write @io)"}
# What KFX has not, the shim's in its place.
PRIMS = {"keyword?": "mx-keyword?", "keyword->symbol": "mx-keyword->symbol",
         "symbol->keyword": "mx-symbol->keyword", "datum-int?": "mx-datum-int?", "eq?": "mx-eq?",
         "datum-dotted": "mx-datum-dotted", "parse-nat": "mx-parse-nat",
         "%macros:expand-procedural": "mx-expand-procedural", "%macros:call-procedural": "mx-call-procedural",
         "%macros:limit": "mx-limit", "parse-number": "mx-parse-number",
         "datum-vector->list": "mx-datum-vector->list"}
SHIM = os.path.join(ROOT, "fx26", "macros-shim.kfx")
# The engine's driver of the expander, written by hand, read in last.
DRIVER = os.path.join(ROOT, "fx26", "macros-driver.kfx")
# Names a local may have in FX-26 and not in KFX, whose Rust kayak-rt or the
# emitter writes (`kfx-name?`): renamed where they are bound and named.
# The forms a region is written in: a region parameter's name in one of
# them is the region; anywhere else, a value's name (values and
# descriptions are named apart).
REGION_HEADS = {"ref", "listof", "arrayof", "flatarrayof", "narrayof", "read", "write", "alloc",
                "goto", "comefrom", "prompt-tag", "pairof", "bytevectorof", "icell", "mark-key",
                "bloblet", "blobref", "init", "await", "make-continuation-prompt-tag"}
# Procedures whose effect is written, `(subr E …)`, where Fixpt's `->*` has
# it found: a module's value names them as the module is made, which a
# module may not of its own `->*` procedures (its effects are found after
# its values are made). In Fixpt each is another file's, a module made
# before; here every file is one module's. Each effect is what Fixpt's
# checker finds; kfx checks it.
WRITTEN = {("parser.fx", "mk-symbol"): "pure", ("macros-nodes.fx", "def-env-top"): "pure"}
# Fixpt's text changed before it is read, each where KFX cannot say what
# it says: the file, the text, what it is made, and why. Each must be
# found once (the pin's moving says so when it is not).
PATCHES = {
    # The parse's prompt allows its delimited computations its host's
    # effect too (C449): an expansion calls the host's asker, of the file's
    # effect parameter `mxe`, where Fixpt's calls a primitive of its
    # runtime's, whose effect is the runtime's own.
    "parser.fx": [
        ("(define parse-tag (prompt-tag presult presult (maxeff tree-builds (write rs) spin) rp)",
         "(define parse-tag (prompt-tag presult presult (maxeff tree-builds (write rs) spin mxe) rp)"),
    ],
    "macros-shapes.fx": [
        # Syntax compared by identity, a shortcut that keeps a form as it
        # was where nothing in it changed: KFX has no `eq?` (C75), and the
        # form made again of the same parts is the same syntax.
        ("(if (and (same-items? xs items) (eq? t2 t)) s",
         "(if (and (same-items? xs items) #f) s"),
        ("(else (and (eq? (car xs) (car ys)) (same-items? (cdr xs) (cdr ys))))",
         "(else (and #f (same-items? (cdr xs) (cdr ys))))"),
    ],
    # Procedures that hold syntax, which holds `datum`s, `Value`s in Rust,
    # which no other thread may share: of an effect a process may have
    # (`pure`), kfx writes them `Send + Sync` (C102) and so refuses them.
    # Given an effect no process has, `(read rs)`, which they may well
    # have, they are not (C429; TODO §51: a datum shared, in Rust).
    "macros-nodes.fx": [
        ("(subr pure (symbol) int)", "(subr (read rs) (symbol) int)", 3),
    ],
    # `s-typed`'s closure holds its syntax (`h`), so `one-var`'s procedure
    # is given the same effect (C441).
    "macros-types.fx": [
        ("(define one-var (->* (stys (-> (sty) sty)) stys)", "(define one-var (->* (stys (subr (read rs) (sty) sty)) stys)"),
    ],
    "macros-align.fx": [
        ('(shown (lambda () (string-append "`" (show-syn s) "`"))))',
         '(shown (the (subr (read rs) () string) (lambda () (string-append "`" (show-syn s) "`")))))'),
    ],
}
# Names a local may have that shadow what KFX takes only as itself (the
# primitive `list`): renamed where a `lambda` or `let` binds them, there.
SCOPED = {"list": "mx-list"}
# FX-26's operations of any number of operands that KFX's take two of:
# `(op a b c)` written `(op a (op b c))`, as they associate.
NESTED = {"string-append", "append", "+", "*"}
# Definitions named as a form's word of kayak's checker (`region`), which
# it reads as the form: prefixed, as a name another file defines is.
WORDS = {"region",
         # And as the names kfx's restriction pass refuses (kfx/restrict.kfx's
         # lists: continuation marks, keywords, evidence, sessions, what
         # KFX has not yet, what it would copy).
         "current-marks", "marks-of", "keyword?", "symbol->keyword", "keyword->symbol",
         "effect-bound", "type-bound", "make-session", "session-redefines",
         "call-with-oneshot-continuation", "rmake-array", "datum-int?", "oneshot", "assume",
         "assert", "char-in?", "nat?", "certify-nat", "rmake-icell", "pair-identity",
         "make-flatarray", "flatarray-ref", "flatarray-set!", "flatarray-length", "flatlayout",
         "no-pair", "eq?", "set-car!", "set-cdr!", "stay-cellular", "list"}
CTORS = {}
LOCALS = {"fail": "mx-fail", "once": "mx-once", "knot": "mx-knot", "unwind": "mx-unwind"}


# ------------------------------------------------------------ reading

TOKEN = re.compile(r"""
    (?P<ws>\s+)
  | (?P<comment>;[^\n]*)
  | (?P<block>\#\|.*?\|\#)
  | (?P<string>"(?:[^"\\]|\\.)*")
  | (?P<char>\#\\(?:[a-zA-Z][a-zA-Z0-9]+|x[0-9a-fA-F]+|.))
  | (?P<open>\#\(|\()
  | (?P<close>\))
  | (?P<quote>'|`|,@|,)
  | (?P<pipe>\|[^|]*\|)
  | (?P<atom>[^\s()";']+)
""", re.X | re.S)


def tokens(text):
    out, i = [], 0
    while i < len(text):
        m = TOKEN.match(text, i)
        if not m:
            raise SystemExit("macros-port: cannot read at %d: %r" % (i, text[i:i + 40]))
        out.append([m.lastgroup, m.group()])
        i = m.end()
    return out


class Node:
    def __init__(self, start):
        self.start, self.end, self.items = start, None, []


def tree(toks):
    stack = [Node(-1)]
    for i, (k, t) in enumerate(toks):
        if k == "open":
            n = Node(i)
            stack[-1].items.append(n)
            stack.append(n)
        elif k == "close":
            stack.pop().end = i
        elif k in ("atom", "string", "char", "pipe", "quote"):
            stack[-1].items.append(i)
    return stack[0].items


def head(toks, n):
    if isinstance(n, Node) and n.items and not isinstance(n.items[0], Node):
        return toks[n.items[0]][1]
    return None


def atom_of(toks, x):
    return None if isinstance(x, Node) else toks[x][1]


# ------------------------------------------------------------ the files

class File:
    def __init__(self, name):
        self.name = name
        self.text = open(os.path.join(CLONE, SRC, name), encoding="utf-8").read()
        for patch in PATCHES.get(name, []):
            old, new, times = patch if len(patch) == 3 else patch + (1,)
            if self.text.count(old) != times:
                raise SystemExit("macros-port: %s: a patch's text is not there %d time(s): %s" % (name, times, old))
            self.text = self.text.replace(old, new)
        self.toks = tokens(self.text)
        self.forms = []     # (node, wrapped-in-hide)
        self.modules = {}   # alias -> ("load", file) | ("with", alias, member)
        # Each by name and namespace: "v" a value's, "t" a description's (a
        # type's or an effect's), which FX-26 names apart.
        self.aliases = {}   # (name, ns) -> ("with"/"select", module alias, member)
        self.imports = {}   # (name, ns) -> module alias
        self.defs = {}      # (name, ns) -> the form defining it (its start)
        self.params = []    # region parameters, in order
        self.loads = []
        for f in tree(self.toks):
            if isinstance(f, Node):
                if head(self.toks, f) == "hide":
                    self.forms.extend((x, True) for x in f.items[1:] if isinstance(x, Node))
                else:
                    self.forms.append((f, False))
        self.read()

    def read(self):
        t = self.toks
        for f, hid in self.forms:
            h = head(t, f)
            if h == "module-parameters":
                for p in f.items[1].items:
                    self.params.append(atom_of(t, p.items[0]))
            elif h in ("define", "define-type", "define-effect") and len(f.items) >= 3 and not isinstance(f.items[1], Node):
                name, body = atom_of(t, f.items[1]), f.items[-1]
                load = self.load_of(body) if isinstance(body, Node) else None
                if load:
                    self.modules[name] = ("load", load)
                    self.loads.append(load)
                elif (isinstance(body, Node) and head(t, body) in ("with", "select") and len(body.items) == 3
                      and len(f.items) == 3
                      and (atom_of(t, body.items[1]) in self.modules or (atom_of(t, body.items[1]), "v") in self.aliases)
                      and not isinstance(body.items[2], Node)):
                    self.aliases[(name, "v" if h == "define" else "t")] = (head(t, body), atom_of(t, body.items[1]), atom_of(t, body.items[2]))
                else:
                    self.defs[(name, "v" if h == "define" else "t")] = f.start
            elif h == "import":
                alias = atom_of(t, f.items[1])
                for x in f.items[2:]:
                    for y in (x.items[1:] if isinstance(x, Node) else [x]):
                        self.imports[(atom_of(t, y), "t" if isinstance(x, Node) else "v")] = alias
            elif h in ("define", "define-type", "define-effect", "define-datatype", "define-generative"):
                n = f.items[1]
                self.defs[(atom_of(t, n) if not isinstance(n, Node) else atom_of(t, n.items[0]),
                           "v" if h == "define" else "t")] = f.start
                if h == "define-datatype":
                    for c in f.items[2:]:
                        if isinstance(c, Node):
                            self.defs[(atom_of(t, c.items[0]), "v")] = f.start
            elif h == "define-rec":
                for b in f.items[1:]:
                    if isinstance(b, Node):
                        self.defs[(atom_of(t, b.items[0]), "v")] = f.start

    def load_of(self, body):
        t = self.toks
        if head(t, body) == "load-module":
            return t[body.items[1]][1].strip('"')[len("fx26:"):]
        if body.items and isinstance(body.items[0], Node) and head(t, body.items[0]) == "proj":
            inner = body.items[0].items[1]
            if isinstance(inner, Node) and head(t, inner) == "load-module":
                return t[inner.items[1]][1].strip('"')[len("fx26:"):]
        return None


class Program:
    def __init__(self):
        self.files, todo = {}, list(ROOTS)
        while todo:
            n = todo.pop()
            if n in self.files or not os.path.exists(os.path.join(CLONE, SRC, n)):
                continue
            self.files[n] = File(n)
            todo.extend(self.files[n].loads)
        self.order, seen = [], set()

        def visit(n):
            if n in seen or n not in self.files:
                return
            seen.add(n)
            for d in self.files[n].loads:
                visit(d)
            self.order.append(n)
        for n in ROOTS:
            visit(n)

    def module_file(self, fname, alias, seen=()):
        """The file module alias `alias` of file `fname` names."""
        f = self.files.get(fname)
        if f is None:
            return None
        m = f.modules.get(alias)
        if m is None:
            # An alias of a module, `(define m (with n m2))`.
            a = f.aliases.get((alias, "v"))
            if a and a[0] == "with" and (fname, alias) not in seen:
                g = self.module_file(fname, a[1], seen + ((fname, alias),))
                return self.module_file(g, a[2], seen + ((fname, alias),)) if g else None
            # A module imported: `(import m n)` where `n` is a module.
            if (alias, "v") in f.imports and (fname, alias) not in seen:
                g = self.module_file(fname, f.imports[(alias, "v")], seen + ((fname, alias),))
                return self.module_file(g, alias, seen + ((fname, alias),)) if g else None
            return None
        return m[1]

    def resolve(self, fname, name, ns="v", seen=()):
        """The (file, name, ns) definition `name` in file `fname` is, in
        namespace `ns` if it is one there, else in the other; or None."""
        r = self.resolve_in(fname, name, ns, seen)
        return r if r else self.resolve_in(fname, name, "t" if ns == "v" else "v", seen)

    def resolve_in(self, fname, name, ns, seen=()):
        f = self.files.get(fname)
        if f is None or (fname, name, ns) in seen:
            return None
        seen = seen + ((fname, name, ns),)
        if (name, ns) in f.defs:
            return (fname, name, ns)
        if (name, ns) in f.aliases:
            how, alias, member = f.aliases[(name, ns)]
            g = self.module_file(fname, alias)
            return self.resolve_in(g, member, ns, seen) if g else None
        if (name, ns) in f.imports:
            g = self.module_file(fname, f.imports[(name, ns)])
            return self.resolve_in(g, name, ns, seen) if g else None
        return None


def unit_names(units=((os.path.join("fx26", "fx26.unit"), False),)):
    """Every name a unit that loads the expander defines, but the
    expander's: the engine's; with kc's (C445), its types' names alone,
    where one of theirs and one of Fixpt's the same would be one type in
    Rust."""
    names = set()
    for unit, types_only in units:
        here = os.path.dirname(os.path.join(ROOT, unit))
        for line in open(os.path.join(ROOT, unit)):
            line = line.strip()
            if not line or line.startswith((";", "--")) or line.startswith("macros/") or os.path.basename(line) in ("macros-shim.kfx", "macros-bridge.kfx"):
                continue
            s = open(os.path.join(here, line), encoding="utf-8").read()
            names |= defined_in(s, types_only)
    return names


def defined_in(s, types_only=False):
    """The names a KFX file's text defines at its top level, its
    `define-rec`s' and datatypes' constructors too; or, `types_only`, its
    types' names alone (kc's, whose values the expander's do not meet)."""
    names = set()
    t = tokens(s)
    for f in tree(t):
        if not isinstance(f, Node) or len(f.items) < 2:
            continue
        h = head(t, f)
        n = f.items[1]
        if types_only:
            if h in ("define-type", "define-datatype", "define-generative"):
                names.add(atom_of(t, n) if not isinstance(n, Node) else atom_of(t, n.items[0]))
            continue
        if h in ("define", "define-type", "define-effect", "define-datatype", "define-generative"):
            names.add(atom_of(t, n) if not isinstance(n, Node) else atom_of(t, n.items[0]))
            if h == "define-datatype":
                for c in f.items[2:]:
                    if isinstance(c, Node) and c.items:
                        names.add(atom_of(t, c.items[0]))
        elif h == "define-rec":
            for b in f.items[1:]:
                if isinstance(b, Node) and b.items:
                    names.add(atom_of(t, b.items[0]))
    return names - {None}


def short(fname):
    return fname[:-3].replace("macros-", "").replace("macros", "top")


# ------------------------------------------------------------ what is kept

def quoted_spans(t, form):
    """The token indices of `form` that are data: quoted, a `case`'s."""
    skip = set()

    def mark(n):
        if isinstance(n, Node):
            skip.update(range(n.start, n.end + 1))
        else:
            skip.add(n)

    def walk(n):
        for k, x in enumerate(n.items):
            if not isinstance(x, Node) and t[x][0] == "quote" and k + 1 < len(n.items):
                mark(n.items[k + 1])
        h = head(t, n)
        if h == "quote":
            mark(n)
            return
        if h == "case":
            for c in n.items[2:]:
                if isinstance(c, Node) and c.items:
                    mark(c.items[0])
        for x in n.items:
            if isinstance(x, Node):
                walk(x)
    walk(form)
    return skip


TYPE_HEADS = {"->*", "->", "subr", "poly", "ref", "listof", "pairof", "arrayof", "flatarrayof",
              "narrayof", "tupleof", "productof", "sumof", "oneof", "union", "prompt-tag",
              "maxeff", "read", "write", "alloc", "goto", "comefrom", "moduleof", "mark-key",
              "icell", "bloblet", "blobref", "bytevectorof", "composable", "oneshot", "select"}


def type_positions(t, form):
    """The token indices of `form` that name descriptions, types and
    effects, which FX-26 names apart from values: a definition's type, a
    `the`'s, a parameter's, a binding's in a `letrec` or `define-rec`; a
    type's or effect's definition; a datatype's fields; and what the type
    forms are made of."""
    out = set()

    def all_of(n):
        if isinstance(n, Node):
            out.update(range(n.start, n.end + 1))
        else:
            out.add(n)

    def walk(n):
        h = head(t, n)
        items = n.items
        if h in TYPE_HEADS:
            all_of(n)
            return
        if h in ("define", "define*") and len(items) == 4:
            all_of(items[2])
        elif h in ("define-type", "define-effect"):
            for x in items[1:]:
                all_of(x)
        elif h == "define-datatype":
            all_of(items[1])
            for c in items[2:]:
                if isinstance(c, Node):
                    for x in c.items[1:]:
                        all_of(x)
        elif h == "the" and len(items) == 3:
            all_of(items[1])
        elif h in ("lambda", "rlambda") and len(items) >= 2 and isinstance(items[1], Node):
            for p in items[1].items:
                if isinstance(p, Node) and len(p.items) == 2:
                    all_of(p.items[1])
        elif h in ("letrec", "define-rec"):
            bs = items[1].items if h == "letrec" and isinstance(items[1], Node) else items[1:]
            for b in bs:
                if isinstance(b, Node) and len(b.items) == 3:
                    all_of(b.items[1])
        for x in items:
            if isinstance(x, Node):
                walk(x)
    walk(form)
    return out


def form_refs(prog, fname, form):
    """The definitions form `form` of file `fname` names."""
    t = prog.files[fname].toks
    skip = quoted_spans(t, form)
    types = type_positions(t, form)
    out = set()

    def walk(n):
        h = head(t, n)
        if h in ("with", "select") and len(n.items) == 3 and not isinstance(n.items[1], Node) and not isinstance(n.items[2], Node):
            g = prog.module_file(fname, t[n.items[1]][1])
            if g is not None:
                r = prog.resolve(g, t[n.items[2]][1], "v" if h == "with" else "t")
                if r:
                    out.add(r)
                return
        for x in n.items:
            if isinstance(x, Node):
                walk(x)
            elif t[x][0] == "atom" and x not in skip:
                r = prog.resolve(fname, t[x][1], "t" if x in types else "v")
                if r:
                    out.add(r)
    walk(form)
    return out


def kept(prog):
    """The forms kept, by file: every definition of the expander's files,
    and what they reach."""
    keep = {n: set() for n in prog.order}
    todo = []
    for n in prog.order:
        if WHOLE.match(n):
            f = prog.files[n]
            for form, hid in f.forms:
                if form.start in f.defs.values():
                    todo.append((n, form))
    by_start = {n: {form.start: form for form, h in prog.files[n].forms} for n in prog.order}
    while todo:
        n, form = todo.pop()
        if form.start in keep[n]:
            continue
        keep[n].add(form.start)
        for (g, name, ns) in form_refs(prog, n, form):
            s = prog.files[g].defs[(name, ns)]
            if s not in keep[g]:
                todo.append((g, by_start[g][s]))
    return keep


def constructors_of(prog, keep, names):
    """Each kept datatype's variants, by name, where no two datatypes have
    one of the name: a `tagcase`'s tag is its type's variant, which a file
    may name without importing it (FX-26 finds it through the type)."""
    seen, out = {}, {}
    for n in prog.order:
        f = prog.files[n]
        for form, h in f.forms:
            if form.start in keep[n] and head(f.toks, form) == "define-datatype":
                for c in form.items[2:]:
                    if isinstance(c, Node) and c.items:
                        cn = atom_of(f.toks, c.items[0])
                        seen[cn] = seen.get(cn, 0) + 1
                        out[cn] = names[(n, cn, "v")]
    return {c: g for c, g in out.items() if seen[c] == 1}


def globals_of(prog, keep, taken, taken_types=frozenset()):
    count = {}
    for n in prog.order:
        f = prog.files[n]
        for (d, ns), s in f.defs.items():
            if s in keep[n]:
                count[d] = count.get(d, 0) + 1
    out = {}
    for n in prog.order:
        f = prog.files[n]
        for (d, ns), s in f.defs.items():
            if s in keep[n]:
                out[(n, d, ns)] = "mx-%s-%s" % (short(n), d) if (count[d] > 1 or d in taken or (ns == "t" and d in taken_types) or d in PRIMS or d in WORDS) else d
    return out


# ------------------------------------------------------------ writing

def translate(prog, fname, keep, names):
    f = prog.files[fname]
    t = f.toks
    regions = dict(zip(f.params, REGIONS))
    pieces = []
    for group in ordered(prog, fname, keep):
        texts = [form_text(prog, fname, form, names, regions) for form in group]
        kind = "type" if all(head(t, form) in TYPE_HEADS for form in group) else "item"
        if len(group) == 1 and not recursive_group(prog, fname, group):
            pieces.append((texts[0][0] + texts[0][1], kind))
        elif kind == "type":
            pieces.append((type_group(prog, fname, group, texts, names), kind))
        else:
            pieces.append(("(define-rec\n" + "\n".join(rec_member(prog, fname, form, x) for form, x in zip(group, texts)) + ")", kind))
    whole = "whole" if WHOLE.match(fname) else "the definitions the expander reaches"
    return (";;; ============================================================ %s (%s)\n\n"
            % (fname, whole)), pieces


# The forms that define types: a maker's module may define only those
# that name its place (C436); the rest are defined before it.
TYPE_HEADS = ("define-type", "define-datatype", "define-generative")
REGION_NAMED = re.compile(r"@m[sempc]\b")
TYPE_NAMED = re.compile(r"\(define-(?:type|datatype|generative)\s+\(?([^\s()]+)")


def hoisted(files):
    """The files' pieces parted (C437): the types defined before the maker,
    each that names no region of the expander's, nor a type or effect that
    does; and,
    per file, what its module keeps."""
    # The effects that name a region, which the maker's module defines.
    inside = {m.group(1) for header, pieces in files for text, kind in pieces
              for m in [re.match(r"\s*(?:;[^\n]*\n\s*)*\(define-effect\s+([^\s()]+)", text)]
              if m and REGION_NAMED.search(text)}
    for header, pieces in files:
        for text, kind in pieces:
            if kind == "type":
                mentioned = {tok for k, tok in tokens(text) if k == "atom"}
                if REGION_NAMED.search(text) or mentioned & inside:
                    inside.update(TYPE_NAMED.findall(text))
    outside, kept_in = [], []
    for header, pieces in files:
        out = [x for x, k in pieces if k == "type" and not (set(TYPE_NAMED.findall(x)) & inside)]
        keep = [x for x, k in pieces if not (k == "type" and not (set(TYPE_NAMED.findall(x)) & inside))]
        if out:
            outside.append(header + "\n".join(out) + "\n")
        kept_in.append(header + "\n".join(keep) + "\n")
    return outside, kept_in


def placed(text):
    """The expander's regions, Fixpt's module parameters, the maker's one
    place (C437): what Fixpt shares among its modules, made one instance's."""
    return REGION_NAMED.sub(PLACE, text)


def ordered(prog, fname, keep):
    """File `fname`'s kept forms, each after those it names (KFX sees only
    what is before), in Fixpt's order where that is so; forms that name
    each other, together (Tarjan's strongly connected components)."""
    f = prog.files[fname]
    forms = [form for form, h in f.forms if form.start in keep[fname]]
    index = {form.start: k for k, form in enumerate(forms)}
    deps = {}
    for form in forms:
        deps[form.start] = sorted({index[f.defs[(n, ns)]] for (g, n, ns) in form_refs(prog, fname, form)
                                   if g == fname and f.defs[(n, ns)] in index and f.defs[(n, ns)] != form.start})
    out, stack, on, low, num = [], [], set(), {}, {}
    counter = [0]

    def strong(v):
        num[v] = low[v] = counter[0]
        counter[0] += 1
        stack.append(v)
        on.add(v)
        for w in deps[forms[v].start]:
            if w not in num:
                strong(w)
                low[v] = min(low[v], low[w])
            elif w in on:
                low[v] = min(low[v], num[w])
        if low[v] == num[v]:
            group = []
            while True:
                w = stack.pop()
                on.discard(w)
                group.append(w)
                if w == v:
                    break
            out.append([forms[k] for k in sorted(group)])
    sys.setrecursionlimit(100000)
    for v in range(len(forms)):
        if v not in num:
            strong(v)
    return out


def type_group(prog, fname, group, texts, names):
    """Types that name each other, a datatype and abbreviations of it
    (`(define-datatype shape (sh-fun (listof shapes acyclic) shapes))`,
    `(define-type shapes (listof shape acyclic))`): the datatype first, each
    abbreviation in it written out, then the abbreviations (KFX reads a
    type's name only after its definition)."""
    t = prog.files[fname].toks
    abbrevs = {}
    for form, (lead, body) in zip(group, texts):
        if head(t, form) == "define-type":
            name = names[(fname, atom_of(t, form.items[1]), "t")]
            inner = body[1:-1].strip()[len("define-type"):].strip()
            abbrevs[name] = inner[len(name):].strip()
    def expand(text, depth=0):
        if depth > 8:
            raise SystemExit("macros-port: %s: abbreviations that name each other" % fname)
        out = []
        for k, tok in tokens(text):
            out.append("(" + expand(abbrevs[tok], depth + 1) + ")" if False else (expand(abbrevs[tok], depth + 1) if k == "atom" and tok in abbrevs else tok))
        return "".join(out)
    first, then = [], []
    for form, (lead, body) in zip(group, texts):
        if head(t, form) == "define-datatype":
            first.append(lead + expand(body))
        else:
            then.append(lead + body)
    return "\n".join(first + then)


def recursive_group(prog, fname, group):
    """Whether a lone form is a `define-rec` already (kept as one)."""
    return False


def rec_member(prog, fname, form, text):
    """A form, `(define name type lambda)`, as a `define-rec`'s member,
    `(name type lambda)`; a `define-rec`'s own members as they are."""
    t = prog.files[fname].toks
    h = head(t, form)
    lead, body = text
    assert body.startswith("(") and body.endswith(")")
    body = body[1:-1]
    if h == "define-rec":
        return lead + body[len("define-rec"):].strip("\n")
    if h == "define":
        return lead + "(" + body[len("define"):].lstrip() + ")"
    raise SystemExit("macros-port: %s: a %s among forms that name each other" % (fname, h))


def form_text(prog, fname, form, names, regions):
    """Form `form` of file `fname`, its names made global ones."""
    t = prog.files[fname].toks
    if True:
        skip = quoted_spans(t, form)
        types = type_positions(t, form)
        out = {}
        in_region = set()

        def mark_regions(n):
            if head(t, n) in REGION_HEADS:
                for x in n.items[1:]:
                    if not isinstance(x, Node):
                        in_region.add(x)
            for x in n.items:
                if isinstance(x, Node):
                    mark_regions(x)
        mark_regions(form)

        def walk(n):
            h = head(t, n)
            if h in ("with", "select") and len(n.items) == 3 and not isinstance(n.items[1], Node) and not isinstance(n.items[2], Node):
                g = prog.module_file(fname, t[n.items[1]][1])
                if g is not None:
                    r = prog.resolve(g, t[n.items[2]][1], "v" if h == "with" else "t")
                    out[n.start] = names[r] if r else PRIMS.get(t[n.items[2]][1], t[n.items[2]][1])
                    for i in range(n.start + 1, n.end + 1):
                        out[i] = ""
                    return
            for x in n.items:
                if isinstance(x, Node):
                    walk(x)
        walk(form)
        # The operations of more than two operands, nested.
        before, closes = {}, {}

        def nest(n):
            h = head(t, n)
            if h in ("string-append", "append") and len(n.items) == 2 and n.items[0] not in skip:
                # Of one operand, the operand.
                out[n.start] = out[n.items[0]] = out[n.end] = ""
            if h in NESTED and len(n.items) > 3 and n.items[0] not in skip:
                for x in n.items[2:-1]:
                    k = x.start if isinstance(x, Node) else x
                    before[k] = before.get(k, "") + "(" + h + " "
                closes[n.end] = closes.get(n.end, "") + ")" * (len(n.items) - 3)
            for x in n.items:
                if isinstance(x, Node):
                    nest(x)
        nest(form)
        # Locals that shadow a name KFX takes only as itself, renamed in
        # their scope.
        def bound_by(n):
            h = head(t, n)
            names = []
            if h in ("lambda", "rlambda") and len(n.items) > 2 and isinstance(n.items[1], Node):
                for p in n.items[1].items:
                    names.append(atom_of(t, p) if not isinstance(p, Node) else atom_of(t, p.items[0]))
            elif h in ("let", "let*") and len(n.items) > 2 and isinstance(n.items[1], Node):
                for b in n.items[1].items:
                    if isinstance(b, Node) and b.items:
                        names.append(atom_of(t, b.items[0]))
            return [x for x in names if x in SCOPED]

        def scoped(n, live):
            live = live | set(bound_by(n))
            for x in n.items:
                if isinstance(x, Node):
                    scoped(x, live)
                elif t[x][0] == "atom" and t[x][1] in live and x not in skip:
                    out[x] = SCOPED[t[x][1]]
        scoped(form, frozenset())
        text = []
        defname = atom_of(t, form.items[1]) if len(form.items) > 1 else None
        written = WRITTEN.get((fname, defname))
        for i in range(form.start, form.end + 1):
            k, s = t[i]
            if i in before:
                text.append(before[i])
            if i in closes:
                text.append(closes[i])
            if written and k == "atom" and s == "->*":
                text.append("subr " + written)
                written = None
                continue
            if i in out:
                text.append(out[i])
                continue
            # `'|x y|`, a symbol named as no name reads: made of its name.
            if k == "quote" and s == "'" and i + 1 <= form.end and t[i + 1][0] == "pipe":
                out[i + 1] = '(string->symbol "%s")' % t[i + 1][1].strip("|")
                continue
            if k == "atom" and i not in skip:
                if s in regions and i in in_region:
                    s = regions[s]
                else:
                    r = prog.resolve(fname, s, "t" if i in types else "v")
                    if r is not None:
                        s = names[r]
                    elif s in PRIMS:
                        s = PRIMS[s]
                    elif s in CTORS:
                        s = CTORS[s]
                    elif s in LOCALS:
                        s = LOCALS[s]
            elif k == "pipe" and i not in skip:
                s = s.strip("|")
            text.append(s)
        # The comments just before the form, kept with it.
        lead = []
        j = form.start - 1
        while j >= 0 and t[j][0] in ("ws", "comment"):
            lead.append(t[j][1])
            j -= 1
        lead = "".join(reversed(lead)).lstrip("\n")
        return (lead, "".join(text))


def made():
    prog = Program()
    keep = kept(prog)
    taken = unit_names() | (defined_in(open(SHIM, encoding="utf-8").read()) if os.path.exists(SHIM) else set())
    names = globals_of(prog, keep, taken, unit_names(((os.path.join("kc", "kc.unit"), True),)))
    CTORS.clear()
    CTORS.update(constructors_of(prog, keep, names))
    files = [n for n in prog.order if keep[n]]
    rev = open(os.path.join(ROOT, "FIXPT_REV")).read().strip()[:7]
    marks = "\n".join(" fixpt-source: %s/%s@%s" % (SRC, n, rev) for n in files)
    shim = open(SHIM, encoding="utf-8").read()
    outside, inside = hoisted([translate(prog, n, keep, names) for n in files])
    body = "\n".join(inside)
    header = ('"expander.fx: Fixpt\'s macro expander written in FX-26, its files made one\n'
              ' KFX module file by scripts/macros-port.py (C429): edit that, not this.\n'
              ' Its files are modules in Fixpt, their items no globals; here they are\n'
              ' one file\'s items, so that naming one reads no global either, as there.\n'
              ' The shim (fx26/macros-shim.kfx) first; then each file, in the order\n'
              ' its loads need, the expander\'s whole, the parser\'s files\' what it\n'
              ' reaches; then the driver (fx26/macros-driver.kfx). A file of a place\n'
              ' (C442, C443), Fixpt\'s five regions its one, each load an instance in\n'
              ' the place given, its state its own: a process may load one, as a\n'
              ' load sees the standard environment alone. Its types, but those that\n'
              ' name the place, are expander-types.fx\'s, loaded once for all. Of\n'
              ' one host\'s asker\'s effect (C449, `ASKS`), which answers a\n'
              ' procedural macro\'s transformer\'s request: expander.fx the\n'
              ' engine\'s, expander-kc.fx kc\'s.\n' + marks + '"\n')
    text = ("(module-parameters ((mxp place)))\n" + header +
            "(include (load-module \"expander-types.fx\"))\n\n" +
            placed(shim + "\n" + body + "\n" + open(DRIVER, encoding="utf-8").read()))
    types = (';; expander-types.fx: the types of Fixpt\'s macro expander that name no\n'
             ';; region of its, written by scripts/macros-port.py (C429, C443): edit\n'
             ';; that, not this. A file of no parameters, loaded once for every\n'
             ';; instance of expander.fx: its constructors are no globals, as in Fixpt,\n'
             ';; where each is another file\'s item.\n' + "\n".join(outside))
    prelude = open(os.path.join(CLONE, SRC, "macros", "procedural-prelude.fx"), encoding="utf-8").read()
    escaped = prelude.replace("\\", "\\\\").replace('"', '\\"')
    phase1 = ('"phase1-prelude.kfx: what a procedural macro\'s transformer sees besides\n'
              ' the standard environment, Fixpt\'s prelude as its text, written by\n'
              ' scripts/macros-port.py (C446): edit that, not this. The engine\'s\n'
              ' `--phase1` defines it before a transformer (main.kfx).\n'
              ' fixpt-source: %s/macros/procedural-prelude.fx@%s"\n'
              '(define phase1-prelude string "%s")\n' % (SRC, rev, escaped))
    made_texts = {name: re.sub(r"(?<![A-Za-z0-9!?*<>=/+-])mxe(?![A-Za-z0-9!?*<>=/+-])", atoms, text) for name, atoms in ASKS.items()}
    return files, dict(made_texts, **{"expander-types.fx": types, "phase1-prelude.kfx": phase1})


def main():
    check = sys.argv[1:] == ["--check"]
    if sys.argv[1:] not in ([], ["--check"]):
        sys.exit(__doc__)
    order, texts = made()
    if check:
        wrong = [n for n, s in texts.items()
                 if not os.path.exists(os.path.join(OUT, n)) or open(os.path.join(OUT, n), encoding="utf-8").read() != s]
        if wrong:
            sys.exit("macros-port: not the pinned Fixpt's: " + " ".join(sorted(wrong)))
        print("macros-port: %d file(s), the pinned Fixpt's" % len(texts))
        return
    os.makedirs(OUT, exist_ok=True)
    for old in os.listdir(OUT):
        if old.endswith((".kfx", ".fx")) and old not in texts:
            os.remove(os.path.join(OUT, old))
    for n, s in texts.items():
        with open(os.path.join(OUT, n), "w", encoding="utf-8") as f:
            f.write(s)
    print("macros-port: %d file(s) written" % len(texts))


UNIT = os.path.join(ROOT, "fx26", "fx26.unit")


def unit_listed(order):
    """fx26/fx26.unit with the files, in order, after the shim."""
    lines = open(UNIT).read().split("\n")
    i = lines.index("; The macro expander (C429; docs/research/fx26-macros.md), between the reader and the checker.")
    j = i + 1
    while j < len(lines) and (lines[j].startswith("macros/") or lines[j] == "macros-shim.kfx"):
        j += 1
    # The expander is loaded where it is used (C443); the unit has the
    # bridge to it.
    lines[i + 1:j] = ["macros-bridge.kfx"]
    with open(UNIT, "w") as f:
        f.write("\n".join(lines))


if __name__ == "__main__":
    main()
