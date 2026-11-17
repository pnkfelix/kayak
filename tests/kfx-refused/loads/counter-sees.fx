;; A counter of a place that names its loader's global, `seen` (C442):
;; a file loaded sees the standard environment alone, so both refuse it.
(module-parameters ((p place)))
(define-effect kfx (maxeff (read @globals) spin))
(define n (ref i64 p) (new 0))
(define bump (subr (maxeff kfx (read p) (write p)) (i64) i64)
  (lambda (k) (begin (set n (i64+ k (get n))) (get n) seen)))
