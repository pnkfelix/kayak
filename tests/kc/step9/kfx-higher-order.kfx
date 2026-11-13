;; => 18
(define twice (subr pure ((subr pure (int) int) int) int) (lambda (f x) (f (f x))))
(twice (lambda ((x int)) (* x 3)) 2)
