;; => (1 2 3 4 0 5 (6 7))
;; A `case`'s symbols are data, not names (C469): a datum that names a
;; local, a global, a keyword or a primitive is the symbol still. kc once
;; compared the key with the variable of that name.
(define expr 99)
(define which (subr pure (symbol) int)
  (lambda (n)
    (let ((data 'other))
      (case n
        ((expr) 1)
        ((data) 2)
        ((lambda if) 3)
        ((car nil) 4)
        (else 0)))))
(define kinds (subr pure (symbol) int)
  (lambda (k)
    (case k ((region place effect type data sendable flat size conv) 5) (else 0))))
`(,(which 'expr) ,(which 'data) ,(which 'if) ,(which 'nil) ,(which 'other)
  ,(kinds 'size)
  (,(case 'x ((y) 0) ((x) 6) (else 8)) ,(case #f ((#t) 0) ((#f) 7) (else 8))))
