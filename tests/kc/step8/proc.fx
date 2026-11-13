;; => 5
(define f (subr pure (int) int) (lambda (n) (+ n 1))) (f 4)
