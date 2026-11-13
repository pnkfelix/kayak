;; => 42
(define* f (subr pure (int) int) (lambda (n) (* n 3)))
(f 14)
