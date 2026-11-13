;; => (#t #t)
(letrec ((ev (subr spin (int) bool) (lambda (n) (if (zero? n) #t (od (- n 1)))))
         (od (subr spin (int) bool) (lambda (n) (if (zero? n) #f (ev (- n 1))))))
  (list (ev 1000000) (od 7)))
