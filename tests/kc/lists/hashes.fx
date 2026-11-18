;; => (918498814959166002 1040888010961589620 1151149506240833654 1040888010961589620 513307110889576267 571206600946375502 1038768129606968786 860922984064492325)
;; Hashes (C470), the expander's, Fixpt's values: `string-hash`, FNV-1a of
;; the code points, of the contents (a string made at run time hashes as
;; its literal does); `symbol-name-hash`, of the name's bytes, the one kept
;; with the symbol, a symbol made at run time's too (the empty name by
;; `string->symbol`: kc's reader has no `|…|` yet, TODO §54).
`(,(string-hash "") ,(string-hash "abc") ,(string-hash "λx") ,(string-hash (string-append "ab" "c"))
  ,(symbol-name-hash 'abc) ,(symbol-name-hash 'expr) ,(symbol-name-hash (string->symbol "λx")) ,(symbol-name-hash (string->symbol "")))
