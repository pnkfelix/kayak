;; => 3
(define m (module (hide (define y int 2)) (define x int (+ y 1))))
(with m x)
