;; => (a (1 "s" #\x) #t ())
;; kc's step 15 (C447): a quoted datum of any kind, its pairs made when it
;; is met, as a transformer's code quotes a form.
(define d datum '(a (1 "s" #\x) #t ()))
d
