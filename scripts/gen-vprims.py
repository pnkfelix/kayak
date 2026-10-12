"""Generates fx26/vprims.kfx, the engine's primitives over words (C64), from
the list in fx26/table.kfx: `op-need`, the room each may allocate, and
`apply-op`, each by its number.

usage: python3 -I scripts/gen-vprims.py
"""
import os
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
import re
src=open(os.path.join(ROOT,'fx26','table.kfx')).read()
m=re.search(r'\(define prim-list strings\s*\(list(.*?)\)\)', src, re.S)
names=re.findall(r'"([^"]*)"', m.group(1))
I=lambda e:f"(word-int {e})"
W=lambda e:f"(int-word! {e})"
I64=lambda e:f"(word-i64 {e})"
W64=lambda e:f"(i64-word! {e})"
B=lambda e:f"(bool-word {e})"
C=lambda e:f"(word-char {e})"
a,b,c="a","b","c"
spec={}
for op,f in [("i64+","i64+"),("i64-","i64-"),("i64*","i64*"),("i64-quotient","i64-quotient"),("i64-remainder","i64-remainder"),("i64-and","i64-and"),("i64-or","i64-or"),("i64-xor","i64-xor")]:
    spec[op]=W64(f"({f} {I64(a)} {I64(b)})")
spec["i64-not"]=W64(f"(i64-not {I64(a)})")
for op in ["i64=","i64<","i64>","i64<=","i64>="]: spec[op]=B(f"({op} {I64(a)} {I64(b)})")
spec["i64-shl"]=W64(f"(i64-shl {I64(a)} {I(b)})"); spec["i64-shr"]=W64(f"(i64-shr {I64(a)} {I(b)})")
for op in ["+","-","*","quotient","remainder","modulo","max","min","bitwise-and","bitwise-ior","bitwise-xor","arithmetic-shift"]:
    spec[op]=W(f"({op} {I(a)} {I(b)})")
