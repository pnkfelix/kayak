;; => 172
;; More procedural macros than kc keeps sealed (C456: 16): each its own
;; transformer's image (its definition's program sealed, run and freed);
;; the least recently used freed as the 17th and 18th are sealed, and
;; `m0`'s, used first, sealed again for its last use, freeing `m2`'s.
;; In process, 38 images sealed and 22 freed (kc run's own, after it).
(define-syntax m0 (er-macro-transformer (lambda (form rename compare) `(,(rename '+) ,(car (cdr form)) 0))))
(define-syntax m1 (er-macro-transformer (lambda (form rename compare) `(,(rename '+) ,(car (cdr form)) 1))))
(define-syntax m2 (er-macro-transformer (lambda (form rename compare) `(,(rename '+) ,(car (cdr form)) 2))))
(define-syntax m3 (er-macro-transformer (lambda (form rename compare) `(,(rename '+) ,(car (cdr form)) 3))))
(define-syntax m4 (er-macro-transformer (lambda (form rename compare) `(,(rename '+) ,(car (cdr form)) 4))))
(define-syntax m5 (er-macro-transformer (lambda (form rename compare) `(,(rename '+) ,(car (cdr form)) 5))))
(define-syntax m6 (er-macro-transformer (lambda (form rename compare) `(,(rename '+) ,(car (cdr form)) 6))))
(define-syntax m7 (er-macro-transformer (lambda (form rename compare) `(,(rename '+) ,(car (cdr form)) 7))))
(define-syntax m8 (er-macro-transformer (lambda (form rename compare) `(,(rename '+) ,(car (cdr form)) 8))))
(define-syntax m9 (er-macro-transformer (lambda (form rename compare) `(,(rename '+) ,(car (cdr form)) 9))))
(define-syntax m10 (er-macro-transformer (lambda (form rename compare) `(,(rename '+) ,(car (cdr form)) 10))))
(define-syntax m11 (er-macro-transformer (lambda (form rename compare) `(,(rename '+) ,(car (cdr form)) 11))))
(define-syntax m12 (er-macro-transformer (lambda (form rename compare) `(,(rename '+) ,(car (cdr form)) 12))))
(define-syntax m13 (er-macro-transformer (lambda (form rename compare) `(,(rename '+) ,(car (cdr form)) 13))))
(define-syntax m14 (er-macro-transformer (lambda (form rename compare) `(,(rename '+) ,(car (cdr form)) 14))))
(define-syntax m15 (er-macro-transformer (lambda (form rename compare) `(,(rename '+) ,(car (cdr form)) 15))))
(define-syntax m16 (er-macro-transformer (lambda (form rename compare) `(,(rename '+) ,(car (cdr form)) 16))))
(define-syntax m17 (er-macro-transformer (lambda (form rename compare) `(,(rename '+) ,(car (cdr form)) 17))))
(+ (m0 1) (+ (m1 1) (+ (m2 1) (+ (m3 1) (+ (m4 1) (+ (m5 1) (+ (m6 1) (+ (m7 1) (+ (m8 1) (+ (m9 1) (+ (m10 1) (+ (m11 1) (+ (m12 1) (+ (m13 1) (+ (m14 1) (+ (m15 1) (+ (m16 1) (+ (m17 1) (m0 1)))))))))))))))))))
