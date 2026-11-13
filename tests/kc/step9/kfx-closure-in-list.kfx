;; => (10 21)
(define apply-all (subr (maxeff spin (read (globals apply-all))) ((listof (subr pure (int) int) acyclic) int) (listof int acyclic)) (lambda (fs x) (if (null? fs) nil (cons ((car fs) x) (apply-all (cdr fs) x)))))
(let ((k 7)) (apply-all (list (lambda ((x int)) (+ x k)) (lambda ((x int)) (* x k))) 3))
