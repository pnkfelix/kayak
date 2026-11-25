;;; Strings and bytevectors in Fixpt's layout (C320): a length word, then
;;; code points two to a word or bytes eight to a word. Lengths, an odd
;;; element of each, a copy's length, and text through UTF-8 and back.
(define b (bytevectorof @heap) (make-bytevector 13 7))
(define s (stringof @heap) (make-string 5 #\a))
(define main (subr (maxeff (read @heap) (write @heap) (alloc @heap) (read (globals b s))) () string)
  (lambda ()
    (begin
      (bytevector-u8-set! b 12 200)
      (string-set! s 3 #\z)
      (string-append
        (int->string (+ 0 (bytevector-length b))) " "
        (int->string (u8->int (bytevector-u8-ref b 12))) " "
        (int->string (u8->int (bytevector-u8-ref b 3))) " "
        (int->string (+ 0 (bytevector-length (bytevector-copy b 2 13)))) " "
        (char->string (string-ref s 3)) (char->string (string-ref s 4)) " "
        (int->string (string-length (substring "héllo, wörld" 1 10)))
        (tagcase (bytes->utf8 (utf8->bytes (string->utf8 "héllo, wörld")))
          (text t-utf8 (let ((t (utf8->string t-utf8))) (string-append " " t)))
          (bad k " bad"))))))
(main)
