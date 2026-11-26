;; => 7
;; Fixpt's tests/fx26/programs/processes/race.fx, as kc's tasks run it (C544).
;; `race` of two waits: the first I-cell is filled, by a process, and the
;; second never is; so the first branch wins, and the second, waiting
;; still, is cancelled.
(define a (icell int @c) (make-icell))
(define b (icell int @c) (make-icell))
(define w (addr int @p) (spawn (box int r) (icell-put! a (receive box))))
(send w 7)
(race (icell-get a) (icell-get b))
