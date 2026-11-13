;; => 4
(tagcase (the (sumof (a int) (b bool)) (sum a 3)) (a n (+ n 1)) (b v 0))
