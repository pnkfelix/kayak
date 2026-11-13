;; => (11 3 2)
;; `#%toplevel` where something binds the name (C397): the program's global,
;; a lambda's parameter shadowing it; and, where the program defines none, the
;; standard binding (`car`), a `let` shadowing it.
(define x 10)
(define f (subr (read (globals x)) (int) int) (lambda (x) (+ (with #%toplevel x) x)))
(define g (subr (read @heap) ((listof int @heap)) int)
  (lambda (l) (let ((car 3)) (+ car (#%toplevel:car (cdr l))))))
(list (f 1) (g (list 0 0)) (let ((x 1)) (+ #%toplevel:x (- x 9))))
