;;; `car` of a union with a pair member, an expression's (C256;
;;; `scheme-bench/peval.fx`'s `not-a-pair`): the pair's head, checked when
;;; it runs.
(define-type ip (pairof int bool @heap))
(define* first (subr (read @heap) ((union nil ip)) int) (lambda (x) (car (the (union nil ip) x))))
(first (cons 7 #t))
