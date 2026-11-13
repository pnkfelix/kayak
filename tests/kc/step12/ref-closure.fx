;; => (3 2)
(define make-counter (subr (alloc @x) () (subr (maxeff (read @x) (write @x)) () int)) (lambda () (let ((r (the (ref int @x) (new 0)))) (lambda () (begin (set r (+ (get r) 1)) (get r))))))
(let ((a (make-counter)) (b (make-counter))) (begin (a) (a) (b) (list (a) (b))))
