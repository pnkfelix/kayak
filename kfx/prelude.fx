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

"Flat arrays of i64s (FX-26's flatarrayof), their sizes and indices at
 i64. In Rust, a slice of Cells: FX-26's flat arrays may be aliased, and
 written through any alias, as a Cell may."

(define make-flatarray-i64 (poly ((r region)) (subr (alloc r) (i64 i64) (flatarrayof i64 r)))
  (lambda (n v) (make-flatarray (i64-flat) (i64->int n) v)))

(define flatarray-ref-i64 (poly ((r region)) (subr (read r) ((flatarrayof i64 r) i64) i64))
  (lambda (a i) (flatarray-ref a (i64->int i))))

(define flatarray-set-i64! (poly ((r region)) (subr (write r) ((flatarrayof i64 r) i64 i64) unit))
  (lambda (a i v) (flatarray-set! a (i64->int i) v)))

(define flatarray-length-i64 (poly ((r region)) (subr pure ((flatarrayof i64 r)) i64))
  (lambda (a) (int->i64 (flatarray-length a))))

"A suspension (CLAUDES_CHOICES.md C45, docs/research/async-rust.md): the
 rest of the computation up to `tag`'s prompt, `k`, given to `f`, and an
 abort there with `f k`. FX-26 would let `k` be resumed again and again;
 KFX's, a Rust future, once: so here a second resume fails, as in Rust.
 Its value is what `k` is resumed with."
(define abort-with-continuation
  (poly ((v type) (a type) (h type) (d effect) (r region) (e effect))
    (subr (maxeff e (goto r) (comefrom r) (alloc r))
          ((prompt-tag a h d r) (subr e ((subr (maxeff d (goto r) (comefrom r) (read r) (write r)) (v) a)) h))
          v))
  (plambda ((v type) (a type) (h type) (d effect) (r region) (e effect))
    (lambda (tag f)
      (call-with-composable-continuation
        (lambda ((k (composable v a d r)))
          (let ((used (the (ref bool r) (new #f))))
            (the v (abort-current-continuation tag
              (f (lambda ((x v))
                   (if (get used)
                       (begin (error "a continuation resumed twice") (k x))
                       (begin (set used #t) (k x)))))))))
        tag))))
