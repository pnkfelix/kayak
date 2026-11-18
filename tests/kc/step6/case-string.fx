;; => (1 2 0)
;; A `case` on strings compares by contents, as Fixpt's does (C469): kc
;; once compared them by `eq?`, so a string made at run time never matched.
(define f (subr pure (string) int) (lambda (s) (case s (("a" "b") 1) (("cd") 2) (else 0))))
`(,(f (string-append "" "a")) ,(f (string-append "c" "d")) ,(f "x"))
