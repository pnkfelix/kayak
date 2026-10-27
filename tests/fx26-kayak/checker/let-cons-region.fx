;;; A pair `cons` makes where nothing says its region is at a region of its
;;; own (FX-26's implicit projection, `docs/fx26.md`; C257), not its tail's:
;;; so it is no list at `acyclic`, its tail's region. Refused, as Fixpt
;;; refuses it (C254: kayak's checker had accepted it).
(define-type ints (listof int acyclic))
(define* g (subr pure (ints) int) (lambda (xs) (if (null? xs) 0 (car xs))))
(define* f (subr pure (int ints) int) (lambda (v path) (let ((p (cons v path))) (g p))))
(f 1 nil)
