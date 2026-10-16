;; A family a loaded module gives, named by an abbreviation with no
;; parameters of its own and applied, as Fixpt's front end names `table`
;; (C119).
(hide (define fam (load-module "families.fx")))
(define-type two (select fam two))
(define swap (poly ((a type) (b type)) (subr pure ((two a b)) (two b a)))
  (lambda (p) (product (fst (extract p snd)) (snd (extract p fst)))))
(define made (subr pure () (two i64 string)) (lambda () (product (fst 1) (snd "one"))))
