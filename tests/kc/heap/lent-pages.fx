;; => (3000150000 1250025000)
;; kc's code in process makes its objects in kayak-rt's heap (C538): its
;; pages lent, each object's start noted as it gives them back, its frames
;; and globals roots; a list kept across collections (a global's) and
;; lists made and dropped, past a nursery's worth.
(define* build (subr (maxeff spin (read (globals build))) (int (listof int acyclic)) (listof int acyclic))
  (lambda (n acc) (if (zero? n) acc (build (- n 1) (cons n acc)))))
(define* total (subr (maxeff spin (read (globals total))) ((listof int acyclic) int) int)
  (lambda (xs s) (if (null? xs) s (total (cdr xs) (+ s (car xs))))))
(define keep (listof int acyclic) (build 50000 nil))
(define* churn (subr (maxeff spin (read (globals churn build total))) (int int) int)
  (lambda (k s) (if (zero? k) s (churn (- k 1) (+ s (total (build 20000 nil) 0))))))
(list (churn 15 0) (total keep 0))
