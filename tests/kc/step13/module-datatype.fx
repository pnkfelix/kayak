;; => 12
(define shapes (module (define-datatype shape (circle int) (square int)) (define area (subr pure (shape) int) (lambda (s) (tagcase s (circle (r) (* 3 (* r r))) (square (w) (* w w)))))))
(with shapes (area (circle 2)))
