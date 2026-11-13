;; => 25
(define sq (subr pure (int) int) (lambda (n) (let ((m n)) (* m m)))) (sq 5)
