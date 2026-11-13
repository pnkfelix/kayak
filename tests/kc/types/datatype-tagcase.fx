;; => (12 10)
(define-datatype shape (circle int) (square int int))
(define area (subr pure (shape) int) (lambda (s) (tagcase s (circle (r) (* 3 (* r r))) (square (w h) (* w h)))))
(list (area (circle 2)) (area (square 2 5)))
