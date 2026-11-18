;; => (1 "two" #\3 (4 5) 7 "seven" 12 (9 . "nine") 300000 20000)
;; Tuples (C461): `(tuple E ...)`, Fixpt's `(product (1 E) ...)`, its
;; fields read by `extract`, from 1. Fields of each kind; a tuple in a
;; tuple; one made by a procedure and taken apart by its caller; tuples
;; in a list; a datatype's payload a tuple; the expander's way, a datum,
;; a string and an int; and tuples made and dropped past a heap of 64
;; words.
(define t (tupleof int string char) (tuple 1 "two" #\3))
(define nest (tupleof (tupleof int int) bool) (tuple (tuple 4 5) #t))
(define swap (subr pure ((tupleof int string)) (tupleof string int))
  (lambda (p) (tuple (extract p 2) (extract p 1))))
(define s (tupleof string int) (swap (tuple 7 "seven")))
(define* total (subr (maxeff spin (read (globals total))) ((listof (tupleof int int) acyclic) int) int)
  (lambda (ps n) (if (null? ps) n (total (cdr ps) (+ n (* (extract (car ps) 1) (extract (car ps) 2)))))))
(define-datatype entry (named (tupleof int string)) (none))
(define e entry (named (tuple 9 "nine")))
(define shown (subr pure (entry) datum)
  (lambda (x) (tagcase x (named (p) `(,(extract p 1) . ,(extract p 2))) (none () (the datum nil)))))
(define drive (subr pure (datum int) (tupleof datum string int))
  (lambda (d n) (tuple d "drove" n)))
(define* churn (subr (maxeff spin (read (globals churn))) (int int) int)
  (lambda (k acc)
    (if (zero? k) acc (let ((p (tuple k (tuple acc 1)))) (churn (- k 1) (+ (extract (extract p 2) 1) (extract (extract p 2) 2)))))))
`(,(extract t 1) ,(extract t 2) ,(extract t 3)
  (,(extract (extract nest 1) 1) ,(extract (extract nest 1) 2))
  ,(extract s 2) ,(extract s 1)
  ,(total (list (tuple 1 2) (tuple 2 5)) 0)
  ,(shown e)
  ,(extract (drive `x 300000) 3)
  ,(churn 20000 0))
