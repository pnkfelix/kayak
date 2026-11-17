;; => 42
;; kc's step 15 (C446): a procedural macro, explicit renaming. Its
;; transformer, FX-26, is run as the file is expanded, a form at a time:
;; each request it makes a query's input, answered once by kc's own
;; native code (C447; the engine's, C446).
;; `x` it binds is its own; the use's `x` is the use's.
(define-syntax twice
  (er-macro-transformer
    (lambda (form rename compare)
      (let ((e (car (cdr form))))
        `(,(rename 'let) ((,(rename 'x) ,e))
           (,(rename '+) ,(rename 'x) ,(rename 'x)))))))
(define x int 20)
(+ (twice (+ x 1)) 0)
