;; => (5549928 "kept!" (1 4 4 4 4 2))
;; What kcrt makes for kc's code (C455), a string appended, a substring,
;; a `,@`'s pairs, made in kc's heap and collected with it: their churn
;; past a heap of 64 words, what is kept still whole after.
(define keep string (string-append "kept" "!"))
(define four (listof datum acyclic) (list 4 4 4 4))
(define keepl datum `(1 ,@four 2))
(define spin-str (subr (maxeff spin (read (globals spin-str four))) (int int) int)
  (lambda (k acc)
    (if (zero? k)
        acc
        (let* ((s (string-append (substring "abcdefghijklmnopqrstuvwxyz" 0 (+ 1 (modulo k 26))) "-tail"))
               (l `(,@four ,@four 1)))
          (spin-str (- k 1) (+ acc (string-length s)))))))
`(,(spin-str 300000 0) ,keep ,keepl)
