;; => ((3 2 1) () (1 2 3 4 5 9) #t () (1 2 3) 6 0 "42" "-7" "0" "1152921504606846975" 42 100 2 #t #f #t #t #f "n42" 48893)
;; Lists and numbers (C462), as Fixpt's procedures: `reverse`; `append` of
;; no lists, one, two and more, its last shared; `length`; `int->string`
;; of each sign and the largest fixnum; an `i64` and its operations, the
;; expander's way, `int->i64` and back; and each made over and over past
;; a heap of 64 words.
(define xs (listof int acyclic) (list 1 2 3))
(define last (listof int acyclic) (list 9))
(define joined (listof int acyclic) (append xs (list 4 5) nil last))
(define* upto (subr (maxeff spin (read (globals upto))) (int (listof int acyclic)) (listof int acyclic))
  (lambda (n acc) (if (zero? n) acc (upto (- n 1) (cons n acc)))))
(define* churn (subr (maxeff spin (read (globals churn upto))) (int int) int)
  (lambda (k acc)
    (if (zero? k)
        acc
        (let* ((l (upto 20 nil))
               (r (reverse (append l (list k))))
               (s (int->string (car r))))
          (churn (- k 1) (+ acc (+ (length r) (string-length s))))))))
(define h i64 (i64+ (int->i64 40) (int->i64 2)))
(define step (subr pure (i64 i64) i64)
  (lambda (a b) (i64-quotient (i64* (i64- a b) 10) 3)))
`(,(reverse xs) ,(reverse (the (listof int acyclic) nil)) ,joined
  ,(eq? (cdr (cdr (cdr (cdr (cdr joined))))) last)
  ,(append) ,(append xs) ,(length joined) ,(length (the (listof int acyclic) nil))
  ,(int->string 42) ,(int->string -7) ,(int->string 0) ,(int->string 1152921504606846975)
  ,(i64->int h) ,(i64->int (step h 12)) ,(i64->int (i64-remainder h 5))
  ,(i64= h 42) ,(i64< h 42) ,(i64<= h 42) ,(i64> h 41) ,(i64>= h 43)
  ,(string-append "n" (int->string (i64->int h)))
  ,(churn 2000 0))
