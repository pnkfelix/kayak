;; => (#<sum empty> #<sum circle>)
(define-datatype shape (circle int) (square int int) (empty))
(list (empty) (circle 1))
