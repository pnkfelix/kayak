;; => (#(1 "two" three) #() #t #f #f #f (#(1 (0) "c") #(2 (1) "c") #(3 (2) "c")))
;; Vectors (C470), the expander's: `datum-list->vector`, Fixpt's layout
;; (kind 3, the elements, a trailer; none, a header alone); `f64?` of
;; what is not one. Vectors made in a loop, past a heap of 64 words, each
;; kept and its elements moved with it.
(define* made (subr (maxeff spin (read (globals made))) (int datum) datum)
  (lambda (k acc)
    (if (zero? k)
        acc
        (made (- k 1) (cons (datum-list->vector (list k (list (- k 1)) "c")) (if (< k 3) acc nil))))))
`(,(datum-list->vector '(1 "two" three)) ,(datum-list->vector '()) ,(vector? (datum-list->vector '(1)))
  ,(vector? '(1)) ,(f64? 1) ,(f64? "x")
  ,(made 1000 nil))
