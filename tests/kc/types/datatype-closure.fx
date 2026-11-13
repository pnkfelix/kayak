;; => (7 12)
(define-datatype op (add int) (mul int))
(define apply-op (subr pure (op) (subr pure (int) int)) (lambda (o) (tagcase o (add (n) (lambda ((x int)) (+ x n))) (mul (n) (lambda ((x int)) (* x n))))))
(list ((apply-op (add 3)) 4) ((apply-op (mul 3)) 4))
