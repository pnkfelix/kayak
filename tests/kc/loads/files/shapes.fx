;; Loaded by tests/kc/loads/*.fx (C471): a file of no parameters, its
;; datatype and procedures; one module for all its loads.
(define-datatype shape (circle int) (rect int int))
(define area (subr pure (shape) int)
  (lambda (s) (tagcase s (circle (r) (* 3 (* r r))) (rect (w h) (* w h)))))
(define areas (subr pure ((listof shape acyclic)) int)
  (lambda (ss) (if (null? ss) 0 (+ (area (car ss)) (areas (cdr ss))))))
