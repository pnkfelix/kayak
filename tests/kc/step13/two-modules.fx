;; => (1 3)
(define a (module (define v int 1) (define get (subr pure () int) (lambda () v))))
(define b (module (define v int 2) (define get (subr (read (globals a)) () int) (lambda () (+ v (with a (get)))))))
(list (with a (get)) (with b (get)))
