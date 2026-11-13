;; => 1110000
(define steps (subr (maxeff spin (read (globals steps))) (int int) int) (lambda (n k) (cond ((= n 1) k) ((zero? (remainder n 2)) (steps (quotient n 2) (+ k 1))) (else (steps (+ (* 3 n) 1) (+ k 1))))))
(define many (subr (maxeff spin (read (globals many steps))) (int int) int) (lambda (i acc) (if (zero? i) acc (many (- i 1) (+ acc (steps 27 0))))))
(many 10000 0)
