;; => 285
(define xs (arrayof int @x) (make-array 10 0))
(define fill (subr (maxeff spin (read (globals fill xs)) (write @x)) (int) unit) (lambda (i) (if (= i (array-length xs)) #u (begin (array-set! xs i (* i i)) (fill (+ i 1))))))
(define total (subr (maxeff spin (read (globals total xs)) (read @x)) (int int) int) (lambda (i acc) (if (= i (array-length xs)) acc (total (+ i 1) (+ acc (array-ref xs i))))))
(begin (fill 0) (total 0 0))
