;; => #t
(define kind (subr pure (int) symbol) (lambda (n) (if (< n 0) 'negative (if (= n 0) 'zero 'positive))))
(eq? (kind 5) 'positive)
