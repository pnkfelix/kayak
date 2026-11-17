;; A counter whose state is in the place it is given (C442; Fixpt's
;; docs/research/closed-code.md): loaded in a process, an instance its own.
(module-parameters ((p place)))
(define-effect kfx (maxeff (read @globals) spin))
(define n (ref i64 p) (new 0))
(define bump (subr (maxeff kfx (read p) (write p)) (i64) i64)
  (lambda (k) (begin (set n (i64+ k (get n))) (get n))))
