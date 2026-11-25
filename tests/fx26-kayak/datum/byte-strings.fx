; `#u8"…"` and `#u8(…)` (Fixpt 3569ce51) as KFX takes them: a constant
; of type `bytevector`, `(bytevectorof const)`, made once; quoted, itself.
; Fixpt's run/byte-strings.fx has them within quoted data too, which KFX's
; `datum` has not yet, and `utf8`'s operations, which kayak has not
; carried yet (TODO §54).
(define t #u8"ab\x80;\xff;\n\\z")
t
(bytevector-length t)
(bytevector=? t #u8(97 98 128 255 10 92 122))
(bytevector-u8-ref #u8"A" 0)
'#u8"quoted"
'#u8(1 2)
(define once (subr pure () bytevector) (lambda () #u8"once"))
(bytevector-length (once))
#u8""
(bytevector=? #u8"" #u8())
(tagcase (bytes->utf8 #u8"h\xc3;\xa9;") (text u (utf8->string u)) (bad n "bad"))
(define first (subr pure (bytevector) int) (lambda (b) (u8->int (bytevector-u8-ref b 0))))
(first #u8"Z")
