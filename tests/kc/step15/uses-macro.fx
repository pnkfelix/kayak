;; => 5
;; A macro whose template uses another, defined before it, and a
;; procedure between them that names neither.
(define-syntax inc
  (macro (_ expr) expr)
  (syntax-rules ((_ e) (+ e 1))))
(define double (subr pure (int) int) (lambda (n) (* n 2)))
(define-syntax inc2
  (macro (_ expr) expr)
  (syntax-rules ((_ e) (inc (inc e)))))
(define f (subr pure (int) int) (lambda (n) (inc2 n)))
(+ (f 3) (- (double 1) 2))
