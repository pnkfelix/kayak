;; => "hello, modules"
(define greet (module (define hello string "hello") (define say (subr pure (string) string) (lambda (n) (string-append hello (string-append ", " n))))))
(with greet (say "modules"))
