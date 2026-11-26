;; => 42
;; Fixpt's tests/fx26/programs/processes/wait-masked-spin.fx, as kc's tasks run it (C544).
;; A wait on an I-cell nothing outside sees, said: masking drops the
;; `(await @q)` and leaves `spin` (TODO §224).
(define ask-self (subr spin (int) int)
  (lambda (n)
    (let ((c (the (icell int @q) (make-icell))))
      (begin
        (spawn (box int p) (icell-put! c (* 2 n)))
        (icell-get c)))))
(ask-self 21)
