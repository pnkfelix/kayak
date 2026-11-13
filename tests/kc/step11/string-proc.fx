;; => "hello, kc"
(define greet (subr pure (string) string) (lambda (n) (string-append "hello, " n)))
(greet "kc")
