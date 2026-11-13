;; => 6
(define squares (subr (alloc @x) (int) (arrayof int @x)) (lambda (n) (make-array n 3)))
(array-length (squares 6))
