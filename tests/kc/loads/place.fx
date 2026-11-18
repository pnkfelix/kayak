;; => (7 17 1)
;; A file of a place (C442, C471), loaded under `letrena` as kc's driver
;; loads the expander: each load an instance, its state its own.
(define counted (subr (maxeff (alloc @w) (read @globals) spin) (int int) int)
  (lambda (a b)
    (letrena q
      (let ((x ((proj (load-module "files/counter-place.fx") q))))
        (begin (with x (bump a)) (with x (bump b)))))))
`(,(counted 3 4) ,(counted 10 7) ,(counted 0 1))
