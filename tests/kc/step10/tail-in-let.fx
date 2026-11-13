;; => 2000000
(define count (subr (maxeff spin (read (globals count))) (int int) int) (lambda (i acc) (let ((j (- i 1))) (if (< j 0) acc (count j (+ acc 2))))))
(count 1000000 0)
