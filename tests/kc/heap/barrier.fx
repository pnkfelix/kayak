;; => (522000 1275 465)
;; kc's stores into the heap pass kayak-rt's write barrier in process
;; (C538, `kc_remember`): a box and an array made first, old after the
;; first collection, given young lists again and again; a minor
;; collection, which follows no old object, keeps them as the barrier
;; noted their slots.
(define* build (subr (maxeff spin (read (globals build))) (int (listof int acyclic)) (listof int acyclic))
  (lambda (n acc) (if (zero? n) acc (build (- n 1) (cons n acc)))))
(define* total (subr (maxeff spin (read (globals total))) ((listof int acyclic) int) int)
  (lambda (xs s) (if (null? xs) s (total (cdr xs) (+ s (car xs))))))
(define held (ref (listof int acyclic) @x) (new nil))
(define slots (arrayof (listof int acyclic) @x) (make-array 8 nil))
(define* churn (subr (maxeff spin (read @x) (write @x) (read (globals churn build total held slots))) (int int) int)
  (lambda (k s)
    (if (zero? k)
        s
        (begin
          (set held (build 50 nil))
          (array-set! slots (remainder k 8) (build 30 nil))
          (total (build 500 nil) 0)
          (churn (- k 1) (+ s (+ (total (get held) 0) (total (array-ref slots (remainder k 8)) 0))))))))
(list (churn 300 0) (total (get held) 0) (total (array-ref slots 5) 0))
