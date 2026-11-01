;;; `pselect` on programs run beside this one (C296; Heist's F6): an arm
;;; for any process's news, then one for a process's output and one for
;;; its end, the output all taken before the end is seen. Fixpt's value,
;;; and the engine's: "0 one\ntwo\nexited 4".
(define* drain (subr (maxeff (io process) nondet spin) (int string) string)
  (lambda (p acc)
    (pselect ((process-output p) text (drain p (string-append acc text)))
             ((process-exit p) how (tagcase how (exited c (string-append acc (string-append "exited " (int->string c))))
                                     (signalled s "signalled") (unknown u "unknown"))))))
(define* news (subr (maxeff (io process) nondet) () int)
  (lambda () (pselect ((process-news) q q))))
(define* run (subr (maxeff (io process) nondet spin) () string)
  (lambda ()
    (tagcase (process-start (list "/bin/sh" "-c" "echo one; sleep 0.2; echo two; exit 4") "")
      (started p (let ((first (news))) (string-append (int->string first) (string-append " " (drain p "")))))
      (failed e "failed"))))
(run)
