;; => (6540 2007)
;; Arguments past the sixth (C464) under capture: a continuation taken
;; inside a procedure of eight parameters, deep in calls of eight, each
;; frame's seventh and eighth read after it is resumed, twice; and the
;; continuation itself called by a lambda of seven.
(define t (prompt-tag int int (maxeff spin (read @globals)) @p) (make-continuation-prompt-tag))
(define* deep8 (subr (maxeff spin (goto @p) (comefrom @p) (read @globals)) (int int int int int int int int) int)
  (lambda (n a b c d e g h)
    (if (zero? n)
        (+ (* 100 g) (+ h (call-with-composable-continuation
                            (lambda ((k (composable int int (maxeff spin (read @globals)) @p))) (+ (k 1) (k 2)))
                            t)))
        (+ g (deep8 (- n 1) a b c d e (+ g 1) (* 2 h))))))
(define captured int (prompt t (deep8 10 0 0 0 0 0 1 1) (lambda ((v int)) v)))
(define called7 int
  (prompt t
    (+ 1000 (call-with-composable-continuation
              (lambda ((k (composable int int (maxeff spin (read @globals)) @p)))
                (let ((via7 (lambda ((k2 (composable int int (maxeff spin (read @globals)) @p)) (a int) (b int) (c int) (d int) (e int) (g int))
                              (k2 (+ a g)))))
                  (via7 k 1 2 3 4 5 6)))
              t))
    (lambda ((v int)) v)))
(list captured called7)
