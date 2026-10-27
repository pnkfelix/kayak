;;; `string->list` and `symbol->string` (C258), on tier 1 and the tiers
;;; after: a string's characters, and a symbol's name, made a string.
(define* count-of (subr (maxeff (read @heap) spin) (char (listof char @heap)) int)
  (lambda (c cs) (if (null? cs) 0 (+ (if (char=? c (car cs)) 1 0) (count-of c (cdr cs))))))
(define cs (listof char @heap) (string->list "abracadabra"))
(+ (* 100 (count-of #\a cs)) (string-length (symbol->string 'hello)))
