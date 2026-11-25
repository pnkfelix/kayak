; `#utf8"…"` and `#u8(…)` given to a macro (C532): atoms to the expander,
; as Fixpt's reader reads them, not the lists kayak's reader makes of them;
; a pattern's list does not match one (Fixpt refuses `(first-of #utf8"x")`,
; and kayak so). Quoted by a template, themselves.
(define-syntax q (syntax-rules ((_ x) (quote x))))
(q #utf8"héllo")
(utf8->string (q #utf8"héllo"))
(q #u8(1 2 255))
(bytevector-length (q #u8"abc"))
(q (#utf8"a" #utf8"b"))
(define-syntax first-of
  (syntax-rules
    ((_ (a b ...)) 'a)))
(first-of (#utf8"x" #utf8"y"))
(first-of (#u8"ab" #u8(3)))
