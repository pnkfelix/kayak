;; => 42
(define f (subr pure (int) int) (let ((k 2)) (lambda ((x int)) (* x k))))
(f 21)
