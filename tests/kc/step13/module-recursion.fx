;; => 3628800
(define m (module (define fact (subr (maxeff spin (read (globals fact))) (int) int) (lambda (n) (if (zero? n) 1 (* n (fact (- n 1))))))))
(with m (fact 10))
