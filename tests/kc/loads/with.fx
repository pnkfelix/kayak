;; => (6 12)
;; A load named and reached by `with` (C471), twice: one module.
(define s (load-module "files/shapes.fx"))
(define t (load-module "files/shapes.fx"))
`(,(with s (area (rect 2 3))) ,(with t (area (circle 2))))
