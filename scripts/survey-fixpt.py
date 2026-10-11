"""Which of Fixpt's FX-26 test programs kfx accepts, and why not the rest
(docs/survey.md). Each program Fixpt checks is given to kfx; its first
refusal is kept, its place dropped, and refusals are counted by kind.

usage: python3 -I scripts/survey-fixpt.py [ROWS.tsv]
"""
import os, re, subprocess, sys, collections
ROOT=os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
fixpt=subprocess.run([ROOT+'/scripts/fixpt.sh'],capture_output=True,text=True).stdout.strip()
PROGS=os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(fixpt))),'crates','fixpt-fx26','tests','programs')
kfx=ROOT+'/target/release/kfx'
rows=[]
for d,_,fs in sorted(os.walk(PROGS)):
    for f in sorted(fs):
        if not f.endswith('.fx'): continue
        p=os.path.join(d,f); rel=os.path.relpath(p,PROGS)
        try:
            fx=subprocess.run([fixpt,'--dialect','fx26','check',p],capture_output=True,text=True,timeout=30)
        except subprocess.TimeoutExpired:
            continue
        if fx.returncode!=0 or 'error' in fx.stdout.lower() and 'agree' not in fx.stdout: continue
        k=subprocess.run([kfx,p],capture_output=True,text=True,timeout=60)
        if k.returncode==0:
            rows.append((rel,'ACCEPTED')); continue
        msg=[l for l in k.stderr.splitlines() if l.startswith('kfx:')]
        msg=msg[0] if msg else k.stderr.strip().splitlines()[0]
        msg=re.sub(r'^kfx: [^ ]*:\d+: ','kfx: ',msg)
        msg=re.sub(r'^kfx: in [^:]+: ','',msg)
        msg=re.sub(r'^kfx: ','',msg)
        rows.append((rel,msg))
cats=collections.Counter()
def cat(m):
    m=re.sub(r'"[^"]*"','"…"',m)
    m=re.sub(r'named \S+','named X',m)
    m=re.sub(r': .*','',m) if not m.startswith('a type KFX') else m[:40]
    return m[:70]
for r,m in rows: cats[cat(m)]+=1
print(len(rows),'programs Fixpt accepts;',sum(1 for r,m in rows if m=='ACCEPTED'),'kfx accepts')
for c,n in cats.most_common(): print('%4d  %s'%(n,c))
if sys.argv[1:]:
    open(sys.argv[1],'w').write('\n'.join('%s\t%s'%r for r in rows)+'\n')
