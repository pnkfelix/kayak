;; => 7
(define r (ref (ref int @x) @x) (new (new 1)))
(begin (set (get r) 7) (get (get r)))
