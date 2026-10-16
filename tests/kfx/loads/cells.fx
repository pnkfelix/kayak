;; A family whose body is a bloblet, as Fixpt's `table` is, and a type
;; (C119).
(define-type (cell (k type) (r region)) (bloblet (fields k) r))
(define-type score i64)
