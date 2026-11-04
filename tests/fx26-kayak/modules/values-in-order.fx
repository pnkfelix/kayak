;;; Module values in Fixpt's layout (C328): a product, its values in its
;;; type's order. A module given where a module type orders its values
;;; otherwise, or names fewer, is reshaped to that type (`module-arg`): a
;;; module named, one made in place, one bound to a typed name; a type
;;; written `(define-type ab (moduleof …))` as well as a module's own. An
;;; `include` and a `with` find each value at its place.
(define-type ab (moduleof (val b int) (val a int)))
(define m (module (define a int 2) (define c int 4) (define b int 7)))
(define first-b (subr pure (ab) int) (lambda (x) (with x b)))
(define second-a (subr pure (ab) int) (lambda (x) (with x a)))
(define k ab (module (define z int 1) (define a int 0) (define b int 6)))
(define n (module (include m) (define d int 9)))
;; Each reshape in a definition of its own: one in a `list`
;; stops Fixpt's checker (docs/requests/fixpt-reshape-in-list.md).
(define x int (first-b m))
(define y int (second-a m))
(define z int (first-b (module (define z int 1) (define a int 0) (define b int 5))))
(define main (subr (maxeff (read (globals x y z first-b second-a k n))) () (listof int acyclic))
  (lambda ()
    (list x y z (first-b k) (second-a k) (with n (- c 1)) (with n d))))
(main)
