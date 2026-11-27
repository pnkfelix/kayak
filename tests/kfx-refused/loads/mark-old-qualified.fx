;; A module parameter marked as before keywords, the symbol `qualified`
;; (C557): refused, as Fixpt refuses it since 23af3a07.
(module-parameters ((m-base (load-moduleof "mark-base.fx") qualified)))
(hide (import m-base twice))
(define* four (subr pure () int) (lambda () (twice 2)))
