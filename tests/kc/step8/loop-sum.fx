;; => 5050
(define total (subr (maxeff spin (read (globals total))) (int) int) (lambda (n) (if (zero? n) 0 (+ n (total (- n 1)))))) (total 100)
