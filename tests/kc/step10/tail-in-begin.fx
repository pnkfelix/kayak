;; => -1000000
(define count (subr (maxeff spin (read (globals count))) (int int) int) (lambda (i acc) (begin (+ i 1) (if (zero? i) acc (begin acc (count (- i 1) (- acc 1)))))))
(count 1000000 0)
