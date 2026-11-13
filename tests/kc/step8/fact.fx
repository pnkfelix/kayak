;; => 120
(define fact (subr (maxeff spin (read (globals fact))) (int) int) (lambda (n) (if (zero? n) 1 (* n (fact (- n 1)))))) (fact 5)
