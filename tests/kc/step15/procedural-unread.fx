;; => (1 #u)
;; kc's step 15 (C448): a use kc's reader does not read (one that holds
;; `#u`): the transformer called as the engine calls it, the call a form of
;; its program, built and run.
(define-syntax rest-of
  (er-macro-transformer
    (lambda (form rename compare) `(,(rename 'quote) ,(cdr form)))))
(rest-of 1 #u)
