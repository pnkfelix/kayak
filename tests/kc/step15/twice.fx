;; => 42
;; kc's step 15 (C445): a macro, its use expanded by kc, a form at a time.
(define-syntax twice
  (macro (_ (many expr)) expr)
  (syntax-rules ((_ e) (+ e e))))
(twice 21)
