;; => ! abort: no prompt for this tag
;; An abort no prompt of its tag delimits (C458): a failure, in Fixpt's
;; native machines' words; here past a prompt of another tag, whose
;; record is passed over.
(define t (prompt-tag int int (maxeff spin) @p) (make-continuation-prompt-tag))
(define u (prompt-tag int int (maxeff spin (read (globals t))) @p) (make-continuation-prompt-tag))
(prompt u (+ 1 (abort-current-continuation t 5)) (lambda ((v int)) v))
