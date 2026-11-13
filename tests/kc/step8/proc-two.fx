;; => 7
(define f (subr pure (int int) int) (lambda (a b) (- a b))) (f 10 3)
