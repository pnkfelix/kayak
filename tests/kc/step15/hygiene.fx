;; => 11
;; The template's `tmp` against the use's own `tmp`.
(define-syntax add-one
  (macro (_ expr) expr)
  (syntax-rules ((_ e) (let ((tmp 1)) (+ tmp e)))))
(let ((tmp 10)) (add-one tmp))
