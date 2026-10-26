;;; Adapters both ways (C230; Fixpt's `native/adapters.fx`, without its
;;; procedure defined twice, which tier 1's reader of listings refuses): a
;;; procedure given where one of the other convention is expected,
;;; converted by an adapter that calls it; an adapter converted back, an
;;; adapter of the adapter; each called by every convention; standard
;;; operations as values. One answer, so that each machine's is compared.
(define inc (subr pure (int) int) (lambda (x) (+ x 1)))
(define n (subr (conv native) pure (int) int) inc)
(define c (subr (conv cellular) pure (int) int) inc)
(define call-n (subr pure ((subr (conv native) pure (int) int)) int) (lambda (f) (f 1)))
(define call-c (subr pure ((subr (conv cellular) pure (int) int)) int) (lambda (f) (f 1)))
(define twice (subr pure ((subr (conv fx) pure (int) int) int) int) (lambda (f x) (f (f x))))
(define m (subr pure ((subr pure (int int) int) int) int) (lambda (k x) (k x 3)))
(define back (subr (conv cellular) pure (int) int) (convention cellular n))
(list (call-n n) (call-n inc) (call-c c) (call-c inc) ((convention native inc) 5)
      ((convention cellular n) 6) (back 7) (call-c back) (call-n (convention native back))
      (twice n 10) (twice c 20) (m * 5))
