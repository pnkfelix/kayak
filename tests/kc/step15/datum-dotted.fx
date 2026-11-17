;; => ((a b . c) (1 2 . 3) (x . 7))
;; kc's step 15 (C448): a dotted datum, quoted and quasiquoted, its last
;; pair's cdr the item after the `.`.
(define n int 7)
(list '(a b . c) '(1 2 . 3) `(x . ,n))
