; `#utf8"…"` (Fixpt 06a3666) as KFX takes it: a constant of type `utf8`,
; made once, a string's escapes; quoted, itself. Fixpt's
; run/utf8-literals.fx has `utf8`'s operations too, which kayak has not
; carried yet (TODO §54): its conversions alone here.
(define t #utf8"aé\x1F600;z")
t
(utf8->string t)
(bytevector-length (utf8->bytes t))
(string=? (utf8->string t) (utf8->string (string->utf8 "aé\x1F600;z")))
'#utf8"quoted"
(define once (subr pure () utf8) (lambda () #utf8"once"))
(utf8->string (once))
#utf8""
(utf8->string #utf8"\x3BB;q\t")
(utf8->string #utf8"back \"quoted\"")