for op in ["=","<",">","<=",">="]: spec[op]=B(f"({op} {I(a)} {I(b)})")
spec["zero?"]=B(f"(zero? {I(a)})"); spec["bitwise-not"]=W(f"(bitwise-not {I(a)})")
spec["int->i64"]=W64(f"(int->i64 {I(a)})"); spec["i64->int"]="a"
spec["int->string"]="(int-text! a)"; spec["i64->string"]="(int-text! a)"
spec["not"]=B("(not (truthy? a))"); spec["bool=?"]=B("(i64= a b)")
spec["string-append"]="(string-append-w! a b)"
spec["string=?"]=B("(i64= (string-compare-w a b 0) 0)")
spec["string<?"]=B("(i64< (string-compare-w a b 0) 0)"); spec["string>?"]=B("(i64> (string-compare-w a b 0) 0)")
spec["string<=?"]=B("(i64<= (string-compare-w a b 0) 0)"); spec["string>=?"]=B("(i64>= (string-compare-w a b 0) 0)")
spec["string-compare"]="(fixnum (string-compare-w a b 0))"
spec["string-search"]="(fixnum (string-search-w a b (fixnum-value c)))"
spec["string-downcase"]="(string-word! (string-downcase (word-string a)))"
spec["string-length"]="(fixnum (string-length-w a))"; spec["string-length-i64"]=spec["string-length"]
spec["string-ref"]="(string-ref-w a (fixnum-value b))"; spec["string-ref-i64"]=spec["string-ref"]
spec["substring"]="(substring-w! a (fixnum-value b) (fixnum-value c))"
spec["char->i64"]="(fixnum (immediate-value a))"; spec["char->integer"]=spec["char->i64"]
spec["i64->char"]="(immediate imm-char (fixnum-value a))"; spec["integer->char"]=spec["i64->char"]
spec["char->string"]="(char-string! a)"
spec["char=?"]=B("(i64= a b)"); spec["char<?"]=B("(i64< a b)"); spec["char>?"]=B("(i64> a b)")
spec["char<=?"]=B("(i64<= a b)"); spec["char>=?"]=B("(i64>= a b)")
spec["char-upcase"]=f"(char-word (char-upcase {C(a)}))"; spec["char-downcase"]=f"(char-word (char-downcase {C(a)}))"
spec["char-whitespace?"]=B(f"(char-whitespace? {C(a)})"); spec["char-numeric?"]=B(f"(char-numeric? {C(a)})"); spec["char-alphabetic?"]=B(f"(char-alphabetic? {C(a)})")
spec["string->list"]="(string-list! a (i64- (string-length-w a) 1) w-nil)"
spec["list->string"]="(list-string! a)"
spec["string->symbol"]="(symbol-word (intern! (word-string a)))"
spec["symbol->string"]="(string-word! (name-of-number (immediate-value a)))"
spec["symbol=?"]=B("(i64= a b)"); spec["symbol-compare"]="(fixnum (int->i64 (string-compare (name-of-number (immediate-value a)) (name-of-number (immediate-value b)))))"
spec["cons"]="(pair! a b)"; spec["car"]="(checked-car a)"; spec["cdr"]="(checked-cdr a)"
spec["null?"]=B("(i64= a w-nil)"); spec["pair?"]=B("(is-obj? a ob-pair)")
spec["length"]="(fixnum (list-count a 0))"; spec["list-length"]=spec["length"]
spec["reverse"]="(reverse-words! a w-nil)"; spec["append"]="(append-words! (reverse-words! a w-nil) b)"
spec["new"]="(one-word! ob-ref a)"; spec["get"]="(obj-ref a 0)"; spec["set"]="(begin (obj-set! a 0 b) w-unit)"
spec["rnew"]="(one-word! ob-ref b)"; spec["rcons"]="(pair! b c)"
spec["make-array"]="(filled! ob-array (fixnum-value a) b)"; spec["rmake-array"]="(filled! ob-array (fixnum-value b) c)"
spec["array-ref"]="(element a (fixnum-value b))"; spec["array-set!"]="(begin (element-set! a (fixnum-value b) c) w-unit)"
spec["array-length"]="(fixnum (obj-size a))"
spec["vector?"]=B("(is-obj? a ob-vector)"); spec["datum-list->vector"]="(list-array! ob-vector a)"
spec["array->list"]="(array-list! a (i64- (obj-size a) 1) w-nil)"; spec["list->array"]="(list-array! ob-array a)"
spec["make-flatarray-i64"]="(filled! ob-flat (fixnum-value a) b)"; spec["flatarray-ref-i64"]="(element a (fixnum-value b))"
spec["flatarray-set-i64!"]="(begin (element-set! a (fixnum-value b) c) w-unit)"; spec["flatarray-length-i64"]="(fixnum (obj-size a))"
spec["make-bytevector"]="(filled-raw! ob-bytes (fixnum-value a) (fixnum-value b))"
spec["bytevector-length"]="(fixnum (obj-size a))"
spec["bytevector-u8-ref"]="(fixnum (element a (fixnum-value b)))"
spec["bytevector-u8-set!"]="(begin (element-set! a (fixnum-value b) (i64-and (fixnum-value c) 255)) w-unit)"
spec["bytevector-copy"]="(bytes-copy! a (fixnum-value b) (fixnum-value c))"
spec["bytevector-copy!"]="(begin (bytes-copy-into! a b c (fourth-arg args) (fifth-arg args)) w-unit)"
spec["bytevector=?"]=B("(raw-equal? a b 0)")
spec["string->utf8"]="(kfx-bytes-word! (string->utf8 (word-string a)))"
spec["utf8->string"]="(utf8-word! a)"
spec["error"]="(begin (error (word-string a)) 0)"
spec["%identity"]="a"; spec["%case="]=B("(same-atom-w? a b)")
spec["int?"]=B("(or (fixnum? a) (is-obj? a ob-big))"); spec["string?"]=B("(is-obj? a ob-string)")
spec["bool?"]=B("(is-imm? a imm-bool)"); spec["char?"]=B("(is-imm? a imm-char)")
spec["symbol?"]=B("(or (is-imm? a imm-symbol) (i64= a w-unit))")
spec["array?"]=B("(or (is-obj? a ob-array) (is-obj? a ob-flat))")
spec["procedure?"]=B("(or (is-obj? a ob-proc) (is-obj? a ob-ctor) (is-obj? a ob-cont) (is-imm? a imm-prim))")
spec["f64?"]=B("(is-obj? a ob-real)"); spec["ref?"]=B("(is-obj? a ob-ref)")
spec["sum?"]=B("(or (is-obj? a ob-sum) (is-obj? a ob-data))"); spec["product?"]=B("(is-obj? a ob-product)")
spec["bytevector?"]=B("(is-obj? a ob-bytes)")
spec["write-out"]="(written-word! (write-out (word-string a)))"; spec["write-err"]="(written-word! (write-err (word-string a)))"
spec["exit"]="(begin (exit (word-int a)) 0)"; spec["clock"]=W64("(clock)")
spec["sleep-until"]="(begin (sleep-until (word-i64 a)) w-unit)"
spec["args"]="(strings-word! (get program-args))"
spec["read-line"]="(read-line-word!)"; spec["read-file"]="(read-file-word! a)"
spec["write-file"]="(written-word! (write-file (word-string a) (word-kfx-bytes b)))"
spec["list-dir"]="(list-dir-word! a)"
spec["int->f64"]="(real-word! (int->f64 (word-int a)))"; spec["f64->int"]=W("(f64->int (word-real a))")
spec["f64->string"]="(string-word! (f64->string (word-real a)))"
for op in ["f64+","f64-","f64*","f64/","f64-min","f64-max"]: spec[op]=f"(real-word! ({op} (word-real a) (word-real b)))"
for op in ["f64<","f64<=","f64>","f64>=","f64="]: spec[op]=B(f"({op} (word-real a) (word-real b))")
for op in ["f64-abs","f64-neg","f64-sqrt","f64-floor","f64-ceiling","f64-truncate","f64-round"]: spec[op]=f"(real-word! ({op} (word-real a)))"
spec["f64-nan?"]=B("(f64-nan? (word-real a))"); spec["f64-infinite?"]=B("(f64-infinite? (word-real a))"); spec["f64-finite?"]=B("(f64-finite? (word-real a))")
spec["string-ci=?"]=B("(string=? (string-downcase (word-string a)) (string-downcase (word-string b)))")
spec["eq?"]=B("(i64= a b)"); spec["set-car!"]="(begin (obj-set! a 0 b) w-unit)"; spec["set-cdr!"]="(begin (obj-set! a 1 b) w-unit)"; spec["stay-cellular"]="a"
spec["make-continuation-mark-key"]="(one-word! ob-ref (fixnum 0))"
special={"with-mark","first-mark","current-marks","marks-of","apply","make-continuation-prompt-tag","abort-current-continuation","call-with-composable-continuation","abort-with-continuation"}
# needs: words a primitive may allocate
need={}
for op in names: need[op]="64"
need.update({"string-append":"(i64+ (i64+ (string-length-w a) (string-length-w b)) 1)",
 "string->list":"(i64* 3 (i64+ (string-length-w a) 1))","list->string":"(i64+ (list-count a 0) 1)",
 "substring":"(i64+ (string-length-w a) 1)","string-downcase":"(i64* 2 (i64+ (string-length-w a) 1))",
 "symbol->string":"(i64+ (string-length-i64 (name-of-number (immediate-value a))) 1)",
 "reverse":"(i64* 3 (i64+ (list-count a 0) 1))","append":"(i64* 6 (i64+ (list-count a 0) 1))",
 "make-array":"(i64+ (fixnum-value a) 1)","rmake-array":"(i64+ (fixnum-value b) 1)","make-flatarray-i64":"(i64+ (fixnum-value a) 1)",
 "make-bytevector":"(i64+ (fixnum-value a) 1)","array->list":"(i64* 3 (i64+ (obj-size a) 1))","list->array":"(i64+ (list-count a 0) 1)","datum-list->vector":"(i64+ (list-count a 0) 1)",
 "bytevector-copy":"(i64+ (obj-size a) 1)","string->utf8":"(i64+ (i64* 4 (string-length-w a)) 2)","utf8->string":"(i64+ (obj-size a) 8)",
 "read-file":"(file-room! a)","list-dir":"(dir-room! a)","args":"(args-room)","read-line":"(line-room!)",
 "f64->string":"64","int->string":"(text-room a)","i64->string":"(text-room a)",
 "+":"(int-room a b)","-":"(int-room a b)","*":"(int-room-mul a b)","quotient":"(int-room a b)","remainder":"(int-room a b)","modulo":"(int-room a b)",
 "arithmetic-shift":"(shift-room a b)","bitwise-and":"(int-room a b)","bitwise-ior":"(int-room a b)","bitwise-xor":"(int-room a b)","bitwise-not":"(int-room a a)"})
