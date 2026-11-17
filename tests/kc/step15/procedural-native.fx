;; => (6 0 1 2 3)
;; kc's step 15 (C447): a transformer kc compiles to native code and runs,
;; built as any program is: it takes its use apart by `typecase` and
;; compares names.
(define-syntax sum-or-list
  (er-macro-transformer
    (lambda (form rename compare)
      (let ((op (car (cdr form))) (args (cdr (cdr form))))
        (typecase args
          (pair p (if (typecase op (symbol s (compare s 'sum)) (else e #f))
                      (cons (rename '+) p)
                      (cons (rename 'list) p)))
          (else e `(,(rename 'quote) 0)))))))
(cons (sum-or-list sum 2 4) (cons (sum-or-list sum) (sum-or-list list 1 2 3)))
