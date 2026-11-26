;; => (7 3 1)
;; A product's fields by label (C545): in the order written, each
;; `extract`'s index the checker's, of the type it found, here one declared,
;; one found by itself, and one a procedure's parameter's.
(define-type point (productof (x int) (y int) (z int)))
(define norm1 (lambda ((p point)) (+ (extract p x) (+ (extract p y) (extract p z)))))
(define q (product (name "b") (size 3)))
(define named-b? (lambda ((s string)) (if (string=? s "b") 1 0)))
(list (norm1 (product (x 1) (y 2) (z 4))) (extract q size) (named-b? (extract q name)))
