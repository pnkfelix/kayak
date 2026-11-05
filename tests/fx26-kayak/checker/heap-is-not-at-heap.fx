;;; The heap's place, written `heap`, and the region constant `@heap` are
;;; two regions (C348; Fixpt's `Region::Heap` and `Region::Const`): a list
;;; at one is not a list at the other, which Fixpt refuses and kayak did not
;;; (C171 had read `heap` as `@heap`).
(define f (subr pure ((listof int heap)) (listof int @heap)) (lambda (l) l))
(f (list 1 2))
