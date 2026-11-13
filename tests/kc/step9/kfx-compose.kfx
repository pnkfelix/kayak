;; => 42
(define compose (subr pure ((subr pure (int) int) (subr pure (int) int)) (subr pure (int) int)) (lambda (f g) (lambda ((x int)) (f (g x)))))
(define dbl (subr pure (int) int) (lambda (n) (* n 2)))
((compose dbl (lambda ((x int)) (+ x 1))) 20)
