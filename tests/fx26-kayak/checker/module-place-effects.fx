;;; A module made in a procedure over a place, opened by a `with`: its
;;; procedures' effects at the place given, `r`, which the `letrena` masks
;;; (C272: kayak's checker had kept them at an inference variable, solved
;;; since, and refused what Fixpt accepts).
(define-type (builder (p place))
  (moduleof (val count-to (subr (maxeff (alloc p) (read p)) (int) (listof int p)))))
(define builder-in (poly ((p place)) (subr (alloc p) ((place p)) (builder p)))
  (lambda (p)
    (module
      (define count-to (subr (maxeff (alloc p) (read p)) (int) (listof int p))
        (lambda (n) (cons n nil))))))
(define probe2 (subr (read (globals builder-in)) () int)
  (lambda () (letrena r (let ((m (builder-in r))) (with m (car (count-to 7)))))))

(probe2)
