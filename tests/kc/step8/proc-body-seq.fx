;; => 3
(define f (subr pure (int) int) (lambda (n) (+ n 1) (+ n 2))) (f 1)
