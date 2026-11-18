;; Loaded by tests/kc/loads/*.fx (C471): a counter whose state is in the
;; place it is given (C442; Fixpt's docs/research/closed-code.md), an
;; instance at each load.
(module-parameters ((p place)))
(define n (ref int p) (new 0))
(define bump (subr (maxeff (read p) (write p)) (int) int)
  (lambda (k) (begin (set n (+ k (get n))) (get n))))
