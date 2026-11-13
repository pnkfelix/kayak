;; => ! division by zero
(let ((d 0)) ((lambda ((x int)) (quotient x d)) 9))