out=[]
out.append('''"vprims.kfx: the primitives over words (C64), generated from the list in
 table.kfx by scripts/gen-vprims.py: `op-need`, the
 room each may allocate, made sure of before it runs; `apply-op`, each by
 its number. The helpers they use are in vwords.kfx."

(define op-need (subr vm (i64 i64) i64)
  (lambda (op args)
    (let ((a (if (i64= args w-nil) (the i64 0) (pair-car args)))
          (b (if (or (i64= args w-nil) (i64= (pair-cdr args) w-nil)) (the i64 0) (pair-car (pair-cdr args)))))
      (cond''')
for i,n in enumerate(names):
    if n in special: continue
    if need[n]!="64":
        out.append(f"        ((i64= op {i}) {need[n]})")
out.append('''        ((i64>= op fixed-base) 8)
        (else 64)))))

(define apply-op (subr vm (i64 i64) i64)
  (lambda (op args)
    (let* ((a (if (i64= args w-nil) (the i64 0) (pair-car args)))
           (r1 (if (i64= args w-nil) w-nil (pair-cdr args)))
           (b (if (i64= r1 w-nil) (the i64 0) (pair-car r1)))
           (r2 (if (i64= r1 w-nil) w-nil (pair-cdr r1)))
           (c (if (i64= r2 w-nil) (the i64 0) (pair-car r2))))
      (cond''')
for i,n in enumerate(names):
    if n in special: continue
    out.append(f"        ((i64= op {i}) {spec[n]})")
out.append('''        ((i64>= op fixed-base) (fixed-op-word (name-of-number (i64- op fixed-base)) a b))
        (else (begin (error "the engine: a primitive of no number") 0))))))
''')
missing=[n for n in names if n not in special and n not in spec]
assert not missing, missing
open(os.path.join(ROOT,'fx26','vprims.kfx'),'w').write('\n'.join(out))
print(len(names),"primitives")
