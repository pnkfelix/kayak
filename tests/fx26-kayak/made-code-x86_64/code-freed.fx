;;; Code a program makes and drops, freed (kayak's, C229): twenty thousand
;;; adders, each a bloblet of 4096 bytes sealed into the code area, more
;;; than the area holds at once (tier 1's, 4194304 words, about eight
;;; thousand of them), each called once and dropped; so the area's code
;;; that nothing reaches must be freed, and its blocks used again, for the
;;; last to be made. One made first is kept and called last: code reached
;;; stays where it is, and stays whole. Where this machine runs no native
;;; code, or runs another machine's, the answers it would give.

;; Whether the code this makes runs here: native code, x86_64's.
(define here bool
  (and (native-code?) (string=? (car (the (listof string @heap) (native-convention)))
                                "(target x86_64)")))

;; The bytes `bs` from byte `at` of `b`'s suffix.
(define* put-all (subr (maxeff (write @heap) (read @heap) spin)
                      ((bloblet (fields) @heap) int (listof int @heap)) unit)
  (lambda (b at bs)
    (if (null? bs) #u (begin (bloblet-set-byte! b at (car bs)) (put-all b (+ at 1) (cdr bs))))))

(define-type adds (subr (conv native) pure (int int) int))

;; `lea rax, [rdi + rsi]; ret`, then zeros to 4096 bytes: two fixnums' sum.
(define* adder (subr (maxeff (alloc @heap) (write @heap) (read @heap) spin (io native)) () adds)
  (lambda ()
    (let ((b (the (bloblet (fields) @heap) (make-bloblet 4096))))
      (begin (put-all b 0 (list 72 141 4 55 195))
             (the adds (native-procedure (seal-code b)))))))

;; `n` adders made, each called once and dropped: the sum of what they gave.
(define* many (subr (maxeff (alloc @heap) (write @heap) (read @heap) spin (io native))
                    (int int) int)
  (lambda (n acc)
    (if (= n 0) acc (many (- n 1) (+ acc ((adder) n 1))))))

(define kept (listof adds @heap) (if here (list (adder)) nil))

;; The sum of n + 1 for n from 1 to 20000, and 42 by the adder kept.
(if here (+ (many 20000 0) ((car kept) 20 22)) 200030042)
