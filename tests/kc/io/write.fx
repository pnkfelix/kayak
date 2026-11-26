;; => (1 1 0)
;; The standard output and error (C546): each write's text in order with
;; the values written before and after it, in process as built (the error
;; given no text: kc-steps reads Fixpt's answer after its error); a read of
;; the standard input, empty, at its end (`eof`, kc-steps gives none).
(define* done? (subr pure ((sumof (done unit) (failed io-error))) int)
  (lambda (r) (tagcase r (done u 1) (failed e 0))))
(define* byte? (subr pure ((sumof (byte u8) (eof unit) (failed io-error))) int)
  (lambda (b) (tagcase b (byte k 1) (eof u 0) (failed e 2))))
(done? (write-out "a line\n"))
(write-out "no line, then ")
(list (done? (write-out "two\n")) (done? (write-err "")) (byte? (read-byte)))
