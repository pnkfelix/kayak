"""Fixpt's front end's FX-26 files, each loaded by a wrapper program through
kfx (TODO §31 step 1: what strategy A, Fixpt's front end through KFX,
would cost): which kfx accepts, and the first refusal of the rest, by kind.

usage: python3 -I scripts/survey-front-end.py FIXPT-SRC OUT-DIR KFX
  FIXPT-SRC  the clone's crates/fixpt-fx26/src
  OUT-DIR    an empty directory: the files are copied there, `fx26:` paths
             made relative, each with its wrapper, and RESULTS written
  KFX        target/release/kfx"""
import collections, os, re, subprocess, sys
src, out, kfx = sys.argv[1], sys.argv[2], sys.argv[3]
files = sorted(f for f in os.listdir(src) if f.endswith('.fx'))
for f in files:
    t = open(os.path.join(src, f)).read()
    t = re.sub(r'"fx26:([\w.-]+)"', r'"\1"', t)
    open(os.path.join(out, f), 'w').write(t)
def one_expression(t):
    # A file whose first form, past comments, is a let or a module or a lambda.
    s = re.sub(r';[^\n]*', '', t).strip()
    return s.startswith('(let') or s.startswith('(module ') or s.startswith('(module\n') or s.startswith('(lambda') or s.startswith('(plambda')
def params(t):
    import re as r
    s = r.sub(r';[^\n]*', '', t).strip()
    m = r.match(r'\(module-parameters \(((?:\(\w+ region\)\s*)+)\)\)', s)
    return len(r.findall(r'\(\w+ region\)', m.group(1))) if m else None
results = {}
def run(f):
    t = open(os.path.join(out, f)).read()
    w = os.path.join(out, 'W-' + f[:-3] + '.kfx')
    n = params(t)
    if n is not None:
        how = 'parameters'
        open(w, 'w').write('(define m ((proj (load-module "%s") %s)))\n' % (f, ' '.join('@r%d' % i for i in range(n))))
    else:
        how = 'load-input' if one_expression(t) else 'load-module'
        open(w, 'w').write('(define m (%s "%s"))\n' % (how, f))
    r = subprocess.run([kfx, w], capture_output=True, text=True, timeout=120)
    if r.returncode == 0:
        return f, how, None
    said = [l for l in r.stderr.splitlines() if l.startswith('kfx:')]
    return f, how, (said[0] if said else r.stderr.strip().splitlines()[-1] if r.stderr.strip() else 'fails')
from concurrent.futures import ThreadPoolExecutor
with ThreadPoolExecutor(8) as ex:
    for f, how, e in ex.map(run, files):
        results[f] = (how, e)
ok = [f for f, (h, e) in results.items() if e is None]
print('accepted %d of %d' % (len(ok), len(files)))
def kind(e):
    m = re.sub(r'^kfx: ', '', e)
    m = re.sub(r'^\S+:\d+: ', '', m)
    m = re.sub(r'^in [^:]+: ', '', m)
    m = re.sub(r'`[^`]*`', '`…`', m)
    return m[:110]
c = collections.Counter(kind(e) for f, (h, e) in results.items() if e)
for k, n in c.most_common():
    print('%4d  %s' % (n, k))
open(os.path.join(out, 'RESULTS'), 'w').write('\n'.join('%s %s %s' % (f, h, e) for f, (h, e) in sorted(results.items())))
