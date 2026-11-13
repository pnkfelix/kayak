;; => 7
(define m (module (define x int 3) (define f (subr pure (int) int) (lambda (n) (+ n x)))))
(with m (f 4))
