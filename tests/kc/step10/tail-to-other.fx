;; => 1000007
(define loop (subr (maxeff spin (read (globals loop))) (int int) int) (lambda (i acc) (if (zero? i) acc (loop (- i 1) (+ acc 1)))))
(define start (subr (maxeff spin (read (globals loop))) (int) int) (lambda (n) (loop n 7)))
(start 1000000)
