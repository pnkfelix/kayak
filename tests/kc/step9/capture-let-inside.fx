;; => 13
(let ((a 5)) ((lambda ((x int)) (let ((y (* x 2))) (+ a y))) 4))
