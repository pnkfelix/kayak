;; => 7
(define inc (subr pure (int) int) (lambda (n) (+ n 1)))
(define twice (subr pure ((subr pure (int) int) int) int) (lambda (f x) (f (f x))))
(twice inc 5)
