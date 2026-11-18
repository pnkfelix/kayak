;; => ! a one-shot continuation resumed twice
;; A one-shot continuation (C459) resumed twice: the second resume fails,
;; as on Fixpt's every machine, rather than running the rest again.
(define t (prompt-tag int int (read @globals) @p) (make-continuation-prompt-tag))
(prompt t
  (+ 1 (call-with-oneshot-continuation
         (lambda ((k (oneshot int int (read @globals) @p))) (k (k 20)))
         t))
  (lambda ((v int)) v))
