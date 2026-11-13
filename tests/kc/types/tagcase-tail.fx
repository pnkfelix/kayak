;; => 100000
(define-datatype nat-list (nil-l) (cons-l int nat-list))
(define len (subr (maxeff spin (read (globals len))) (nat-list int) int) (lambda (l acc) (tagcase l (nil-l () acc) (cons-l (h t) (len t (+ acc 1))))))
(define build (subr (maxeff spin (read (globals build cons-l))) (int nat-list) nat-list) (lambda (n acc) (if (zero? n) acc (build (- n 1) (cons-l n acc)))))
(len (build 100000 (nil-l)) 0)
