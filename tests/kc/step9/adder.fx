;; => 7
(define adder (subr pure (int) (subr pure (int) int)) (lambda (n) (lambda ((x int)) (+ x n))))
((adder 3) 4)
