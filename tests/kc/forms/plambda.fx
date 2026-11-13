;; => (3 4)
(define id (poly ((t type)) (subr pure (t) t)) (plambda ((t type)) (lambda (x) x)))
(list ((proj id int) 3) ((proj id int) 4))
