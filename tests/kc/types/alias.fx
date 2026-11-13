;; => 3
(define-type t int)
(define f (subr pure (t) t) (lambda (x) (+ x 1)))
(f 2)
