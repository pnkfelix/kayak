;; Types named by `select` under their own names, one in the other's
;; instance, as Fixpt's `check-holds-types.fx` names `table` and
;; `k-region` (C119: the emitter, writing `score` out, met the select that
;; defines it, wrote that `score` out, and so on, until its stack
;; overflowed).
(hide (define cells (load-module "cells.fx")))
(define-type cell (select cells cell))
(define-type score (select cells score))
(define-type counts (cell score @t))
(define bump (subr (maxeff (read @t) (write @t)) (counts) i64)
  (lambda (c) (begin (bloblet-set! c 0 (i64+ (bloblet-ref c 0) 1)) (bloblet-ref c 0))))
(define fresh (subr (alloc @t) () counts) (lambda () (the counts (make-bloblet 0 41))))
