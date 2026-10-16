;; A file of module-parameters that loads another at its own region, hidden,
;; and names a type of it by `select`, as Fixpt's `parser-types.fx` names
;; the reader's `syn` (C118).
(module-parameters ((q region)))
(hide (define base ((proj (load-module "shapes.fx") q))))
(define-type shape (select base shape))
(define area (subr pure (shape) int)
  (lambda (s) (tagcase s (dot (x) 0) (box (w h) (* w h)))))
(define square (subr (read @globals) (int) shape) (lambda (n) (with base (make-box n n))))
