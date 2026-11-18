;; => (1 2 3 2 1 10 4 0 7 5)
;; Continuation marks in kc (C460), as Fixpt's native convention has
;; them, a record on the prompts' chain: a mark in tail position replaces
;; one of its key the return is to, so a loop that marks each round runs
;; in constant space and sees only the latest (as Fixpt's
;; `run/tail-marks.fx`); marks not in tail position, each kept; the
;; innermost, or the default; a mark under an abort, gone with its
;; frames; and the marks a continuation took (`run/marks-of.fx`).
(define-effect marking (maxeff (write @m) (read @m) (alloc @m) (read (globals key t))))
(define key (mark-key int @m) (make-continuation-mark-key))
(define t (prompt-tag (listof int acyclic) int marking @p) (make-continuation-prompt-tag))
(define* loop (subr (maxeff (read @m) (write @m) (alloc @m) spin) (int) (listof int @m))
  (lambda (n) (if (= n 0) (current-marks key) (with-mark key n (lambda () (loop (- n 1)))))))
(define* nested (subr (maxeff (read @m) (write @m) (alloc @m) spin) () (listof int @m))
  (lambda () (with-mark key 1 (lambda () (with-mark key 2 (lambda () (current-marks key)))))))
(define* kept (subr (maxeff (read @m) (write @m) (alloc @m) spin) () (listof int @m))
  (lambda ()
    (with-mark key 1 (lambda () (cons 0 (with-mark key 2 (lambda () (cons 0 (with-mark key 3 (lambda () (current-marks key)))))))))))
(define* strip (subr (maxeff (read @m) spin) ((listof int @m)) (listof int @m))
  (lambda (xs) (if (null? xs) xs (if (= (car xs) 0) (strip (cdr xs)) xs))))
(define outside int (first-mark key 10))
(define inside int (with-mark key 4 (lambda () (+ 0 (first-mark key 10)))))
(define u (prompt-tag int int (maxeff (read @m) (write @m) (alloc @m) (read (globals key u))) @p) (make-continuation-prompt-tag))
(define after-abort int
  (prompt u
    (with-mark key 9 (lambda () (+ 1 (begin (abort-current-continuation u 0) 0))))
    (lambda ((v int)) (+ v (first-mark key 0)))))
(define taken (listof int acyclic)
  (prompt t
    (with-mark key 7
      (lambda () (call-with-composable-continuation (lambda ((k (composable (listof int acyclic) (listof int acyclic) marking @p))) (marks-of k key)) t)))
    (lambda ((v int)) nil)))
(define* total (subr (maxeff (read @m) spin) ((listof int @m) int) int)
  (lambda (xs n) (if (null? xs) n (total (cdr xs) (+ n (car xs))))))
(define* onto (subr (maxeff (read @m) (alloc @m) spin) ((listof int @m) (listof int @m)) (listof int @m))
  (lambda (xs ys) (if (null? xs) ys (cons (car xs) (onto (cdr xs) ys)))))
(the (listof int @m)
  (cons (car (loop 20000))
    (onto (nested)
      (onto (strip (kept))
        (list outside inside after-abort (car taken) (total (loop 5) 4))))))
