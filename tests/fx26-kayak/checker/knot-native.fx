;;; A procedure kept in `@c` that reads `@c` could reach itself, and must say
;;; `spin`: of a convention written too (C255; Fixpt's `k-knot-in`).
(define-type hc (subr (conv native) (read @c) () int))
(define cell (ref (listof hc @c) @c) (new nil))
1
