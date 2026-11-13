;; => (7 7 3)
(define xs (arrayof int @x) (make-array 3 7))
(list (array-ref xs 0) (array-ref xs 2) (array-length xs))
