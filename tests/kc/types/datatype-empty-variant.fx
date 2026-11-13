;; => (0 5)
(define-datatype opt (none) (some int))
(define get (subr pure (opt) int) (lambda (o) (tagcase o (none () 0) (some (v) v))))
(list (get (none)) (get (some 5)))
