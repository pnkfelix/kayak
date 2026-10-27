;;; The same pair, taken apart where it is (C257): FX-26's `car` takes any
;;; pair, its tail a list or not.
(define* f (subr pure ((listof int acyclic)) int) (lambda (xs) (let ((p (cons 41 xs))) (+ 1 (car p)))))
(f nil)
