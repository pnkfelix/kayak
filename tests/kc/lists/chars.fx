;; => ("a" "λ " #t #t #t #t #f #t #t #f #t ("x" "y" "z"))
;; Characters (C470), the expander's: `char->string`, a string made in
;; kc's heap; `char-whitespace?` and `char-numeric?` as Fixpt's, Unicode's
;; (an ideographic space, an Arabic-Indic three, a half); and strings of
;; characters made in a loop, past a heap of 64 words.
(define* each (subr (maxeff spin (read (globals each))) (string int (listof string acyclic)) (listof string acyclic))
  (lambda (s k acc)
    (if (zero? k) acc (each s (- k 1) (cons (char->string (string-ref s (- k 1))) acc)))))
(define* churn (subr (maxeff spin (read (globals churn each))) (int) (listof string acyclic))
  (lambda (k) (if (zero? k) (each "xyz" 3 nil) (begin (each "abcdefgh" 8 nil) (churn (- k 1))))))
`(,(char->string #\a) ,(string-append (char->string #\λ) (char->string #\space))
  ,(char-whitespace? #\space) ,(char-whitespace? #\newline) ,(char-whitespace? #\tab) ,(char-whitespace? #\x3000) ,(char-whitespace? #\a)
  ,(char-numeric? #\7) ,(char-numeric? #\x663) ,(char-numeric? #\x) ,(char-numeric? #\½)
  ,(churn 500))
