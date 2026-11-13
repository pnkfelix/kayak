;; => (1 2)
(define two (subr pure (int) (listof int acyclic)) (lambda (n) (list (- n 1) n))) (two 2)
