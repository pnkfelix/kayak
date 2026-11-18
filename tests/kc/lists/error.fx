;; => ! bad: 3
;; `error` (C470): the run fails, saying the message, made at run time.
(define f (subr pure (int) int) (lambda (n) (begin (error (string-append "bad: " (int->string n))) n)))
(f 3)
