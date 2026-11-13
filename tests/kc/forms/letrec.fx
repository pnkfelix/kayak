;; => 3628800
(letrec ((fact (subr spin (int) int) (lambda (n) (if (zero? n) 1 (* n (fact (- n 1))))))) (fact 10))
