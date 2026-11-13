;; => -1
(define f (subr pure (int int) int) (lambda (a b) (- a b))) (f (+ 1 1) (* 1 3))
