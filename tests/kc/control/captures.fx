;; => (4 43 303 14 -199 1)
;; Capturing continuations in kc (C459), as Fixpt's native convention
;; takes them: the frames up to the tag's prompt copied into the heap, put
;; back on the caller's to resume. Resumed twice; once only; from 100
;; frames deep, twice; a prompt inside the copy, its record put on the
;; current ones at each resume (one resume aborting to it); and frames
;; holding lists, collected while copied (with a heap of 64 words).
(define t (prompt-tag int int (maxeff spin (read @globals)) @p) (make-continuation-prompt-tag))
(define u (prompt-tag int int (maxeff spin (read @globals)) @p) (make-continuation-prompt-tag))
(define twice int
  (prompt t
    (+ 1 (call-with-composable-continuation
           (lambda ((k (composable int int (maxeff spin (read @globals)) @p))) (k (k 1)))
           t))
    (lambda ((v int)) v)))
(define once int
  (prompt t
    (+ 1 (call-with-oneshot-continuation
           (lambda ((k (oneshot int int (maxeff spin (read @globals)) @p))) (* 2 (k 20)))
           t))
    (lambda ((v int)) v)))
(define* deep (subr (maxeff spin (goto @p) (comefrom @p) (read @globals)) (int) int)
  (lambda (n)
    (if (zero? n)
        (call-with-composable-continuation
          (lambda ((k (composable int int (maxeff spin (read @globals)) @p))) (+ (k 1) (k 2)))
          t)
        (+ 1 (deep (- n 1))))))
(define deeply int (prompt t (deep 100) (lambda ((v int)) v)))
(define nested1 int
    (prompt t
      (prompt u
        (+ 1 (let ((x (call-with-composable-continuation
                        (lambda ((k (composable int int (maxeff spin (read @globals)) @p))) (+ (k 5) (k 6)))
                        t)))
               x))
        (lambda ((v int)) (- 0 v)))
      (lambda ((v int)) v)))
(define nested2 int
    (prompt t
      (prompt u
        (+ 1 (let ((x (call-with-composable-continuation
                        (lambda ((k (composable int int (maxeff spin (read @globals)) @p))) (k 200))
                        t)))
               (if (> x 100) (begin (abort-current-continuation u x) 0) x)))
        (lambda ((v int)) (- 0 v)))
      (lambda ((v int)) v)))
(define* build (subr (maxeff spin (goto @p) (comefrom @p) (read @globals)) (int (listof int acyclic)) int)
  (lambda (n acc)
    (if (zero? n)
        (call-with-composable-continuation
          (lambda ((k (composable int int (maxeff spin (read @globals)) @p))) (+ (k 0) (k 0)))
          t)
        (let ((more (cons n acc))) (+ (build (- n 1) more) (if (= (car more) 1) 1 0))))))
(define listed int (prompt t (build 300 nil) (lambda ((v int)) v)))
(list twice once deeply nested1 nested2 (- listed 2))
