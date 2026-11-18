;; => ("" "abc" #f "abcd" "1abc2abc3" 4764)
;; `string-append` of any count (C464), as Fixpt's `vsubr`: none, one,
;; two, five; one's a fresh string; and many made past a heap of 64 words.
(define s string "abc")
(define* churn (subr (maxeff spin (read (globals churn))) (int int) int)
  (lambda (k acc)
    (if (zero? k) acc (churn (- k 1) (+ acc (string-length (string-append "a" (int->string k) "-" "b" (int->string acc))))))))
`(,(string-append) ,(string-append s) ,(eq? (string-append s) s) ,(string-append s "d")
  ,(string-append "1" s "2" s "3") ,(churn 500 0))
