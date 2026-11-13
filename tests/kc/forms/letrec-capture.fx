;; => (10 20 30 40)
(define count-to (subr spin (int) (listof int acyclic)) (lambda (k) (letrec ((go (subr spin (int (listof int acyclic)) (listof int acyclic)) (lambda (i acc) (if (= i 0) acc (go (- i 1) (cons (* i k) acc)))))) (go 4 nil))))
(count-to 10)
