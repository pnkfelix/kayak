;; => (negative zero positive)
(define kind (subr pure (int) symbol) (lambda (n) (if (< n 0) 'negative (if (= n 0) 'zero 'positive))))
(list (kind -3) (kind 0) (kind 7))
