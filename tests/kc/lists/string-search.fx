;; => (0 3 -1 4 4 -1 0 3 -1)
;; `string-search` (C474), Fixpt's, which the expander's paths use: where a
;; string is first found, by code points, at or after a start (below 0, 0).
`(,(string-search "a::b" "a" 0) ,(string-search "λ::b" "b" 0) ,(string-search "abc" "x" 0)
  ,(string-search "a::b::c" "::" 2) ,(string-search "a::b::c" "::" 4) ,(string-search "ab" "abc" 0)
  ,(string-search "abc" "" -5) ,(string-search "abc" "" 3) ,(string-search "abc" "" 4))
