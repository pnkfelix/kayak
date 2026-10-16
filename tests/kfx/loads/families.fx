;; A family of types, which another file names by `select` (C119).
(define-type (two (a type) (b type)) (productof (fst a) (snd b)))
