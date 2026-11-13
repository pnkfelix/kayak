;; => 6
(define count-a (subr (maxeff spin (read (globals count-a))) (string int int) int) (lambda (s i n) (if (= i (string-length s)) n (count-a s (+ i 1) (if (char=? (string-ref s i) #\a) (+ n 1) n)))))
(count-a "banana bandana" 0 0)
