;; => 3
(define counter (module (hide (define c (ref int @x) (new 0))) (define next (subr (maxeff (read @x) (write @x) (read (globals c))) () int) (lambda () (begin (set c (+ (get c) 1)) (get c))))))
(with counter (begin (next) (next) (next)))
