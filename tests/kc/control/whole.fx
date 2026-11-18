;; => (6 6 208 6 413 7)
;; Whole continuations in kc (C460), as Fixpt's `native/whole-captures.fx`
;; takes them, but on one machine: thrown through frames, deep too, and
;; through a prompt; kept, then called again after `cwcc` has returned,
;; three times round, each resume replacing the frames there are then and
;; returning where the current run does.
(define-type k-int (subr (goto @k) (int) void))
(define-effect jumping
  (maxeff (goto @k) (comefrom @k) (read @globals) (read @g) (write @g) (alloc @g) spin))
(define saved (ref (listof k-int @g) @g) (new nil))
(define count (ref int @g) (new 0))
(define* throw (subr jumping (k-int int) int) (lambda (k v) (+ 1000 (k v))))
(define* via (subr jumping ((subr jumping () int)) int) (lambda (g) (+ 100 (g))))
(define* cap (subr jumping ((subr jumping (k-int) int)) int)
  (lambda (r) (+ 1 (cwcc (lambda ((k k-int)) (r k))))))
(define* deep (subr jumping (k-int int) int)
  (lambda (k d) (if (= d 0) (throw k 7) (+ 1 (deep k (- d 1))))))
(define* again (subr jumping (int) int) (lambda (v) (+ 1000 ((car (get saved)) v))))
(define* keeper (subr jumping (k-int) int) (lambda (k) (begin (set saved (list k)) 0)))
(define* keep (subr jumping () int)
  (lambda () (+ 10 (cwcc (lambda ((k k-int)) (keeper k))))))
(define* loop (subr jumping () int)
  (lambda ()
    (let ((v (keep)))
      (begin (set count (+ (get count) 1))
             (if (< v 13) (again (+ (- v 10) 1)) (+ v (* 100 (get count))))))))
(define u (prompt-tag int int jumping @p) (make-continuation-prompt-tag))
(list (cap (lambda (k) (via (lambda () (throw k 5)))))
      (cap (lambda (k) (via (lambda () (via (lambda () (throw k 5)))))))
      (+ 200 (cap (lambda (k) (deep k 200))))
      (cap (lambda (k) (prompt u (via (lambda () (throw k 5))) (lambda ((v int)) v))))
      (begin (set count 0) (loop))
      (+ 1 (cap (lambda (k) 5))))
