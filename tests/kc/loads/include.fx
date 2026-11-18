;; => (9 22 3)
;; A file of a place that includes a load (C471), as the expander's file
;; includes its types file: the included file's datatype and procedures
;; its own, each instance's total its own.
(define summed (subr (maxeff (alloc @w) (read @globals) spin) ((listof int acyclic)) int)
  (lambda (ws)
    (letrena q
      (let ((x ((proj (load-module "files/area-place.fx") q))))
        (letrec ((go (subr (maxeff (read q) (write q) spin) ((listof int acyclic) int) int)
                     (lambda (ws acc) (if (null? ws) acc (go (cdr ws) (with x (add-rect (car ws) 2)))))))
          (go ws 0))))))
`(,(summed (list 3)) ,(summed (list 5 3)) ,(summed (list 0)))
