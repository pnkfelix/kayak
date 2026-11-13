;; => 55
(define fib (subr (maxeff spin (read (globals fib))) (int) int) (lambda (n) (if (< n 2) n (+ (fib (- n 1)) (fib (- n 2)))))) (fib 10)
