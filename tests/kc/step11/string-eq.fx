;; => (#t #f #t #f)
(list (string=? "ab" "ab") (string=? "ab" "abc") (string<? "ab" "b") (string<? "b" "ab"))
