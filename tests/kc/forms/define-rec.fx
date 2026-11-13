;; => (#t #t #f)
(define-rec
  (even? (subr (maxeff spin (read (globals even? odd?))) (int) bool) (lambda (n) (if (zero? n) #t (odd? (- n 1)))))
  (odd? (subr (maxeff spin (read (globals even? odd?))) (int) bool) (lambda (n) (if (zero? n) #f (even? (- n 1))))))
(list (even? 10) (odd? 7) (even? 100001))
