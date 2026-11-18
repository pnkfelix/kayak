;; Loaded by tests/kc/loads/include.fx (C471): a file of a place that
;; includes a load of a file of none, as the expander's file includes its
;; types file; its running total in the place it is given.
(module-parameters ((p place)))
(include (load-module "shapes.fx"))
(define total (ref int p) (new 0))
(define add-rect (subr (maxeff (read p) (write p)) (int int) int)
  (lambda (w h) (begin (set total (+ (get total) (areas (list (rect w h) (circle 1))))) (get total))))
