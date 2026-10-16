"""The modules Fixpt builds in, which a program loads by name, as
`(load-module "fx26:conditions.fx")` (C107): kfx/built-in.kfx, written from
Fixpt's own files at the pinned commit (FIXPT_REV), each a KFX string. And
Fixpt's native convention's declaration (C116), the one a program's
`define-convention` is compared with: in kfx/built-in.kfx too, and as
crates/kayak-jit/src/native-arm64.conv, which kayak-jit's constants are
tested against.

usage: python3 -I scripts/built-in.py [--check]

--check: exits 1 if kfx/built-in.kfx is not what the pinned Fixpt's files
make (check.sh runs it, when there is a clone).
"""

import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, "kfx", "built-in.kfx")
# Each built-in module KFX has: its name, and its file in Fixpt.
MODULES = [
    ("fx26:conditions.fx", "crates/fixpt-fx26/src/conditions.fx"),
    # The code generators' (C116): the arm64 encoder, and a native
    # convention's declaration read as data.
    ("fx26:arm64-types.fx", "crates/fixpt-fx26/src/arm64-types.fx"),
    ("fx26:arm64.fx", "crates/fixpt-fx26/src/arm64.fx"),
    ("fx26:convention-types.fx", "crates/fixpt-fx26/src/convention-types.fx"),
    ("fx26:convention.fx", "crates/fixpt-fx26/src/convention.fx"),
]
CONVENTION = "crates/fixpt-runtime/src/native-arm64.conv"
CONVENTION_OUT = os.path.join(ROOT, "crates", "kayak-jit", "src", "native-arm64.conv")


def kfx_string(text):
    return '"' + text.replace("\\", "\\\\").replace('"', '\\"') + '"'


def made(clone):
    rev = open(os.path.join(ROOT, "FIXPT_REV")).read().strip()
    out = [
        '"built-in.kfx: the modules Fixpt builds in that KFX has (C107), each\n'
        ' loaded by its name, `(load-module \\"fx26:NAME\\")`, its text Fixpt\'s at\n'
        " " + rev[:7] + ". Written by scripts/built-in.py: edit that, not this.\"\n",
    ]
    names = []
    for i, (name, path) in enumerate(MODULES):
        with open(os.path.join(clone, path), encoding="utf-8") as f:
            text = f.read()
        names.append((name, "built-in-" + str(i)))
        out.append("(define built-in-%d string\n  %s)\n" % (i, kfx_string(text)))
    with open(os.path.join(clone, CONVENTION), encoding="utf-8") as f:
        out.append('"Fixpt\'s native convention, declared (C116): one clause a line."\n'
                   "(define native-convention-text string\n  %s)\n" % kfx_string(f.read()))
    out.append('(define built-in-text (subr kfx (string) string)\n'
               '  (lambda (path)\n'
               '    "A built-in module\'s text, by its name; `` for none."\n'
               '    (cond ' + "\n          ".join('((string=? path "%s") %s)' % (n, v) for n, v in names)
               + '\n          (else ""))))\n')
    return "\n".join(out)


def main():
    check = sys.argv[1:] == ["--check"]
    if sys.argv[1:] not in ([], ["--check"]):
        sys.exit("usage: built-in.py [--check]")
    clone = os.environ.get("KAYAK_FIXPT_CLONE", os.path.join(ROOT, "..", "kayak-refs", "fixpt"))
    if subprocess.run([os.path.join(ROOT, "scripts", "fixpt.sh")], capture_output=True).returncode != 0:
        sys.exit("built-in: the Fixpt clone is not at FIXPT_REV (scripts/fixpt.sh)")
    text = made(clone)
    with open(os.path.join(clone, CONVENTION), encoding="utf-8") as f:
        convention = f.read()
    if check:
        have = open(OUT, encoding="utf-8").read() if os.path.exists(OUT) else ""
        if have != text:
            sys.exit("built-in: kfx/built-in.kfx is not the pinned Fixpt's modules (python3 -I scripts/built-in.py)")
        have = open(CONVENTION_OUT, encoding="utf-8").read() if os.path.exists(CONVENTION_OUT) else ""
        if have != convention:
            sys.exit("built-in: kayak-jit's native-arm64.conv is not the pinned Fixpt's (python3 -I scripts/built-in.py)")
        print("built-in: %d module(s) and the native convention, the pinned Fixpt's" % len(MODULES))
    else:
        with open(OUT, "w", encoding="utf-8") as f:
            f.write(text)
        with open(CONVENTION_OUT, "w", encoding="utf-8") as f:
            f.write(convention)


if __name__ == "__main__":
    main()
