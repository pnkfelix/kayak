;; => 3000000
(define go (subr (maxeff spin (read (globals go))) ((subr pure (int) int) int int) int) (lambda (f i acc) (if (zero? i) acc (go f (- i 1) (f acc)))))
(let ((k 3)) (go (lambda ((x int)) (+ x k)) 1000000 0))
