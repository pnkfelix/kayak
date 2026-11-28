; Tagged literals (Fixpt 9025322f; C564): `#Ltag"text"` read as
; `(literal:tag #utf8"text")`, a use of the name `literal:tag`, which a
; procedure may be; quoted, that list, written back as it was read; `#l` as
; `#L`; a list of the shape whose name is no tag's, or whose second is no
; `utf8`, written as a list.
(define literal:twice (subr pure (utf8) string)
  (lambda (u) (string-append (utf8->string u) (utf8->string u))))
#Ltwice"ab"
'#Limg"iVBORw0KGgo="
(car '#Limg"x")
(utf8? (car (cdr '#Limg"x")))
'(a #l-A1"y" (literal:x "plain") (literal:x #utf8"z" 1) (literal: #utf8"w") (literal:a_b #utf8"v"))
'#Lq"a\"b"
