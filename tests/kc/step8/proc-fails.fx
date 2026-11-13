;; => ! division by zero
(define f (subr pure (int) int) (lambda (n) (quotient 1 n))) (f 0)
