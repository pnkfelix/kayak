;; => 1000000
(define six (subr (maxeff spin (read (globals six))) (int int int int int int) int) (lambda (a b c d e f) (if (zero? a) (+ b (+ c (+ d (+ e f)))) (six (- a 1) c d e f (+ b 1)))))
(six 1000000 0 0 0 0 0)
