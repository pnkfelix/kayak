;; => (int string symbol pair nil char bool)
;; kc's step 15 (C447): a `typecase` of a datum, each arm its shape's test,
;; as a transformer takes a form apart.
(define shape (subr pure (datum) symbol)
  (lambda (d)
    (typecase d
      (int i (if (int? i) 'int 'no))
      (string s 'string)
      (symbol y (if (symbol=? y 'a) 'symbol 'no))
      (pair p 'pair)
      (nil n 'nil)
      (char c 'char)
      (else e 'bool))))
(list (shape 1) (shape "s") (shape 'a) (shape '(b)) (shape '()) (shape #\c) (shape #t))
