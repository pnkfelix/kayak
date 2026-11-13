;; => (3 11)
(define m (module (define x int 3) (define f (subr pure (int) int) (lambda (x) (+ x 1)))))
(with m (list x (f 10)))
