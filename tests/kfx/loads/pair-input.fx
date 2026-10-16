;; A file of one expression, which loads another file beside it (C121: kfx
;; joined that load's path to its directory twice, when given an absolute
;; one).
(let ((fam (load-module "families.fx")))
  (module
    (define-type two (select fam two))
    (define made (subr pure () (two i64 i64)) (lambda () (product (fst 4) (snd 2))))))
