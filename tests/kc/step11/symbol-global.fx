;; => #t
(define s symbol 'shared)
(define t symbol (string->symbol "shared"))
(eq? s t)
