; One macro's template expanded more than once (C557, TODO.md §57): the
; checker's facts of a form keyed by its place, every expansion of one
; template was one place, and the second's fact the first's too, the last
; field each `extract` here. Two uses in one form, uses in two forms, and a
; macro whose recursion expands its template within itself.
(define-syntax get1 (syntax-rules ((_ e k) (extract e k))))
(define-syntax get*
  (macro (_ e @ expr k @ label ...) expr)
  (syntax-rules
    ((_ e k) (extract e k))
    ((_ e k k2 ...) (get* (extract e k) k2 ...))))
(define-type inner (productof (refs int) (defs int)))
(define-type outer (productof (ann inner) (name string)))
(define i inner (product (refs 3) (defs 4)))
(define o outer (product (ann i) (name "xy")))
(define a int (get1 i refs))
(define b int (get1 i defs))
(list a b (get1 i refs) (get1 i defs)
      (get* o ann refs) (get* o ann defs) (string-length (get* o name)))
