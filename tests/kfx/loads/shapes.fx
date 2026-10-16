;; Shapes at a region of the loader's (C118): a file of module-parameters.
(module-parameters ((r region)))
(define-datatype shape (dot int) (box int int))
(define make-box (subr pure (int int) shape) (lambda (w h) (box w h)))
