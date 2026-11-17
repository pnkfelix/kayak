;; => (sum product box bloblet other)
;; kc's step 15 (C448): a `typecase` of a bloblet's kind: a sum, a product,
;; a box, an array, each by its header, as Fixpt's heap has them.
(define-type thing (oneof (sumof (some int)) (productof (1 int) (2 int)) (ref int @x) (arrayof int @x) int))
(define kind-of (subr (read @x) (thing) symbol)
  (lambda (x)
    (typecase x
      (sum s 'sum)
      (product p 'product)
      (box b 'box)
      (bloblet a 'bloblet)
      (else e 'other))))
(list (kind-of (sum some 1)) (kind-of (product (1 2) (2 3))) (kind-of (new 4)) (kind-of (make-array 2 0)) (kind-of 5))
