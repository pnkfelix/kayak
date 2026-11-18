;; => (3 42 1052 9 105 8)
;; Delimited control in kc (C458): prompts and aborts, as Fixpt's native
;; convention has them. A body's value, its handler not called; an abort
;; from deep in calls, its handler given the value; the innermost prompt
;; of a tag; an abort past another tag's prompt; an abort from a closure;
;; and a prompt in a loop, each round aborted, the stack as it was.
(define t (prompt-tag int int (maxeff spin (read (globals t u deep))) @p) (make-continuation-prompt-tag))
(define u (prompt-tag int int (maxeff spin (read (globals t u deep))) @p) (make-continuation-prompt-tag))
(define* deep (subr (maxeff spin (goto @p) (read (globals t deep))) (int) int)
  (lambda (n)
    (if (zero? n)
        (begin (abort-current-continuation t 42) 0)
        (+ 1 (deep (- n 1))))))
(define plain int (prompt t (+ 1 2) (lambda ((v int)) (+ v 100))))
(define from-deep int (prompt t (deep 1000) (lambda ((v int)) v)))
(define innermost int
  (prompt t
    (+ 1000 (prompt t (deep 5) (lambda ((v int)) (+ v 10))))
    (lambda ((v int)) (- 0 v))))
(define past-u int
  (prompt t
    (prompt u (+ 1 (deep 3)) (lambda ((v int)) (- 0 v)))
    (lambda ((v int)) (- v 33))))
(define by-closure int
  (let ((f (lambda ((k int)) (begin (abort-current-continuation u (+ k 100)) 0))))
    (prompt u (+ 1 (f 5)) (lambda ((v int)) v))))
(define* rounds (subr (maxeff spin (goto @p) (read (globals t deep rounds))) (int int) int)
  (lambda (n acc)
    (if (zero? n)
        acc
        (rounds (- n 1) (+ acc (prompt t (deep 50) (lambda ((v int)) (- v 41))))))))
(list plain from-deep innermost past-u by-closure (- (rounds 2000 0) 1992))
