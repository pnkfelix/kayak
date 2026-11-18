;; => ! call-with-composable-continuation: no prompt for this tag
;; A capture no prompt of its tag delimits (C459): a failure, past a
;; prompt of another tag.
(define t (prompt-tag int int (read @globals) @p) (make-continuation-prompt-tag))
(define u (prompt-tag int int (read @globals) @p) (make-continuation-prompt-tag))
(prompt u
  (+ 1 (call-with-composable-continuation
         (lambda ((k (composable int int (read @globals) @p))) (k 1))
         t))
  (lambda ((v int)) v))
