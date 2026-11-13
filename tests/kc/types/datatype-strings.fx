;; => ("rex barks" "tom meows")
(define-datatype pet (dog string) (cat string int))
(define say (subr pure (pet) string) (lambda (p) (tagcase p (dog (n) (string-append n " barks")) (cat (n k) (string-append n " meows")))))
(list (say (dog "rex")) (say (cat "tom" 9)))
