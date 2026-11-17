;; => 7
;; kc's step 15 (C446): a transformer calls a helper of phase 1's
;; (`begin-for-syntax`), in force where its macro is defined: kc's
;; expansion of a use replays it before the macro's definition.
(begin-for-syntax
  (define double (subr pure (datum) datum) (lambda (d) (if (int? d) (* 2 d) d))))
(define-syntax dbl (er-macro-transformer (lambda (form rename compare) (double (car (cdr form))))))
(+ (dbl 3) 1)
