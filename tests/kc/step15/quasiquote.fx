;; => (1 2 3 4 (nested 2) (quasiquote (x (unquote y))))
;; kc's step 15 (C447): a quasiquote, as a transformer writes its
;; expansion: `,e` its value, `,@e` its list appended, a nested one kept.
(define xs (listof datum acyclic) (list 2 3))
`(1 ,@xs ,(+ 2 2) (nested ,(car xs)) `(x ,y))
