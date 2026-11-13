;; => 10
(define f (subr pure (int int) int) (lambda (a b) (+ a b))) (f (f 1 2) (f 3 4))
