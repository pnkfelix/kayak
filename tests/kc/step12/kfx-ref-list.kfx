;; => (3 2 1)
(define acc (ref (listof int acyclic) @x) (new nil))
(define push (subr (maxeff (read @x) (write @x) (read (globals acc))) (int) unit) (lambda (n) (set acc (cons n (get acc)))))
(begin (push 1) (push 2) (push 3) (get acc))
