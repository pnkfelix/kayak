;; => #t
(define even (subr pure (int) bool) (lambda (n) (zero? (remainder n 2)))) (even 4)
