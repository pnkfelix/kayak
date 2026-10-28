;;; A `plambda` where nothing says its type: its type found from its body,
;;; closed over its binders, as Fixpt finds it (C273: kayak's checker had
;;; taken a plambda only where a poly type is expected).
(define id (plambda ((t type)) (lambda ((x t)) x)))
(define pick (plambda ((a type) (b type)) (lambda ((x a) (y b)) y)))
(+ ((proj id int) 5) ((proj pick bool int) #t 37))
