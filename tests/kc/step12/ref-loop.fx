;; => 5000050000
(define total (ref int @x) (new 0))
(define add-up (subr (maxeff spin (read @x) (write @x) (read (globals total add-up))) (int) int) (lambda (n) (if (zero? n) (get total) (begin (set total (+ (get total) n)) (add-up (- n 1))))))
(add-up 100000)
