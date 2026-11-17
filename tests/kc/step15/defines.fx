;; => 120
;; Macros that make definitions, named by the forms after them.
(define-syntax define-const
  (macro (_ bvar expr) defn)
  (syntax-rules ((_ n e) (define n e))))
(define-syntax define-fact
  (macro (_ bvar) defn)
  (syntax-rules
    ((_ f)
     (define f (->* (int) int) (lambda ((n int)) (if (= n 0) 1 (* n (f (- n 1)))))))))
(define-const five 5)
(define-fact fact)
(fact five)
