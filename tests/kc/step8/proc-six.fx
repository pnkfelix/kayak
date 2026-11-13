;; => 21
(define s (subr pure (int int int int int int) int) (lambda (a b c d e f) (+ a (+ b (+ c (+ d (+ e f))))))) (s 1 2 3 4 5 6)
