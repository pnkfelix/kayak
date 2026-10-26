;;; Quoted data interned (C231; Fixpt's `datum/quote-identity.fx`), and
;;; kept one object across collections: the table's entries are weak
;;; pairs, which a collection moves on with what they hold, rehashed as
;;; their parts' places change. Equal quotes, made at run time (a string, a
;;; vector in them) or when compiled (all literals), one object before and
;;; after many collections; a bit for each.
(define* f (subr pure () datum) (lambda () '(1 (a b) 2 "s")))
(define* g (subr pure () datum) (lambda () '#(1 "t" (c))))
(define* h (subr pure () datum) (lambda () '(x (y z))))
(define* churn (subr (maxeff (alloc @heap) spin) (int (listof int @heap)) int)
  (lambda (n acc) (if (= n 0) 0 (churn (- n 1) (if (= (modulo n 100) 0) nil (cons n acc))))))
(define* bit (subr pure (bool int) int) (lambda (b n) (if b n 0)))
(define f0 datum (f))
(define g0 datum (g))
(define h0 datum (h))
(define z int (churn 300000 nil))
(+ (bit (eq? f0 (f)) 1)
   (+ (bit (eq? g0 (g)) 2)
      (+ (bit (eq? h0 (h)) 4)
         (+ (bit (eq? (f) '(1 (a b) 2 "s")) 8)
            (+ (bit (eq? (h) '(x (y z))) 16) (bit (eq? (g) '#(1 "t" (c))) 32))))))
