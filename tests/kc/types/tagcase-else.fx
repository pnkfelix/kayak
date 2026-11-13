;; => 7
(tagcase (the (sumof (a int) (b bool)) (sum b #t)) (a n (+ n 1)) (else s 7))
