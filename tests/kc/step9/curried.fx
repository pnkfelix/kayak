;; => 7
(define curry3 (subr pure (int) (subr pure (int) (subr pure (int) int))) (lambda (a) (lambda ((b int)) (lambda ((c int)) (+ a (* b c))))))
(((curry3 1) 2) 3)
