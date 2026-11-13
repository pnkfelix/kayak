;; => 9
(define inc (subr pure (int) int) (lambda (n) (+ n 1))) (define twice (subr (read (globals inc)) (int) int) (lambda (n) (inc (inc n)))) (twice 7)
