;; => 22
;; kc's step 15 (C446): a procedural macro by syntactic closures, what its
;; transformer gives closed in the macro's environment, the operand in the
;; use's: the use's `x` is not the template's.
(define-syntax twice
  (sc-macro-transformer
    (lambda (form env)
      (let ((e (make-syntactic-closure env '() (car (cdr form)))))
        `(let ((x ,e)) (+ x x))))))
(let ((x 10)) (twice (+ x 1)))
