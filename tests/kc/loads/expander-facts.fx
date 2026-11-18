;; => (((22848371) (x) ()) ((1303289370) () (m)))
;; Fixpt's expander, a file of a place, loaded by a kc program as kc's
;; `kc-instance` loads it (C471), under a `letrena`, an instance its own:
;; a file's facts, what each form defines, a macro or a value.
(define-effect asks (maxeff (read @globals) spin nondet (read @q) (write @q) (alloc @q) (io process) (io fs-read) (io fs-write)
                            (io stderr) (io exit) (alloc @io) (read @io) (write @io) (io native) (io sys)))
(define facts (subr (maxeff asks (alloc @w)) (datum) datum)
  (lambda (forms)
    (letrena p
      (let ((x ((proj (load-module "../../../fx26/macros/expander-kc.fx") p))))
        (with x (drive-written-facts forms))))))
(facts '((1 "f" 1 0 (0 "f" 0 1 define) (0 "f" 0 8 x) (0 "f" 0 10 1))
         (1 "f" 2 12 (0 "f" 0 13 define-syntax) (0 "f" 0 27 m)
            (1 "f" 2 29 (0 "f" 0 30 syntax-rules) (1 "f" 2 43)
               (1 "f" 2 46 (1 "f" 2 47 (0 "f" 0 48 _) (0 "f" 0 50 a)) (0 "f" 0 53 a))))))
