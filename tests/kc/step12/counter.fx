;; => 3
(define c (ref int @x) (new 0))
(define bump (subr (maxeff (read @x) (write @x) (read (globals c))) () int) (lambda () (begin (set c (+ (get c) 1)) (get c))))
(begin (bump) (bump) (bump))
