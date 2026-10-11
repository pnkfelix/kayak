"prelude.fx: what KFX has that FX-26's standard environment has only at
 `int`. KFX has no `int` (PLAN.md, section 2): it writes `i64`, whose
 arithmetic wraps in FX-26 as in Rust.

 This file is FX-26, not KFX: it is where `int` meets `i64`. Fixpt reads
 it before a KFX program (stage 0). The transpiler never sees it: each
 name here is a function of kayak-rt, by the same mangling, and this file
 is that function's meaning."

(define string-length-i64 (subr pure (string) i64)
  (lambda (s) (int->i64 (string-length s))))

(define string-ref-i64 (subr pure (string i64) char)
  (lambda (s i) (string-ref s (i64->int i))))

(define char->i64 (subr pure (char) i64)
  (lambda (c) (int->i64 (char->integer c))))

(define i64->string (subr pure (i64) string)
  (lambda (n) (int->string (i64->int n))))
