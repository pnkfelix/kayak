(module-parameters ((mxp place)))
"expander.fx: Fixpt's macro expander written in FX-26, its files made one
 KFX module file by scripts/macros-port.py (C429): edit that, not this.
 Its files are modules in Fixpt, their items no globals; here they are
 one file's items, so that naming one reads no global either, as there.
 The shim (fx26/macros-shim.kfx) first; then each file, in the order
 its loads need, the expander's whole, the parser's files' what it
 reaches; then the driver (fx26/macros-driver.kfx). A file of a place
 (C442, C443), Fixpt's five regions its one, each load an instance in
 the place given, its state its own: a process may load one, as a
 load sees the standard environment alone. Its types, but those that
 name the place, are expander-types.fx's, loaded once for all. Of
 one host's asker's effect (C449, `ASKS`), which answers a
 procedural macro's transformer's request: expander.fx the
 engine's, expander-kc.fx kc's.
 fixpt-source: crates/fixpt-fx26/src/eager-reader-types.fx@6f7bd73
 fixpt-source: crates/fixpt-fx26/src/syn.fx@6f7bd73
 fixpt-source: crates/fixpt-fx26/src/parser-types.fx@6f7bd73
 fixpt-source: crates/fixpt-fx26/src/parser-loads.fx@6f7bd73
 fixpt-source: crates/fixpt-fx26/src/parser.fx@6f7bd73
 fixpt-source: crates/fixpt-fx26/src/macros-kernel.fx@6f7bd73
 fixpt-source: crates/fixpt-fx26/src/macros-syntax.fx@6f7bd73
 fixpt-source: crates/fixpt-fx26/src/macros-types.fx@6f7bd73
 fixpt-source: crates/fixpt-fx26/src/macros-env-types.fx@6f7bd73
 fixpt-source: crates/fixpt-fx26/src/macros-env-indexed.fx@6f7bd73
 fixpt-source: crates/fixpt-fx26/src/macros-nodes.fx@6f7bd73
 fixpt-source: crates/fixpt-fx26/src/macros-align.fx@6f7bd73
 fixpt-source: crates/fixpt-fx26/src/macros-scopes.fx@6f7bd73
 fixpt-source: crates/fixpt-fx26/src/macros-match.fx@6f7bd73
 fixpt-source: crates/fixpt-fx26/src/macros-exports.fx@6f7bd73
 fixpt-source: crates/fixpt-fx26/src/macros-facts.fx@6f7bd73
 fixpt-source: crates/fixpt-fx26/src/macros-files.fx@6f7bd73
 fixpt-source: crates/fixpt-fx26/src/macros-infer.fx@6f7bd73
 fixpt-source: crates/fixpt-fx26/src/macros-show.fx@6f7bd73
 fixpt-source: crates/fixpt-fx26/src/macros-shown.fx@6f7bd73
 fixpt-source: crates/fixpt-fx26/src/macros-solve.fx@6f7bd73
 fixpt-source: crates/fixpt-fx26/src/macros-fresh.fx@6f7bd73
 fixpt-source: crates/fixpt-fx26/src/macros-contracts.fx@6f7bd73
 fixpt-source: crates/fixpt-fx26/src/macros-procedural.fx@6f7bd73
 fixpt-source: crates/fixpt-fx26/src/macros.fx@6f7bd73
 fixpt-source: crates/fixpt-fx26/src/macros-templates.fx@6f7bd73
 fixpt-source: crates/fixpt-fx26/src/macros-imports.fx@6f7bd73
 fixpt-source: crates/fixpt-fx26/src/macros-walk.fx@6f7bd73
 fixpt-source: crates/fixpt-fx26/src/macros-tree.fx@6f7bd73
 fixpt-source: crates/fixpt-fx26/src/macros-shaper.fx@6f7bd73
 fixpt-source: crates/fixpt-fx26/src/macros-shapes.fx@6f7bd73"
(include (load-module "expander-types.fx"))

"macros-shim.kfx: what Fixpt's expander, made KFX (fx26/macros/,
 scripts/macros-port.py, C429), uses of FX-26 that KFX has not, written
 by hand, and read into the expander's module first: keywords, which
 kfx's reader reads as names, `#:seal` a symbol whose name begins `#:`
 (C291); `datum-int?`; `eq?` of what the expander compares. Primitives
 alone, so that naming one reads no global."

(define mx-keyword-name? (subr pure (string) bool)
  (lambda (n)
    (and (> (string-length n) 2) (string=? (substring n 0 2) "#:") (not (char-numeric? (string-ref n 2))))))
(define mx-keyword? (subr pure (datum) bool)
  (lambda (d) (and (symbol? d) (mx-keyword-name? (symbol->string d)))))
(define mx-keyword->symbol (subr pure (datum) symbol)
  (lambda (d)
    (if (symbol? d)
        (let ((n (symbol->string d))) (string->symbol (substring n 2 (string-length n))))
        (string->symbol ""))))
(define mx-symbol->keyword (subr pure (symbol) symbol)
  (lambda (n) (string->symbol (string-append "#:" (symbol->string n)))))
(define mx-datum-int? (subr pure (datum) bool) (lambda (d) (int? d)))
"`eq?` of two booleans, as the expander compares a binder's namespace, an
 address's steps (KFX has no `eq?`: C75)."
(define mx-eq? (subr pure (bool bool) bool) (lambda (a b) (if a b (not b))))
"FX-26's `datum-dotted`: the list of `xs` ending in `tail`."
(define mx-datum-dotted (->* ((listof datum acyclic) datum) datum)
  (lambda (xs tail) (if (null? xs) tail (the datum (cons (car xs) (mx-datum-dotted (cdr xs) tail))))))
"FX-26's `parse-nat`, as the expander calls it, of radix 10 or less: the
 natural number `s`'s digits write, or -1 if they do not."
(define mx-parse-nat-from (->* (string int int int) int)
  (lambda (s radix i acc)
    (if (= i (string-length s))
        acc
        (let ((d (- (char->integer (string-ref s i)) 48)))
          (if (and (>= d 0) (< d radix)) (mx-parse-nat-from s radix (+ i 1) (+ (* acc radix) d)) -1)))))
(define mx-parse-nat (->* (string int) int)
  (lambda (s radix) (if (= (string-length s) 0) -1 (mx-parse-nat-from s radix 0 0))))
"The runtime's for procedural macros (Fixpt's `standard.rs`, `TODO.md`
 §228; C446, C449): a request of the expander's (`%forms`, `%define`,
 `%apply`, Fixpt's `procedural::asked`) answered as Fixpt's runtime
 answers it, by a call, and the expansion goes on: by the asker its host
 gives (`drive-asker!`), of its host's effect, which scripts/macros-port.py
 writes in (`ASKS`: expander.fx's, the engine's `--phase1`, a process;
 expander-kc.fx's, kc's `phase1` query, `qe`). A request's answer is of
 its text alone, as a transformer is deterministic, so each is kept, and
 one asked again is answered at once. An asker that answers `` (none
 given, or a REPL's draft, which runs nothing) is said not yet. None is in
 a phase-1 namespace, `apart`. And a limit of expansion, Fixpt's default,
 the environment not read (`syscall::macro_limit`'s `FIXPT_MACRO_*`)."
(define mx-phase1-given (ref datum mxp) (new (the datum nil)))
(define mx-phase1-asker (ref (subr (maxeff (read @globals) (io process) spin) (string) string) mxp) (new (lambda ((r string)) "")))
"Request `r` as it is asked and answered: of a syntactic-closure
 transformer's `%apply` (style 1 or 2), its expansion's tag `0`, which its
 transformer is not given (Fixpt's `call_transformer`), so its answer is
 the same for every tag, and one asked again on what it gave (a use it
 returns, expanded again) is answered at once."
(define mx-phase1-tag-end (->* (string int) int)
  (lambda (r i) (cond ((>= i (string-length r)) i) ((char=? (string-ref r i) #\space) i) (else (mx-phase1-tag-end r (+ i 1))))))
(define mx-phase1-key (->* (string) string)
  (lambda (r)
    (if (and (> (string-length r) 9) (or (string=? (substring r 0 9) "%apply 1 ") (string=? (substring r 0 9) "%apply 2 ")))
        (string-append (substring r 0 9) (string-append "0" (substring r (mx-phase1-tag-end r 9) (string-length r))))
        r)))
(define mx-phase1-in (->* (string datum) string)
  (lambda (request xs)
    (if (pair? xs)
        (let* ((x (car xs)) (k (if (pair? x) (car x) (the datum nil))) (v (if (pair? x) (cdr x) (the datum nil))))
          (if (string? k)
              (if (string? v)
                  (if (string=? k request) v (mx-phase1-in request (cdr xs)))
                  (mx-phase1-in request (cdr xs)))
              (mx-phase1-in request (cdr xs))))
        "")))
(define mx-expand-procedural (->* (string) string)
  (lambda (asked)
    (let* ((request (mx-phase1-key asked)) (a (mx-phase1-in request (get mx-phase1-given))))
      (if (not (string=? a ""))
          a
          (let ((b ((get mx-phase1-asker) request)))
            (if (string=? b "")
                "!a procedural macro's transformer, run as the program is read: not yet here"
                (begin
                  (set mx-phase1-given (the datum (cons (the datum (cons (the datum request) (the datum b))) (get mx-phase1-given))))
                  b)))))))
(define mx-call-procedural (subr spin (int string datum string (listof string acyclic)) (sumof (ok datum) (failed string) (apart unit)))
  (lambda (style code d x forms) (sum apart #u)))
(define mx-limit (subr spin (string) string)
  (lambda (name)
    (cond ((string=? name "USES") "=10000") ((string=? name "DEPTH") "=1000")
          ((string=? name "RECHECK") "=0") ((string=? name "CONTRACTS") "=1")
          (else (string-append "!no limit of macro expansion is named `" (string-append name "`"))))))
"FX-26's `parse-number`, as the expander's reading of a procedural
 transformer's answer calls it: an integer of radix 10 or less, signed or
 not, in a list of one; else none (a real, not yet: stage 5)."
(define mx-parse-number (->* (string int) (listof datum acyclic))
  (lambda (s radix)
    (let* ((neg (and (> (string-length s) 1) (char=? (string-ref s 0) #\-)))
           (digits (if (or neg (and (> (string-length s) 1) (char=? (string-ref s 0) #\+))) (substring s 1 (string-length s)) s))
           (n (mx-parse-nat digits radix)))
      (if (< n 0) nil (list (the datum (if neg (- 0 n) n)))))))
"FX-26's `datum-vector->list`: a vector's elements. Called only where
 `vector?` holds of a datum, which a procedural transformer's answer may
 be (stage 5); none, here."
(define mx-datum-vector->list (subr pure (datum) (listof datum acyclic)) (lambda (d) nil))
"A list of datums as one datum (a vector's, as the reader keeps it)."
(define mx-datum-list (->* ((listof datum acyclic)) datum)
  (lambda (xs) (mx-datum-dotted xs (the datum nil))))

;;; ============================================================ eager-reader-types.fx (the definitions the expander reaches)



;;; ============================================================ syn.fx (the definitions the expander reaches)

(define syn->datum (->* (syn) datum)
  (lambda (s)
    (tagcase s
      (atom (d a b xpn) d)
      (mx-eager-reader-types-lst (items d a b xpn) d)
      (dotted (items tail d a b xpn) d)
      (vec (items d a b xpn) d))))

;;; ============================================================ parser-types.fx (the definitions the expander reaches)

;; What a parse may do: read what was read and build a tree
;; (`tree-builds`), keep what the files it loads define, and give up.
(define-effect tree-builds (maxeff (read @globals) spin (read mxp) (alloc mxp)))
(define-effect parses (maxeff tree-builds (write mxp) (goto mxp)))

;;; ============================================================ parser-loads.fx (the definitions the expander reaches)

;; The positions of one file and the next apart.
(define load-base int 1000000000)
;; The file read at `base`, in a list of one.
(define loaded-based (->* (loaded-files int) loaded-files)
  (lambda (fs base)
    (cond ((null? fs) nil)
          ((and (> base 0) (= (extract (car fs) 2) base)) (the loaded-files (cons (car fs) nil)))
          (else (loaded-based (cdr fs) base)))))
;; Where position `at` of `text` is: `line:column`, each from 1.
(define text-place (->* (string int) string)
  (lambda (text at)
    (letrec ((go (subr (maxeff (read @globals) spin) (int int int) string)
               (lambda (i line col)
                 (cond ((or (>= i at) (>= i (string-length text)))
                        (string-append (int->string line) (string-append ":" (int->string col))))
                       ((char=? (string-ref text i) #\newline) (go (+ i 1) (+ line 1) 1))
                       (else (go (+ i 1) line (+ col 1)))))))
      (go 0 1 1))))
;; The same, the file named `path`, as the load being read wrote it: a
;; file read once may be loaded under more than one path (`TODO.md` §71).
(define in-loaded-as
  (->* (loaded-files int string string int) string)
  (lambda (fs base path m at)
    (let ((f (loaded-based fs base)))
      (if (or (null? f) (< at base) (>= at (+ base load-base)))
          m
          (string-append "in `" (string-append path (string-append "`, "
                         (string-append (text-place (extract (car f) 6) (- at base)) (string-append ": " m)))))))))

;;; ============================================================ parser.fx (the definitions the expander reaches)

(define parse-tag (prompt-tag presult presult (maxeff tree-builds (write mxp) spin (read @globals) (io process)) mxp)
  (make-continuation-prompt-tag))
;;; -------------------------------------------------------- looking at syn

(define syn-start (->* (syn) int)
  (lambda (s)
    (tagcase s
      (atom (d a b x) a) (mx-eager-reader-types-lst (i d a b x) a) (dotted (i t d a b x) a) (vec (i d a b x) a))))
(define syn-end (->* (syn) int)
  (lambda (s)
    (tagcase s
      (atom (d a b x) b) (mx-eager-reader-types-lst (i d a b x) b) (dotted (i t d a b x) b) (vec (i d a b x) b))))
;; The expansion that made `s`, 0 if none did.
(define syn-expansion (->* (syn) int)
  (lambda (s)
    (tagcase s
      (atom (d a b x) x) (mx-eager-reader-types-lst (i d a b x) x) (dotted (i t d a b x) x) (vec (i d a b x) x))))
(define pfail (->* (string syn) void)
  (lambda (message s)
    (abort-current-continuation parse-tag (p-err message (syn-start s) (syn-end s)))))
(define pfail-at (->* (string int int) void)
  (lambda (message a b) (abort-current-continuation parse-tag (p-err message a b))))
(define syn-symbol? (->* (syn) bool)
  (lambda (s) (tagcase s (atom (d a b xpn) (symbol? d)) (else x #f))))
(define syn-name (->* (syn) string)
  (lambda (s)
    (tagcase s
      (atom (d a b xpn) (if (symbol? d) (symbol->string d) ""))
      (else x ""))))
;; A form's head as a symbol, to compare with the keywords: any that is
;; not a name is taken as `()`, which is none of them.
(define syn-head (->* (syn) symbol)
  (lambda (s)
    (tagcase s
      (atom (d a b xpn) (if (symbol? d) d (string->symbol "()")))
      (else x (string->symbol "()")))))
(define syn-symbol (->* (syn) symbol)
  (lambda (s) (if (syn-symbol? s) (syn-head s) (pfail "a name" s))))
;; A proper list's items; `what`, when it is not one.
(define syn-items (->* (syn string) syns-a)
  (lambda (s what)
    (tagcase s
      (mx-eager-reader-types-lst (items d a b xpn) items)
      (else x (pfail (string-append what ": expected a list") s)))))
(define len (->* (syns-a) int)
  (lambda (xs) (if (null? xs) 0 (+ 1 (len (cdr xs))))))
(define nth (->* (syns-a int) syn)
  (lambda (xs i) (if (= i 0) (car xs) (nth (cdr xs) (- i 1)))))
(define drop (->* (syns-a int) syns-a)
  (lambda (xs i) (if (= i 0) xs (drop (cdr xs) (- i 1)))))
(define mk-symbol (subr pure (string int int) syn)
  (lambda (n a b) (atom (string->symbol n) a b 0)))
(define syn-datums (->* (syns-a) (listof datum acyclic))
  (lambda (xs) (if (null? xs) nil (cons (syn->datum (car xs)) (syn-datums (cdr xs))))))
(define mk-list (->* (syns-a int int) syn)
  (lambda (items a b) (mx-eager-reader-types-lst items (syn-datums items) a b 0)))
;; A form's head, as `syn-head` gives it: `()` if it is not a list, or is
;; empty.
(define form-head (->* (syn) symbol)
  (lambda (s)
    (tagcase s
      (mx-eager-reader-types-lst (items d a b xpn) (if (null? items) (string->symbol "()") (syn-head (car items))))
      (else x (string->symbol "()")))))
;; Whether `s` is a form headed `keyword`.
(define form-of? (->* (syn symbol) bool)
  (lambda (s keyword) (symbol=? (form-head s) keyword)))
;; Whether `s` is a string literal; and the string it is.
(define syn-string? (->* (syn) bool)
  (lambda (s) (tagcase s (atom (d a b xpn) (string? d)) (else x #f))))
(define syn-string (->* (syn) string)
  (lambda (s)
    (tagcase s (atom (d a b xpn) (if (string? d) d "")) (else x ""))))
;;; ------------------------------------------------------------ expressions

;;; FX-91's `(define-datatype name (tag type …) …)`, expanded as the Rust
;;; reader expands it (`top.rs`'s `expand_datatype`), as syntax, into the
;;; forms it stands for: a sum of products, each variant's members labelled
;;; from 1, `(define-type name (sumof (tag (productof (1 T) …)) …))`, and a
;;; constructor per tag, `(define tag (subr pure (T …) name) (lambda (%x1 …)
;;; (sum tag (product (1 %x1) …))))`. Where the form is: a program's, a
;;; module's, or a module file's, each parsing what it is made into.

(define datatype? (->* (syn) bool)
  (lambda (s) (form-of? s 'define-datatype)))
(define mk-int (->* (int int int) syn) (lambda (i a b) (atom i a b 0)))
;; `(keyword item …)`; `(subr pure (member …) result)`; and `(poly (binder
;; …) body)`: each spanning `a`..`b`.
(define mk-form (->* (string syns-a int int) syn)
  (lambda (keyword items a b) (mk-list (cons (mk-symbol keyword a b) items) a b)))
(define mk-pure-subr (->* (syns-a syn int int) syn)
  (lambda (mx-parser-types-members result a b)
    (let ((pure (mk-symbol "pure" a b)) (args (mk-list mx-parser-types-members a b)))
      (mk-form "subr" (list pure args result) a b))))
(define mk-poly (->* (syns-a syn int int) syn)
  (lambda (binders body a b)
    (mk-form "poly" (list (mk-list binders a b) body) a b)))
;; A constructor's parameters, `%x1 …`, from `i`.
(define mx-parser-dt-vars (->* (syns-a int int int) syns-a)
  (lambda (ms i a b)
    (if (null? ms)
        nil
        (let ((x (mk-symbol (string-append "%x" (int->string i)) a b)))
          (cons x (mx-parser-dt-vars (cdr ms) (+ i 1) a b))))))
;; `(1 m1) (2 m2) …`, from `i`.
(define dt-labelled (->* (syns-a int int int) syns-a)
  (lambda (ms i a b)
    (if (null? ms)
        nil
        (let* ((pair (mk-list (list (mk-int i a b) (car ms)) a b))
               (rest (dt-labelled (cdr ms) (+ i 1) a b)))
          (cons pair rest)))))
;; A variant's tag: a name, or a keyword, `#:circle`, which is its name (the
;; same label, and the constructor `circle`, as `top.rs` reads it).
(define dt-tag (->* (syn) syn)
  (lambda (t)
    (tagcase t
      (atom (d a b xpn) (if (mx-keyword? d) (atom (mx-keyword->symbol d) a b 0) t))
      (else x t))))
;; Each variant's arm of the sum, `(tag (productof (1 T) …))`.
(define dt-arms (->* (syns-a int int) syns-a)
  (lambda (vs a b)
    (if (null? vs)
        nil
        (let* ((v (car vs)) (parts (syn-items v "a variant")))
          (if (or (null? parts) (not (syn-symbol? (dt-tag (car parts)))))
              (pfail "a variant is `(tag type …)` or `(#:tag type …)`" v)
              (let* ((prod (mk-form "productof" (dt-labelled (cdr parts) 1 a b) a b))
                     (arm (mk-list (list (dt-tag (car parts)) prod) a b))
                     (rest (dt-arms (cdr vs) a b)))
                (cons arm rest)))))))
;; Each constructor: of type `(subr pure (member …) used)`, where `used` is
;; the type as it is used, `name` or `(name param …)`; polymorphic in the
;; parameters if there are any (`family?`).
(define mx-parser-dt-constructors (->* (syn bool syns-a syns-a int int) syns-a)
  (lambda (used family? params vs a b)
    (if (null? vs)
        nil
        (let* ((parts (syn-items (car vs) "a variant"))
               (tag (dt-tag (car parts)))
               (mx-parser-types-members (cdr parts))
               (mono (mk-pure-subr mx-parser-types-members used a b))
               (ty (if family? (mk-poly params mono a b) mono))
               (xs (mx-parser-dt-vars mx-parser-types-members 1 a b))
               (fields (mk-form "product" (dt-labelled xs 1 a b) a b))
               ;; At its variant: no two constructors' bodies at one place.
               (body (mk-form "sum" (list tag fields) (syn-start (car vs)) (syn-end (car vs))))
               (make (mk-form "lambda" (list (mk-list xs a b) body) a b))
               (ctor (mk-form "define" (list tag ty make) a b))
               (rest (mx-parser-dt-constructors used family? params (cdr vs) a b)))
          (cons ctor rest)))))
;; The parameters' names, from `(name kind) …`.
(define dt-param-names (->* (syns-a) syns-a)
  (lambda (ps)
    (if (null? ps)
        nil
        (let ((p (syn-items (car ps) "a parameter")))
          (if (and (= (len p) 2) (syn-symbol? (car p)))
              (cons (car p) (dt-param-names (cdr ps)))
              (pfail "a parameter is `(name kind)`" (car ps)))))))
;; The type forms, as `parse.rs`'s `TYPE_FORMS`: no variant's tag (`TODO.md`
;; §190), as `(define-datatype var (oneof symbol pat))` would be one variant
;; `oneof`, not the type.
(define dt-type-forms (listof symbol acyclic)
  (list 'arrayof 'narrayof 'bloblet 'composable 'dletrec 'icell 'listof 'mark-key 'moduleof 'mu
        'nat 'nlist 'oneshot 'pairof 'place 'poly 'productof 'prompt-tag 'proves 'ref 'select 'subr
        'sumof 'tupleof 'bool 'oneof 'union 'anyof 'allof '-> '->* 'restrict 'modnames))
(define dt-in? (->* (symbol (listof symbol acyclic)) bool)
  (lambda (x xs) (and (not (null? xs)) (or (symbol=? x (car xs)) (dt-in? x (cdr xs))))))
;; #t, once no variant of `vs` has a type form's name for its tag.
(define dt-no-type-forms (->* (syns-a syn) bool)
  (lambda (vs name)
    (if (null? vs)
        #t
        (let* ((parts (syn-items (car vs) "a variant"))
               (tag (if (null? parts) (car vs) (dt-tag (car parts)))))
          (if (and (syn-symbol? tag) (dt-in? (syn-symbol tag) dt-type-forms))
              (let ((t (syn-name tag)))
                (pfail (string-append "`" (string-append t (string-append "` is a type form: this would be a variant named `" (string-append t
                                      (string-append "`, not the type; to name a type, write `(define-type "
                                      (string-append (syn-name name) (string-append " (" (string-append t " …))`"))))))))
                       (car vs)))
              (dt-no-type-forms (cdr vs) name))))))
;; `(define-datatype name (tag type …) …)`, or with parameters,
;; `(define-datatype (name (param kind) …) …)`, a type family, which its
;; variants may mention with the same parameters: the forms it stands for.
(define expand-datatype (->* (syn) syns-a)
  (lambda (s)
    (let* ((items (syn-items s "a datatype")) (a (syn-start s)) (b (syn-end s))
           (usage "`(define-datatype name (tag type …) …)`"))
      (if (< (len items) 3)
          (pfail usage s)
          (let* ((head (nth items 1))
                 (family? (not (syn-symbol? head)))
                 (hs (if family? (syn-items head "a datatype's name") (the syns-a nil)))
                 (name (cond ((not family?) head)
                             ((and (not (null? hs)) (syn-symbol? (car hs))) (car hs))
                             (else (pfail usage s))))
                 (params (if family? (cdr hs) (the syns-a nil)))
                 (used (if family? (mk-list (cons name (dt-param-names params)) a b) name))
                 (named (dt-no-type-forms (drop items 2) name))
                 (sum (mk-form "sumof" (dt-arms (drop items 2) a b) a b))
                 (ctors (mx-parser-dt-constructors used family? params (drop items 2) a b)))
            (cons (mk-form "define-type" (list head sum) a b) ctors))))))

;;; ============================================================ macros-kernel.fx (whole)

;;; The macros' kernel (`macros.rs`, `macros.fx`): the forms a template may
;;; use, with their binding structure, each form's signature of its
;;; operands in Herman's notation (`define` names an abbreviation, `form` a
;;; form, `definition` a definition's form); the words that are forms'
;;; names, which no macro, binder or variable may be; and the words of
;;; descriptions, which a template's descriptions write as they are. Data,
;;; read by both front ends' expanders.

(define kernel-forms datum
  (quote
   (
    (define exprs (mu es (union () (expr . es))))
    ;; As named: (union nil (exports (pairof (named x bvar) (named xs formals)) (rib x xs)) …);
    ;; each parameter in scope in the types of those after it, as the
    ;; checkers bind them (`Checker::synth_lambda_as`'s `(select m t)` of an
    ;; earlier one).
    (define formals (mu f (union (export () (rho))
                                 (export (bvar . (import f (rho (A var)))) (rho (A var) (D rib)))
                                 (export ((bvar desc) . (import f (rho (AA var))))
                                         (rho (AA var) (D rib))))))
    (define let-bindings (mu b (union (export () (rho))
                                      (export ((bvar expr) . b) (rho (AA var) (D rib))))))
    ;; Herman's let*-cl: each clause's expression in the scope of those
    ;; before; the whole a ribcage, innermost first.
    (define let*-clauses (mu c (union (export () (cage))
                                      (export ((bvar expr) . (import c (rho (AA var))))
                                              (cage (at D) (rho (AA var)))))))
    ;; Each bound to a lambda, as the checkers take one.
    (define letrec-bindings (mu b (union (export () (rho))
                                         (export ((bvar desc lambda) . b) (rho (AA var) (D rib))))))
    ;; A `define-rec`'s, each lambda in the scope of its type's `poly`s'
    ;; binders, as a definition's is (`Checker::bind_signature`).
    (define defrec-bindings
      (mu b (union (export () (rho))
                   (export ((bvar desc (import lambda (rho (ADA rib)))) . b)
                           (rho (AA var) (D rib))))))
    ;; A description's binders, `((name kind) …)`, a third the binder's place
    ;; or `distinct`; each in the scope of those before it.
    (define dbinders (mu b (union (export () (rho))
                                  (export ((dvar kind) . b) (rho (AA dvar) (D rib)))
                                  (export ((dvar kind desc) . b) (rho (AA dvar) (D rib))))))
    (define descs (mu ds (union () (desc . ds))))
    ;; A `tagcase` arm, `(#:tag x body …)` or `(else x body …)`: `x` in scope in
    ;; the body.
    (define arms (mu a (union ()
                              ((label bvar . (import exprs (rho (ADA var)))) . a)
                              (((literal else) bvar . (import exprs (rho (ADA var)))) . a))))
    (define cond-clauses (mu c (union ()
                                      ((expr . exprs) . c)
                                      (((literal else) . exprs) . c))))
    (define fields (mu p (union () ((label expr) . p))))
    (form lambda (formals . (import exprs (rho (A rib)))))
    (form let (let-bindings . (import exprs (rho (A rib)))))
    (form let* (let*-clauses . (import exprs (cage (at A)))))
    (form letrec ((import letrec-bindings (rho (A rib))) . (import exprs (rho (A rib)))))
    (form if (expr expr expr))
    (form begin exprs)
    (form and exprs)
    (form or exprs)
    (form quote (data))
    (form the (desc expr))
    (form sum (label expr))
    (form extract (expr label))
    (form product fields)
    (form tuple exprs)
    (form tagcase (expr . arms))
    (form cond cond-clauses)
    (form plambda (dbinders . (import exprs (rho (A rib)))))
    (form proj (expr . descs))
    ;; The forms that bind a region (`letregion`, `letfreeze`, the latter
    ;; into a place if `(r p)`) or a place, which is also a value of type
    ;; `(place r)` (`letrena`, `letreap`), in their bodies; a lambda whose
    ;; closure is made in a place.
    (form letregion (dvar . (import exprs (rho (A dvar)))))
    (form letfreeze (union (dvar . (import exprs (rho (A dvar))))
                           ((dvar desc) . (import exprs (rho (AA dvar))))))
    (form letrena (dvar . (import exprs (rho (A place)))))
    (form letreap (dvar . (import exprs (rho (A place)))))
    (form rlambda (expr formals . (import exprs (rho (AD rib)))))
    (form %app (expr . exprs))
    ;; A keyword's construction, `(#:tag e …)`, its tag written or a
    ;; pattern variable's (a `label`).
    (form %construct (label . exprs))
    ;; The definitions a template may make, a macro's whose signature says
    ;; `(macro σ defn)`, each exporting what it defines: a value, seeing those
    ;; before it, `(define x T lambda)` and `define*` itself, a `define-rec`
    ;; group the whole group; a `begin`'s, each in the scope of those before
    ;; it. The address `()` is the part the annotation is on. A definition's
    ;; type's `poly`s' binders are in scope in what it is defined as.
    (define defns (mu d (union (export () (cage))
                               (export (defn . (import d (cage (at A)))) (cage (at D) (at A))))))
    (definition define (union (export (bvar expr) (rho (A var)))
                              (export (bvar desc (import expr (rho (A var) (AD rib))))
                                      (rho (A var)))))
    (definition define* (export (bvar desc (import lambda (rho (A var) (AD rib)))) (rho (A var))))
    (definition define-rec (import defrec-bindings (rho (() rib))))
    ;; A type, its name in scope in itself (types are declared ahead), a
    ;; family's parameters too; an effect.
    (definition define-type
      (union (export (dvar (import desc (rho (A dvar)))) (rho (A dvar)))
             (export ((dvar . dbinders) (import desc (rho (AA dvar) (DA rib)))) (rho (AA dvar)))))
    (definition define-effect (export (dvar desc) (rho (A dvar))))
    ;; A generative type, its conversions named, `#:seal mk #:unseal get`,
    ;; values each (`docs/research/generative-types.md` §7).
    (definition define-generative
      (union (export (dvar (import desc (rho (A dvar))) label bvar label bvar)
                     (rho (A dvar) (ADDD var) (ADDDDD var)))
             (export ((dvar . dbinders) (import desc (rho (AA dvar) (DA rib)))
                      label bvar label bvar)
                     (rho (AA dvar) (ADDD var) (ADDDDD var)))))
    (definition begin defns))))
;; The binding structure of the forms `kernel-forms` has not (yet), for a
;; program's walk (`binding-shapes.md`, step 1; `Macros::alpha_renamed`),
;; read by the expander written in Rust alone, for now: a template may not
;; use these. Its notation is the kernel's, and two more: `item`, a
;; module's item (a definition, a string, or an expression run as the module
;; is made), and `(shape A)`, what the module at address `A` gives, which
;; `with` and `include` bind. A `union` of one member is there to start its
;; addresses from where it is.
(define scope-forms datum
  (quote
   (
    (define items (mu d (union (export () (cage)) (export (item . d) (cage (at A) (at D))))))
    ;; `(name body …)`, `(name body)`: the name in scope in the body.
    (define clause (union (bvar . (import exprs (rho (A var))))))
    (define arm1 (union (bvar (import expr (rho (A var))))))
    (define ids (mu i (union () (id . i))))
    (define given-opt (union () (((literal given) . ids))))
    (define dvars (mu d (union (export () (rho)) (export (dvar . d) (rho (A dvar) (D rib))))))
    (define import-names
      (mu n (union (export () (rho))
                   (export (bvar . n) (rho (A var) (D rib)))
                   (export (((literal type) . dvars) . n) (rho (DA rib) (D rib)))
                   (export (((literal effect) . dvars) . n) (rho (DA rib) (D rib))))))
    (define variants (mu v (union (export () (rho))
                                  (export ((bvar . descs) . v) (rho (AA var) (D rib))))))
    (define case-clauses (mu c (union ()
                                      (((data . data) . exprs) . c)
                                      (((literal else) . exprs) . c))))
    (define typecase-arm (union (kind bvar (import expr (rho (AD var)))) (kind expr)))
    ;; A program's `tagcase` arm, `(tag x body …)` or `(tag (x …) body …)`,
    ;; the latter taking the tag's product apart; `else` a tag here.
    (define bvars (mu n (union (export () (rho)) (export (bvar . n) (rho (A var) (D rib))))))
    (define tag-arm (union (label bvar . (import exprs (rho (AD var))))
                           (label bvars . (import exprs (rho (AD rib))))))
    (define tag-arms (mu a (union () (tag-arm . a))))
    (define typecase-arms (mu a (union () (typecase-arm . a))))
    (define select-arm (union ((data . exprs) bvar . (import exprs (rho (AD var))))))
    (define select-arms (mu a (union () (select-arm . a))))
    (form vlambda (union (bvar . (import exprs (rho (A var))))
                         ((bvar desc) . (import exprs (rho (AA var))))))
    (form stage (expr desc desc expr clause clause . given-opt))
    (form session-check (expr desc desc expr clause clause . given-opt))
    (form reify-effect (desc . given-opt))
    (form reify-region (desc . given-opt))
    (form reify-type (desc . given-opt))
    (form session-commit! (expr expr expr))
    (form syscall-available? (data))
    (form send (expr expr))
    (form receive (expr))
    (form self (expr))
    (form icell-get (expr))
    (form icell-put! (expr expr))
    ;; `(spawn (mailbox message-type place) body …)`: the mailbox and the
    ;; place (a description, and a value of it) in the body, the place in
    ;; the message's type.
    (form spawn (union ((bvar (import desc (rho (ADDA place))) dvar)
                        . (import exprs (rho (AA var) (ADDA place))))))
    (form spawn-monitored (union (expr expr (bvar (import desc (rho (ADDADD place))) dvar)
                                       . (import exprs (rho (AADD var) (ADDADD place))))))
    (form par (let-bindings . (import exprs (rho (A rib)))))
    (form race exprs)
    (form attempt (expr expr))
    (form attempt-within (expr expr expr expr))
    (form pselect select-arms)
    (form tagcase (expr . tag-arms))
    (form case (expr . case-clauses))
    (form typecase (expr . typecase-arms))
    (form confirm-length (expr expr arm1 expr))
    (form acyclic (expr arm1 expr))
    (form confirm-nat (expr arm1 expr))
    (form convention (desc expr))
    (form make-bloblet exprs)
    (form rmake-bloblet exprs)
    (form bloblet-ref exprs)
    (form bloblet-set! exprs)
    (form bloblet-freeze exprs)
    (form bloblet-byte exprs)
    (form bloblet-set-byte! exprs)
    (form bloblet-bytes exprs)
    (form blobref-at exprs)
    (form blobref+ exprs)
    (form blobref-owner exprs)
    (form blobref-offset exprs)
    (form blobref-get exprs)
    (form blobref-set! exprs)
    (form blobref-index exprs)
    (form blobref-length exprs)
    (form blobref-variant! exprs)
    ;; A module's items, each in scope in all of them (values as a
    ;; `letrec*`'s, types as declared ahead).
    (form module (import items (cage (at ()))))
    (form extend (expr (import expr (shape A))))
    (form load-module (data))
    (form load-input (data))
    (form %loaded (data data))
    (form with (expr . (import exprs (shape A))))
    (form %datum (data))
    (form prompt (expr expr expr))
    (definition define-datatype
      (union (export (dvar . (import variants (rho (A dvar)))) (rho (A dvar) (D rib)))
             (export ((dvar . dbinders) . (import variants (rho (AA dvar) (DA rib))))
                     (rho (AA dvar) (D rib)))))
    (definition define-convention (export (dvar . data) (rho (A dvar))))
    (definition define-syscall (export (bvar data desc) (rho (A var))))
    (definition define-optional-syscall (export (bvar data desc) (rho (A var))))
    (definition include (union (export (expr) (shape A)) (export (expr data) (shape A))))
    (definition import (export (id . import-names) (rho (D rib))))
    (definition hide items)
    (definition module-parameters (data))
    (definition import-with-syntax (union (export (expr) (shape A)) (export (expr data) (shape A))))
    (definition define-syntax data)
    (definition define-syntax-rec data))))
;; The kernel's keywords (`top::KEYWORDS`, which a test holds this to); with
;; `#%fx`, `#%toplevel` and `define-syntax`, the names that are reserved.
(define reserved-words datum
  (quote
   (
    lambda plambda proj if letrec let begin define define* define-type define-generative subr ->
    ->* poly ref pairof dletrec void pure maxeff read write alloc goto comefrom region effect type
    prompt prompt-tag composable oneshot mark-key listof cond case else and or let* define-effect
    define-convention define-syscall define-optional-syscall syscall-available? module-parameters
    the bloblet fields frozen arrayof icell await define-rec letrena letreap rlambda quote
    productof sumof product tupleof tuple extract sum tagcase module moduleof with select
    load-module import define-datatype make-bloblet bloblet-ref bloblet-set! bloblet-freeze
    bloblet-byte bloblet-set-byte! bloblet-bytes rmake-bloblet dlambda => blobref-at blobref+
    blobref-owner blobref-offset blobref-get blobref-set! blobref-index blobref-length
    blobref-variant! vlambda stage session-check session-commit! typecase reify-type reify-effect
    reify-region confirm-length confirm-nat convention extend load-input quasiquote letregion
    letfreeze spawn spawn-monitored send receive self par race attempt attempt-within pselect
    icell-get icell-put! acyclic)))
;; The words of descriptions (`typed-macros.md`, "Decided: a template's free
;; description names, one qualified name each"), which a template's
;; descriptions write as they are, not the top level's: the heads of the
;; forms, each read by its name alone at a list's head (both checkers'
;; `TYPE_FORMS` and `EFFECT_HEADS`, and the parts of forms), and the words
;; read before any name is looked up. Any other name in a template's
;; description is the top level's, or a binder's the template writes.
(define description-heads datum
  (quote
   (
    arrayof narrayof bloblet composable dletrec icell listof mark-key moduleof mu nat nlist
    oneshot pairof place poly productof prompt-tag proves ref select subr sumof tupleof bool oneof
    union anyof allof -> ->* restrict modnames read write alloc goto comefrom await init send
    receive io
    maxeff globals conv fields frozen requires ensures <= + - dlambda acyclic const)))
;; The type forms' heads (`parse.rs`'s `TYPE_FORMS`, which a test holds
;; this to): a template whose head is one makes a description, which is
;; how a signature inferred says so (`macros/infer.rs`'s `makes`).
(define type-forms datum
  (quote
   (
    arrayof narrayof bloblet composable dletrec icell listof mark-key moduleof mu nat nlist
    oneshot pairof place poly productof prompt-tag proves ref select subr sumof tupleof bool oneof
    union anyof allof -> ->* restrict modnames)))
(define description-words datum
  (quote (void pure spin nondet stage gensym heap const acyclic finite cellular native fx
          distinct)))

;;; ============================================================ macros-syntax.fx (whole)

;; Whether `s` is `()`.
(define syn-nil? (->* (syn) bool)
  (lambda (s)
    (tagcase s
      (atom (d a b x) (null? d))
      (mx-eager-reader-types-lst (items d a b x) (null? items))
      (else y #f))))
;; Whether `s` is a keyword, `#:name`.
(define syn-keyword? (->* (syn) bool)
  (lambda (s) (tagcase s (atom (d a b x) (mx-keyword? d)) (else y #f))))
;; Whether `s` is an integer, as a program's label by position may be.
(define syn-number? (->* (syn) bool)
  (lambda (s) (tagcase s (atom (d a b x) (mx-datum-int? d)) (else y #f))))
;; A list's car and cdr, the cdr a list of the rest (or the tail), in a list
;; of two; none if `s` is no pair.
(define split (->* (syn) syns-a)
  (lambda (s)
    (tagcase s
      (mx-eager-reader-types-lst (items d a b x)
        (if (null? items)
            nil
            (list (car items)
                  (if (null? (cdr items))
                      (atom '() a b x)
                      (mx-eager-reader-types-lst (cdr items) (if (pair? d) (cdr d) d) a b x)))))
      (dotted (items tail d a b x)
        (list (car items)
              (if (null? (cdr items))
                  tail
                  (dotted (cdr items) tail (if (pair? d) (cdr d) d) a b x))))
      (else y nil))))
(define-rec
;; `d`, a datum, as syntax at no place.
(datum->syn (->* (datum) syn)
  (lambda (d)
    (if (pair? d)
        (let ((parts (datum-parts d)))
          (if (null? (cdr parts))
              (mx-eager-reader-types-lst (car parts) d 0 0 0)
              (dotted (car parts) (car (car (cdr parts))) d 0 0 0)))
        (atom d 0 0 0))))
;; A pair's elements as syntax, and its tail if it is not `()`, in a list.
(datum-parts (->* (datum) (listof syns-a acyclic))
  (lambda (d)
    (letrec ((go (subr (maxeff (read @globals) spin) (datum) syns-a)
               (lambda (x) (if (pair? x) (cons (datum->syn (car x)) (go (cdr x))) nil)))
             (end (subr (maxeff (read @globals) spin) (datum) datum)
               (lambda (x) (if (pair? x) (end (cdr x)) x))))
      (let ((e (end d)))
        (if (null? e)
            (list (go d))
            (list (go d) (the syns-a (list (atom e 0 0 0))))))))))
(define show-datum (->* (datum) string)
  (lambda (d)
    (cond ((null? d) "()")
          ((symbol? d) (symbol->string d))
          ((mx-keyword? d) (string-append "#:" (symbol->string (mx-keyword->symbol d))))
          ((string? d) (string-append "\"" (string-append d "\"")))
          ((char? d) (string-append "#\\" (char->string d)))
          ((int? d) (int->string d))
          ((f64? d) (f64->string d))
          ((bool? d) (if d "#t" "#f"))
          (else "…"))))
(define-rec
;; `s` written as the Rust front end's `write_syntax` writes it, for what
;; is said of it.
(show-syn (->* (syn) string)
  (lambda (s)
    (tagcase s
      (mx-eager-reader-types-lst (items d a b x) (if (null? items) "()" (string-append "(" (string-append (mx-syntax-show-each items) ")"))))
      (dotted (items tail d a b x)
        (string-append "(" (string-append (mx-syntax-show-each items) (string-append " . " (string-append (show-syn tail) ")")))))
      (vec (items d a b x) (string-append "#(" (string-append (mx-syntax-show-each items) ")")))
      (atom (d a b x) (show-datum d)))))
(mx-syntax-show-each (->* (syns-a) string)
  (lambda (xs)
    (cond ((null? xs) "")
          ((null? (cdr xs)) (show-syn (car xs)))
          (else (string-append (show-syn (car xs)) (string-append " " (mx-syntax-show-each (cdr xs)))))))))
(define words-have? (->* (symbol datum) bool)
  (lambda (n ws)
    (and (pair? ws)
         (or (and (symbol? (car ws)) (symbol=? n (car ws))) (words-have? n (cdr ws))))))
(define reserved-sym? (->* (symbol) bool)
  (lambda (n)
    (or (symbol=? n (string->symbol "#%fx")) (symbol=? n (string->symbol "#%toplevel")) (symbol=? n 'define-syntax)
        (symbol=? n 'define-syntax-rec)
        (words-have? n reserved-words))))
;;; ------------------------------------------------------------ names

;; Whether `name` is a form's, `#%fx`, `#%toplevel` or `define-syntax`: never
;; bound, so never a binder or a variable.
(define reserved? (->* (string) bool)
  (lambda (name) (reserved-sym? (string->symbol name))))
;; Whether `name` is one of words `ws`: compared as symbols, no word's name
;; copied to a string.
(define word-in? (->* (string datum) bool)
  (lambda (name ws) (words-have? (string->symbol name) ws)))
;; Whether `name` is a word of descriptions (the kernel's): one read
;; anywhere (`pure`, `heap`, a region constant `@r`, or a name already
;; `#%`'s), or, `head`, one a list's head may be (`subr`, `listof`).
(define desc-word? (->* (string bool) bool)
  (lambda (name head)
    (or (and (> (string-length name) 0) (char=? (string-ref name 0) #\@))
        (and (> (string-length name) 1) (string=? (substring name 0 2) "#%"))
        (word-in? name description-words)
        (and head (word-in? name description-heads)))))
;; Whether `name` is a type form's head (the kernel's `type-forms`), whose
;; template makes a description.
(define type-form? (->* (string) bool)
  (lambda (name) (word-in? name type-forms)))
;; Whether `s` is a name, and a reserved one.
(define reserved-syn? (->* (syn) bool)
  (lambda (s) (and (syn-symbol? s) (reserved-sym? (syn-symbol s)))))
(define mx-syntax-name-in? (->* (symbol mx-parser-types-names) bool)
  (lambda (n ns) (and (not (null? ns)) (or (symbol=? n (car ns)) (mx-syntax-name-in? n (cdr ns))))))
;; Whether `i` is one of `is`.
(define mx-syntax-int-in? (->* (int (listof int acyclic)) bool)
  (lambda (i is) (and (not (null? is)) (or (= i (car is)) (mx-syntax-int-in? i (cdr is))))))

;;; ============================================================ macros-types.fx (whole)

;; How a part of the expander fails: the caller's `pfail-at`.
(define-type fails (subr parses (string int int) void))
(define fail-at (->* (fails string syn) void)
  (lambda (mx-fail message s) (mx-fail message (syn-start s) (syn-end s))))
(define one-var (->* (stys (subr (read mxp) (sty) sty)) stys)
  (lambda (c f) (if (null? c) nil (list (f (car c))))))
(define or-same (->* (stys sty) sty) (lambda (c t) (if (null? c) t (car c))))
(define nth-sty (->* (stys int) sty)
  (lambda (xs i) (if (= i 0) (car xs) (nth-sty (cdr xs) (- i 1)))))
(define-rec
;; Of `t`, under `k` `mu`s more than it is, each variable `j` changed as
;; `op` says, in a list of one; none if none is (what is not is shared). Op
;; 0 raises one `k` or more out by `d`; 1 replaces `k`'s by `x` and lowers
;; those further out, as its `mu` is gone; 2 closes one `k` or more out, by
;; `env`'s `mu`s (closed, the innermost first); 3 finds one that is `k`.
(vars-map (->* (sty int int int sty stys) stys)
  (lambda (t k op d x env)
    (tagcase t
      (s-var (j)
        (cond ((< j k) nil)
              ((= op 0) (list (s-var (+ j d))))
              ((= op 2) (list (nth-sty env (- j k))))
              ((= op 3) (if (= j k) (list t) nil))
              ((> j k) (list (s-var (- j 1))))
              ((= k 0) (list x))
              (else (list (shift x k 0)))))
      (s-mu (b r p) (one-var (vars-map b (+ k 1) op d x env) (lambda ((c sty)) (s-mu c r p))))
      (s-pair (a b)
        (let ((a2 (vars-map a k op d x env)) (b2 (vars-map b k op d x env)))
          (if (and (null? a2) (null? b2)) nil (list (s-pair (or-same a2 a) (or-same b2 b))))))
      (s-import (b y) (one-var (vars-map b k op d x env) (lambda ((c sty)) (s-import c y))))
      (s-export (b y) (one-var (vars-map b k op d x env) (lambda ((c sty)) (s-export c y))))
      (s-many (b) (one-var (vars-map b k op d x env) (lambda ((c sty)) (s-many c))))
      (s-typed (b h) (one-var (vars-map b k op d x env) (lambda ((c sty)) (s-typed c h))))
      (s-union (ms)
        (let ((c (vars-map-each ms k op d x env))) (if (null? c) nil (list (s-union (car c))))))
      (else y nil))))
(vars-map-each (->* (stys int int int sty stys) (listof stys acyclic))
  (lambda (ms k op d x env)
    (if (null? ms)
        nil
        (let ((m (vars-map (car ms) k op d x env)) (r (vars-map-each (cdr ms) k op d x env)))
          (if (and (null? m) (null? r))
              nil
              (list (cons (or-same m (car ms)) (if (null? r) (cdr ms) (car r)))))))))
;; `t` placed under `d` `mu`s more, its variables `k` or more out raised.
(shift (->* (sty int int) sty) (lambda (t d k) (or-same (vars-map t k 0 d t nil) t))))
;; `t`, under the `mu` `k` around it, `x` for its variable, that `mu` gone.
(define subst (->* (sty int sty) sty) (lambda (t k x) (or-same (vars-map t k 1 0 x nil) t)))
;; `mu` `t` unfolded once: its body, its variable `t` again.
(define mx-types-unfold-mu (->* (sty) sty)
  (lambda (t) (tagcase t (s-mu (b r p) (subst b 0 t)) (else y t))))
;; `t` closed by the `mu`s `env` around it, closed, the innermost first.
(define closed-by (->* (sty stys) sty) (lambda (t env) (or-same (vars-map t 0 2 0 t env) t)))
;; Whether `t` refers to a `mu` `k` or more out from it; to the one `k`.
(define free-at? (->* (sty int) bool) (lambda (t k) (not (null? (vars-map t k 0 1 t nil)))))
(define refers? (->* (sty int) bool) (lambda (t k) (not (null? (vars-map t k 3 0 t nil)))))
;; `t`, under one `mu` it does not refer to, without it, in a list of one;
;; none if it refers to it.
(define lowered (->* (sty) stys)
  (lambda (t) (if (refers? t 0) nil (list (subst t 0 (s-nil))))))
(define marks (ref marked mxp) (new nil))
(define mark-count (ref int mxp) (new 2))
;; What is said of a name a macro takes for a trailmark that is none.
(define no-trailmark (->* (string) string)
  (lambda (n)
    (string-append "`" (string-append n (string-append "` names no trailmark here: `(define-syntax " (string-append n " (trailmark))`"))))))
(define mark-in (->* (symbol marked) int)
  (lambda (n ms)
    (cond ((null? ms) (cond ((symbol=? n 'else) 1) ((symbol=? n '=>) 2) (else 0)))
          ((symbol=? n (extract (car ms) 1)) (extract (car ms) 2))
          (else (mark-in n (cdr ms))))))
;; Whether typed hole `h` (none or one) has its effect written `pure`.
(define pure-hole? (->* ((listof hole acyclic)) bool)
  (lambda (h)
    (and (not (null? h)) (not (null? (extract (car h) 2)))
         (syn-symbol? (car (extract (car h) 2)))
         (string=? (syn-name (car (extract (car h) 2))) "pure"))))
;; A kind's number, for the cases of `overlap-base?`: 0 expr, 1 bvar, 2 id,
;; 3 data, 4 type, 5 label, 6 nil, 7 literal, 8 pair, 9 anything else, 10
;; lambda, 11 defn, 12 dvar, 13 kind, 14 item, 15 trailmark.
(define mx-types-kind-of (->* (sty) int)
  (lambda (t)
    (tagcase t
      (s-expr () 0) (s-bvar () 1) (s-id () 2) (s-data () 3) (s-type () 4) (s-label () 5)
      (s-nil () 6) (s-literal (n) 7) (s-pair (a d) 8) (s-lambda () 10) (s-defn () 11)
      (s-dvar () 12) (s-kind () 13) (s-item () 14) (s-mark (n i) 15) (s-rules () 16)
      (else y 9))))
;; Whether a pattern variable of type `t` has an expression in it: not a
;; binder, name, label, datum or type.
(define affine-kind? (->* (sty) bool)
  (lambda (t)
    (let ((k (mx-types-kind-of t)))
      (not (or (= k 1) (= k 2) (= k 5) (= k 3) (= k 4) (= k 12) (= k 13))))))
(define oneof-apart string
  (string-append "a syntax type: a `oneof`'s members must be told apart by their shapes in the "
                 "source (an `expr` may be anything)"))
(define any-free? (->* (stys) bool)
  (lambda (ms) (and (not (null? ms)) (or (free-at? (car ms) 0) (any-free? (cdr ms))))))
(define mx-types-closed-each (->* (stys stys) stys)
  (lambda (ms env) (if (null? ms) nil (cons (closed-by (car ms) env) (mx-types-closed-each (cdr ms) env)))))
;;; ------------------------------------------------------------ comparing

;; The type at `t`, its annotations, `mu`s seen through to what is under
;; them.
(define unfold (->* (sty) sty)
  (lambda (t)
    (tagcase t
      (s-import (b x) (unfold b)) (s-export (b x) (unfold b)) (s-many (b) (unfold b))
      (s-typed (b h) (unfold b))
      (s-mu (b r p) (unfold (mx-types-unfold-mu t)))
      (else y t))))
(define literal-of (->* (sty) symbol)
  (lambda (t) (tagcase t (s-literal (n) n) (else y (string->symbol "")))))
(define mark-of (->* (sty) int) (lambda (t) (tagcase t (s-mark (n i) i) (else y 0))))
;; Whether kind `k` is a binder's, `bvar` or `dvar`; one any syntax may be
;; of, `expr`, `data`, `desc`, `kind` or `defn`.
(define binder-kind? (->* (int) bool) (lambda (k) (or (= k 1) (= k 12))))
(define any-kind? (->* (int) bool)
  (lambda (k) (or (= k 0) (= k 3) (= k 4) (= k 11) (= k 13) (= k 14))))
(define-rec
;; Whether some source syntax could be of both `a` and `b` (Herman's
;; shapes, §5.2.3): `expr` may be anything but a form's name; looked at to
;; `depth` pairs, past which they are taken to overlap.
(overlap? (->* (sty sty int) bool)
  (lambda (a0 b0 depth)
    (let ((a (unfold a0)) (b (unfold b0)))
      (tagcase a
        (s-union (ms) (any-overlaps? ms b depth))
        (else y
          (tagcase b
            (s-union (ms) (any-overlaps-of? a ms depth))
            (else z (overlap-base? a b depth))))))))
(any-overlaps? (->* (stys sty int) bool)
  (lambda (ms b depth)
    (and (not (null? ms)) (or (overlap? (car ms) b depth) (any-overlaps? (cdr ms) b depth)))))
(any-overlaps-of? (->* (sty stys int) bool)
  (lambda (a ms depth)
    (and (not (null? ms)) (or (overlap? a (car ms) depth) (any-overlaps-of? a (cdr ms) depth)))))
(overlap-base? (->* (sty sty int) bool)
  (lambda (a b depth)
    (let ((ka (mx-types-kind-of a)) (kb (mx-types-kind-of b)))
      (cond
        ;; A trailmark is itself: no expression, datum, binder or other
        ;; trailmark; an `id` may be any name.
        ((and (= ka 15) (= kb 15)) (= (mark-of a) (mark-of b)))
        ((or (and (= ka 15) (= kb 2)) (and (= ka 2) (= kb 15))) #t)
        ((or (= ka 15) (= kb 15)) #f)
        ((and (= ka 7) (or (= kb 0) (binder-kind? kb))) #f)
        ((and (= kb 7) (or (= ka 0) (binder-kind? ka))) #f)
        ;; A lambda is a list: no name, `()` or label, and any list.
        ((or (= ka 10) (= kb 10))
         (let ((other (if (= ka 10) kb ka)))
           (not (or (= other 7) (binder-kind? other) (= other 2) (= other 6) (= other 5)))))
        ((and (= ka 7) (= kb 7)) (symbol=? (literal-of a) (literal-of b)))
        ((or (and (= ka 7) (= kb 2)) (and (= ka 2) (= kb 7))) #t)
        ((or (any-kind? ka) (any-kind? kb)) #t)
        ((and (= ka 8) (= kb 8))
         (or (= depth 0)
             (tagcase a
               (s-pair (a1 d1)
                 (tagcase b
                   (s-pair (a2 d2) (and (overlap? a1 a2 (- depth 1)) (overlap? d1 d2 (- depth 1))))
                   (else y #f)))
               (else y #f))))
        ((and (= ka 6) (= kb 6)) #t)
        ((and (= ka 5) (= kb 5)) #t)
        ((or (= ka 5) (= kb 5)) #f)
        ((and (or (binder-kind? ka) (= ka 2)) (or (binder-kind? kb) (= kb 2))) #t)
        (else #f))))))
;; Whether two of `ms` overlap.
(define any-overlap? (->* (stys) bool)
  (lambda (ms)
    (and (not (null? ms))
         (or (any-overlaps-of? (car ms) (cdr ms) 6) (any-overlap? (cdr ms))))))
(define-rec
;; Whether, of `t` under the `mu`s `env` (closed, the innermost first),
;; each `oneof` whose members refer to one of them has them apart.
(oneofs-apart? (->* (sty stys) bool)
  (lambda (t env)
    (tagcase t
      (s-mu (b r p) (oneofs-apart? b (cons (closed-by t env) env)))
      (s-union (ms)
        (and (or (not (any-free? ms)) (not (any-overlap? (mx-types-closed-each ms env))))
             (all-apart? ms env)))
      (s-pair (a d) (and (oneofs-apart? a env) (oneofs-apart? d env)))
      (s-import (b x) (oneofs-apart? b env)) (s-export (b x) (oneofs-apart? b env))
      (s-many (b) (oneofs-apart? b env)) (s-typed (b h) (oneofs-apart? b env))
      (else y #t))))
(all-apart? (->* (stys stys) bool)
  (lambda (ms env) (or (null? ms) (and (oneofs-apart? (car ms) env) (all-apart? (cdr ms) env))))))
;; `mu` `t`, read at `s` under `scope`: if it is closed, each `oneof` in it
;; whose members refer to a `mu` around them, which its reading left, its
;; members closed, told apart by their shapes (`macros.rs`'s `mu_read`).
(define mu-read (->* (fails sty syn mu-scope) sty)
  (lambda (mx-fail t s scope)
    (begin (if (and (null? scope) (not (oneofs-apart? t nil))) (fail-at mx-fail oneof-apart s) #u)
           t)))
(define cars-at (->* (entries (tupleof addr int)) entries)
  (lambda (es d)
    (if (null? es) (list d)
        (cons (tuple (the addr (cons #t (extract (car es) 1))) (extract (car es) 2))
              (cars-at (cdr es) d)))))
;; A list's `mu`'s body, `elem` under it: `()`, and the pair of `elem` and the list; an
;; element exporting a rib, the pair its entries from the pair's root, `(A …)`, then the
;; rest's, `(D rib)`: an element written out is in the pair's region (`macros.rs`'s
;; `seq_members`).
(define seq-body (->* (sty) stys)
  (lambda (elem)
    (let ((none (s-export (s-nil) (bind-rib nil))) (d (tuple (the addr (list #f)) 2)))
      (tagcase elem
        (s-export (e bd)
          (tagcase bd
            (bind-rib (es) (list none (s-export (s-pair e (s-var 0)) (bind-rib (cars-at es d)))))
            (else y (list none (s-export (s-pair elem (s-var 0))
                                         (bind-rib (list (tuple (the addr (list #t)) 2) d)))))))
        (else y (list (s-nil) (s-pair elem (s-var 0))))))))
;; `(elem ...)`: `(mu l (union () (elem . l)))`, and if `elem` exports,
;; each element's export together, as a `let`'s bindings do; a binder
;; alone, `bvar` or `dvar`, exports itself.
(define seq-type (->* (sty) sty)
  (lambda (elem0)
    (let ((elem (tagcase elem0
                  (s-bvar () (s-export elem0 (bind-rib (list (tuple (the addr nil) 1)))))
                  (s-dvar () (s-export elem0 (bind-rib (list (tuple (the addr nil) 4)))))
                  (else y elem0))))
      (s-mu (s-union (seq-body elem)) -1 -1))))
;; Whether `s` is the name `...`.
(define ellipsis? (->* (syn) bool)
  (lambda (s) (and (syn-symbol? s) (string=? (syn-name s) "..."))))
(define scope-of (->* (symbol mu-scope) int)
  (lambda (n scope)
    (cond ((null? scope) -1)
          ((and (extract (car scope) 2) (symbol=? n (extract (car scope) 1))) 0)
          (else (let ((i (scope-of n (cdr scope)))) (if (< i 0) i (+ i 1)))))))
(define mx-types-named-of (->* (symbol mx-types-named) stys)
  (lambda (n mx-types-named)
    (cond ((null? mx-types-named) nil)
          ((symbol=? n (extract (car mx-types-named) 1)) (list (extract (car mx-types-named) 2)))
          (else (mx-types-named-of n (cdr mx-types-named))))))
(define read-sty-name (->* (fails syn mx-types-named mu-scope) sty)
  (lambda (mx-fail s mx-types-named scope)
    (let ((n (syn-symbol s)))
      (let ((mu (scope-of n scope)))
        (if (>= mu 0)
            (s-var mu)
            (let ((t (mx-types-named-of n mx-types-named)))
              (if (null? t)
                  (case n
                    ((expr) (s-expr)) ((bvar) (s-bvar)) ((id) (s-id)) ((data) (s-data))
                    ((lambda) (s-lambda))
                    ;; A description, kinded after expansion as an
                    ;; expression is typed.
                    ((desc) (s-type)) ((label) (s-label)) ((dvar) (s-dvar)) ((kind) (s-kind))
                    ((item) (s-item))
                    ((type)
                     (begin (fail-at mx-fail (string-append "a syntax type: a part that is a "
                                                         (string-append "description is written `desc`, not "
                                                         "`type`"))
                                     s)
                            (s-nil)))
                    (else
                     (begin
                       (fail-at mx-fail (string-append "a syntax type: `" (string-append (symbol->string n) (string-append "` is "
                                                    (string-append "none (`expr`, `bvar`, `id`, `data`, `desc`, "
                                                    (string-append "`label`, `lambda`, `dvar`, `kind`, `item`, "
                                                    "or a `mu`'s variable)")))))
                                s)
                       (s-nil))))
                  (car t))))))))
;; A proper list's elements; none if `s` is not one.
(define proper-items (->* (syn) syns-a)
  (lambda (s) (tagcase s (mx-eager-reader-types-lst (items d a b x) items) (else y nil))))
(define-rec
;; Whether `t` uses the variable of the `mu` `id` around it only under a
;; pair, a `mu` inside it looked into too: none unfolds to itself.
(guarded? (->* (sty int) bool)
  (lambda (t id)
    (tagcase t
      (s-var (j) (not (= j id)))
      (s-mu (b r p) (guarded? b (+ id 1)))
      (s-import (b x) (guarded? b id)) (s-export (b x) (guarded? b id)) (s-many (b) (guarded? b id))
      (s-typed (b h) (guarded? b id))
      (s-union (ms) (all-guarded? ms id))
      (else y #t))))
(all-guarded? (->* (stys int) bool)
  (lambda (ms id) (or (null? ms) (and (guarded? (car ms) id) (all-guarded? (cdr ms) id))))))
;; Whether `s` is a proper list of one or more.
(define proper? (->* (syn) bool)
  (lambda (s) (tagcase s (mx-eager-reader-types-lst (items d a b x) (not (null? items))) (else y #f))))
;; A keyword's name.
(define keyword-name (->* (syn) string)
  (lambda (s)
    (tagcase s
      (atom (d a b x) (if (mx-keyword? d) (symbol->string (mx-keyword->symbol d)) ""))
      (else y ""))))
;; A typed hole's keywords and what each says, `#:type T` and, of an `expr`,
;; `#:effect E`, each once: from `parts`, onto `ty` and `eff`; `kind`, the
;; part's.
(define read-hole (->* (fails syns-a string syn syns-a syns-a) hole)
  (lambda (mx-fail parts kind s ty eff)
    (let* ((expr? (string=? kind "expr"))
           (usage (string-append "a syntax type: "
                                 (if expr?
                                     "`(expr #:type T #:effect E)`, either left out"
                                     (string-append "`(" (string-append kind " #:type T)`"))))))
      (cond
        ((null? parts) (tuple ty eff))
        ((null? (cdr parts)) (begin (fail-at mx-fail usage s) (tuple ty eff)))
        (else
         (let* ((k (car parts)) (word (if (syn-keyword? k) (keyword-name k) "")))
           (cond ((and (string=? word "type") (null? ty))
                  (read-hole mx-fail (cdr (cdr parts)) kind s (list (nth parts 1)) eff))
                 ((and expr? (string=? word "effect") (null? eff))
                  (read-hole mx-fail (cdr (cdr parts)) kind s ty (list (nth parts 1))))
                 ((or (string=? word "type") (and expr? (string=? word "effect")))
                  (begin (fail-at mx-fail (string-append usage ", each once") k) (tuple ty eff)))
                 (else (begin (fail-at mx-fail usage k) (tuple ty eff))))))))))
;; A binding, `(rho (address δ) …)` or `(cage (rho …) (at address) …)`.
(define bind-usage string
  "a binding is `(rho (address δ) …)` or `(cage (rho …) (at address) …)`")
(define syntax-sigs (ref syns-a mxp) (new nil))
;; δ, by number: 0 `none`, 1 `var`, 2 `rib`, 3 `env`, 4 `dvar`, 5 `place`;
;; 100 + k a macro's binder, its signature `(macro σ made)` (`Delta::Syntax`)
;; the `k`th of `syntax-sigs`, as written.
(define read-delta (->* (fails syn) int)
  (lambda (mx-fail d)
    (let ((n (if (syn-symbol? d) (syn-name d) "")) (ds (proper-items d)))
      (cond ((string=? n "none") 0) ((string=? n "var") 1) ((string=? n "rib") 2)
            ((string=? n "env") 3) ((string=? n "dvar") 4) ((string=? n "place") 5)
            ;; A macro's binder: its kind the macro's signature.
            ((and (>= (len ds) 3) (syn-symbol? (car ds))
                  (or (string=? (syn-name (car ds)) "macro/addressed")
                      (string=? (syn-name (car ds)) "macro")))
             (let ((k (len (get syntax-sigs))))
               (begin (set syntax-sigs (append (get syntax-sigs) (list d))) (+ 100 k))))
            ((and (pair? ds) (syn-symbol? (car ds)) (string=? (syn-name (car ds)) "syntax"))
             (begin (fail-at mx-fail (string-append "a macro's binder is written with its signature, "
                                                 "`(macro …)`, not `(syntax …)`")
                             d)
                    0))
            (else (begin (fail-at mx-fail (string-append "δ is `none`, `var`, `dvar`, `place`, "
                                                      (string-append "`rib`, `env` or a macro's signature, "
                                                      "`(macro σ made)`"))
                                  d)
                         0))))))
(define path-usage string
  (string-append "an address is `(path #:a #:d …)`, its steps from the root, `#:a` the car "
                 "and `#:d` the cdr"))
(define path-steps (->* (fails syns-a) addr)
  (lambda (mx-fail ks)
    (if (null? ks)
        nil
        (let ((k (keyword-name (car ks))))
          (cond ((string=? k "a") (cons #t (path-steps mx-fail (cdr ks))))
                ((string=? k "d") (cons #f (path-steps mx-fail (cdr ks))))
                (else (begin (fail-at mx-fail path-usage (car ks)) (the addr nil))))))))
(define a-or-d? (->* (string int) bool)
  (lambda (s i)
    (or (= i (string-length s))
        (and (or (char=? (string-ref s i) #\A) (char=? (string-ref s i) #\D))
             (a-or-d? s (+ i 1))))))
;; The steps of `s`'s letters, the last first: from `j` on, onto `out`.
(define mx-types-steps (->* (string int addr) addr)
  (lambda (s j out)
    (if (= j (string-length s))
        out
        (mx-types-steps s (+ j 1) (cons (char=? (string-ref s j) #\A) out)))))
;; An address, `A`s and `D`s read right to left, or `()`, none: its steps,
;; first first.
(define read-addr (->* (fails syn) addr)
  (lambda (mx-fail a)
    (let ((name (if (syn-symbol? a) (syn-name a) "")))
      (cond
        ((syn-nil? a) nil)
        ;; `(path #:a #:d …)`: its steps from the root, first first.
        ((let ((ps (proper-items a))) (and (pair? ps) (syn-symbol? (car ps))
                                           (string=? (syn-name (car ps)) "path")))
         (path-steps mx-fail (cdr (proper-items a))))
        ((and (> (string-length name) 0) (a-or-d? name 0)) (mx-types-steps name 0 nil))
        (else (begin (fail-at mx-fail path-usage a) (the addr nil)))))))
(define read-rib (->* (fails syns-a) entries)
  (lambda (mx-fail es)
    (if (null? es)
        nil
        (let ((parts (proper-items (car es))))
          (if (= (len parts) 2)
              (cons (tuple (read-addr mx-fail (car parts)) (read-delta mx-fail (nth parts 1)))
                    (read-rib mx-fail (cdr es)))
              (begin (fail-at mx-fail bind-usage (car es)) (the entries nil)))))))
(define read-rings (->* (fails syns-a) rings)
  (lambda (mx-fail rs)
    (if (null? rs)
        nil
        (let* ((parts (proper-items (car rs)))
               (head (if (and (pair? parts) (syn-symbol? (car parts))) (syn-name (car parts)) ""))
               (r (cond ((string=? head "rho") (ring-rib (read-rib mx-fail (cdr parts))))
                        ((and (string=? head "at") (= (len parts) 2))
                         (ring-at (read-addr mx-fail (nth parts 1))))
                        ((and (string=? head "shape") (= (len parts) 2))
                         (ring-shape (read-addr mx-fail (nth parts 1))))
                        (else (begin (fail-at mx-fail bind-usage (car rs)) (ring-rib nil))))))
          (cons r (read-rings mx-fail (cdr rs)))))))
(define read-bind (->* (fails syn) mx-types-bind)
  (lambda (mx-fail b)
    (let* ((items (proper-items b)) (head (if (and (pair? items) (syn-symbol? (car items)))
                                             (syn-name (car items))
                                             "")))
      (cond ((string=? head "rho") (bind-rib (read-rib mx-fail (cdr items))))
            ((string=? head "cage") (bind-cage (read-rings mx-fail (cdr items))))
            ((and (string=? head "shape") (= (len items) 2))
             (bind-shape (read-addr mx-fail (nth items 1))))
            (else (begin (fail-at mx-fail bind-usage b) (bind-rib nil)))))))
;; Whether one of entries `es` is a macro's binder's.
(define syntax-entry? (->* (entries) bool)
  (lambda (es) (and (not (null? es)) (or (>= (extract (car es) 2) 100) (syntax-entry? (cdr es))))))
(define-rec
(read-sty (->* (fails syn mx-types-named mu-scope) sty)
  (lambda (mx-fail s mx-types-named scope)
    (cond
      ((syn-nil? s) (s-nil))
      ((syn-symbol? s) (read-sty-name mx-fail s mx-types-named scope))
      (else
       (let* ((parts (split s)) (hd (if (null? parts) s (car parts)))
              ;; The forms below are proper lists; a dotted one is a pair.
              (head (if (and (syn-symbol? hd) (proper? s)) (syn-name hd) "")))
         (cond
           ((null? parts) (begin (fail-at mx-fail "a syntax type: not one" s) (s-nil)))
           ;; `(σ … σ ...)`: a list ending in any number of the last `σ`.
           ((let ((items (proper-items s)))
              (and (>= (len items) 2) (ellipsis? (nth items (- (len items) 1)))))
            (read-seq-list mx-fail s (proper-items s) mx-types-named scope))
           ((or (string=? head "import") (string=? head "export"))
            (let ((args (proper-items s)))
              (if (= (len args) 3)
                  (let ((body (read-sty mx-fail (nth args 1) mx-types-named scope))
                        (b (read-bind mx-fail (nth args 2))))
                    (cond ((not (string=? head "import")) (s-export body b))
                          ((tagcase b (bind-rib (es) (syntax-entry? es)) (else y #f))
                           (begin (fail-at mx-fail (string-append "a syntax type: a macro's "
                                                               (string-append "binder, `(macro …)`, says what a "
                                                               (string-append "use defines, in what its "
                                                               "signature makes, `(defn D …)`")))
                                           s)
                                  (s-import body b)))
                          (else (s-import body b))))
                  (read-pair mx-fail s parts mx-types-named scope))))
           ((string=? head "mu") (read-mu mx-fail s mx-types-named scope))
           ((or (string=? head "union") (string=? head "oneof"))
            (let ((ms (read-each mx-fail (cdr (proper-items s)) mx-types-named scope)))
              ;; Members that refer to a `mu` around them, checked when it
              ;; is read whole (`mu-read`).
              (begin
                (if (and (not (any-free? ms)) (any-overlap? ms)) (fail-at mx-fail oneof-apart s) #u)
                (s-union ms))))
           ;; `(expr #:type T #:effect E)`, `(bvar #:type T)`: a list whose
           ;; second is a keyword, which no syntax type is.
           ((and (or (string=? head "expr") (string=? head "bvar") (string=? head "lambda"))
                 (> (len (proper-items s)) 1) (syn-keyword? (nth (proper-items s) 1)))
            (s-typed (cond ((string=? head "expr") (s-expr))
                           ((string=? head "bvar") (s-bvar))
                           (else (s-lambda)))
                     (read-hole mx-fail (cdr (proper-items s)) head s nil nil)))
           ((and (string=? head "many") (= (len (proper-items s)) 2))
            (s-many (read-sty mx-fail (nth (proper-items s) 1) mx-types-named scope)))
           ((and (string=? head "literal") (= (len (proper-items s)) 2))
            (let ((n (nth (proper-items s) 1)))
              (cond ((not (syn-symbol? n))
                     (begin (fail-at mx-fail "a syntax type: `(literal name)`" s) (s-nil)))
                    ;; The kernel's words, as its signatures write them, and
                    ;; a reserved name; a name a macro matches, a trailmark.
                    ((and (null? mx-types-named) (not (reserved-syn? n)))
                     (begin (fail-at mx-fail (string-append "a syntax type: `(literal " (string-append (syn-name n)
                                                         (string-append ")` is a kernel form's word; a name a "
                                                         (string-append "macro's pattern matches is a "
                                                         (string-append "trailmark's, `(trailmark " (string-append (syn-name n)
                                                         ")`"))))))
                                     s)
                            (s-nil)))
                    (else (s-literal (syn-symbol n))))))
           ((and (string=? head "trailmark") (= (len (proper-items s)) 2))
            (let ((n (nth (proper-items s) 1)))
              (cond ((not (syn-symbol? n))
                     (begin (fail-at mx-fail "a syntax type: `(trailmark name)`" s) (s-nil)))
                    ((= (mark-in (syn-symbol n) (get marks)) 0)
                     (begin (fail-at mx-fail (string-append "a syntax type: "
                                                         (no-trailmark (syn-name n)))
                                     s)
                            (s-nil)))
                    (else (s-mark (syn-symbol n) (mark-in (syn-symbol n) (get marks)))))))
           (else (read-pair mx-fail s parts mx-types-named scope))))))))
;; The items of a list type ending in `σ ...`: the last `σ` repeated
;; (`macros.rs`'s `seq`), read under its `mu`, after the ones before it,
;; each once; `s` the whole.
(read-seq-list (->* (fails syn syns-a mx-types-named mu-scope) sty)
  (lambda (mx-fail s items mx-types-named scope)
    (if (null? (cdr (cdr items)))
        (mu-read mx-fail (seq-type (read-sty mx-fail (car items) mx-types-named
                                          (cons (tuple (string->symbol "") #f) scope)))
                 s scope)
        (let ((first (read-sty mx-fail (car items) mx-types-named scope)))
          (s-pair first (read-seq-list mx-fail s (cdr items) mx-types-named scope))))))
(read-pair (->* (fails syn syns-a mx-types-named mu-scope) sty)
  (lambda (mx-fail s parts mx-types-named scope)
    (s-pair (read-sty mx-fail (car parts) mx-types-named scope) (read-sty mx-fail (nth parts 1) mx-types-named scope))))
(read-each (->* (fails syns-a mx-types-named mu-scope) stys)
  (lambda (mx-fail xs mx-types-named scope)
    (if (null? xs)
        nil
        (cons (read-sty mx-fail (car xs) mx-types-named scope) (read-each mx-fail (cdr xs) mx-types-named scope)))))
;; `(mu v body)`.
(read-mu (->* (fails syn mx-types-named mu-scope) sty)
  (lambda (mx-fail s mx-types-named scope)
    (let ((args (proper-items s)))
      (if (and (= (len args) 3) (syn-symbol? (nth args 1)))
          (let ((body (read-sty mx-fail (nth args 2) mx-types-named
                                (cons (tuple (syn-symbol (nth args 1)) #t) scope))))
            (begin
              (if (guarded? body 0)
                  #u
                  (fail-at mx-fail (string-append "a syntax type: a `mu`'s variable is used where no "
                                               (string-append "pair is around it (Herman's types are "
                                               "contractive)"))
                           s))
              (mu-read mx-fail (s-mu body -1 -1) s scope)))
          (begin (fail-at mx-fail "a syntax type: `(mu variable type)`" s) (s-nil)))))))
;; Elements `es`, found under list `mu` `t`, in a list of one, outside it.
(define out-of-mu (->* (stys sty) stys)
  (lambda (es t) (if (null? es) nil (list (subst (car es) 0 t)))))
;; Whether `r`, under a list's `mu`, is the list.
(define itself? (->* (sty) bool) (lambda (r) (tagcase r (s-var (j) (= j 0)) (else y #f))))
;; Entries `es`, each at its car's, from the car, onto `out`, in a list of
;; one; none if one is not its car's.
(define cars-of (->* (entries entries) (listof entries acyclic))
  (lambda (es out)
    (cond ((null? es) (list (the entries (reverse out))))
          ((and (not (null? (extract (car es) 1))) (car (extract (car es) 1)))
           (cars-of (cdr es) (cons (tuple (cdr (extract (car es) 1)) (extract (car es) 2)) out)))
          (else nil))))
;; The signature a macro's binder's δ `d` says, as written.
(define syntax-sig-of (->* (int) syn) (lambda (d) (nth (get syntax-sigs) (- d 100))))
;; Whether two δs say the same: two macros' binders, by what their
;; signatures write (`bind_same`).
(define delta=? (->* (int int) bool)
  (lambda (d e)
    (or (= d e)
        (and (>= d 100) (>= e 100)
             (string=? (show-syn (syntax-sig-of d)) (show-syn (syntax-sig-of e)))))))
(define mx-types-addr=? (->* (addr addr) bool)
  (lambda (a b)
    (if (null? a)
        (null? b)
        (and (not (null? b)) (mx-eq? (car a) (car b)) (mx-types-addr=? (cdr a) (cdr b))))))
(define entries=? (->* (entries entries) bool)
  (lambda (es fs)
    (if (null? es)
        (null? fs)
        (and (not (null? fs))
             (mx-types-addr=? (extract (car es) 1) (extract (car fs) 1))
             (delta=? (extract (car es) 2) (extract (car fs) 2))
             (entries=? (cdr es) (cdr fs))))))
(define rings=? (->* (rings rings) bool)
  (lambda (rs ss)
    (if (null? rs)
        (null? ss)
        (and (not (null? ss))
             (tagcase (car rs)
               (ring-rib (es) (tagcase (car ss) (ring-rib (fs) (entries=? es fs)) (else y #f)))
               (ring-at (a) (tagcase (car ss) (ring-at (b) (mx-types-addr=? a b)) (else y #f)))
               (ring-shape (a) (tagcase (car ss) (ring-shape (b) (mx-types-addr=? a b)) (else y #f))))
             (rings=? (cdr rs) (cdr ss))))))
(define bind=? (->* (mx-types-bind mx-types-bind) bool)
  (lambda (p q)
    (tagcase p
      (bind-rib (es) (tagcase q (bind-rib (fs) (entries=? es fs)) (else y #f)))
      (bind-cage (rs) (tagcase q (bind-cage (ss) (rings=? rs ss)) (else y #f)))
      ;; Found, never written in a signature.
      (bind-direct (as) #f)
      (bind-earlier (as) #f)
      (bind-shape (a) (tagcase q (bind-shape (b) (mx-types-addr=? a b)) (else y #f))))))
;; Pair type `core` of a ribcage's list, under its `mu`, which exports `rings`: its
;; element, with what it exports, in a list of one; none if it is not one.
(define cage-pair (->* (sty rings) stys)
  (lambda (core rings)
    (tagcase core
      (s-pair (a d)
        (tagcase d
          (s-import (r inner)
            (if (and (itself? r) (not (null? (cdr rings)))
                     (null? (cdr (cdr rings)))
                     (tagcase (car rings) (ring-at (x) (mx-types-addr=? x (list #f))) (else y #f)))
                (let ((own (car (cdr rings))))
                  (tagcase own
                    (ring-rib (es)
                      (let ((stripped (cars-of es nil)))
                        (if (and (bind=? inner (bind-rib es)) (not (null? stripped)))
                            (list (s-export a (bind-rib (car stripped))))
                            nil)))
                    (ring-at (x)
                      (if (and (mx-types-addr=? x (list #t)) (bind=? inner (bind-cage (list own))))
                          (list a)
                          nil))
                    (ring-shape (x) nil)))
                nil))
          (else y nil)))
      (else y nil))))
(define cage-members (->* (stys stys bool) stys)
  (lambda (ms es none)
    (if (null? ms)
        (if (and none (not (null? es)) (null? (cdr es))) es nil)
        (tagcase (car ms)
          (s-export (core b)
            (tagcase b
              (bind-cage (rings)
                (cond ((and (null? rings) (= (mx-types-kind-of core) 6)) (cage-members (cdr ms) es #t))
                      ((null? rings) nil)
                      (else
                       (let ((e (cage-pair core rings)))
                         (if (null? e) nil (cage-members (cdr ms) (cons (car e) es) none))))))
              (else y nil)))
          (else y nil)))))
;; A ribcage's list (`macros.rs`'s `cage_seq_of`): `(export () (cage))`, and
;; pairs `(export (e . (import t β)) (cage (at D) β'))`, the rest in the
;; scope of what the element binds, `β`, which is also the list's innermost
;; rib or ribcage, `β'`, after the rest's. The element type, with what it
;; exports, in a list of one.
(define cage-seq-elem (->* (sty) stys)
  (lambda (t)
    (tagcase t
      (s-mu (b r p)
        (tagcase b (s-union (ms) (out-of-mu (cage-members ms nil #f) t)) (else y nil)))
      (else y nil))))
;; Entries `es` at their car's, from the pair's root.
(define car-entries (->* (entries) entries)
  (lambda (es)
    (cond ((null? es) nil)
          ((and (not (null? (extract (car es) 1))) (car (extract (car es) 1)))
           (cons (car es) (car-entries (cdr es))))
          (else (car-entries (cdr es))))))
;; Whether `d`, member `m`'s pair's rest, is list `id` itself, perhaps in
;; the scope of what the pair's element binds (a `lambda`'s parameters,
;; each in the later ones' types: `seq-scoped?`).
(define seq-rest? (->* (sty sty) bool)
  (lambda (m d)
    (tagcase d
      (s-var (j) (= j 0))
      (s-import (r b)
        (and (itself? r)
             (tagcase b
               (bind-rib (fs)
                 (tagcase m
                   (s-export (c x)
                     (tagcase x (bind-rib (es) (entries=? fs (car-entries es))) (else y #f)))
                   (else y #f)))
               (else y #f))))
      (else y #f))))
;; A pair's export's entries `es` that are its car's, from the car, onto
;; `out`, in a list of one; none if one is neither its car's nor `(D rib)`.
(define own-entries (->* (entries entries) (listof entries acyclic))
  (lambda (es out)
    (if (null? es)
        (list (the entries (reverse out)))
        (let ((a (extract (car es) 1)) (d (extract (car es) 2)))
          (cond ((and (not (null? a)) (car a)) (own-entries (cdr es) (cons (tuple (cdr a) d) out)))
                ((and (not (null? a)) (null? (cdr a)) (= d 2)) (own-entries (cdr es) out))
                (else nil))))))
;; Element `a`, which exports what `own` says of it.
(define with-own (->* (sty entries) sty)
  (lambda (a own)
    (cond ((null? own) a)
          ((and (tagcase a (s-export (b x) #t) (else y #f)) (null? (cdr own))
                (null? (extract (car own) 1)) (= (extract (car own) 2) 2))
           a)
          (else (s-export a (bind-rib own))))))
;; Of members `ms`, from the first: the elements found so far `es`, the last
;; first, and whether `()` is one of them.
(define seq-members (->* (stys stys bool) stys)
  (lambda (ms es none)
    (if (null? ms)
        (cond ((or (null? es) (not none)) nil)
              ((null? (cdr es)) es)
              (else (list (s-union (the stys (reverse es))))))
        (let* ((m (car ms)) (core (tagcase m (s-export (b x) b) (else y m)))
               (own (tagcase m
                      (s-export (b x) (tagcase x (bind-rib (es) (own-entries es nil)) (else z nil)))
                      (s-import (b x) nil)
                      (else y (list (the entries nil))))))
          (cond ((null? own) nil)
                ((= (mx-types-kind-of core) 6) (seq-members (cdr ms) es #t))
                (else
                 (tagcase core
                   (s-pair (a d)
                     (if (seq-rest? m d)
                         (seq-members (cdr ms) (cons (with-own a (car own)) es) none)
                         nil))
                   (else y nil))))))))
;; The type of an element of list type `t`, in a list of one, if a template
;; or pattern may give its elements by ellipsis: a `mu` of `()` and of pairs
;; of an element and itself, which binds no more than what each element
;; exports, all of them together (a `let`'s); or each element's in the
;; scope of those after it, the list a ribcage (a `let*`'s, a `begin`'s
;; definitions, `seq-cage?`) (`macros.rs`'s `seq_of`).
(define seq-elem (->* (sty) stys)
  (lambda (t)
    (let ((caged (cage-seq-elem t)))
      (if (null? caged)
          (tagcase t
            (s-mu (b r p)
              (tagcase b (s-union (ms) (out-of-mu (seq-members ms nil #f) t)) (else y nil)))
            (else y nil))
          caged))))
(define peel-export (->* (sty) sty)
  (lambda (m) (tagcase m (s-export (c x) c) (else y m))))
(define rest-imported? (->* (stys) bool)
  (lambda (ms)
    (and (not (null? ms))
         (or (tagcase (peel-export (car ms))
               (s-pair (a d) (tagcase d (s-import (r b) #t) (else y #f)))
               (else y #f))
             (rest-imported? (cdr ms))))))
;; Whether list type `t`'s elements are each in the scope of what those
;; before it bind, its pairs' rests imported.
(define seq-scoped? (->* (sty) bool)
  (lambda (t)
    (tagcase t
      (s-mu (b r p) (tagcase b (s-union (ms) (rest-imported? ms)) (else y #f)))
      (else y #f))))
(define seq-cage? (->* (sty) bool)
  (lambda (t)
    (or (not (null? (cage-seq-elem t))) (and (seq-scoped? t) (not (null? (seq-elem t)))))))
;; The type at `t`, its annotations seen through, not its `mu`s.
(define peel (->* (sty) sty)
  (lambda (t)
    (tagcase t
      (s-import (b x) (peel b)) (s-export (b x) (peel b)) (s-many (b) (peel b))
      (s-typed (b h) (peel b))
      (else y t))))
;; `x` of `(x ...)`, a part repeated, in a list of one, if `s` is one.
(define ellipsis-of (->* (syn) syns-a)
  (lambda (s)
    (let ((items (proper-items s)))
      (if (and (= (len items) 2) (ellipsis? (nth items 1))) (list (car items)) nil))))
;; Whether `s` is a lambda as the checkers take one: a `lambda`, `rlambda`
;; or `vlambda`, in `plambda`s and `the`s (`macros.rs`'s `lambda_shaped`).
(define lambda-shaped? (->* (syn) bool)
  (lambda (s)
    (let* ((items (proper-items s))
           (head (if (and (not (null? items)) (syn-symbol? (car items)))
                     (syn-name (car items))
                     "")))
      (cond ((or (string=? head "lambda") (string=? head "rlambda") (string=? head "vlambda")) #t)
            ((and (or (string=? head "plambda") (string=? head "the")) (= (len items) 3))
             (lambda-shaped? (nth items 2)))
            (else #f)))))
;; The kernel's forms' signatures, each by its name, and its definitions'
;; (its `begin` of them not its `begin` of expressions), its abbreviations
;; read on the way, `defn` among them, which no macro's signature names:
;; read once, the first time.
(define kernel-read (ref (tupleof mx-types-named mx-types-named mx-types-named) mxp)
  (new (tuple (the mx-types-named nil) (the mx-types-named nil) (the mx-types-named nil))))
(define read-kernel (->* (fails syns-a mx-types-named mx-types-named mx-types-named) (tupleof mx-types-named mx-types-named mx-types-named))
  (lambda (mx-fail fs mx-types-named out defs)
    (if (null? fs)
        (tuple out defs mx-types-named)
        (let* ((items (proper-items (car fs)))
               (t (read-sty mx-fail (nth items 2) mx-types-named nil))
               (n (syn-symbol (nth items 1)))
               (what (syn-name (car items))))
          (cond ((string=? what "define")
                 (read-kernel mx-fail (cdr fs) (cons (tuple n t) mx-types-named) out defs))
                ((string=? what "definition")
                 (read-kernel mx-fail (cdr fs) mx-types-named out (cons (tuple n t) defs)))
                (else (read-kernel mx-fail (cdr fs) mx-types-named (cons (tuple n t) out) defs)))))))
(define kernel-tables (->* (fails) (tupleof mx-types-named mx-types-named mx-types-named))
  (lambda (mx-fail)
    (begin
      (if (null? (extract (get kernel-read) 1))
          (set kernel-read (read-kernel mx-fail (proper-items (datum->syn kernel-forms))
                                        (list (tuple 'defn (s-defn))) nil nil))
          #u)
      (get kernel-read))))
(define kernel-sigs (->* (fails) mx-types-named)
  (lambda (mx-fail) (extract (kernel-tables mx-fail) 1)))
(define kernel-definitions (->* (fails) mx-types-named)
  (lambda (mx-fail) (extract (kernel-tables mx-fail) 2)))
;; The binding structure of the forms the kernel's list has not, for a
;; program's walk alone (`scope-forms`), its signatures and definitions,
;; read with the kernel's abbreviations: read once, the first time.
(define scope-read (ref (tupleof mx-types-named mx-types-named mx-types-named) mxp)
  (new (tuple (the mx-types-named nil) (the mx-types-named nil) (the mx-types-named nil))))
(define scope-tables (->* (fails) (tupleof mx-types-named mx-types-named mx-types-named))
  (lambda (mx-fail)
    (begin
      (if (null? (extract (get scope-read) 1))
          (set scope-read (read-kernel mx-fail (proper-items (datum->syn scope-forms))
                                       (extract (kernel-tables mx-fail) 3) nil nil))
          #u)
      (get scope-read))))
(define scope-sigs (->* (fails) mx-types-named)
  (lambda (mx-fail) (extract (scope-tables mx-fail) 1)))
(define scope-definitions (->* (fails) mx-types-named)
  (lambda (mx-fail) (extract (scope-tables mx-fail) 2)))
;; Whether `k` is a kind of no parts: one of what `(Expr | Id | Data | Type
;; | Label | Bvar | Dvar | Kind | Lambda | Defn)` names in `macros.rs`.
(define simple-kind? (->* (int) bool) (lambda (k) (or (< k 6) (> k 9))))
(define-rec
;; Whether two types are the same, structurally.
(sty=? (->* (sty sty) bool)
  (lambda (a b)
    (tagcase a
      (s-literal (n) (tagcase b (s-literal (m) (symbol=? n m)) (else y #f)))
      (s-mark (n i) (= i (mark-of b)))
      (s-pair (x y) (tagcase b (s-pair (p q) (and (sty=? x p) (sty=? y q))) (else z #f)))
      (s-import (x p) (tagcase b (s-import (y q) (and (sty=? x y) (bind=? p q))) (else z #f)))
      (s-export (x p) (tagcase b (s-export (y q) (and (sty=? x y) (bind=? p q))) (else z #f)))
      (s-mu (x r p) (tagcase b (s-mu (y q o) (and (= r q) (= p o) (sty=? x y))) (else z #f)))
      (s-var (i) (tagcase b (s-var (j) (= i j)) (else y #f)))
      (s-many (x) (tagcase b (s-many (y) (sty=? x y)) (else z #f)))
      (s-typed (x h) (tagcase b (s-typed (y g) (sty=? x y)) (else z #f)))
      (s-union (ms) (tagcase b (s-union (ns) (all-sty=? ms ns)) (else y #f)))
      (else y (and (or (< (mx-types-kind-of a) 7) (> (mx-types-kind-of a) 9)) (= (mx-types-kind-of a) (mx-types-kind-of b)))))))
(all-sty=? (->* (stys stys) bool)
  (lambda (ms ns)
    (if (null? ms)
        (null? ns)
        (and (not (null? ns)) (sty=? (car ms) (car ns)) (all-sty=? (cdr ms) (cdr ns)))))))
(define mx-types-mu? (->* (sty) bool) (lambda (t) (tagcase t (s-mu (b r p) #t) (else y #f))))
(define assumed? (->* (sty sty assumed) bool)
  (lambda (a b xs)
    (and (not (null? xs))
         (or (and (sty=? a (extract (car xs) 1)) (sty=? b (extract (car xs) 2)))
             (assumed? a b (cdr xs))))))
(define-rec
(sub? (->* (sty sty assumed) bool)
  (lambda (a b assumed)
    (if (or (sty=? a b) (and (or (mx-types-mu? a) (mx-types-mu? b)) (assumed? a b assumed)))
        #t
        (let ((more (if (or (mx-types-mu? a) (mx-types-mu? b)) (the assumed (cons (tuple a b) assumed)) assumed)))
          (tagcase a
            (s-many (x) (sub? x b more))
            (s-typed (x h) (sub? x b more))
            (s-mu (x r p) (sub? (mx-types-unfold-mu a) b more))
            (s-union (ms) (all-sub? ms b more))
            (else y
              (tagcase b
                (s-many (x) (sub? a x more))
                (s-typed (x h) (sub? a x more))
                (s-mu (x r p) (sub? a (mx-types-unfold-mu b) more))
                (s-union (ms) (any-sub? a ms more))
                (else z (sub-base? a b more)))))))))
(sub-base? (->* (sty sty assumed) bool)
  (lambda (a b more)
    (tagcase a
      (s-import (x p) (tagcase b (s-import (y q) (and (bind=? p q) (sub? x y more))) (else z #f)))
      (s-export (x p) (tagcase b (s-export (y q) (and (bind=? p q) (sub? x y more))) (else z #f)))
      (s-pair (a1 d1)
        (tagcase b (s-pair (a2 d2) (and (sub? a1 a2 more) (sub? d1 d2 more))) (else z #f)))
      (else y (or (and (= (mx-types-kind-of a) 10) (= (mx-types-kind-of b) 0)) (sty=? a b))))))
(all-sub? (->* (stys sty assumed) bool)
  (lambda (ms b more) (or (null? ms) (and (sub? (car ms) b more) (all-sub? (cdr ms) b more)))))
(any-sub? (->* (sty stys assumed) bool)
  (lambda (a ms more)
    (and (not (null? ms)) (or (sub? a (car ms) more) (any-sub? a (cdr ms) more))))))

;;; ============================================================ macros-env-types.fx (whole)

;; The name of the binder at a node, in a list of one; none if the node is
;; no binder (`macros-nodes.fx`'s `binder-name`).
(define-type namer (subr (maxeff (read @globals) spin (read mxp)) (int) (listof symbol acyclic)))

;;; ============================================================ macros-env-indexed.fx (whole)

(define down-env (->* (mx-env-indexed-env) (tupleof levels mx-env-indexed-groups)) (lambda (e) (tagcase e (env-of (x) x))))
(define env-empty mx-env-indexed-env (env-of (tuple (the levels nil) (the mx-env-indexed-groups nil))))
;; `ix` with binder `b` of name `n` (hash `h`, a description's if `d`),
;; unless it has one of that name and namespace already.
(define ix-kept (->* (index symbol int bool int) index)
  (lambda (ix n h d b)
    (tagcase ix
      (ix-node (k m e x l r)
        (cond ((< h k) (ix-node k m e x (ix-kept l n h d b) r))
              ((and (= h k) (symbol=? n m) (mx-eq? d e)) ix)
              (else (ix-node k m e x l (ix-kept r n h d b)))))
      (else y (ix-node h n d b (ix-leaf) (ix-leaf))))))
;; Each binder of `as` in `ix`, after those there: the first of a name kept.
(define indexed (->* (atms index namer) index)
  (lambda (as ix name)
    (if (null? as)
        ix
        (indexed (cdr as)
                 (tagcase (car as)
                   (atm-lit (b d)
                     (let ((m (name b)))
                       (if (null? m) ix (ix-kept ix (car m) (symbol-name-hash (car m)) d b))))
                   (else y ix))
                 name))))
;; `ls`'s groups, innermost first: the run of ribs to a module's level
;; one group (`ix`, `open` if it has a level yet), each module's its own.
(define mx-env-indexed-grouped (->* (levels index bool namer) mx-env-indexed-groups)
  (lambda (ls ix open name)
    (if (null? ls)
        (if open (list (gr-names ix)) nil)
        (tagcase (car ls)
          (lv-rib (as) (mx-env-indexed-grouped (cdr ls) (indexed as ix name) #t name))
          (lv-earlier (as) (mx-env-indexed-grouped (cdr ls) (indexed as ix name) #t name))
          (lv-cage (c) (mx-env-indexed-grouped (cdr ls) ix open name))
          (lv-shape (m)
            (let ((rest (cons (gr-shape m) (mx-env-indexed-grouped (cdr ls) (ix-leaf) #f name))))
              (if open (cons (gr-names ix) rest) rest)))))))
(define env-within (->* (mx-env-indexed-env levels namer) mx-env-indexed-env)
  (lambda (e ls name)
    (let ((x (down-env e)))
      (env-of (tuple (append ls (extract x 1))
                     (append (mx-env-indexed-grouped ls (ix-leaf) #f name) (extract x 2)))))))
(define env-levels (->* (mx-env-indexed-env) levels) (lambda (e) (extract (down-env e) 1)))
(define any-shape-group? (->* (mx-env-indexed-groups) bool)
  (lambda (gs)
    (and (not (null? gs))
         (or (tagcase (car gs) (gr-shape (m) #t) (else y #f)) (any-shape-group? (cdr gs))))))
(define env-any-shape? (->* (mx-env-indexed-env) bool)
  (lambda (e) (any-shape-group? (extract (down-env e) 2))))
(define ix-find (->* (index symbol int bool) int)
  (lambda (ix n h d)
    (tagcase ix
      (ix-node (k m e x l r)
        (cond ((< h k) (ix-find l n h d))
              ((and (= h k) (symbol=? n m) (mx-eq? d e)) x)
              (else (ix-find r n h d))))
      (else y -1))))
(define mx-env-indexed-dropped (->* (mx-env-indexed-groups int) mx-env-indexed-groups)
  (lambda (gs k) (if (or (= k 0) (null? gs)) gs (mx-env-indexed-dropped (cdr gs) (- k 1)))))
(define looked-in (->* (mx-env-indexed-groups symbol int bool int) looked)
  (lambda (gs n h d k)
    (if (null? gs)
        (tuple -1 -1 k)
        (let ((next (+ k 1)))
          (tagcase (car gs)
            (gr-names (ix)
              (let ((b (ix-find ix n h d)))
                (if (>= b 0) (tuple 0 b next) (looked-in (cdr gs) n h d next))))
            (gr-shape (m) (tuple 1 m next)))))))
(define env-lookup (->* (mx-env-indexed-env symbol bool int namer) looked)
  (lambda (e n d from name)
    (looked-in (mx-env-indexed-dropped (extract (down-env e) 2) from) n (symbol-name-hash n) d from)))

;;; ============================================================ macros-nodes.fx (whole)

(define def-env-top (subr pure () def-env) (lambda () (tuple 0 'top (the mx-parser-types-names nil))))
;; The macros defined, each by the name it is used by, the last defined
;; first (`macros.fx`).
(define defined (ref macro-env mxp) (new nil))
(define macro-of (->* (symbol macro-env) (listof macro acyclic))
  (lambda (n mx-env-indexed-env)
    (cond ((null? mx-env-indexed-env) nil)
          ((symbol=? n (extract (car mx-env-indexed-env) 1)) (list (extract (car mx-env-indexed-env) 2)))
          (else (macro-of n (cdr mx-env-indexed-env))))))
(define macro? (->* (symbol) bool) (lambda (n) (not (null? (macro-of n (get defined))))))
(define makes (->* (symbol) int)
  (lambda (n)
    (let ((m (macro-of n (get defined)))) (if (null? m) -1 (extract (car m) 4)))))
;; Whether `n` is a macro that makes definitions.
(define makes-defns? (->* (symbol) bool) (lambda (n) (= (makes n) 1)))
;; Whether `n` is a macro a top level's form may make definitions with: one
;; that makes them, or a procedural one (`macros-procedural.fx`), which no
;; signature says.
(define makes-top-defns? (->* (symbol) bool) (lambda (n) (or (= (makes n) 1) (= (makes n) 3))))
;; Whether `n` is a macro that makes a description.
(define makes-desc? (->* (symbol) bool) (lambda (n) (= (makes n) 2)))
(define mx-nodes-inserted (->* (string mx-nodes-strings) mx-nodes-strings)
  (lambda (x ys)
    (cond ((null? ys) (list x))
          ((string<? x (car ys)) (cons x ys))
          ((string=? x (car ys)) ys)
          (else (cons (car ys) (mx-nodes-inserted x (cdr ys)))))))
(define mx-nodes-sorted-strings (->* (mx-nodes-strings mx-nodes-strings) mx-nodes-strings)
  (lambda (xs out) (if (null? xs) out (mx-nodes-sorted-strings (cdr xs) (mx-nodes-inserted (car xs) out)))))
(define mx-nodes-joined (->* (mx-nodes-strings) string)
  (lambda (ss)
    (if (null? (cdr ss)) (car ss) (string-append (car ss) (string-append ", " (mx-nodes-joined (cdr ss)))))))
;; Whether kind `k` is a name's: `bvar`, `dvar`, `id`, `label`.
(define name-kind? (->* (int) bool)
  (lambda (k) (or (= k 1) (= k 12) (= k 2) (= k 5))))
(define names-of (->* (stys) stys)
  (lambda (ms)
    (cond ((null? ms) nil)
          ((name-kind? (mx-types-kind-of (unfold (car ms)))) (cons (car ms) (names-of (cdr ms))))
          (else (names-of (cdr ms))))))
;;; ------------------------------------------------------------ matching

;; Of union `t`'s members, the one a name is of, in a list of one, if one
;; is; none else, or if `t` is no union.
(define name-fit-of (->* (sty) stys)
  (lambda (t)
    (tagcase t
      (s-union (ms)
        (let ((mx-parser-types-names (names-of ms))) (if (and (not (null? mx-parser-types-names)) (null? (cdr mx-parser-types-names))) mx-parser-types-names nil)))
      (else y nil))))
(define atom-at (subr pure (int) syn) (lambda (a) (atom '() a a 0)))
;; The nodes, a field's array each, so that a field set is one store (a
;; program's are a million, each set some times: `TODO.md` §239). `node`,
;; as a tuple, is what a node's fields are, in this order.
(define node-syns (ref (arrayof syn mxp) mxp) (new (make-array 256 (atom-at 0))))
(define node-paths (ref (arrayof addr mxp) mxp) (new (make-array 256 (the addr nil))))
(define node-cores (ref (arrayof sty mxp) mxp) (new (make-array 256 (s-nil))))
(define node-importss (ref (arrayof rooted mxp) mxp) (new (make-array 256 (the rooted nil))))
(define node-exports (ref (arrayof rooted mxp) mxp) (new (make-array 256 (the rooted nil))))
(define node-manys (ref (arrayof bool mxp) mxp) (new (make-array 256 #f)))
(define node-kidss (ref (arrayof mx-nodes-kids mxp) mxp) (new (make-array 256 (k-other))))
(define node-holes (ref (arrayof holes mxp) mxp) (new (make-array 256 (the holes nil))))
;; Whether each node is a pending list's (`pending!`), so that a walk asks
;; in one step (`macros.rs`'s `Node::head`).
(define node-heads (ref (arrayof bool mxp) mxp) (new (make-array 256 #f)))
(define nnodes (ref int mxp) (new 0))
(define node-syn (->* (int) syn) (lambda (i) (array-ref (get node-syns) i)))
(define node-path (->* (int) addr) (lambda (i) (array-ref (get node-paths) i)))
(define node-core (->* (int) sty) (lambda (i) (array-ref (get node-cores) i)))
(define node-imports (->* (int) rooted) (lambda (i) (array-ref (get node-importss) i)))
(define node-export (->* (int) rooted) (lambda (i) (array-ref (get node-exports) i)))
(define node-many? (->* (int) bool) (lambda (i) (array-ref (get node-manys) i)))
(define node-kids (->* (int) mx-nodes-kids) (lambda (i) (array-ref (get node-kidss) i)))
(define node-hole (->* (int) holes) (lambda (i) (array-ref (get node-holes) i)))
;; A node's field, set.
(define with-core (->* (int sty) unit) (lambda (i t) (array-set! (get node-cores) i t)))
(define with-kids (->* (int mx-nodes-kids) unit) (lambda (i k) (array-set! (get node-kidss) i k)))
(define with-import (->* (int mx-types-bind int) unit)
  (lambda (i b root)
    (array-set! (get node-importss) i
                (append (node-imports i) (the rooted (list (tuple b root)))))))
(define with-export (->* (int mx-types-bind int) unit)
  (lambda (i b root) (array-set! (get node-exports) i (the rooted (list (tuple b root))))))
(define with-many (->* (int) unit) (lambda (i) (array-set! (get node-manys) i #t)))
(define with-hole (->* (int hole) unit)
  (lambda (i h) (array-set! (get node-holes) i (the holes (list h)))))
;; `a`, in an array twice as long, the rest `fill`.
(define array-grown
  (poly ((t type)) (subr (maxeff (read mxp) (write mxp) (alloc mxp) spin) ((arrayof t mxp) t)
                         (arrayof t mxp)))
  (plambda ((t type))
    (lambda ((a (arrayof t mxp)) (fill t))
      (let ((b (the (arrayof t mxp) (make-array (* 2 (array-length a)) fill))))
        (letrec ((copied (subr (maxeff (read mxp) (write mxp) spin) (int) unit)
                         (lambda (i)
                           (if (= i (array-length a))
                               #u
                               (begin (array-set! b i (array-ref a i)) (copied (+ i 1)))))))
          (begin (copied 0) b))))))
;; Each node's whether `repeated` has it, by its number: asked of every
;; node a walk reaches, where the list would be searched each time. Grown
;; with the nodes (`node-new`), which clears each new node's.
(define repeated-marks (ref (arrayof bool mxp) mxp) (new (make-array 256 #f)))
;; Each field's array, twice as long.
(define nodes-grown! (->* () unit)
  (lambda ()
    (begin
      (set node-syns (array-grown (get node-syns) (atom-at 0)))
      (set node-paths (array-grown (get node-paths) (the addr nil)))
      (set node-cores (array-grown (get node-cores) (s-nil)))
      (set node-importss (array-grown (get node-importss) (the rooted nil)))
      (set node-exports (array-grown (get node-exports) (the rooted nil)))
      (set node-manys (array-grown (get node-manys) #f))
      (set node-kidss (array-grown (get node-kidss) (k-other)))
      (set node-holes (array-grown (get node-holes) (the holes nil)))
      (set repeated-marks (array-grown (get repeated-marks) #f))
      (set node-heads (array-grown (get node-heads) #f)))))
;; A new node, of `s` at `path`, its type `t`: its number.
(define node-new (->* (syn addr sty) int)
  (lambda (s path t)
    (let ((i (get nnodes)))
      (begin
        (if (= i (array-length (get node-syns))) (nodes-grown!) #u)
        (set nnodes (+ i 1))
        (array-set! (get node-syns) i s)
        (array-set! (get node-paths) i path)
        (array-set! (get node-cores) i t)
        (array-set! (get node-importss) i (the rooted nil))
        (array-set! (get node-exports) i (the rooted nil))
        (array-set! (get node-manys) i #f)
        (array-set! (get node-kidss) i (k-other))
        (array-set! (get node-holes) i (the holes nil))
        (array-set! (get repeated-marks) i #f)
        (array-set! (get node-heads) i #f)
        i))))
;; Whether a program's form is being aligned, after expansion (`macros.rs`'s
;; `Mode::Program`), not a template: its binders and references found as
;; the checkers will read them (`binding-shapes.md`); and the names its
;; walk cannot place, written where an address reaches no part (inside a
;; quasiquoted vector, a `load-moduleof`'s arguments).
(define program-walk (ref bool mxp) (new #f))
(define unplaced (ref mx-parser-types-names mxp) (new nil))
;; A program's form read by its scopes (`macros-scopes.fx`'s
;; `program-scoped`; `macros.rs`'s `Aligner::program`, `TODO.md` §295):
;; whether it is, the paths of the lists a macro's name heads read as
;; applications so far, their heads bound around them, and whether any
;; other such list, a use, is aligned no further (where a form is expanded)
;; or by its signature. Each such list pending.
(define scoped-on (ref bool mxp) (new #f))
(define scoped-apps (ref (listof addr acyclic) mxp) (new nil))
(define scoped-whole (ref bool mxp) (new #f))
(define addr-in? (->* (addr (listof addr acyclic)) bool)
  (lambda (p ps) (and (not (null? ps)) (or (mx-types-addr=? p (car ps)) (addr-in? p (cdr ps))))))
;; The nodes an ellipsis follows, so that each is repeated; and, as a walk
;; finds them, each with how many ellipses it is under, itself counted, the
;; last found first (`macros.rs`'s `Node::repeated`, `walk`'s `seqs`).
(define mx-nodes-repeated (ref (listof int acyclic) mxp) (new nil))
(define seq-depths (ref (listof (tupleof int int) acyclic) mxp) (new nil))
;; The lists given by ellipsis whose elements are each in the scope of those
;; before it, a ribcage's (`macros.rs`'s `Kids::Seq`'s flag).
(define cage-seqs (ref (listof int acyclic) mxp) (new nil))
;; The nodes of the descriptions a template writes, whole (`macros.rs`'s
;; `Node::desc`).
(define desc-roots (ref (listof int acyclic) mxp) (new nil))
(define pending (ref (listof (tupleof int bool) acyclic) mxp) (new nil))
;; The nodes forgotten, for another alignment.
(define nodes-reset! (->* () unit)
  (lambda ()
    (begin (set nnodes 0) (set desc-roots nil) (set mx-nodes-repeated nil) (set seq-depths nil)
           (set cage-seqs nil) (set pending nil) (set scoped-on #f))))
(define repeated! (->* (int) unit)
  (lambda (e)
    (begin (set mx-nodes-repeated (cons e (get mx-nodes-repeated))) (array-set! (get repeated-marks) e #t))))
(define mx-nodes-repeated? (->* (int) bool)
  (lambda (i) (array-ref (get repeated-marks) i)))
(define int-member? (->* (int (listof int acyclic)) bool)
  (lambda (i is) (and (not (null? is)) (or (= i (car is)) (int-member? i (cdr is))))))
(define cage-seq? (->* (int) bool)
  (lambda (i) (int-member? i (get cage-seqs))))
(define-rec
;; Whether signature `t` binds nothing, as a template may use one of
;; `scope-forms` (`bloblet-ref`'s, `TODO.md` §247): no `import` or `export`
;; in it, so neither a module's item nor its names (`item`, `(shape A)`),
;; which a program's walk alone knows; a `mu`'s body once, its variable
;; what is around it. As `macros.rs`'s `binds_nothing`.
(binds-nothing? (->* (sty) bool)
  (lambda (t)
    (tagcase t
      (s-item () #f)
      (s-import (b x) #f)
      (s-export (b x) #f)
      (s-pair (a d) (and (binds-nothing? a) (binds-nothing? d)))
      (s-many (b) (binds-nothing? b))
      (s-typed (b h) (binds-nothing? b))
      (s-union (ms) (all-bind-nothing? ms))
      (s-mu (b r p) (binds-nothing? b))
      (else y #t))))
(all-bind-nothing? (->* (stys) bool)
  (lambda (ms)
    (or (null? ms) (and (binds-nothing? (car ms)) (all-bind-nothing? (cdr ms)))))))
(define pvar-of (->* (symbol pvars) (listof pinfo acyclic))
  (lambda (n ps)
    (cond ((null? ps) nil)
          ((symbol=? n (extract (car ps) 1)) (list (extract (car ps) 2)))
          (else (pvar-of n (cdr ps))))))
(define pvar? (->* (symbol pvars) bool) (lambda (n ps) (not (null? (pvar-of n ps)))))
;; 0 if `n` is no pattern variable of `ps`; 1 if it is a name's (`bvar`,
;; `dvar`, `id`, `label`), which is never a list; else 2.
(define pvar-shape (->* (symbol pvars) int)
  (lambda (n ps)
    (let ((p (pvar-of n ps)))
      (cond ((null? p) 0) ((name-kind? (mx-types-kind-of (extract (car p) 1))) 1) (else 2)))))
(define mark-code (->* (symbol macro-env marked) int)
  (lambda (n ds ms) (if (null? (macro-of n ds)) (mark-in n ms) 0)))
;; The trailmark name `n` is bound to, where it is read: its number, or 0
;; (`macros.rs`'s `mark_of`); and so, `ds` the macros and `ms` the
;; trailmarks, for `fits?`, as `pvar-shape`'s are, a trailmark's minus it.
(define mark-now (->* (symbol) int) (lambda (n) (mark-code n (get defined) (get marks))))
(define mx-nodes-union? (->* (sty) bool) (lambda (t) (tagcase t (s-union (ms) #t) (else y #f))))
;;; ------------------------------------------------------------ shapes

;; Whether `s` has the shape of `t`, in `mode` (0 a pattern, 1 a template,
;; 2 a use, 3 a program's form after expansion): a pattern variable in a
;; template (`pvar?` says 1 for a name's, 2 for another's, 0 for none, and
;; of a trailmark's name minus its number, in any mode) fits
;; anything but, if it is a name, a list's shape; any name in a pattern
;; fits anything. No deeper than `fits-depth` lists: a `union`'s members
;; differ within `overlap?`'s depth, and a long list is not looked through
;; at each of its parts (`macros.rs`'s `FITS_DEPTH`).
(define fits-depth int 8)
(define-rec
(fits-within? (->* (sty syn int (subr (read mxp) (symbol) int) int) bool)
  (lambda (t s mode pvar? depth)
    (let ((u (unfold t)) (x (if (>= mode 2) (the syns-a nil) (ellipsis-of s)))
          (program? (= mode 3)))
      (if (not (null? x))
          ;; `(p ...)`, where a list of elements `p` fits may be.
          (let ((e (seq-elem (peel t))))
            (and (not (null? e)) (fits-within? (car e) (car x) mode pvar? depth)))
      (if (and (syn-symbol? s) (< mode 3) (< (pvar? (syn-symbol s)) 0))
          ;; A trailmark, where a use, a pattern or a template writes one:
          ;; itself, a kernel's word of its name, or an `id`.
          (tagcase u
            (s-union (ms) (any-fits? ms s mode pvar? depth))
            (s-mark (n i) (= i (- 0 (pvar? (syn-symbol s)))))
            (s-literal (n) (symbol=? (syn-symbol s) n))
            (s-id () #t)
            (else y #f))
      (if (and (syn-symbol? s)
               (or (and (= mode 0) (not (= (mx-types-kind-of u) 7)) (not (= (mx-types-kind-of u) 15))
                        (not (string=? (syn-name s) "_")) (not (string=? (syn-name s) "...")))
                   (and (= mode 1) (> (pvar? (syn-symbol s)) 0)
                        (not (and (= (pvar? (syn-symbol s)) 1)
                                  (or (= (mx-types-kind-of u) 8) (= (mx-types-kind-of u) 6) (= (mx-types-kind-of u) 10)))))))
          #t
          (tagcase u
            (s-union (ms) (any-fits? ms s mode pvar? depth))
            (s-pair (a d)
              (let ((parts (split s)))
                (and (not (null? parts))
                     (or (= depth 0)
                         (and (fits-within? a (car parts) mode pvar? (- depth 1))
                              (fits-within? d (nth parts 1) mode pvar? (- depth 1)))))))
            (s-nil () (syn-nil? s))
            (s-literal (n) (and (syn-symbol? s) (symbol=? (syn-symbol s) n)))
            (s-mark (n i) (and (syn-symbol? s) (= i (- 0 (pvar? (syn-symbol s))))))
            ;; A program's binder: any name the checkers let it bind.
            (s-bvar () (and (syn-symbol? s) (or program? (not (reserved-syn? s)))))
            (s-dvar () (and (syn-symbol? s) (or program? (not (reserved-syn? s)))))
            (s-id () (syn-symbol? s))
            (s-label () (or (syn-keyword? s) (and (>= mode 2) (syn-symbol? s))
                            (and program? (syn-number? s))))
            (s-expr () (and (or program? (not (reserved-syn? s))) (not (syn-nil? s))))
            (s-lambda () (lambda-shaped? s))
            (s-defn () (not (null? (split s))))
            (else y #t))))))))
(any-fits? (->* (stys syn int (subr (read mxp) (symbol) int) int) bool)
  (lambda (ms s mode pvar? depth)
    (and (not (null? ms))
         (or (fits-within? (car ms) s mode pvar? depth) (any-fits? (cdr ms) s mode pvar? depth))))))
(define mx-nodes-fits? (->* (sty syn int (subr (read mxp) (symbol) int)) bool)
  (lambda (t s mode pvar?) (fits-within? t s mode pvar? fits-depth)))
;; Those of `ms` that `s` fits.
(define mx-nodes-fitting (->* (stys syn int pvars) stys)
  (lambda (ms s mode ps)
    (let ((ds (get defined)) (ks (get marks)))
      (cond ((null? ms) nil)
            ((mx-nodes-fits? (car ms) s mode
                    (lambda ((n symbol))
                      (let ((p (pvar-shape n ps))) (if (> p 0) p (- 0 (mark-code n ds ks))))))
             (cons (car ms) (mx-nodes-fitting (cdr ms) s mode ps)))
            (else (mx-nodes-fitting (cdr ms) s mode ps))))))
;; A pattern variable written somewhere inside `s`, in a list of one.
(define pvar-inside (->* (syn pvars) (listof symbol acyclic))
  (lambda (s ps)
    (if (syn-symbol? s)
        (if (pvar? (syn-symbol s) ps) (list (syn-symbol s)) nil)
        (let ((parts (split s)))
          (if (null? parts)
              nil
              (let ((first (pvar-inside (car parts) ps)))
                (if (null? first) (pvar-inside (nth parts 1) ps) first)))))))
;;; ------------------------------------------------------------ binding

;; The name of the binder at node `b`, in a list of one; none if it is no
;; binder: what an environment is told of its atoms (`namer`).
(define mx-nodes-binder-name (->* (int) (listof symbol acyclic))
  (lambda (b) (tagcase (node-kids b) (k-binder (m e) (list m)) (else y nil))))
;; Each name in `xs`, onto `out`: names a walk leaves unplaced.
(define symbols-onto (->* (syns-a mx-parser-types-names) mx-parser-types-names)
  (lambda (xs out)
    (if (null? xs)
        out
        (symbols-onto (cdr xs)
                      (tagcase (car xs)
                        (mx-eager-reader-types-lst (items d a b x) (symbols-onto items out))
                        (dotted (items tail d a b x) (symbols-onto (cons tail items) out))
                        (vec (items d a b x) (symbols-onto items out))
                        (else y (if (syn-symbol? (car xs)) (cons (syn-symbol (car xs)) out)
                                  out)))))))
;; Whether `s` is `(with #%fx name)` or `(with #%toplevel name)`.
(define standard-with? (->* (syn) bool)
  (lambda (s)
    (tagcase s
      (mx-eager-reader-types-lst (items d a b x)
        (and (= (len items) 3) (syn-symbol? (nth items 1)) (syn-symbol? (nth items 2))
             (or (string=? (syn-name (nth items 1)) "#%fx")
                 (string=? (syn-name (nth items 1)) "#%toplevel"))))
      (else y #f))))
;;; ------------------------------------------------------------ module parameters

;; Address `p` and one step more, a car's (`#t`) or a cdr's.
(define addr+ (->* (addr bool) addr) (lambda (p step) (append p (the addr (list step)))))
(define addr-falses (->* (addr int) addr)
  (lambda (a k) (if (= k 0) a (addr-falses (addr+ a #f) (- k 1)))))
(define kind-form? (->* (syn) bool)
  (lambda (k)
    (let ((parts (proper-items k)))
      (if (not (null? parts))
          (and (syn-symbol? (car parts)) (string=? (syn-name (car parts)) "=>"))
          (and (syn-symbol? k)
               (mx-syntax-name-in? (syn-symbol k)
                         (list 'region 'place 'effect 'type 'data 'sendable 'flat 'size 'conv)))))))
;; Whether module parameter `b` binds a description, `(name kind)` or `(r
;; region p)`; else it is `(m T)` or `(m T qualified)`, a module
;; (`parse::binder_is_kind`).
(define binder-is-kind? (->* (syn) bool)
  (lambda (b)
    (let ((parts (proper-items b)))
      (cond ((= (len parts) 2) (kind-form? (nth parts 1)))
            ((= (len parts) 3)
             (or (kind-form? (nth parts 1))
                 (not (and (syn-symbol? (nth parts 2)) (string=? (syn-name (nth parts 2))
                                                         "qualified")))))
            (else #t)))))
(define parameters-list (->* (syns-a) sty)
  (lambda (ps)
    (if (null? ps)
        (s-nil)
        (s-pair (s-union (list (if (binder-is-kind? (car ps))
                                   (s-export (s-pair (s-dvar) (s-data))
                                             (bind-rib (list (tuple (list #t) 4))))
                                   (s-export (s-pair (s-bvar) (s-pair (s-type) (s-data)))
                                             (bind-rib (list (tuple (list #t) 1)))))))
                (parameters-list (cdr ps))))))
(define parameter-entries (->* (syns-a int) entries)
  (lambda (ps k)
    (if (null? ps)
        nil
        (cons (tuple (cons #t (addr+ (addr-falses nil k) #t)) 2)
              (parameter-entries (cdr ps) (+ k 1))))))
(define parameter-shapes (->* (syns-a int) rings)
  (lambda (ps k)
    (cond ((null? ps) nil)
          ((or (binder-is-kind? (car ps)) (not (= (len (proper-items (car ps))) 2)))
           (parameter-shapes (cdr ps) (+ k 1)))
          (else (cons (ring-shape (cons #t (addr+ (addr+ (addr-falses nil k) #t) #t)))
                      (parameter-shapes (cdr ps) (+ k 1)))))))
;; The binding structure of a program's `module-parameters`' list `bs`:
;; each a value's binder, its type in the scope of those before it, or a
;; description's; each module not `qualified` included in the items, as the
;; checkers make the file (`macros.rs`'s `parameters_sig`).
(define parameters-sig (->* (syn) sty)
  (lambda (bs)
    (let ((ps (proper-items bs)))
      (s-export (s-pair (parameters-list ps) (s-nil))
                (bind-cage (cons (ring-rib (parameter-entries ps 0)) (parameter-shapes ps 0)))))))
(define sig-of-name (->* (symbol mx-types-named) (listof sty acyclic))
  (lambda (n ts)
    (cond ((null? ts) nil) ((symbol=? n (extract (car ts) 1)) (list (extract (car ts) 2)))
          (else (sig-of-name n (cdr ts))))))
;;; ------------------------------------------------------------ a defined macro's words

;; Whether `n`, in a macro's transformer a template writes, is a word, not a
;; name it refers to: `_`, a form's or a description's word, a macro's
;; definition's, a macro's or a trailmark's name (`Aligner::rules`).
(define rules-word? (->* (fails symbol) bool)
  (lambda (mx-fail n)
    (let ((name (symbol->string n)))
      (or (string=? name "_") (reserved? name)
          (not (null? (sig-of-name n (kernel-sigs mx-fail))))
          (not (null? (sig-of-name n (kernel-definitions mx-fail))))
          (desc-word? name #t) (type-form? name)
          (mx-syntax-name-in? n (list 'else '=> 'quote 'quasiquote 'unquote 'unquote-splicing 'syntax-rules
                            'trailmark 'er-macro-transformer 'sc-macro-transformer
                            'rsc-macro-transformer))
          (macro? n) (> (mark-now n) 0)))))
;; A list's items, a dotted one's before its tail; none of another.
(define rules-items (->* (syn) syns-a)
  (lambda (s)
    (tagcase s (mx-eager-reader-types-lst (items d a b x) items) (dotted (items t d a b x) items) (else y nil))))
(define any-ellipsis? (->* (syns-a) bool)
  (lambda (xs) (and (not (null? xs)) (or (ellipsis? (car xs)) (any-ellipsis? (cdr xs))))))
(define rules-dots string
  (string-append "`...` in a macro a template defines is the template's (R7RS), and repeats "
                 "nothing here: the defined macro's is written `(... ...)`"))
(define rules-procedural string
  (string-append "a template's `define-syntax` defines a `syntax-rules` macro: a procedural "
                 "one's transformer is phase 1's code, not yet in a template"))
;; `define-syntax`'s operands' binding structure, with a signature if
;; `signed`: `(export (bvar [data] rules) (rho (A var)))`.
(define define-syntax-sig (->* (bool) sty)
  (lambda (signed)
    (let ((rest (s-pair (s-rules) (s-nil))))
      (s-export (s-pair (s-bvar) (if signed (s-pair (s-data) rest) rest))
                (bind-rib (list (tuple (list #t) 1)))))))
;; A `define-syntax` with a signature in a transformer a template writes:
;; its signature a datum.
(define rules-define-sig (->* () sty)
  (lambda ()
    (s-pair (s-rules) (s-pair (s-rules) (s-pair (s-data) (s-pair (s-rules) (s-nil)))))))
;; Whether `s` is a `define-syntax` with a signature.
(define rules-define? (->* (syn) bool)
  (lambda (s)
    (let ((items (proper-items s)))
      (and (= (len items) 4) (syn-symbol? (car items))
           (string=? (syn-name (car items)) "define-syntax")))))
(define rules-escape string "`(... template)` other than `(... ...)`: not yet")
;;; ------------------------------------------------------------ forms' words

(define sig-of (->* (symbol mx-types-named) stys)
  (lambda (n mx-types-named)
    (cond ((null? mx-types-named) nil)
          ((symbol=? n (extract (car mx-types-named) 1)) (list (extract (car mx-types-named) 2)))
          (else (sig-of n (cdr mx-types-named))))))
(define percent-found (ref bool mxp) (new #f))
(define percent-names (ref mx-parser-types-names mxp) (new nil))
(define percent-of (->* (mx-types-named) mx-parser-types-names)
  (lambda (ns)
    (cond ((null? ns) nil)
          ((let ((h (symbol->string (extract (car ns) 1))))
             (and (> (string-length h) 0) (string=? (substring h 0 1) "%")))
           (cons (extract (car ns) 1) (percent-of (cdr ns))))
          (else (percent-of (cdr ns))))))
;; The names of `kernel`'s forms that start with `%`, found once.
(define percent-named (->* (mx-types-named) mx-parser-types-names)
  (lambda (kernel)
    (begin
      (if (get percent-found) #u
          (begin (set percent-names (percent-of kernel)) (set percent-found #t)))
      (get percent-names))))
(define-rec
;; Whether one of `xs` has an `unquote` or `unquote-splicing` in it.
(any-unquotes? (->* (syns-a) bool)
  (lambda (xs) (and (not (null? xs)) (or (unquotes? (car xs)) (any-unquotes? (cdr xs))))))
(unquotes? (->* (syn) bool)
  (lambda (s)
    (tagcase s
      (mx-eager-reader-types-lst (items d a b x) (any-unquotes? items))
      (dotted (items tail d a b x) (or (any-unquotes? items) (unquotes? tail)))
      (vec (items d a b x) (any-unquotes? items))
      (else y (and (syn-symbol? s) (or (string=? (syn-name s) "unquote")
                                       (string=? (syn-name s) "unquote-splicing"))))))))
;; Whether a list headed by `hd` constructs: a keyword, or a pattern
;; variable of a `label`.
(define constructs? (->* (syn pvars) bool)
  (lambda (hd ps)
    (or (syn-keyword? hd)
        (and (syn-symbol? hd) (pvar? (syn-symbol hd) ps)
             (= (mx-types-kind-of (extract (car (pvar-of (syn-symbol hd) ps)) 1)) 5)))))
(define-rec
;; Whether `s` is a definition, by its head: a `begin` one if all it holds
;; are (`macros.rs`'s `defines`).
(defines? (->* (fails syn) bool)
  (lambda (mx-fail s)
    (let* ((items (proper-items s))
           (head (if (and (not (null? items)) (syn-symbol? (car items))) (syn-name (car items))
                   "")))
      (cond ((string=? head "") #f)
            ((string=? head "begin") (and (not (null? (cdr items))) (all-define? mx-fail (cdr items))))
            (else (let ((n (syn-symbol (car items))))
                    (or (not (null? (sig-of n (kernel-definitions mx-fail))))
                        (not (null? (sig-of n (scope-definitions mx-fail)))))))))))
(all-define? (->* (fails syns-a) bool)
  (lambda (mx-fail xs) (or (null? xs) (and (defines? mx-fail (car xs)) (all-define? mx-fail (cdr xs)))))))
;;; ------------------------------------------------------------ heads read by scope

;; Whether the template being checked uses a macro whose uses define a macro
;; (`Aligner::heads`); how each list read so far as a use of one is, by its
;; place, with that `(macro σ made)` entry's δ; and the lists found pending,
;; each with whether a definition is there, the first found last
;; (`Aligner::pending`). `macros-scopes.fx`'s `aligned-heads` reads them.
(define heads-on (ref bool mxp) (new #f))
(define heads (ref (listof (tupleof int int int) acyclic) mxp) (new nil))
(define pending! (->* (int bool) unit)
  (lambda (at defn)
    (begin (array-set! (get node-heads) at #t) (set pending (cons (tuple at defn) (get pending))))))
(define pending-node? (->* (int) bool) (lambda (at) (array-ref (get node-heads) at)))
(define head-at (->* (int int (listof (tupleof int int int) acyclic)) (listof int acyclic))
  (lambda (a b hs)
    (cond ((null? hs) nil)
          ((and (= a (extract (car hs) 1)) (= b (extract (car hs) 2))) (list (extract (car hs) 3)))
          (else (head-at a b (cdr hs))))))
;; How list `s` is read so far: the δ of the `(macro σ made)` entry whose macro
;; it is a use of, in a list of one; none, an application.
(define head-read (->* (syn) (listof int acyclic))
  (lambda (s) (head-at (syn-start s) (syn-end s) (get heads))))
;; Each `(macro σ made)` entry's signature, by its δ, read where the signature it
;; is in is (`macros-files.fx`'s `signature-of`): its syntax type, and what
;; it makes.
(define syntax-read (ref (listof (tupleof int sty int) acyclic) mxp) (new nil))
(define syntax-read! (->* (int sty int) unit)
  (lambda (d t made) (set syntax-read (cons (tuple d t made) (get syntax-read)))))
(define syntax-read-in
  (->* (int (listof (tupleof int sty int) acyclic)) (listof (tupleof sty int) acyclic))
  (lambda (d xs)
    (cond ((null? xs) nil)
          ((= d (extract (car xs) 1)) (list (tuple (extract (car xs) 2) (extract (car xs) 3))))
          (else (syntax-read-in d (cdr xs))))))
(define syntax-sig-read (->* (int) (listof (tupleof sty int) acyclic))
  (lambda (d) (syntax-read-in d (get syntax-read))))
;; Whether `h`, at a list's head in a template, may be a name a `(macro σ made)`
;; entry binds: no macro's, form's or trailmark's, and no pattern
;; variable's but a binder's (`Aligner::unknown_head`).
(define unknown-head? (->* (fails symbol pvars) bool)
  (lambda (mx-fail h ps)
    (if (pvar? h ps)
        (= (mx-types-kind-of (extract (car (pvar-of h ps)) 1)) 1)
        (not (or (reserved? (symbol->string h)) (macro? h) (> (mark-now h) 0)
                 (not (null? (sig-of h (kernel-sigs mx-fail))))
                 (not (null? (sig-of h (kernel-definitions mx-fail))))
                 (not (null? (sig-of h (scope-sigs mx-fail))))
                 (not (null? (sig-of h (scope-definitions mx-fail)))))))))
(define-rec
;; Whether template `s` uses a macro whose signature says its uses define a
;; macro, `(ℓ (macro σ made))` (`Macros::uses_syntax_macro`).
(uses-syntax-macro? (->* (syn) bool)
  (lambda (s)
    (let ((items (proper-items s)))
      (and (not (null? items))
           (or (and (syn-symbol? (car items)) (macro? (syn-symbol (car items)))
                    (tagcase (extract (car (macro-of (syn-symbol (car items)) (get defined))) 1)
                      (s-export (b x) (tagcase x (bind-rib (es) (syntax-entry? es)) (else y #f)))
                      (else y #f)))
               (any-uses-syntax-macro? items))))))
(any-uses-syntax-macro? (->* (syns-a) bool)
  (lambda (xs)
    (and (not (null? xs)) (or (uses-syntax-macro? (car xs)) (any-uses-syntax-macro? (cdr xs)))))))
(define logic-word? (->* (string) bool)
  (lambda (h)
    (or (string=? h "<") (string=? h "<=") (string=? h "=") (string=? h ">") (string=? h ">=")
        (string=? h "not") (string=? h "and") (string=? h "or") (string=? h "implies")
        (string=? h "+") (string=? h "-") (string=? h "*"))))
(define-rec
;; The nodes under node `at`, itself first.
(under (->* (int) (listof int acyclic))
  (lambda (at)
    (cons at (tagcase (node-kids at)
               (k-pair (x y) (append (under x) (under y)))
               (k-form (x) (under x))
               (mx-nodes-k-seq (x) (under x))
               (k-desc (xs) (under-each xs))
               (else y (the (listof int acyclic) nil))))))
(under-each (->* ((listof int acyclic)) (listof int acyclic))
  (lambda (xs) (if (null? xs) nil (append (under (car xs)) (under-each (cdr xs)))))))
;; The `i`th of nodes `ks`.
(define kid (->* ((listof int acyclic) int) int)
  (lambda (ks i) (if (= i 0) (car ks) (kid (cdr ks) (- i 1)))))
(define len-kids (->* ((listof int acyclic)) int)
  (lambda (ks) (if (null? ks) 0 (+ 1 (len-kids (cdr ks))))))

;;; ============================================================ macros-align.fx (whole)

;; Node path `p` a step further: in a program's walk, kept innermost step
;; first, so that a step costs one pair however deep (a program's lists may
;; be long); a template's, in order. A signature's address, `addr+`.
(define path+ (->* (addr bool) addr)
  (lambda (p step) (if (get program-walk) (cons step p) (addr+ p step))))
;; A pattern variable at `e` where elements of more than one shape may be:
;; the shape it fits, and what that exports.
(define seq-member (->* (pvars int sty syn int) unit)
  (lambda (ps e elem x mode)
    (tagcase elem
      (s-union (ms)
        (let ((fit (mx-nodes-fitting ms x mode ps)))
          (if (and (not (null? fit)) (null? (cdr fit)))
              (begin (tagcase (car fit) (s-export (b bd) (with-export e bd e)) (else y #u))
                     (with-core e (peel (car fit))))
              #u)))
      (else y #u))))
;; A binder at `at`, of a description if `d`, else of a value: a macro's name
;; a use's or a program's, which it shadows, not a template's yet (§295).
(define fill-binder (->* (fails int int syn bool) unit)
  (lambda (mx-fail mode at s d)
    (cond ((and (syn-symbol? s) (= mode 1) (macro? (syn-symbol s)))
           (fail-at mx-fail (string-append "`" (string-append (syn-name s) "` is a macro's name, which is not bound"))
                    s))
          ;; A program's binder: any name the checkers let it bind, which they
          ;; decide.
          ((and (syn-symbol? s) (= mode 3)) (with-kids at (k-binder (syn-symbol s) d)))
          ((and (syn-symbol? s) (> (mark-now (syn-symbol s)) 0))
           (fail-at mx-fail (string-append "`" (string-append (syn-name s) "` is a trailmark, which is not bound")) s))
          ;; A value's binder a use gives: as a program's, any name the
          ;; checkers let it bind, a keyword too (`fields`, Heist's), which a
          ;; template's form, read by its head, does not see bound.
          ((and (syn-symbol? s) (= mode 2) (not d)) #u)
          ((and (syn-symbol? s) (not (reserved-syn? s)) (not (and d (desc-word? (syn-name s) #f))))
           (if (= mode 1) (with-kids at (k-binder (syn-symbol s) d)) #u))
          (else (fail-at mx-fail (string-append "`" (string-append (show-syn s) "` is not a name that may be bound"))
                         s)))))
(define not-definition (->* (fails syn string) unit)
  (lambda (mx-fail s what)
    (fail-at mx-fail (string-append "`" (string-append (show-syn s) (string-append "` is not a definition: a macro that makes "
                                 (string-append "definitions makes " what))))
             s)))
(define definitions-said string
  (string-append "`define`, `define*`, `define-rec`, a `begin` of them, or a use of another "
                 "such"))
;; A macro's use at `at`, where a form is expanded: its parts aligned no
;; further; where a definition is, defining nothing the form's walk sees.
(define use-whole! (->* (int bool) unit)
  (lambda (at defn)
    (begin (with-kids at (k-other)) (if defn (with-export at (bind-rib nil) at) #u))))
(define mx-align-falses (->* (addr int) addr)
  (lambda (path k) (if (= k 0) path (mx-align-falses (path+ path #f) (- k 1)))))
;; Whether nodes of `mode` are written, their binders and references found:
;; a template's, a program's.
(define writes? (->* (int) bool) (lambda (mode) (or (= mode 1) (= mode 3))))
(define desc-kids-from (->* (syns-a addr (listof int acyclic)) (listof int acyclic))
  (lambda (items path out)
    (if (null? items)
        (the (listof int acyclic) (reverse out))
        (let ((k (node-new (car items) (path+ path #t) (s-type))))
          (desc-kids-from (cdr items) (path+ path #f) (cons k out))))))
;; A node for each of `items`, the parts of the list at node `at`, at their
;; paths in it: `at`'s kids.
(define desc-kids (->* (int syns-a) (listof int acyclic))
  (lambda (at items)
    ;; `p ...` last: `p` repeated, the `...` no part of its own.
    (let* ((n (len items)) (again (and (>= n 2) (ellipsis? (nth items (- n 1)))))
           (mx-nodes-kids (desc-kids-from (if again (the syns-a (reverse (cdr (reverse items)))) items)
                                 (node-path at) nil)))
      (begin
        (if (and again (not (null? mx-nodes-kids)))
            (repeated! (car (the (listof int acyclic) (reverse mx-nodes-kids))))
            #u)
        (with-kids at (k-desc mx-nodes-kids))
        mx-nodes-kids))))
;; Binder `n` at node `at`, a description's if `d`, else a value's.
(define name-binder (->* (fails pvars int syn bool) atm)
  (lambda (mx-fail ps at n d)
    (begin
      (cond ((and (syn-symbol? n) (pvar? (syn-symbol n) ps))
             (begin (with-kids at (k-pvar (syn-symbol n))) (atm-var (syn-symbol n) d 0)))
            ((and (syn-symbol? n)
                  (or (get program-walk)
                      (and (not (reserved-syn? n)) (not (desc-word? (syn-name n) #f)))))
             (begin (with-kids at (k-binder (syn-symbol n) d)) (atm-lit at d)))
            (else (begin (fail-at mx-fail (string-append "`" (string-append (show-syn n) (string-append "` is not a name that may "
                                                      "be bound")))
                                  n)
                         (atm-lit at d)))))))
;; A description's binder `n`, at node `at`: a pattern variable (a `dvar`
;; the use gives), or a name the template binds; what it binds.
(define desc-binder (->* (fails pvars int syn) atm)
  (lambda (mx-fail ps at n) (begin (with-core at (s-dvar)) (name-binder mx-fail ps at n #t))))
(define dletrec-parts (->* (fails pvars (listof int acyclic) syns-a) dparts)
  (lambda (mx-fail ps ks es)
    (if (or (null? es) (null? ks))
        nil
        (let ((parts (proper-items (car es))))
          (if (= (len parts) 2)
              (let* ((p (desc-kids (car ks) parts)) (a (desc-binder mx-fail ps (car p) (car parts))))
                (cons (tuple a (kid p 1) (nth parts 1)) (dletrec-parts mx-fail ps (cdr ks) (cdr es))))
              (begin (fail-at mx-fail "`(name description)`" (car es)) (the dparts nil)))))))
(define dparts-atms (->* (dparts) atms)
  (lambda (ds) (if (null? ds) nil (cons (extract (car ds) 1) (dparts-atms (cdr ds))))))
(define mx-align-spec-clause? (->* (syn) bool)
  (lambda (c)
    (let ((parts (proper-items c)))
      (and (not (null? parts)) (syn-symbol? (car parts))
           (or (string=? (syn-name (car parts)) "requires")
               (string=? (syn-name (car parts)) "ensures"))))))
(define clauses-at-end (->* (syns-a) int)
  (lambda (rs)
    (if (and (not (null? rs)) (mx-align-spec-clause? (car rs))) (+ 1 (clauses-at-end (cdr rs))) 0)))
;; A value's binder `n` a description writes (a procedure type's parameter
;; named, its result's name), at node `at`: a pattern variable (a `bvar` the
;; use gives), or a name the template binds; what it binds.
(define value-binder (->* (fails pvars int syn) atm)
  (lambda (mx-fail ps at n) (begin (with-core at (s-bvar)) (name-binder mx-fail ps at n #f))))
;; Names of values at nodes `ks`, each the top level's if free (`free` 1)
;; or as written (2), or a pattern variable placed.
(define desc-values (->* (fails pvars (listof int acyclic) int) unit)
  (lambda (mx-fail ps ks free)
    (if (null? ks)
        #u
        (let ((s (node-syn (car ks))))
          (if (syn-symbol? s)
              (begin
                (with-core (car ks) (s-id))
                (with-kids (car ks) (if (pvar? (syn-symbol s) ps)
                                        (k-pvar (syn-symbol s))
                                        (k-ref (syn-symbol s) #f free)))
                (desc-values mx-fail ps (cdr ks) free))
              (fail-at mx-fail "a name" s))))))
(define-rec
;; A specification's proposition or integer `s`, at node `at`
;; (`macros.rs`'s `spec`): each name a value's, a parameter's or the
;; result's, else the top level's (which the checkers refuse there); a
;; call's head a global procedure's, written as it is, as a `(globals …)`'s
;; names are; the logic's words, numbers, `#t` and `#f` as they are.
(spec (->* (fails pvars int syn) unit)
  (lambda (mx-fail ps at s)
    (tagcase s
      (mx-eager-reader-types-lst (items d a b x)
        (if (null? items)
            #u
            (let ((mx-nodes-kids (desc-kids at items)) (h (car items)))
              (begin
                (cond ((and (syn-symbol? h) (not (pvar? (syn-symbol h) ps))
                            (logic-word? (syn-name h)))
                       #u)
                      ((syn-symbol? h) (desc-values mx-fail ps (list (car mx-nodes-kids)) 2))
                      (else (fail-at mx-fail "a specification's proposition or integer" s)))
                (spec-each mx-fail ps (cdr mx-nodes-kids) (cdr items))))))
      (dotted (items tail d a b x)
        (fail-at mx-fail "a specification's proposition is a list, not a dotted one" s))
      (else y (if (syn-symbol? s) (desc-values mx-fail ps (list at) 1) #u)))))
(spec-each (->* (fails pvars (listof int acyclic) syns-a) unit)
  (lambda (mx-fail ps ks xs)
    (if (or (null? ks) (null? xs))
        #u
        (begin (spec mx-fail ps (car ks) (car xs)) (spec-each mx-fail ps (cdr ks) (cdr xs)))))))
(define desc-clause (->* (fails pvars int syn atms) unit)
  (lambda (mx-fail ps at c as)
    (let* ((parts (proper-items c)) (q (desc-kids at parts)) (head (syn-name (car parts))))
      (cond ((and (string=? head "requires") (= (len-kids q) 2))
             (begin (with-import (kid q 1) (bind-direct as) (kid q 1))
                    (spec mx-fail ps (kid q 1) (nth parts 1))))
            ((and (string=? head "ensures") (= (len-kids q) 3))
             (let ((a (value-binder mx-fail ps (kid q 1) (nth parts 1))))
               (begin (with-import (kid q 2) (bind-direct (append as (the atms (list a))))
                                   (kid q 2))
                      (spec mx-fail ps (kid q 2) (nth parts 2)))))
            (else (fail-at mx-fail (string-append "a specification is `(requires proposition)`, "
                                               "then `(ensures name proposition)`")
                           c))))))
;; A `select`'s module at node `at`: a name, as `desc-values` reads one, or a
;; path to a module, `(with m x)`, its base `m` read as a `select`'s module
;; is and `x` the label of a value of it (`macros.rs`'s `select_path`).
(define select-path (->* (fails pvars int syn) unit)
  (lambda (mx-fail ps at s)
    (if (syn-symbol? s)
        (desc-values mx-fail ps (list at) 1)
        (let ((items (proper-items s)))
          (if (and (= (len items) 3) (syn-symbol? (car items))
                   (string=? (syn-name (car items)) "with") (syn-symbol? (nth items 2)))
              (select-path mx-fail ps (kid (desc-kids at items) 1) (nth items 1))
              (fail-at mx-fail "a module's name, or a path to one, `(with module name)`" s))))))
(define entry-binders (->* ((listof int acyclic)) atms)
  (lambda (es)
    (if (null? es)
        nil
        (let ((rest (entry-binders (cdr es))))
          (tagcase (node-kids (car es))
            (k-desc (parts)
              (tagcase (node-kids (car parts))
                (k-pvar (n) (cons (atm-var n #t 0) rest))
                (k-binder (n d) (cons (atm-lit (car parts) #t) rest))
                (else y rest)))
            (else y rest))))))
;; The binders of the `poly`s at the top of the description at node `at`,
;; outermost first (`Checker::bind_signature`'s).
(define mx-align-poly-binders (->* (int pvars) atms)
  (lambda (at ps)
    (let ((items (proper-items (node-syn at))))
      (tagcase (node-kids at)
        (k-desc (mx-nodes-kids)
          (if (and (= (len items) 3) (syn-symbol? (car items))
                   (string=? (syn-name (car items)) "poly")
                   (not (pvar? (syn-symbol (car items)) ps)))
              (append (entry-binders (tagcase (node-kids (kid mx-nodes-kids 1))
                                       (k-desc (es) es)
                                       (else y (the (listof int acyclic) nil))))
                      (mx-align-poly-binders (kid mx-nodes-kids 2) ps))
              (the atms nil)))
        (else y (the atms nil))))))
(define-rec
;;; ------------------------------------------------------------ aligning

;; `s`, at `path`, aligned with type `t`, in `mode` (0 a pattern, 1 a
;; template, 2 a use); `root`, the node its addresses are from, if not
;; itself (-1): its node.
(align (->* (fails int pvars sty syn int addr) int)
  (lambda (mx-fail mode ps t s root path)
    (let ((at (node-new s path t)))
      (begin (fill mx-fail mode ps at t s (if (< root 0) at root)) at))))
(fill (->* (fails int pvars int sty syn int) unit)
  (lambda (mx-fail mode ps at t s root)
    (tagcase t
      (s-import (body b) (begin (with-import at b root) (fill mx-fail mode ps at body s root)))
      (s-many (body) (begin (with-many at) (fill mx-fail mode ps at body s root)))
      (s-typed (body h) (begin (with-hole at h) (fill mx-fail mode ps at body s root)))
      (s-export (body b)
        (if (null? (node-export at))
            (begin (with-export at b root) (fill mx-fail mode ps at body s root))
            (fail-at mx-fail "two exports at one part" s)))
      (else y (fill-core mx-fail mode ps at t s root)))))
(fill-core (->* (fails int pvars int sty syn int) unit)
  (lambda (mx-fail mode ps at t s root)
    (begin
      (with-core at t)
      ;; A pattern's variable, or one placed in a template: at any type.
      (cond ((and (= mode 0) (syn-symbol? s) (> (mark-now (syn-symbol s)) 0)
                  (not (or (= (mx-types-kind-of (unfold t)) 15) (mx-nodes-union? (unfold t)))))
             (fail-at mx-fail (string-append "`" (string-append (syn-name s) (string-append "` is a trailmark, which a pattern "
                                          (string-append "matches where its signature writes `(trailmark "
                                          (string-append (syn-name s) ")`, and no pattern variable")))))
                      s))
            ((and (syn-symbol? s)
                  (or (and (= mode 0) (not (= (mx-types-kind-of (unfold t)) 7))
                           (not (= (mx-types-kind-of (unfold t)) 15)) (= (mark-now (syn-symbol s)) 0)
                           (not (string=? (syn-name s) "_")) (not (ellipsis? s)))
                      (and (= mode 1) (pvar? (syn-symbol s) ps))))
             ;; A pattern's variable where a name or a list may be: the
             ;; name's shape, if one member is a name's.
             (let ((one (if (= mode 0) (name-fit-of (unfold t)) (the stys nil))))
               (if (null? one)
                   (with-kids at (k-pvar (syn-symbol s)))
                   (fill mx-fail mode ps at (car one) s root))))
            ((= (mx-types-kind-of t) 16) (fill-rules mx-fail mode ps at s root))
            ((and (< mode 2) (not (null? (ellipsis-of s))))
             (fill-seq mx-fail mode ps at t s (car (ellipsis-of s))))
            ((and (< mode 2) (ellipsis? s))
             (fail-at mx-fail "`...` follows the part it repeats, last in a list" s))
            (else (fill-shape mx-fail mode ps at t s root))))))
;; Transformer `s` a template writes, at `at` (`s-rules`, `Aligner::rules`):
;; each name a reference, but a word (`rules-word?`); a list, its parts so;
;; `(... ...)` the defined macro's ellipsis, a quoted datum as it is.
;; Nothing of it in another mode, where it is the defined macro's alone.
(fill-rules (->* (fails int pvars int syn int) unit)
  (lambda (mx-fail mode ps at s root)
    (let ((items (rules-items s)))
      (cond ((not (= mode 1)) #u)
            ((syn-symbol? s)
             (begin (with-many at)
                    (cond ((ellipsis? s) (fail-at mx-fail rules-dots s))
                          ((rules-word? mx-fail (syn-symbol s)) #u)
                          (else (with-kids at (k-ref (syn-symbol s) #f 4))))))
            ((syn-nil? s) (begin (with-many at) (with-kids at (k-nil))))
            ((null? items) (with-many at))
            ((and (= (len items) 2) (proper? s) (ellipsis? (car items)) (ellipsis? (nth items 1)))
             (with-many at))
            ((ellipsis? (car items)) (fail-at mx-fail rules-escape s))
            ((any-ellipsis? items) (fail-at mx-fail rules-dots s))
            ((and (syn-symbol? (car items)) (string=? (syn-name (car items)) "quote"))
             (begin (with-many at) (fill mx-fail mode ps at (s-data) s root)))
            ((rules-define? s)
             (begin (with-many at) (fill mx-fail mode ps at (rules-define-sig) s root)))
            (else (begin (with-many at)
                         (fill mx-fail mode ps at (s-pair (s-rules) (s-rules)) s root)))))))
;; `(x ...)` at `at`, a pattern's or a template's: `x` aligned with the
;; element type of list type `t`, repeated; the list exports what each
;; element does.
(fill-seq (->* (fails int pvars int sty syn syn) unit)
  (lambda (mx-fail mode ps at t s x)
    (let ((elem (seq-elem t)))
      (if (null? elem)
          (fail-at mx-fail (string-append "`" (string-append (show-syn s) (string-append "` repeats a part by `...` where the form "
                                       "takes no list of them")))
                   s)
          (let ((e (align mx-fail mode ps (car elem) x -1 (path+ (node-path at) #t))))
            (begin
              (repeated! e)
              (tagcase (node-kids e) (k-pvar (n) (seq-member ps e (car elem) x mode)) (else y #u))
              (with-kids at (mx-nodes-k-seq e))
              (if (seq-cage? t) (set cage-seqs (cons at (get cage-seqs))) #u)
              (let ((ex (node-export e)))
                (if (or (not (null? (node-export at))) (null? ex))
                    #u
                    (with-export at (extract (car ex) 1) (extract (car ex) 2))))))))))
(fill-shape (->* (fails int pvars int sty syn int) unit)
  (lambda (mx-fail mode ps at t s root)
    (let ((said (lambda ((why string)) (fail-at mx-fail why s)))
          ;; Written only if refused: `s` may be large.
          (shown (the (subr (read mxp) () string) (lambda () (string-append "`" (string-append (show-syn s) "`"))))))
      (tagcase t
        (s-mu (b r p) (fill mx-fail mode ps at (mx-types-unfold-mu t) s at))
        (s-union (ms)
          ;; A program's name that a form's word may be too, `else` or a
          ;; variable bound to `else`: the form's word, if a member writes it.
          (let* ((all (mx-nodes-fitting ms s mode ps))
                 (fit (if (and (= mode 3) (not (null? all)) (not (null? (cdr all))))
                          (mx-nodes-fitting all s 2 ps)
                          all)))
            (cond ((null? fit) (said (string-append (shown) (string-append " is of none of the shapes this part "
                                                    "may have"))))
                  ((null? (cdr fit)) (fill mx-fail mode ps at (car fit) s at))
                  (else (said (string-append (shown) (string-append " could be of more than one of this part's "
                                             "shapes")))))))
        (s-pair (a d)
          (let ((parts (split s)))
            (if (null? parts)
                (said (string-append (shown) " is not a list as this part must be"))
                ;; A use's paths none: only its fit is wanted (TODO.md §231).
                (let* ((path (node-path at)) (no (= mode 2))
                       (x (align mx-fail mode ps a (car parts) root (if no nil (path+ path #t))))
                       (y (align mx-fail mode ps d (nth parts 1) root (if no nil (path+ path #f)))))
                  (with-kids at (k-pair x y))))))
        (s-nil ()
          (if (syn-nil? s)
              (with-kids at (k-nil))
              (said (string-append (shown) " is more than this part may have"))))
        (s-literal (n)
          (if (and (syn-symbol? s) (symbol=? (syn-symbol s) n))
              #u
              (said (string-append (shown) (string-append " where `" (string-append (symbol->string n) "` is written"))))))
        ;; A trailmark: a name bound to this one, where it is read.
        (s-mark (n i)
          (if (and (syn-symbol? s) (= (mark-now (syn-symbol s)) i))
              #u
              (said (string-append (shown) (string-append " where the trailmark `" (string-append (symbol->string n)
                                   "` is written"))))))
        (s-bvar () (fill-binder mx-fail mode at s #f))
        (s-dvar () (fill-binder mx-fail mode at s #t))
        (s-kind () #u)
        (s-id ()
          (if (syn-symbol? s)
              (if (writes? mode) (with-kids at (k-ref (syn-symbol s) #f 0)) #u)
              (said (string-append (shown) " is not a variable"))))
        (s-label ()
          (cond ((syn-keyword? s) #u)
                ((and (syn-symbol? s) (>= mode 2)) #u)
                ;; A program's field by position.
                ((and (= mode 3) (syn-number? s)) #u)
                ((syn-symbol? s)
                 (said (string-append "a tag or label in a template is a keyword: `#:"
                                      (string-append (syn-name s) (string-append "`, not `" (string-append (syn-name s) "`"))))))
                (else (said (string-append (shown) " is not a tag or label")))))
        ;; A template writes a lambda (its form aligned as any of the kernel's);
        ;; a use gives one, of the shape the checkers take.
        (s-lambda ()
          (cond ((= mode 3) (fill mx-fail mode ps at (s-expr) s root))
                ((and (= mode 1) (not (null? (split s))) (syn-symbol? (car (split s)))
                      (string=? (syn-name (car (split s))) "lambda") (proper? s))
                 (mx-align-form mx-fail ps at s #f))
                ((and (= mode 2) (lambda-shaped? s)) #u)
                (else (said (string-append (shown) " is not a lambda, as this part must be")))))
        (s-expr ()
          (cond ((= mode 3) (program-expr mx-fail ps at s))
                ((reserved-syn? s)
                 (said (string-append "`" (string-append (syn-name s) "` is a form's name, not an expression"))))
                ;; `#u`, unit: a literal, which the parsers read as a name.
                ((and (syn-symbol? s) (string=? (syn-name s) "#u")) #u)
                ((and (syn-symbol? s) (= mode 1)) (with-kids at (k-ref (syn-symbol s) #f 0)))
                ((and (not (syn-nil? s)) (not (null? (split s))) (= mode 1)) (mx-align-form mx-fail ps at s #f))
                ((syn-nil? s) (said "`()` is not an expression"))
                (else #u)))
        (s-defn ()
          (if (and (writes? mode) (not (syn-nil? s)) (not (null? (split s))))
              (mx-align-form mx-fail ps at s #t)
              (said (string-append (shown) " is not a definition"))))
        ;; A description a template writes, name by name; the binders of the
        ;; `poly`s at its top, what it exports (what a definition's type puts
        ;; in scope in what it is defined as).
        (s-type ()
          (if (writes? mode)
              (begin (set desc-roots (cons at (get desc-roots)))
                     (desc mx-fail ps at s)
                     (if (null? (node-export at))
                         (with-export at (bind-direct (mx-align-poly-binders at ps)) at)
                         #u))
              #u))
        ;; A module's item, as a program's walk takes one: a definition, a
        ;; string (a comment, binding nothing), or an expression.
        (s-item ()
          (cond ((defines? mx-fail s) (mx-align-form mx-fail ps at s #t))
                ((syn-string? s) (with-export at (bind-rib nil) at))
                (else (fill mx-fail mode ps at (s-expr) s root))))
        ;; `data`.
        (else z
          (let ((inside (pvar-inside s ps)))
            (if (and (= mode 1) (not (null? inside)))
                (said (string-append "pattern variable `" (string-append (symbol->string (car inside))
                                     "` inside a quoted datum: only a whole one is placed")))
                #u)))))))
;; A form in a template, at node `at`: a kernel form, a macro's use, or an
;; application, aligned with its signature; a definition's, if `defn`,
;; where a template makes definitions, else an expression's.
(mx-align-form (->* (fails pvars int syn bool) unit)
  (lambda (mx-fail ps at s defn)
    ;; A program's form, after expansion: its own binding structure.
    (if (get program-walk) (program-form mx-fail ps at s defn) (template-form mx-fail ps at s defn))))
(template-form (->* (fails pvars int syn bool) unit)
  (lambda (mx-fail ps at s defn)
    (let* ((parts (split s)) (hd (car parts))
           (head (if (and (syn-symbol? hd) (not (pvar? (syn-symbol hd) ps))) (syn-name hd) ""))
           (kernel (kernel-sigs mx-fail)) (defs (kernel-definitions mx-fail))
           (macro (and (not (string=? head "")) (macro? (syn-symbol hd)))))
      (cond
        ((and (get heads-on) (syn-symbol? hd) (unknown-head? mx-fail (syn-symbol hd) ps))
         (pending-form mx-fail ps at s defn))
        ;; A procedural macro (`macros-procedural.fx`): no signature says
        ;; what it binds.
        ((and macro (= (makes (syn-symbol hd)) 3))
         (fail-at mx-fail (string-append "`" (string-append head (string-append "` is a procedural macro, whose binding structure "
                                      "no signature says: not in a template")))
                  s))
        (defn (defn-form mx-fail ps at s 1))
        ;; A keyword at a list's head constructs, a tag written or placed.
        ((constructs? hd ps)
         (with-kids at (k-form (align mx-fail 1 ps (car (sig-of '%construct kernel)) s -1
                                      (node-path at)))))
        ((and macro (makes-defns? (syn-symbol hd)))
         (fail-at mx-fail (string-append "`" (string-append head (string-append "` makes definitions, which are made at the top "
                                      "level, not where an expression is")))
                  s))
        ((and macro (makes-desc? (syn-symbol hd)))
         (fail-at mx-fail (string-append "`" (string-append head "` makes a description, not an expression")) s))
        ((and (not (string=? head "")) (not (null? (sig-of (string->symbol head) defs)))
              (null? (sig-of (string->symbol head) kernel)))
         (fail-at mx-fail (string-append "`" (string-append head (string-append "` makes a definition, made at the top level by a "
                                      "macro that makes definitions, not where an expression is")))
                  s))
        ((string=? head "with")
         (cond ((standard-with? s) #u)
               ;; `(with m name)`: the value `name` of module `m`, a
               ;; template's binder or a use's, the name as written; `m`'s
               ;; or refused, where it is checked (`macros.fx`'s
               ;; `projections`).
               ((and (= (len (proper-items s)) 3) (syn-symbol? (nth (proper-items s) 1))
                     (syn-symbol? (nth (proper-items s) 2)))
                (let ((x (align mx-fail 1 ps (s-pair (s-id) (s-pair (s-data) (s-nil))) (nth parts 1)
                                -1 (path+ (node-path at) #f))))
                  (begin
                    (tagcase (node-kids x)
                      (k-pair (m y)
                        (tagcase (node-kids m)
                          (k-ref (n d f) (with-kids m (k-ref n d 3)))
                          (else z #u)))
                      (else y #u))
                    (with-kids at (k-form x)))))
               (else
                (fail-at mx-fail (string-append "a template's `with` is `(with module name)`, the "
                                             (string-append "value `name` of the module, or `(with #%fx name)` or "
                                             (string-append "`(with #%toplevel name)` (one opening a module over "
                                             "a body waits for scopes of a module's names)")))
                         s))))
        (macro (operands-as mx-fail 1 ps at (extract (car (macro-of (syn-symbol hd) (get defined))) 1)
                            (nth parts 1)))
        ((and (not (string=? head "")) (not (string=? (substring head 0 1) "%"))
              (not (null? (sig-of (string->symbol head) kernel))))
         (operands mx-fail ps at (car (sig-of (string->symbol head) kernel)) (nth parts 1)))
        ;; A form the kernel has not, but `scope-forms`, that binds
        ;; nothing (`bloblet-ref`, `TODO.md` §247).
        ((and (not (string=? head ""))
              (let ((sig (sig-of (string->symbol head) (scope-sigs mx-fail))))
                (and (not (null? sig)) (binds-nothing? (car sig)))))
         (operands mx-fail ps at (car (sig-of (string->symbol head) (scope-sigs mx-fail))) (nth parts 1)))
        ((and (not (string=? head "")) (reserved? head))
         (fail-at mx-fail (string-append "the prototype does not know `" (string-append head (string-append "`'s binding "
                                      "structure: not yet in a template")))
                  s))
        (else
         (with-kids at (k-form (align mx-fail 1 ps (car (sig-of '%app kernel)) s -1
                                      (node-path at)))))))))
;; A list whose head a `(macro σ made)` entry may bind, at `at`: pending, read by
;; its scope (`macros-scopes.fx`'s `aligned-heads`), a use of that macro by
;; its signature once its binder is found one, else an application (where a
;; definition is, unread until then, exporting nothing).
(pending-form (->* (fails pvars int syn bool) unit)
  (lambda (mx-fail ps at s defn)
    (let ((read (head-read s)) (n (syn-name (car (split s)))))
      (begin
        (pending! at defn)
        (cond ((not (null? read))
               (let* ((t (car (syntax-sig-read (car read)))) (made (extract t 2)))
                 (cond ((and (not defn) (= made 1))
                        (fail-at mx-fail (string-append "`" (string-append n (string-append "` makes definitions, which are made "
                                                     (string-append "at the top level, not where an expression "
                                                     "is"))))
                                 s))
                       ((and (not defn) (= made 2))
                        (fail-at mx-fail (string-append "`" (string-append n (string-append "` makes a description, not an "
                                                     "expression")))
                                 s))
                       ((and defn (= made 2))
                        (not-definition mx-fail s (string-append "them by a macro that makes "
                                                              "definitions, not a description's")))
                       ((and defn (= made 0))
                        (not-definition mx-fail s (string-append "them by a macro that makes "
                                                              "definitions, not an expression's")))
                       (else (operands mx-fail ps at (extract t 1) (nth (split s) 1))))))
              (defn (with-export at (bind-rib nil) at))
              (else (with-kids at (k-form (align mx-fail 1 ps (car (sig-of '%app (kernel-sigs mx-fail))) s
                                                 -1 (node-path at))))))))))
;;; ------------------------------------------------------------ programs

;; A program's expression `s` at `at`, after expansion: a name referred to
;; (but the literals the parsers read as names), or a form.
(program-expr (->* (fails pvars int syn) unit)
  (lambda (mx-fail ps at s)
    (cond ((syn-symbol? s)
           (if (let ((n (syn-symbol s)))
                 (or (symbol=? n (string->symbol "#t")) (symbol=? n (string->symbol "#f")) (symbol=? n (string->symbol "#u"))))
               #u
               (with-kids at (k-ref (syn-symbol s) #f 0))))
          ((syn-nil? s) (fail-at mx-fail "`()` is not an expression" s))
          ((not (null? (split s))) (mx-align-form mx-fail ps at s #f))
          (else #u))))
;; A program's form at `at`: its binding structure, the kernel's or
;; `scope-forms`'; a reserved word at its head with none, an application,
;; as the checkers take it (`macros.rs`'s `Aligner::form`, `Mode::Program`).
(program-form (->* (fails pvars int syn bool) unit)
  (lambda (mx-fail ps at s defn)
    (let* ((parts (split s)) (hd (car parts))
           (n (if (syn-symbol? hd) (syn-symbol hd) (string->symbol "")))
           (kernel (kernel-sigs mx-fail)) (sforms (scope-sigs mx-fail))
           (defs (kernel-definitions mx-fail)) (sdefs (scope-definitions mx-fail)))
      (cond
        ((and (get scoped-on) (macro? n) (scoped-read! mx-fail ps at s defn)) #u)
        ((and (macro? n) (= (makes n) 3))
         (fail-at mx-fail (string-append "`" (string-append (symbol->string n) (string-append "` is a procedural macro, whose "
                                      "binding structure no signature says: not in a template")))
                  s))
        (defn (defn-form mx-fail ps at s 3))
        ((syn-keyword? hd)
         (with-kids at (k-form (align mx-fail 3 ps (car (sig-of '%construct kernel)) s -1
                                      (node-path at)))))
        ((and (symbol=? n 'quasiquote) (= (len (proper-items s)) 2))
         (with-kids at (k-desc (unquoted mx-fail ps (nth (proper-items s) 1)
                                         (path+ (path+ (node-path at) #f) #t) 1 nil))))
        ((and (symbol=? n 'with) (standard-with? s)) #u)
        ((and (macro? n) (makes-defns? n))
         (fail-at mx-fail (string-append "`" (string-append (symbol->string n) (string-append "` makes definitions, which are made "
                                      "at the top level, not where an expression is")))
                  s))
        ((and (macro? n) (makes-desc? n))
         (fail-at mx-fail (string-append "`" (string-append (symbol->string n) (string-append "` makes a description, not an "
                                      "expression")))
                  s))
        ((macro? n) (operands-as mx-fail 3 ps at (extract (car (macro-of n (get defined))) 1)
                                 (nth parts 1)))
        ((not (null? (sig-of n sforms)))
         (operands-as mx-fail 3 ps at (car (sig-of n sforms)) (nth parts 1)))
        ;; Of the kernel's, not one the expander writes, `%app` or
        ;; `%construct`, named so.
        ((and (not (null? (sig-of n kernel))) (not (mx-syntax-name-in? n (percent-named kernel))))
         (operands-as mx-fail 3 ps at (car (sig-of n kernel)) (nth parts 1)))
        (else (with-kids at (k-form (align mx-fail 3 ps (car (sig-of '%app kernel)) s -1
                                            (node-path at)))))))))
;; A list a macro's name heads at `at`, where a program's form is read by
;; its scopes (`scoped-on`; `Aligner::form`): pending; an application if
;; read so, or a use whole where a form is expanded (done, #t); else a use
;; by its signature (#f).
(scoped-read! (->* (fails pvars int syn bool) bool)
  (lambda (mx-fail ps at s defn)
    (let ((app (begin (pending! at defn) (addr-in? (node-path at) (get scoped-apps)))))
      (cond ((and app defn) (begin (not-definition mx-fail s definitions-said) #t))
            (app (let ((app-sig (car (sig-of '%app (kernel-sigs mx-fail)))))
                   (begin (with-kids at (k-form (align mx-fail 3 ps app-sig s -1 (node-path at))))
                          #t)))
            ((get scoped-whole) (begin (use-whole! at defn) #t))
            (else #f)))))
;; The operands `rest` of the form at `at`, aligned with its signature, in
;; `mode`.
(operands-as (->* (fails int pvars int sty syn) unit)
  (lambda (mx-fail mode ps at sig rest)
    (with-kids at (k-form (align mx-fail mode ps sig rest -1 (path+ (node-path at) #f))))))
;; The expressions a quasiquote's datum `s`, at `path`, unquotes at `depth`
;; 1, each aligned, onto `kids` (the last first, given back first first):
;; a nested `quasiquote` deepens, an `unquote` or `unquote-splicing`
;; shallows, as R7RS counts (`macros.rs`'s `unquoted`).
(unquoted (->* (fails pvars syn addr int (listof int acyclic)) (listof int acyclic))
  (lambda (mx-fail ps s path depth mx-nodes-kids)
    (the (listof int acyclic) (reverse (unquoted-onto mx-fail ps s path depth mx-nodes-kids)))))
(unquoted-onto (->* (fails pvars syn addr int (listof int acyclic)) (listof int acyclic))
  (lambda (mx-fail ps s path depth mx-nodes-kids)
    (tagcase s
      (mx-eager-reader-types-lst (items d a b x)
        (let ((head (if (and (not (null? items)) (syn-symbol? (car items)))
                        (syn-name (car items))
                        ""))
              (inner (path+ (path+ path #f) #t)))
          (cond ((and (= (len items) 2) (= depth 1)
                      (or (string=? head "unquote") (string=? head "unquote-splicing")))
                 (cons (align mx-fail 3 ps (s-expr) (nth items 1) -1 inner) mx-nodes-kids))
                ((and (= (len items) 2) (or (string=? head "unquote") (string=? head
                                                                        "unquote-splicing")))
                 (unquoted-onto mx-fail ps (nth items 1) inner (- depth 1) mx-nodes-kids))
                ((and (= (len items) 2) (string=? head "quasiquote"))
                 (unquoted-onto mx-fail ps (nth items 1) inner (+ depth 1) mx-nodes-kids))
                (else (unquoted-items mx-fail ps items path depth mx-nodes-kids)))))
      (dotted (items tail d a b x)
        (unquoted-onto mx-fail ps tail (mx-align-falses path (len items)) depth
                       (unquoted-items mx-fail ps items path depth mx-nodes-kids)))
      (vec (items d a b x)
        (begin (if (any-unquotes? items) (set unplaced (symbols-onto items (get unplaced))) #u)
          mx-nodes-kids))
      (else y mx-nodes-kids))))
(unquoted-items (->* (fails pvars syns-a addr int (listof int acyclic)) (listof int acyclic))
  (lambda (mx-fail ps items path depth mx-nodes-kids)
    (if (null? items)
        mx-nodes-kids
        (unquoted-items mx-fail ps (cdr items) (path+ path #f) depth
                        (unquoted-onto mx-fail ps (car items) (path+ path #t) depth mx-nodes-kids)))))
;; A definition's form `s` at `at`, a template's (`mode` 1) or a program's
;; (3): its binding structure, a macro's that makes definitions by its
;; signature (`macros.rs`'s `Aligner::form`).
(defn-form (->* (fails pvars int syn int) unit)
  (lambda (mx-fail ps at s mode)
    (let* ((parts (split s)) (hd (car parts)) (items (proper-items s)) (program (= mode 3))
           (n (if (and (syn-symbol? hd) (not (pvar? (syn-symbol hd) ps)))
                  (syn-symbol hd)
                  (string->symbol "")))
           (defs (kernel-definitions mx-fail)) (sdefs (if program (scope-definitions mx-fail) nil))
           (macro (macro? n)))
      (cond ((and program (symbol=? n 'module-parameters) (= (len items) 2))
             (operands-as mx-fail 3 ps at (parameters-sig (nth items 1)) (nth parts 1)))
            ;; `(define-syntax name [σ] transformer)` an expansion makes
            ;; (`define-syntax-sig`), a `syntax-rules` one in a template.
            ((and (symbol=? n 'define-syntax) (proper? s) (or (= (len items) 3) (= (len items) 4)))
             (let* ((t (nth items (- (len items) 1))) (ts (proper-items t)))
               (if (and (not program)
                        (not (and (pair? ts) (syn-symbol? (car ts))
                                  (string=? (syn-name (car ts)) "syntax-rules"))))
                   (fail-at mx-fail rules-procedural t)
                   (operands-as mx-fail mode ps at (define-syntax-sig (= (len items) 4))
                                (nth parts 1)))))
            ((and (not program) (symbol=? n 'define-generative) (= (len items) 3))
             (fail-at mx-fail (string-append "a template's `define-generative` names its "
                                          "conversions, `#:seal` and `#:unseal`") s))
            ((and macro (makes-defns? n))
             (operands-as mx-fail mode ps at (extract (car (macro-of n (get defined))) 1)
                          (nth parts 1)))
            ((and macro (makes-desc? n))
             (not-definition mx-fail s "them by a macro that makes definitions, not a description's"))
            (macro
             (not-definition mx-fail s "them by a macro that makes definitions, not an expression's"))
            ((not (null? (sig-of n defs))) (operands-as mx-fail mode ps at (car (sig-of n defs))
                                                        (nth parts 1)))
            ((not (null? (sig-of n sdefs))) (operands-as mx-fail mode ps at (car (sig-of n sdefs))
                                                         (nth parts 1)))
            (else (not-definition mx-fail s definitions-said))))))
;; The operands `rest` of the form at `at`, aligned with its signature.
(operands (->* (fails pvars int sty syn) unit)
  (lambda (mx-fail ps at sig rest)
    (with-kids at (k-form (align mx-fail 1 ps sig rest -1 (path+ (node-path at) #f))))))
;;; ------------------------------------------------------------ descriptions

;; Description `s`, which a template writes, at node `at`, aligned name by
;; name (`macros.rs`'s `Aligner::desc`): each name referred to in the
;; description namespace, the top level's if the template does not bind it;
;; but a word of descriptions written as it is, a pattern variable placed,
;; and what `poly`, `dlambda`, `mu` and `dletrec` bind in scope in what
;; follows. Labels, a `restrict`'s names and an `io` effect's kind are
;; written as they are; `(globals g …)`'s names and `(select m n)`'s module
;; are values.
(desc (->* (fails pvars int syn) unit)
  (lambda (mx-fail ps at s)
    (tagcase s
      (mx-eager-reader-types-lst (items d a b x) (if (null? items) #u (desc-list mx-fail ps at s items)))
      (dotted (items tail d a b x) (fail-at mx-fail "a description is a list, not a dotted one" s))
      (else y
        (if (syn-symbol? s)
            (let ((n (syn-symbol s)))
              (with-kids at (cond ((pvar? n ps) (k-pvar n))
                                  ((desc-word? (syn-name s) #f) (k-other))
                                  (else (k-ref n #t 1)))))
            #u)))))
(desc-list (->* (fails pvars int syn syns-a) unit)
  (lambda (mx-fail ps at s items)
    (if (and (syn-symbol? (car items)) (not (pvar? (syn-symbol (car items)) ps))
             (macro? (syn-symbol (car items))))
        (desc-use mx-fail ps at s (syn-symbol (car items)))
        (desc-items mx-fail ps at s items))))
;; A macro's use where a description is: its operands aligned with its
;; signature, if it makes a description; where a form is expanded, whole,
;; as in `scoped-read!`.
(desc-use (->* (fails pvars int syn symbol) unit)
  (lambda (mx-fail ps at s n)
    (cond
      ((and (get scoped-on) (get scoped-whole) (makes-desc? n))
       (begin (pending! at #f) (use-whole! at #f)))
      ((makes-desc? n)
       (with-kids at (k-form (align mx-fail 1 ps (extract (car (macro-of n (get defined))) 1)
                                    (nth (split s) 1) -1 (path+ (node-path at) #f)))))
      (else
       (fail-at mx-fail (string-append "`" (string-append (symbol->string n) (string-append "` makes "
                                    (string-append (if (makes-defns? n) "definitions" "an expression")
                                    ", not a description"))))
                s)))))
(desc-items (->* (fails pvars int syn syns-a) unit)
  (lambda (mx-fail ps at s items)
    (let* ((mx-nodes-kids (desc-kids at items)) (h (car items))
           (head (if (and (syn-symbol? h) (not (pvar? (syn-symbol h) ps))) (syn-name h) "")))
      (cond
        ;; A program's `poly` of several descriptions after its binders.
        ((and (or (string=? head "poly") (string=? head "dlambda")) (get program-walk)
              (> (len items) 3))
         (desc-each-scoped mx-fail ps (cdr (cdr mx-nodes-kids)) (cdr (cdr items))
                           (desc-binders mx-fail ps (kid mx-nodes-kids 1) (nth items 1))))
        ((or (string=? head "poly") (string=? head "dlambda"))
         (if (= (len items) 3)
             (desc-scoped mx-fail ps (kid mx-nodes-kids 2) (nth items 2)
                          (desc-binders mx-fail ps (kid mx-nodes-kids 1) (nth items 1)))
             (fail-at mx-fail (string-append "`(" (string-append head " ((name kind) …) description)`")) s)))
        ((string=? head "mu")
         (if (= (len items) 3)
             (desc-scoped mx-fail ps (kid mx-nodes-kids 2) (nth items 2)
                          (list (desc-binder mx-fail ps (kid mx-nodes-kids 1) (nth items 1))))
             (fail-at mx-fail "`(mu name type)`" s)))
        ((string=? head "dletrec")
         (if (= (len items) 3)
             (desc-dletrec mx-fail ps mx-nodes-kids items)
             (fail-at mx-fail "`(dletrec ((name description) …) description)`" s)))
        ((or (string=? head "productof") (string=? head "sumof"))
         (desc-fields mx-fail ps (cdr mx-nodes-kids) (cdr items)))
        ((and (string=? head "restrict") (>= (len items) 2))
         (desc mx-fail ps (kid mx-nodes-kids 1) (nth items 1)))
        ((string=? head "io") #u)
        ;; Values: a global's name, written as it is, since no local can
        ;; shadow it; a module's, the top level's.
        ((string=? head "globals") (desc-values mx-fail ps (cdr mx-nodes-kids) 2))
        ((string=? head "select")
         (if (null? (cdr mx-nodes-kids)) #u (select-path mx-fail ps (kid mx-nodes-kids 1) (nth items 1))))
        ;; A file's module type, given descriptions and module values, which
        ;; a name alone does not tell apart: the checker decides by what
        ;; binds it, so a program's walk leaves the names unplaced.
        ((and (get program-walk)
              (or (string=? head "load-moduleof") (string=? head "load-input-moduleof")))
         (set unplaced (symbols-onto (cdr (cdr items)) (get unplaced))))
        ((string=? head "moduleof")
         (desc-members mx-fail ps (cdr mx-nodes-kids) (cdr items) nil (not (get program-walk))))
        ((or (string=? head "subr") (string=? head "->") (string=? head "->*"))
         (desc-subr mx-fail ps s items mx-nodes-kids))
        ((and (not (string=? head "")) (desc-word? head #t))
         (desc-each mx-fail ps (cdr mx-nodes-kids) (cdr items)))
        ;; A description function applied, its name the head.
        (else (desc-each mx-fail ps mx-nodes-kids items))))))
(desc-each (->* (fails pvars (listof int acyclic) syns-a) unit)
  (lambda (mx-fail ps ks xs)
    (if (or (null? xs) (null? ks))
        #u
        (begin (desc mx-fail ps (car ks) (car xs)) (desc-each mx-fail ps (cdr ks) (cdr xs))))))
;; Description `s` at node `at`, in the scope of `as`.
(desc-scoped (->* (fails pvars int syn atms) unit)
  (lambda (mx-fail ps at s as) (begin (with-import at (bind-direct as) at) (desc mx-fail ps at s))))
;; `((name kind) …)` at node `at`, a binder's place or `distinct` third if
;; any, each in the scope of those before it: what they bind.
(desc-binders (->* (fails pvars int syn) atms)
  (lambda (mx-fail ps at bs)
    (let ((entries (proper-items bs)))
      (desc-binders-from mx-fail ps (desc-kids at entries) entries nil))))
(desc-binders-from (->* (fails pvars (listof int acyclic) syns-a atms) atms)
  (lambda (mx-fail ps ks es out)
    (if (or (null? es) (null? ks))
        out
        (let* ((parts (proper-items (car es))) (n (len parts)))
          (if (or (= n 2) (= n 3))
              (let* ((p (desc-kids (car ks) parts))
                     (a (desc-binder mx-fail ps (car p) (car parts))))
                (begin
                  (if (= n 3) (desc-scoped mx-fail ps (kid p 2) (nth parts 2) out) #u)
                  (desc-binders-from mx-fail ps (cdr ks) (cdr es) (append out (the atms (list a))))))
              (begin (fail-at mx-fail "a binder is `(name kind)`" (car es)) out))))))
(desc-dletrec (->* (fails pvars (listof int acyclic) syns-a) unit)
  (lambda (mx-fail ps mx-nodes-kids items)
    (let* ((entries (proper-items (nth items 1)))
           (parts (dletrec-parts mx-fail ps (desc-kids (kid mx-nodes-kids 1) entries) entries))
           (as (dparts-atms parts)))
      (begin (dparts-each mx-fail ps parts as) (desc-scoped mx-fail ps (kid mx-nodes-kids 2) (nth items 2) as)))))
(dparts-each (->* (fails pvars dparts atms) unit)
  (lambda (mx-fail ps ds as)
    (if (null? ds)
        #u
        (begin (desc-scoped mx-fail ps (extract (car ds) 2) (extract (car ds) 3) as)
               (dparts-each mx-fail ps (cdr ds) as)))))
;; `(label type)`s at nodes `ks`.
(desc-fields (->* (fails pvars (listof int acyclic) syns-a) unit)
  (lambda (mx-fail ps ks fs)
    (if (or (null? fs) (null? ks))
        #u
        (let ((parts (proper-items (car fs))))
          (if (= (len parts) 2)
              (let ((p (desc-kids (car ks) parts)))
                (begin
                  ;; A label a use gives, placed.
                  (if (and (syn-symbol? (car parts)) (pvar? (syn-symbol (car parts)) ps))
                      (begin (with-core (car p) (s-label))
                             (with-kids (car p) (k-pvar (syn-symbol (car parts)))))
                      #u)
                  (desc mx-fail ps (kid p 1) (nth parts 1))
                  (desc-fields mx-fail ps (cdr ks) (cdr fs))))
              ;; A program's flat sum's tag type, `(sumof u8 …)`.
              (if (get program-walk)
                  (begin (desc mx-fail ps (car ks) (car fs)) (desc-fields mx-fail ps (cdr ks) (cdr fs)))
                  (fail-at mx-fail "`(label type)`" (car fs))))))))
;; Descriptions `xs` at nodes `ks`, each in the scope of `as`.
(desc-each-scoped (->* (fails pvars (listof int acyclic) syns-a atms) unit)
  (lambda (mx-fail ps ks xs as)
    (if (or (null? xs) (null? ks))
        #u
        (begin (desc-scoped mx-fail ps (car ks) (car xs) as)
               (desc-each-scoped mx-fail ps (cdr ks) (cdr xs) as)))))
;; A `moduleof`'s members `ms` at nodes `ks`: each's type in the scope of
;; `as`, the abstract and transparent types before it, a value's name a
;; label; an `abs`'s or `desc`'s label written as a keyword apart from its
;; binder after it, `(abs #:t u type)`, as a `template`'s are, or a name a
;; use gives (`macros.rs`'s, `typed-macros.md`).
(desc-members (->* (fails pvars (listof int acyclic) syns-a atms bool) unit)
  (lambda (mx-fail ps ks ms as template)
    (if (or (null? ms) (null? ks))
        #u
        (let* ((parts (proper-items (car ms)))
               (word (if (and (not (null? parts)) (syn-symbol? (car parts)))
                         (syn-name (car parts)) "")))
          (if (< (len parts) 2)
              (begin (desc mx-fail ps (car ks) (car ms))
                     (desc-members mx-fail ps (cdr ks) (cdr ms) as template))
              (let* ((p (desc-kids (car ks) parts)) (n (nth parts 1))
                     (binds (or (string=? word "abs") (string=? word "desc")))
                     (kw (syn-keyword? n))
                     (apart (and binds kw (= (len parts) 4) (syn-symbol? (nth parts 2))))
                     (given (and (syn-symbol? n) (pvar? (syn-symbol n) ps))))
                (begin
                  (cond ((or (not template) kw given) #u)
                        ((not (syn-symbol? n))
                         (fail-at mx-fail "a list of names in a template's `moduleof`: not yet" n))
                        (binds (fail-at mx-fail (string-append "a template's `abs` or `desc` writes "
                                                            (string-append "its label as a keyword, its binder "
                                                            "after it: `(abs #:t t type)`"))
                                        n))
                        (else (fail-at mx-fail (string-append "a template's `val` writes its label "
                                                           "as a keyword: `(val #:x type)`")
                                       n)))
                  ;; A value's label a use gives, placed.
                  (if (and given (not binds))
                      (begin (with-core (kid p 1) (s-label))
                             (with-kids (kid p 1) (k-pvar (syn-symbol n))))
                      #u)
                  ;; An `abs`'s last part is a kind, of words alone.
                  (if (string=? word "abs") #u
                      (desc-each-scoped mx-fail ps (if apart (cdr (cdr (cdr p))) (cdr (cdr p)))
                                        (if apart (cdr (cdr (cdr parts))) (cdr (cdr parts))) as))
                  (desc-members mx-fail ps (cdr ks) (cdr ms)
                                (cond (apart (append as (the atms (list (desc-binder mx-fail ps
                                                                          (kid p 2)
                                                                          (nth parts 2))))))
                                      ((and binds (syn-symbol? n))
                                       (append as (the atms (list (desc-binder mx-fail ps (kid p 1)
                                                                                n)))))
                                      (else as))
                                template))))))))
;; A procedure type, `(subr …)`, `(-> …)` or `(->* …)`, its parts `items`
;; at nodes `kids` (`macros.rs`'s `desc_subr`): each parameter named, `(x :
;; T)`, a value's binder in scope in the types after it, the result and the
;; specification's clauses, and a repeated one's in its own type only as
;; the repetitions before it bound it, which a name cannot say; `(ensures r
;; Q)`'s `r` in scope in `Q`. A `(bool …)` result's propositions are written
;; as they are.
(desc-subr (->* (fails pvars syn syns-a (listof int acyclic)) unit)
  (lambda (mx-fail ps s items mx-nodes-kids)
    (let ((plain (- (len items) (clauses-at-end (reverse items)))))
      (if (or (< plain 3) (not (= (len-kids mx-nodes-kids) (len items))))
          (fail-at mx-fail (string-append "`(subr effect (param …) result)` or `(-> (param …) "
                                       "result)`, then a specification's clauses")
                   s)
          (desc-subr-parts mx-fail ps mx-nodes-kids items 0 (- plain 2) nil)))))
;; The `i`th part on, the parameters the `pi`th; `as`, what the parameters
;; before bind.
(desc-subr-parts (->* (fails pvars (listof int acyclic) syns-a int int atms) unit)
  (lambda (mx-fail ps ks xs i pi as)
    (cond ((or (null? ks) (null? xs)) #u)
          ((= i pi)
           (desc-subr-parts mx-fail ps (cdr ks) (cdr xs) (+ i 1) pi
                            (desc-params mx-fail ps (car ks) (car xs))))
          (else
           (begin
             (cond ((= i 0) #u)
                   ((< i pi) (desc mx-fail ps (car ks) (car xs)))
                   ((= i (+ pi 1)) (desc-result mx-fail ps (car ks) (car xs) as))
                   (else (desc-clause mx-fail ps (car ks) (car xs) as)))
             (desc-subr-parts mx-fail ps (cdr ks) (cdr xs) (+ i 1) pi as))))))
(desc-params (->* (fails pvars int syn) atms)
  (lambda (mx-fail ps at s)
    (cond ((syn-nil? s) nil)
          ((null? (proper-items s))
           (begin (fail-at mx-fail "a procedure type's parameters are a list" s) nil))
          (else (desc-params-from mx-fail ps (desc-kids at (proper-items s)) (proper-items s) nil)))))
(desc-params-from (->* (fails pvars (listof int acyclic) syns-a atms) atms)
  (lambda (mx-fail ps ks xs as)
    (if (or (null? ks) (null? xs))
        as
        (let ((parts (proper-items (car xs))))
          (if (and (= (len parts) 3) (syn-symbol? (nth parts 1))
                   (string=? (syn-name (nth parts 1)) ":"))
              (let* ((q (desc-kids (car ks) parts)) (a (value-binder mx-fail ps (car q) (car parts))))
                (begin
                  (with-import (kid q 2) (bind-direct as) (kid q 2))
                  (if (mx-nodes-repeated? (car ks))
                      (with-import (kid q 2) (bind-earlier (list a)) (kid q 2))
                      #u)
                  (desc mx-fail ps (kid q 2) (nth parts 2))
                  (desc-params-from mx-fail ps (cdr ks) (cdr xs) (append as (the atms (list a))))))
              (begin (desc-scoped mx-fail ps (car ks) (car xs) as)
                     (desc-params-from mx-fail ps (cdr ks) (cdr xs) as)))))))
(desc-result (->* (fails pvars int syn atms) unit)
  (lambda (mx-fail ps at r as)
    (let ((parts (proper-items r)))
      (if (and (not (null? parts)) (syn-symbol? (car parts))
               (string=? (syn-name (car parts)) "bool")
               (not (pvar? (syn-symbol (car parts)) ps)))
          (let ((inside (pvar-inside r ps)))
            (if (null? inside)
                #u
                (fail-at mx-fail (string-append "pattern variable `" (string-append (symbol->string (car inside))
                                             "` in a `(bool …)` result's propositions: not yet"))
                         r)))
          (desc-scoped mx-fail ps at r as))))))

;;; ============================================================ macros-scopes.fx (whole)

(define earlier-atoms (->* (levels) atms)
  (lambda (ls)
    (if (null? ls)
        nil
        (append (tagcase (car ls)
                  (lv-rib (as) as) (lv-earlier (as) as) (lv-cage (n) nil) (lv-shape (m) nil))
                (earlier-atoms (cdr ls))))))
(define earlier-cages (->* (levels) levels)
  (lambda (ls)
    (cond ((null? ls) nil)
          ((tagcase (car ls) (lv-cage (n) #t) (lv-shape (m) #t) (else y #f))
           (cons (car ls) (earlier-cages (cdr ls))))
          (else (earlier-cages (cdr ls))))))
;; Whether δ `delta` binds a variable: a value's, a description's, or both.
(define binder-delta? (->* (int) bool)
  (lambda (delta) (or (= delta 1) (= delta 4) (= delta 5) (>= delta 100))))
;; What the binder at node `t` binds, which the signature says binds a value
;; (δ 1), a description (4) or both, a place (5): refused if it binds what
;; the signature does not say.
(define binder-atms (->* (fails int int) atms)
  (lambda (mx-fail t delta)
    (let* ((d (and (not (= delta 1)) (< delta 100)))
           (desc? (tagcase (node-kids t)
                    (k-binder (n e) e)
                    (k-pvar (n) (= (mx-types-kind-of (node-core t)) 12))
                    (else y (not d))))
           (one (lambda ((e bool))
                  (tagcase (node-kids t) (k-pvar (n) (atm-var n e 0)) (else y (atm-lit t e))))))
      (begin
        (if (mx-eq? desc? d)
            #u
            (fail-at mx-fail (string-append "the signature says a " (string-append (if d "description's" "value's")
                                         " binder is here"))
                     (node-syn t)))
        (if (= delta 5) (list (one #t) (one #f)) (list (one d)))))))
(define env-in-rib (->* (fails syn) unit)
  (lambda (mx-fail s) (fail-at mx-fail "a ribcage (`env`) in a rib: write it `(at ℓ)` in a `cage`" s)))
(define gathered-each (->* (atms) atms)
  (lambda (as)
    (if (null? as)
        nil
        (cons (tagcase (car as) (atm-var (n d g) (atm-var n d (+ g 1))) (else y (car as)))
              (gathered-each (cdr as))))))
;; Atoms `as` node `t` exports: if `t` is a list by ellipsis, what its
;; elements export, every repetition's (`macros.rs`'s `gathered`).
(define gathered (->* (int atms) atms)
  (lambda (t as)
    (tagcase (node-kids t)
      (mx-nodes-k-seq (x)
        (let ((e (node-export t)))
          (if (or (null? e) (= (extract (car e) 2) x)) (gathered-each as) as)))
      (else y as))))
;; The node whose export is what node `t` exports: a form's operands' (a
;; definition's), else its own.
(define exporter (->* (int) int)
  (lambda (t) (tagcase (node-kids t) (k-form (x) x) (else y t))))
(define mx-scopes-form-node? (->* (int) bool)
  (lambda (t) (tagcase (node-kids t) (k-form (x) #t) (else y #f))))
;; The node at `a` from `root`.
(define nav (->* (fails int addr syn) int)
  (lambda (mx-fail root a s)
    (if (null? a)
        root
        (tagcase (node-kids root)
          (k-pair (x y) (nav mx-fail (if (car a) x y) (cdr a) s))
          (k-pvar (n)
            (begin (fail-at mx-fail (string-append "an address into pattern variable `"
                                                (string-append (symbol->string n) (string-append "`: the prototype names a part "
                                                "only where the pattern names it")))
                            s)
                   root))
          (else y (begin (fail-at mx-fail "an address to no part" s) root))))))
(define-rec
(rib (->* (fails entries int syn) atms)
  (lambda (mx-fail es root s)
    (if (null? es)
        nil
        (let* ((t (nav mx-fail root (extract (car es) 1) s)) (delta (extract (car es) 2))
               (ts (node-syn t)) (rest (rib mx-fail (cdr es) root s)))
          (tagcase (node-kids t)
            (k-pvar (n)
              (cond ((binder-delta? delta) (append (binder-atms mx-fail t delta) rest))
                    ;; A use's description: what its own `poly`s bind is the
                    ;; use's, which nothing the template writes can name.
                    ((and (= delta 2) (= (mx-types-kind-of (node-core t)) 4)) rest)
                    ((= delta 2) (cons (atm-rib-of n) rest))
                    ((= delta 3) (begin (env-in-rib mx-fail ts) rest))
                    (else rest)))
            (k-binder (n d)
              (cond ((binder-delta? delta) (append (binder-atms mx-fail t delta) rest))
                    ((= delta 2) (append (exported-rib mx-fail t ts s) rest))
                    ((= delta 3) (begin (env-in-rib mx-fail ts) rest))
                    (else rest)))
            (else y
              (cond ((binder-delta? delta) (append (binder-atms mx-fail t delta) rest))
                    ((= delta 2) (append (exported-rib mx-fail t ts s) rest))
                    ((= delta 3) (begin (env-in-rib mx-fail ts) rest))
                    (else rest))))))))
;; The rib node `t` exports, its addresses read at `s`'s place, as the
;; import's are: a form's, its operands' (a definition's); none, a use of
;; a macro that makes definitions, which the template sees none of.
(exported-rib (->* (fails int syn syn) atms)
  (lambda (mx-fail t ts s)
    (let ((e (node-export (exporter t))))
      (if (null? e)
          (if (mx-scopes-form-node? t)
              (the atms nil)
              (begin (fail-at mx-fail (string-append "the signature says what is here exports a rib, "
                                                  "and it exports none")
                              ts)
                     (the atms nil)))
          (tagcase (extract (car e) 1)
            (bind-rib (es) (gathered t (rib mx-fail es (extract (car e) 2) s)))
            (bind-direct (as) as)
            (else y (begin (fail-at mx-fail (string-append "the signature says what is here exports "
                                                        "a rib, and it exports none")
                                    ts)
                           (the atms nil)))))))))
(define gathered-levels (->* (int levels) levels)
  (lambda (t ls)
    (if (null? ls)
        nil
        (cons (tagcase (car ls)
                (lv-rib (as) (lv-rib (gathered t as)))
                (lv-earlier (as) (lv-earlier (gathered t as)))
                (else y (car ls)))
              (gathered-levels t (cdr ls))))))
(define-rec
;; What `b` binds, its addresses from `root`: levels, innermost first.
(evaluate (->* (fails mx-types-bind int syn) levels)
  (lambda (mx-fail b root s)
    (tagcase b
      (bind-rib (es) (list (lv-rib (rib mx-fail es root s))))
      (bind-cage (rings) (cage mx-fail rings root s))
      (bind-direct (as) (list (lv-rib as)))
      (bind-earlier (as) (list (lv-earlier as)))
      (bind-shape (a) (list (lv-shape (nav mx-fail root a s)))))))
(cage (->* (fails rings int syn) levels)
  (lambda (mx-fail rings root s)
    (if (null? rings)
        nil
        (append (tagcase (car rings)
                  (ring-rib (es) (the levels (list (lv-rib (rib mx-fail es root s)))))
                  (ring-at (a) (levels-of mx-fail (nav mx-fail root a s) s))
                  (ring-shape (a) (the levels (list (lv-shape (nav mx-fail root a s))))))
                (cage mx-fail (cdr rings) root s)))))
(levels-of (->* (fails int syn) levels)
  (lambda (mx-fail t s)
    (tagcase (node-kids t)
      (k-pvar (n) (list (lv-cage n)))
      ;; A ribcage's list by ellipsis: what each element exports.
      (mx-nodes-k-seq (x)
        (gathered-levels t (if (and (cage-seq? t) (null? (node-export t)))
                               (levels-of mx-fail x s)
                               (exported-levels mx-fail t s))))
      (else y (exported-levels mx-fail t s)))))
(exported-levels (->* (fails int syn) levels)
  (lambda (mx-fail t s)
    (let ((e (node-export (exporter t))))
      (cond ((not (null? e)) (evaluate mx-fail (extract (car e) 1) (extract (car e) 2) s))
            ((mx-scopes-form-node? t) (the levels nil))
            (else
             (begin (fail-at mx-fail (string-append "the signature says what is here exports a "
                                                 "ribcage, and it exports none")
                             (node-syn t))
                    (the levels nil))))))))
;; `env` with what the repetitions of element `x` before each bound, which
;; its binders' names cannot tell from its own.
(define earlier (->* (fails int mx-env-indexed-env) mx-env-indexed-env)
  (lambda (mx-fail x mx-env-indexed-env)
    (let ((ls (levels-of mx-fail x (node-syn x))))
      (env-within mx-env-indexed-env (cons (lv-earlier (earlier-atoms ls)) (earlier-cages ls)) mx-nodes-binder-name))))
(define levels-shaped (->* (bool levels) levels)
  (lambda (shaped ls)
    (cond ((null? ls) nil)
          ((mx-eq? shaped (tagcase (car ls) (lv-shape (m) #t) (else y #f)))
           (cons (car ls) (levels-shaped shaped (cdr ls))))
          (else (levels-shaped shaped (cdr ls))))))
;; In a program's walk, what one import binds of its own before the modules
;; it opens: a module's items before what its `include`s give, as the
;; checkers read them (`macros/shapes.rs`'s `segments_of`).
(define own-first (->* (levels) levels)
  (lambda (ls)
    (if (get program-walk)
        (append (levels-shaped #f ls) (levels-shaped #t ls))
        ls)))
;; `env` with what each import binds before it, the last import innermost.
(define mx-scopes-imported (->* (fails rooted mx-env-indexed-env syn) mx-env-indexed-env)
  (lambda (mx-fail is mx-env-indexed-env s)
    (if (null? is)
        mx-env-indexed-env
        (mx-scopes-imported mx-fail (cdr is)
                  (env-within mx-env-indexed-env
                              (own-first
                                (evaluate mx-fail (extract (car is) 1) (extract (car is) 2) s))
                              mx-nodes-binder-name)
                  s))))
(define-rec
;;; ------------------------------------------------------------ binding

;; The tree under `at`, walked with environment `env`, under `depth0`
;; ellipses: what it finds, in the order a walk finds them; each node an
;; ellipsis follows noted in `seq-depths`.
(mx-scopes-walk (->* (fails int mx-env-indexed-env bool int) mx-nodes-found)
  (lambda (mx-fail at mx-env-indexed-env many0 depth0)
    (let* ((many (or many0 (node-many? at)))
           (depth (if (mx-nodes-repeated? at) (+ depth0 1) depth0))
           (noted (if (mx-nodes-repeated? at) (set seq-depths (cons (tuple at depth) (get seq-depths))) #u))
           (env2 (mx-scopes-imported mx-fail (node-imports at) mx-env-indexed-env (node-syn at)))
           ;; A pending list's scope, for its head (`Node::head`).
           (head (if (and (not (null? (get pending))) (pending-node? at))
                     (the mx-nodes-found (list (tuple at env2 many depth)))
                     nil)))
      (append head (tagcase (node-kids at)
        (k-pair (x y) (append (mx-scopes-walk mx-fail x env2 many depth) (mx-scopes-walk mx-fail y env2 many depth)))
        (k-form (x) (mx-scopes-walk mx-fail x env2 many depth))
        ;; An element of a ribcage's list, in the scope of what the
        ;; repetitions before it bound.
        (mx-nodes-k-seq (x) (mx-scopes-walk mx-fail x (if (cage-seq? at) (earlier mx-fail x env2) env2) many depth))
        (k-desc (xs) (walk-each mx-fail xs env2 many depth))
        (k-pvar (n) (the mx-nodes-found (list (tuple at env2 many depth))))
        (k-binder (n d) (the mx-nodes-found (list (tuple at env2 many depth))))
        (k-ref (n d free) (the mx-nodes-found (list (tuple at env2 many depth))))
        (else y (the mx-nodes-found nil)))))))
(walk-each (->* (fails (listof int acyclic) mx-env-indexed-env bool int) mx-nodes-found)
  (lambda (mx-fail xs mx-env-indexed-env many depth)
    (if (null? xs)
        nil
        (let ((first (mx-scopes-walk mx-fail (car xs) mx-env-indexed-env many depth)))
          (append first (walk-each mx-fail (cdr xs) mx-env-indexed-env many depth)))))))
;; Whether `as` binds pattern variable `n` (a description's if `d`), by its
;; name: a reference finds its binder whichever repetitions are gathered.
(define mx-scopes-named-in? (->* (symbol bool atms) bool)
  (lambda (n d as)
    (and (not (null? as))
         (or (tagcase (car as) (atm-var (m e g) (and (symbol=? n m) (mx-eq? d e))) (else y #f))
             (mx-scopes-named-in? n d (cdr as))))))
;; The first level of `env`, from `i`, whose rib binds pattern variable
;; `n`; -1 if none.
(define rib-binding (->* (symbol bool levels int) int)
  (lambda (n d mx-env-indexed-env i)
    (cond ((null? mx-env-indexed-env) -1)
          ((tagcase (car mx-env-indexed-env)
             (lv-rib (as) (mx-scopes-named-in? n d as))
             (lv-earlier (as) (mx-scopes-named-in? n d as))
             (else y #f))
           i)
          (else (rib-binding n d (cdr mx-env-indexed-env) (+ i 1))))))
(define nth-level (->* (levels int) level)
  (lambda (mx-env-indexed-env i) (if (= i 0) (car mx-env-indexed-env) (nth-level (cdr mx-env-indexed-env) (- i 1)))))
;; Why a name bound by a repeated part (an element of a ribcage's list, a
;; parameter named) is refused where that part refers to it.
(define mx-scopes-repeats-said string
  (string-append "is bound by the repetitions before this one, which its name does not tell "
                 "apart: a repeated part refers to no binder it repeats in its own scope"))
(define any-pattern-atom? (->* (atms pvars bool) bool)
  (lambda (as ps d)
    (and (not (null? as))
         (or (tagcase (car as)
               (atm-lit (t e) #f)
               (atm-var (m e g) (mx-eq? e d))
               (else y #t))
             (any-pattern-atom? (cdr as) ps d)))))
;; Whether a level of `env` before the `i`th is a ribcage, or a rib with one
;; of the pattern's binders of a description's namespace if `d`, else a
;; value's: a value's binder never shadows a description's.
(define shadowing? (->* (levels int pvars bool) bool)
  (lambda (mx-env-indexed-env i ps d)
    (and (> i 0)
         (or (tagcase (car mx-env-indexed-env)
               (lv-cage (n) #t)
               (lv-rib (as) (any-pattern-atom? as ps d))
               (lv-earlier (as) (any-pattern-atom? as ps d))
               (lv-shape (m) #f))
             (shadowing? (cdr mx-env-indexed-env) (- i 1) ps d)))))
(define pattern-atoms (->* (atms) atms)
  (lambda (as)
    (cond ((null? as) nil)
          ((tagcase (car as) (atm-lit (t d) #t) (else y #f)) (pattern-atoms (cdr as)))
          (else (cons (car as) (pattern-atoms (cdr as)))))))
;; An environment as compared: the template's own binders left out (made
;; fresh, they bind nothing a use wrote), empty ribs gone.
(define normal (->* (levels) levels)
  (lambda (mx-env-indexed-env)
    (if (null? mx-env-indexed-env)
        nil
        (tagcase (car mx-env-indexed-env)
          (lv-rib (as)
            (let ((kept (pattern-atoms as)))
              (if (null? kept) (normal (cdr mx-env-indexed-env)) (cons (lv-rib kept) (normal (cdr mx-env-indexed-env))))))
          (lv-earlier (as)
            (let ((kept (pattern-atoms as)))
              (if (null? kept) (normal (cdr mx-env-indexed-env)) (cons (lv-earlier kept) (normal (cdr mx-env-indexed-env))))))
          (lv-cage (n) (cons (car mx-env-indexed-env) (normal (cdr mx-env-indexed-env))))
          (lv-shape (m) (normal (cdr mx-env-indexed-env)))))))
(define atm=? (->* (atm atm) bool)
  (lambda (a b)
    (tagcase a
      (atm-var (n d g)
        (tagcase b (atm-var (m e h) (and (symbol=? n m) (mx-eq? d e) (= g h))) (else y #f)))
      (atm-rib-of (n) (tagcase b (atm-rib-of (m) (symbol=? n m)) (else y #f)))
      (atm-lit (t d) (tagcase b (atm-lit (u e) (and (= t u) (mx-eq? d e))) (else y #f))))))
(define without (->* (atm atms) atms)
  (lambda (a bs)
    (cond ((null? bs) nil)
          ((atm=? a (car bs)) (cdr bs))
          (else (cons (car bs) (without a (cdr bs)))))))
(define len-atms (->* (atms) int)
  (lambda (as) (if (null? as) 0 (+ 1 (len-atms (cdr as))))))
(define same-atoms? (->* (atms atms) bool)
  (lambda (as bs)
    (if (null? as)
        (null? bs)
        (let ((rest (without (car as) bs)))
          (and (= (len-atms rest) (- (len-atms bs) 1)) (same-atoms? (cdr as) rest))))))
;; Whether two environments are the same, each rib's atoms in any order.
(define levels=? (->* (levels levels) bool)
  (lambda (xs ys)
    (if (null? xs)
        (null? ys)
        (and (not (null? ys))
             (tagcase (car xs)
               (lv-rib (as) (tagcase (car ys) (lv-rib (bs) (same-atoms? as bs)) (else y #f)))
               (lv-earlier (as)
                 (tagcase (car ys) (lv-earlier (bs) (same-atoms? as bs)) (else y #f)))
               (lv-cage (n) (tagcase (car ys) (lv-cage (m) (symbol=? n m)) (else y #f)))
               (lv-shape (n) (tagcase (car ys) (lv-shape (m) (= n m)) (else y #f))))
             (levels=? (cdr xs) (cdr ys))))))
;; Pattern variable `n`, of `info`, placed in the template where type
;; `want` is, in environment `env`: Herman's rules. `ps`, the pattern's.
(define mx-scopes-placed (->* (fails symbol pinfo sty levels syn pvars) unit)
  (lambda (mx-fail n info want mx-env-indexed-env s ps)
    (let* ((name (symbol->string n)) (have (extract info 1)) (kh (mx-types-kind-of have))
           (kw (mx-types-kind-of (unfold want)))
           (said (lambda ((why string)) (fail-at mx-fail (string-append "`" (string-append name (string-append "` " why))) s)))
           (simple? (lambda ((k int)) (simple-kind? k))))
      (cond
        ((or (and (= kh 1) (= kw 1)) (and (= kh 12) (= kw 12))) #u)
        ;; A pattern's binder referred to, a value's or a description's: in
        ;; its scope, and where no other of the pattern's binders of its
        ;; namespace may shadow it (the shadow restriction).
        ((and (or (= kh 1) (= kh 12)) (or (= kw 0) (= kw 2) (and (= kh 12) (= kw 4))))
         (let* ((d (= kw 4)) (i (rib-binding n d mx-env-indexed-env 0)))
           (cond ((< i 0) (said "is referred to where the template does not bind it"))
                 ((tagcase (nth-level mx-env-indexed-env i) (lv-earlier (as) #t) (else y #f))
                  (said mx-scopes-repeats-said))
                 ((shadowing? mx-env-indexed-env i ps d)
                  (said (string-append "is referred to where another of the pattern's binders "
                                       (string-append "may shadow it (Herman's shadow restriction): a use "
                                       "giving both one name would change what it refers to"))))
                 (else #u))))
        ;; An untyped part where a lambda is wanted: it promises nothing, and
        ;; its uses' expansions are checked; a typed hole says it is a lambda
        ;; (`typed-macros.md`, "Typed holes": two tiers).
        ((and (= kh 0) (= kw 10) (not (null? (extract info 4))))
         (said (string-append "is placed where a lambda is wanted: a typed hole there is a "
                              "lambda, `(lambda #:type T)`")))
        ((or (and (= kh 3) (= kw 3)) (and (= kh 4) (= kw 4)) (and (= kh 5) (= kw 5))
             (and (= kh 13) (= kw 13)))
         #u)
        (else
         ;; In a macro the template defines, whose own check is the defined
         ;; macro's, where it is defined.
         (let ((fits (cond ((= kw 16) #t)
                           ((and (= kh 0) (= kw 0)) #t)
                           ((and (= kh 10) (or (= kw 10) (= kw 0))) #t)
                           ((and (= kh 0) (= kw 10)) #t)
                           ((and (= kh 2) (or (= kw 0) (= kw 2))) #t)
                           ((or (simple? kh) (simple? kw)) #f)
                           (else (sub? have want nil)))))
           (cond ((not fits)
                  (said "is placed where its syntax type is not one the form takes there"))
                 ((not (levels=? (normal mx-env-indexed-env) (normal (extract info 2))))
                  (said "is placed in the scope of other bindings than its signature says it is"))
                 (else #u))))))))
(define lit-binding (->* (symbol bool atms) (listof int acyclic))
  (lambda (n d as)
    (if (null? as)
        nil
        (let ((rest (lit-binding n d (cdr as))))
          (tagcase (car as)
            (atm-lit (t e)
              (tagcase (node-kids t)
                (k-binder (m f) (if (and (symbol=? m n) (mx-eq? e d)) (list t) rest))
                (else y rest)))
            (else y rest))))))
;; The template's binder of `n` in `env`, innermost first, in a
;; description's namespace if `d`, else a value's: its node, in a list of
;; one; refused, at `s`, if the repetitions before an element bind it.
(define mx-scopes-binder-of (->* (fails symbol bool levels syn) (listof int acyclic))
  (lambda (mx-fail n d mx-env-indexed-env s)
    (if (null? mx-env-indexed-env)
        nil
        (tagcase (car mx-env-indexed-env)
          (lv-rib (as)
            (let ((here (lit-binding n d as)))
              (if (null? here) (mx-scopes-binder-of mx-fail n d (cdr mx-env-indexed-env) s) here)))
          (lv-earlier (as)
            (if (null? (lit-binding n d as))
                (mx-scopes-binder-of mx-fail n d (cdr mx-env-indexed-env) s)
                (begin (fail-at mx-fail (string-append "`" (string-append (symbol->string n) (string-append "` " mx-scopes-repeats-said))) s)
                       nil)))
          (else y (mx-scopes-binder-of mx-fail n d (cdr mx-env-indexed-env) s))))))
(define atm-in? (->* (atm atms) bool)
  (lambda (a as) (and (not (null? as)) (or (atm=? a (car as)) (atm-in? a (cdr as))))))
;; The macro readings of `rs` not yet kept, each by its list's place; a
;; reading kept that is now another, refused.
(define new-heads (->* (fails head-readings) (listof (tupleof int int int) acyclic))
  (lambda (mx-fail rs)
    (if (null? rs)
        nil
        (let* ((s (node-syn (extract (car rs) 1))) (was (head-read s)) (now (extract (car rs) 2))
               (said (lambda ((r (listof int acyclic)))
                       (if (null? r)
                           "an application"
                           "a use of the macro a use before it defines")))
               (rest (new-heads mx-fail (cdr rs))))
          (cond ((and (null? was) (null? now)) rest)
                ((null? was) (cons (tuple (syn-start s) (syn-end s) (car now)) rest))
                ((and (not (null? now)) (= (car was) (car now))) rest)
                (else
                 (begin (fail-at mx-fail (string-append "this template's scopes have no one reading: `"
                                                     (string-append (show-syn s) (string-append "`, headed by `"
                                                     (string-append (symbol->string (syn-symbol (car (split s))))
                                                     (string-append "`, is read as " (string-append (said was) (string-append " and then as "
                                                     (said now))))))))
                                 s)
                        rest)))))))
(define syntax-entries (->* (fails entries int syn (listof syntax-binder acyclic))
                            (listof syntax-binder acyclic))
  (lambda (mx-fail es root s out)
    (if (null? es)
        out
        (let ((d (extract (car es) 2)) (rest (syntax-entries mx-fail (cdr es) root s out)))
          (if (< d 100)
              rest
              (let ((t (nav mx-fail root (extract (car es) 1) s)))
                (tagcase (node-kids t)
                  (k-pvar (m) (cons (tuple #t m 0 d) rest))
                  (k-binder (m e) (if e rest (cons (tuple #f m t d) rest)))
                  (else y rest))))))))
(define syntax-binders (->* (fails int int (listof syntax-binder acyclic))
                            (listof syntax-binder acyclic))
  (lambda (mx-fail i n out)
    (if (= i n)
        out
        (syntax-binders mx-fail (+ i 1) n
          (let ((ex (node-export i)))
            (if (null? ex)
                out
                (tagcase (extract (car ex) 1)
                  (bind-rib (es) (syntax-entries mx-fail es (extract (car ex) 2) (node-syn i) out))
                  (else y out))))))))
(define found-env (->* (int mx-nodes-found) mx-env-indexed-env)
  (lambda (at fs)
    (cond ((null? fs) env-empty) ((= at (extract (car fs) 1)) (extract (car fs) 2))
          (else (found-env at (cdr fs))))))
(define syntax-of-pvar (->* (symbol (listof syntax-binder acyclic)) (listof int acyclic))
  (lambda (n sx)
    (cond ((null? sx) nil)
          ((and (extract (car sx) 1) (symbol=? n (extract (car sx) 2))) (list (extract (car sx) 4)))
          (else (syntax-of-pvar n (cdr sx))))))
(define syntax-of-node (->* (int (listof syntax-binder acyclic)) (listof int acyclic))
  (lambda (b sx)
    (cond ((null? sx) nil)
          ((and (not (extract (car sx) 1)) (= b (extract (car sx) 3))) (list (extract (car sx) 4)))
          (else (syntax-of-node b (cdr sx))))))
(define atoms-binder (->* (symbol atms (listof syntax-binder acyclic))
                          (listof (tupleof (listof int acyclic) (listof int acyclic)) acyclic))
  (lambda (h as sx)
    (if (null? as)
        nil
        (tagcase (car as)
          (atm-var (m d g)
            (if (and (symbol=? m h) (not d))
                (list (tuple (syntax-of-pvar h sx) (the (listof int acyclic) nil)))
                (atoms-binder h (cdr as) sx)))
          (atm-lit (b d)
            (tagcase (node-kids b)
              (k-binder (m e)
                (if (and (symbol=? m h) (not d))
                    (list (tuple (syntax-of-node b sx) (list b)))
                    (atoms-binder h (cdr as) sx)))
              (else y (atoms-binder h (cdr as) sx))))
          (else y (atoms-binder h (cdr as) sx))))))
;; Head `h`'s innermost binder in `ls`: its reading, and its node if a
;; template's binder.
(define head-binder (->* (symbol levels (listof syntax-binder acyclic))
                         (tupleof (listof int acyclic) (listof int acyclic)))
  (lambda (h ls sx)
    (if (null? ls)
        (tuple (the (listof int acyclic) nil) (the (listof int acyclic) nil))
        (let ((as (tagcase (car ls) (lv-rib (as) as) (lv-earlier (as) as) (else y nil))))
          (let ((r (atoms-binder h as sx)))
            (if (null? r) (head-binder h (cdr ls) sx) (car r)))))))
(define readings-of (->* (fails (listof (tupleof int bool) acyclic) mx-nodes-found
                           (listof syntax-binder acyclic))
                       head-readings)
  (lambda (mx-fail ps fs sx)
    (if (null? ps)
        nil
        (let* ((at (extract (car ps) 1))
               (h (syn-symbol (car (split (node-syn at)))))
               (mx-env-indexed-env (found-env at fs))
               (r (head-binder h (env-levels mx-env-indexed-env) sx)))
          (cons (tuple at (extract r 1) (extract r 2)) (readings-of mx-fail (cdr ps) fs sx))))))
;; How each pending list is read by the scopes the walk found, `fs`
;; (`Aligner::heads_read`), the first found first: a use of a macro if its
;; head's innermost binder is what a `(macro σ made)` entry binds, a pattern
;; variable or a template's binder at that part of a use (its δ, in a list
;; of one); else an application (none). Each with its node, and its
;; binder, if a template's, in a list of one.
(define mx-scopes-heads-read (->* (fails mx-nodes-found) head-readings)
  (lambda (mx-fail fs)
    (let ((sx (syntax-binders mx-fail 0 (get nnodes) nil)))
      (readings-of mx-fail (reverse (get pending)) fs sx))))
(define aligned-heads (->* (fails pvars sty syn) (tupleof int mx-nodes-found head-readings))
  (lambda (mx-fail ps made template)
    (let* ((root (begin (nodes-reset!) (align mx-fail 1 ps made template -1 nil)))
           (mx-nodes-found (mx-scopes-walk mx-fail root env-empty #f 0))
           (read (if (null? (get pending)) (the head-readings nil) (mx-scopes-heads-read mx-fail mx-nodes-found)))
           (new (new-heads mx-fail read)))
      (if (null? new)
          (tuple root mx-nodes-found read)
          (begin (set heads (append (get heads) new)) (aligned-heads mx-fail ps made template))))))
;; Whether value `n` is bound in `env` from its `from`th group on, past any
;; module's names.
(define mx-scopes-bound-in? (->* (symbol mx-env-indexed-env int) bool)
  (lambda (n mx-env-indexed-env from)
    (let ((l (env-lookup mx-env-indexed-env n #f from mx-nodes-binder-name)))
      (case (extract l 1) ((0) #t) ((1) (mx-scopes-bound-in? n mx-env-indexed-env (extract l 3))) (else #f)))))
(define head-bound? (->* (int mx-env-indexed-env mx-parser-types-names) bool)
  (lambda (at mx-env-indexed-env bound)
    (let* ((hd (car (split (node-syn at)))) (h (syn-symbol hd)))
      (and (= (syn-expansion hd) 0) (not (makes-desc? h))
           (or (mx-syntax-name-in? h bound) (mx-scopes-bound-in? h mx-env-indexed-env 0))))))
;; Each pending list the walk found, `fs`: its node, whether its head is
;; bound around it, and its environment.
(define heads-bound (->* (mx-nodes-found mx-parser-types-names mx-scopes-heads-read) mx-scopes-heads-read)
  (lambda (fs bound out)
    (if (null? fs)
        out
        (heads-bound (cdr fs) bound
                     (let ((at (extract (car fs) 1)) (mx-env-indexed-env (extract (car fs) 2)))
                       (if (pending-node? at)
                           (cons (tuple at (head-bound? at mx-env-indexed-env bound) mx-env-indexed-env) out)
                           out))))))
;; The paths of the lists of `rs` read otherwise than they are now.
(define heads-changed (->* (mx-scopes-heads-read (listof addr acyclic)) (listof addr acyclic))
  (lambda (rs out)
    (cond ((null? rs) out)
          ((mx-eq? (extract (car rs) 2) (addr-in? (node-path (extract (car rs) 1)) (get scoped-apps)))
           (heads-changed (cdr rs) out))
          (else (heads-changed (cdr rs) (cons (node-path (extract (car rs) 1)) out))))))
(define addr-len (->* (addr) int) (lambda (p) (if (null? p) 0 (+ 1 (addr-len (cdr p))))))
(define addr-drop (->* (addr int) addr)
  (lambda (p k) (if (= k 0) p (addr-drop (cdr p) (- k 1)))))
;; Whether the list at path `q` is around another at `p`: a program walk's
;; paths are kept innermost step first, so `q` ends `p`.
(define addr-around? (->* (addr addr) bool)
  (lambda (q p)
    (let ((d (- (addr-len p) (addr-len q)))) (and (> d 0) (mx-types-addr=? q (addr-drop p d))))))
(define around-any? (->* (addr (listof addr acyclic)) bool)
  (lambda (p qs) (and (not (null? qs)) (or (addr-around? (car qs) p) (around-any? p (cdr qs))))))
(define addr-without (->* (addr (listof addr acyclic)) (listof addr acyclic))
  (lambda (p ps)
    (cond ((null? ps) nil)
          ((mx-types-addr=? p (car ps)) (cdr ps))
          (else (cons (car ps) (addr-without p (cdr ps)))))))
;; `apps`, each of `ps` around which none of `all` is read anew.
(define apps-toggled
  (->* ((listof addr acyclic) (listof addr acyclic) (listof addr acyclic)) (listof addr acyclic))
  (lambda (ps all apps)
    (cond ((null? ps) apps)
          ((around-any? (car ps) all) (apps-toggled (cdr ps) all apps))
          ((addr-in? (car ps) apps) (apps-toggled (cdr ps) all (addr-without (car ps) apps)))
          (else (apps-toggled (cdr ps) all (cons (car ps) apps))))))
(define program-scoped-from (->* (fails sty syn mx-parser-types-names) (tupleof int mx-nodes-found mx-scopes-heads-read))
  (lambda (mx-fail core s bound)
    (let* ((root (begin (nodes-reset!) (set unplaced nil) (set program-walk #t) (set scoped-on #t)
                        (align mx-fail 3 (the pvars nil) core s -1 nil)))
           (mx-nodes-found (mx-scopes-walk mx-fail root env-empty #f 0))
           (read (begin (set program-walk #f) (set scoped-on #f) (heads-bound mx-nodes-found bound nil)))
           (changed (heads-changed read nil)))
      (if (null? changed)
          (tuple root mx-nodes-found read)
          (begin (set scoped-apps (apps-toggled changed changed (get scoped-apps)))
                 (program-scoped-from mx-fail core s bound))))))
(define program-scoped (->* (fails sty syn bool mx-parser-types-names) (tupleof int mx-nodes-found mx-scopes-heads-read))
  (lambda (mx-fail core s whole bound)
    (begin (set scoped-apps nil) (set scoped-whole whole) (program-scoped-from mx-fail core s bound))))
;; Whether `n` is a path, `a::b::c`: a program's, not a name of the
;; expander's own (`%…`).
(define path-named? (->* (string) bool)
  (lambda (n)
    (and (> (string-length n) 0) (not (char=? (string-ref n 0) #\%))
         (>= (string-search n "::" 0) 0))))
(define indices (->* (addr int) ixs)
  (lambda (steps k)
    (cond ((null? steps) (if (= k 0) nil (list (- -1 k))))
          ((car steps) (cons k (indices (cdr steps) 0)))
          (else (indices (cdr steps) (+ k 1))))))
;; Path `p`, a program walk's (innermost step first), as the indices of the
;; items it goes through, list by list, outermost first; a list's tail
;; after `k` items, `-1 - k`.
(define index-path (->* (addr) ixs) (lambda (p) (indices (reverse p) 0)))
;; Each path from node `i` to `n`, onto `out`: a reference's; a binder's,
;; refused.
(define path-parts (->* (fails int int mx-scopes-uses) mx-scopes-uses)
  (lambda (mx-fail i n out)
    (if (>= i n)
        out
        (path-parts mx-fail (+ i 1) n
                    (tagcase (node-kids i)
                      (k-ref (x d free)
                        (if (path-named? (symbol->string x))
                            (cons (tuple (index-path (node-path i)) (pt-path d)) out)
                            out))
                      (k-binder (x d)
                        (if (path-named? (symbol->string x))
                            (begin
                              (fail-at mx-fail (string-append "`" (string-append (symbol->string x) (string-append "` is a path, "
                                                           (string-append "`module::name`, not a name that may "
                                                           "be bound"))))
                                       (node-syn i))
                              out)
                            out))
                      (else y out))))))
;; `bound`, and those of `ns` that `env` binds.
(define names-bound (->* (mx-parser-types-names mx-env-indexed-env mx-parser-types-names) mx-parser-types-names)
  (lambda (ns mx-env-indexed-env bound)
    (cond ((null? ns) bound)
          ((and (not (mx-syntax-name-in? (car ns) bound)) (mx-scopes-bound-in? (car ns) mx-env-indexed-env 0))
           (cons (car ns) (names-bound (cdr ns) mx-env-indexed-env bound)))
          (else (names-bound (cdr ns) mx-env-indexed-env bound)))))
(define-rec
;; The macros' names the program wrote in `s`, onto `out`, each once.
(macro-names-in (->* (syn mx-parser-types-names) mx-parser-types-names)
  (lambda (s out)
    (tagcase s
      (mx-eager-reader-types-lst (items d a b x) (macro-names-each items out))
      (dotted (items t d a b x) (macro-names-each items (macro-names-in t out)))
      (vec (items d a b x) out)
      (atom (d a b x)
        (if (and (syn-symbol? s) (= x 0) (macro? (syn-symbol s))
                 (not (mx-syntax-name-in? (syn-symbol s) out)))
            (cons (syn-symbol s) out)
            out)))))
(macro-names-each (->* (syns-a mx-parser-types-names) mx-parser-types-names)
  (lambda (xs out) (if (null? xs) out (macro-names-each (cdr xs) (macro-names-in (car xs) out))))))
(define uses-of (->* (mx-scopes-heads-read mx-parser-types-names mx-scopes-uses) mx-scopes-uses)
  (lambda (rs bound out)
    (if (null? rs)
        out
        (uses-of (cdr rs) bound
                 (if (extract (car rs) 2)
                     out
                     (let ((at (extract (car rs) 1)))
                       (cons (tuple (index-path (node-path at))
                                    (pt-use (names-bound (macro-names-in (node-syn at) nil)
                                                         (extract (car rs) 3) bound)))
                             out)))))))
(define mx-scopes-halves (->* (mx-scopes-uses mx-scopes-uses mx-scopes-uses) (tupleof mx-scopes-uses mx-scopes-uses))
  (lambda (us a b) (if (null? us) (tuple a b) (mx-scopes-halves (cdr us) b (cons (car us) a)))))
;; An index's place among a list's parts: its items in order, then its tail.
(define ix-order (->* (int) int) (lambda (i) (if (< i 0) (+ (* 2 (- -1 i)) 1) (* 2 i))))
(define ixs<? (->* (ixs ixs) bool)
  (lambda (p q)
    (cond ((null? q) #f)
          ((null? p) #t)
          ((= (car p) (car q)) (ixs<? (cdr p) (cdr q)))
          (else (< (ix-order (car p)) (ix-order (car q)))))))
(define uses-merged (->* (mx-scopes-uses mx-scopes-uses) mx-scopes-uses)
  (lambda (a b)
    (cond ((null? a) b)
          ((null? b) a)
          ((ixs<? (extract (car b) 1) (extract (car a) 1)) (cons (car b) (uses-merged a (cdr b))))
          (else (cons (car a) (uses-merged (cdr a) b))))))
;; Uses `us` in the order they are written, by their index paths.
(define uses-sorted (->* (mx-scopes-uses) mx-scopes-uses)
  (lambda (us)
    (if (or (null? us) (null? (cdr us)))
        us
        (let ((h (mx-scopes-halves us nil nil)))
          (uses-merged (uses-sorted (extract h 1)) (uses-sorted (extract h 2)))))))
(define uses-found (->* (fails sty syn mx-parser-types-names) mx-scopes-uses)
  (lambda (mx-fail core s bound)
    (let ((read (extract (program-scoped mx-fail core s #t bound) 3)))
      (uses-sorted (path-parts mx-fail 0 (get nnodes) (uses-of read bound nil))))))
;; The part found here, of uses `us` from here: in a list of one.
(define part-here (->* (mx-scopes-uses) (listof mx-scopes-part acyclic))
  (lambda (us)
    (if (and (not (null? us)) (null? (extract (car us) 1)))
        (list (extract (car us) 2))
        nil)))
(define quote-headed? (->* (syns-a) bool)
  (lambda (xs)
    (and (not (null? xs)) (syn-symbol? (car xs)) (string=? (syn-name (car xs)) "quote"))))
(define-rec
;; Whether a macro's name or a path is written in `s`, outside a quoted
;; datum.
(names-macro? (->* (syn) bool)
  (lambda (s)
    (tagcase s
      (mx-eager-reader-types-lst (items d a b x) (and (not (quote-headed? items)) (any-names-macro? items)))
      (dotted (items t d a b x)
        (and (not (quote-headed? items)) (or (any-names-macro? items) (names-macro? t))))
      (vec (items d a b x) #f)
      (atom (d a b x)
        (and (syn-symbol? s)
             (or (macro? (syn-symbol s)) (path-named? (symbol->string (syn-symbol s)))))))))
(any-names-macro? (->* (syns-a) bool)
  (lambda (xs) (and (not (null? xs)) (or (names-macro? (car xs)) (any-names-macro? (cdr xs)))))))
(define uses-at? (->* (int mx-scopes-uses) bool)
  (lambda (k us)
    (and (not (null? us)) (not (null? (extract (car us) 1))) (= (car (extract (car us) 1)) k))))
;; Of uses `us`, in order, those in the item of index `k` (or the tail, a
;; negative one) first, each by its path from it; and those after them.
(define uses-in (->* (int mx-scopes-uses) mx-scopes-uses)
  (lambda (k us)
    (if (uses-at? k us)
        (cons (tuple (cdr (extract (car us) 1)) (extract (car us) 2)) (uses-in k (cdr us)))
        nil)))
(define uses-past (->* (int mx-scopes-uses) mx-scopes-uses)
  (lambda (k us) (if (uses-at? k us) (uses-past k (cdr us)) us)))
;; `s` at `a`..`b`, of expansion `x`.
(define at-place (->* (syn int int int) syn)
  (lambda (s a b x)
    (tagcase s
      (atom (d c e y) (atom d a b x))
      (mx-eager-reader-types-lst (items d c e y) (mx-eager-reader-types-lst items d a b x))
      (dotted (items tail d c e y) (dotted items tail d a b x))
      (vec (items d c e y) (vec items d a b x)))))
(define at-places (->* (syns-a int int) syns-a)
  (lambda (xs a b) (if (null? xs) nil (cons (at-place (car xs) a b 0) (at-places (cdr xs) a b)))))
;; Items `xs` and a tail (none, or one), as one list at `a`..`b` of
;; expansion `x`: a tail that is a list joined to it, an empty one dropped.
(define list-at (->* (int int int syns-a syns-a) syn)
  (lambda (a b x xs t)
    (cond
      ;; `()`, as the reader reads it: a list of none.
      ((and (null? t) (null? xs)) (mx-eager-reader-types-lst nil '() a b x))
      ((null? t) (mx-eager-reader-types-lst xs (syn-datums xs) a b x))
      ((syn-nil? (car t)) (list-at a b x xs nil))
      (else
       (tagcase (car t)
         (mx-eager-reader-types-lst (more d c e y) (list-at a b x (append xs more) nil))
         (dotted (more tail d c e y) (list-at a b x (append xs more) (list tail)))
         (else z
           (if (null? xs)
               (car t)
               (dotted xs (car t) (mx-datum-dotted (syn-datums xs) (syn->datum (car t)))
                       a b x))))))))
;; `s`, a list, with items `xs` and, if it is dotted, tail `t`.
(define rebuilt (->* (syn syns-a syns-a) syn)
  (lambda (s xs t)
    (list-at (syn-start s) (syn-end s) (syn-expansion s) xs t)))

;;; ============================================================ macros-match.fx (whole)

(define bound-of (->* (symbol mx-match-binds) (listof syn acyclic))
  (lambda (n bs)
    (cond ((null? bs) nil)
          ((symbol=? n (extract (car bs) 1)) (list (extract (car bs) 2)))
          (else (bound-of n (cdr bs))))))
;; The trailmark name `n` is in `ks`, or 0; and each of names `ns` with the
;; one it is bound to where it is read (`macros.rs`'s `Clause::marks`).
(define marked-id (->* (symbol marked) int)
  (lambda (n ks)
    (cond ((null? ks) 0)
          ((symbol=? n (extract (car ks) 1)) (extract (car ks) 2))
          (else (marked-id n (cdr ks))))))
(define bound-in-each (->* (symbol (listof mx-match-binds acyclic)) syns-a)
  (lambda (n each)
    (if (null? each) nil (cons (car (bound-of n (car each))) (bound-in-each n (cdr each))))))
;; `bs`, each of `ns` bound to the list of what each of `each` bound it to,
;; at `s`'s place.
(define seq-binds (->* (mx-parser-types-names (listof mx-match-binds acyclic) syn mx-match-binds) mx-match-binds)
  (lambda (ns each s bs)
    (if (null? ns)
        bs
        (let ((items (bound-in-each (car ns) each)))
          (seq-binds (cdr ns) each s
                     (cons (tuple (car ns) (mk-list items (syn-start s) (syn-end s))) bs))))))
;; A list's items (a dotted list's before its tail); none if `s` is no list.
(define list-items (->* (syn) syns-a)
  (lambda (s)
    (tagcase s (mx-eager-reader-types-lst (items d a b x) items) (dotted (items t d a b x) items) (else y nil))))
(define-rec
;; The pattern variables in pattern `p`: its names but `_`, `...` and the
;; forms' (which match themselves).
(pattern-names (->* (syn) mx-parser-types-names)
  (lambda (p)
    (cond ((syn-symbol? p)
           (if (or (string=? (syn-name p) "_") (ellipsis? p) (reserved-syn? p))
               nil
               (list (syn-symbol p))))
          (else (pattern-names-each (list-items p))))))
(pattern-names-each (->* (syns-a) mx-parser-types-names)
  (lambda (xs) (if (null? xs) nil (append (pattern-names (car xs)) (pattern-names-each (cdr xs)))))))
;; Whether two atoms are the same datum.
(define mx-match-same-atom? (->* (syn syn) bool)
  (lambda (p s)
    (tagcase p
      (atom (d a b x)
        (tagcase s
          (atom (e c f y)
            (cond ((mx-keyword? d)
                   (and (mx-keyword? e) (symbol=? (mx-keyword->symbol d) (mx-keyword->symbol e))))
                  ((string? d) (and (string? e) (string=? d e)))
                  ((int? d) (and (int? e) (= d e)))
                  ((char? d) (and (char? e) (char=? d e)))
                  ((bool? d) (and (bool? e) (mx-eq? d e)))
                  (else #f)))
          (else z #f)))
      (else y #f))))
(define-rec
;; Whether `s` matches pattern `p`, onto `bs`: its bindings, in a list of
;; one; none if not. A name is a pattern variable (`_` anything; one of
;; `ns`, the names', a name alone), a form's name or a keyword itself, or
;; one of `ks`, the clause's trailmarks, a name bound to the same one.
(matched (->* (syn syn mx-parser-types-names marked mx-match-binds) (listof mx-match-binds acyclic))
  (lambda (p s ns ks bs)
    (cond
      ;; `(x ...)`: each element of a list by `x`, each of its pattern
      ;; variables then the list of what it matched.
      ((not (null? (ellipsis-of p)))
       (if (or (syn-nil? s) (not (null? (proper-items s))))
           (let ((each (matched-each (car (ellipsis-of p)) (proper-items s) ns ks nil)))
             (if (null? each)
                 nil
                 (list (seq-binds (pattern-names (car (ellipsis-of p))) (car each) s bs))))
           nil))
      ((syn-symbol? p)
       (cond ((string=? (syn-name p) "_") (list bs))
             ((reserved-syn? p)
              (if (and (syn-symbol? s) (symbol=? (syn-symbol s) (syn-symbol p))) (list bs) nil))
             ((> (marked-id (syn-symbol p) ks) 0)
              (if (and (syn-symbol? s) (= (mark-now (syn-symbol s)) (marked-id (syn-symbol p) ks)))
                  (list bs)
                  nil))
             ((and (mx-syntax-name-in? (syn-symbol p) ns) (not (syn-symbol? s))) nil)
             (else (list (cons (tuple (syn-symbol p) s) bs)))))
      ((syn-nil? p) (if (syn-nil? s) (list bs) nil))
      ((not (null? (split p)))
       (let ((ps (split p)) (ss (split s)))
         (if (null? ss)
             nil
             (let ((first (matched (car ps) (car ss) ns ks bs)))
               (if (null? first) nil (matched (nth ps 1) (nth ss 1) ns ks (car first)))))))
      ((mx-match-same-atom? p s) (list bs))
      (else nil))))
;; What each of `xs` matched by `x`, in order, onto `out` (reversed), in a
;; list of one; none if one does not match.
(matched-each
  (->* (syn syns-a mx-parser-types-names marked (listof mx-match-binds acyclic)) (listof (listof mx-match-binds acyclic) acyclic))
  (lambda (x xs ns ks out)
    (if (null? xs)
        (list (the (listof mx-match-binds acyclic) (reverse out)))
        (let ((one (matched x (car xs) ns ks nil)))
          (if (null? one) nil (matched-each x (cdr xs) ns ks (cons (car one) out))))))))
(define mx-match-marks-of (->* (mx-parser-types-names) marked)
  (lambda (ns) (if (null? ns) nil (cons (tuple (car ns) (mark-now (car ns))) (mx-match-marks-of (cdr ns))))))
;; The first of `ls` from the `i`th whose length is not the first's; -1.
(define odd-length (->* ((listof syns-a acyclic) int) int)
  (lambda (ls i)
    (cond ((or (null? ls) (null? (cdr ls))) -1)
          ((= (len (car (cdr ls))) (len (car ls)))
           (odd-length (cons (car ls) (cdr (cdr ls))) (+ i 1)))
          (else (+ i 1)))))
(define mx-match-nth-name (->* (mx-parser-types-names int) symbol)
  (lambda (ns i) (if (= i 0) (car ns) (mx-match-nth-name (cdr ns) (- i 1)))))
(define nth-list (->* ((listof syns-a acyclic) int) syns-a)
  (lambda (ls i) (if (= i 0) (car ls) (nth-list (cdr ls) (- i 1)))))
(define mx-match-len-ints (->* ((listof int acyclic)) int)
  (lambda (is) (if (null? is) 0 (+ 1 (mx-match-len-ints (cdr is))))))
(define addr-under? (->* (addr addr) bool)
  (lambda (a p)
    (or (null? p) (and (not (null? a)) (mx-eq? (car a) (car p)) (addr-under? (cdr a) (cdr p))))))
(define ints=? (->* ((listof int acyclic) (listof int acyclic)) bool)
  (lambda (a b)
    (if (null? a) (null? b) (and (not (null? b)) (= (car a) (car b)) (ints=? (cdr a) (cdr b))))))
(define all-ints? (->* ((listof int acyclic) int) bool)
  (lambda (is k) (or (null? is) (and (= (car is) k) (all-ints? (cdr is) k)))))
;; The first `d` of `is`.
(define ints-prefix (->* ((listof int acyclic) int) (listof int acyclic))
  (lambda (is d) (if (or (= d 0) (null? is)) nil (cons (car is) (ints-prefix (cdr is) (- d 1))))))

;;; ============================================================ macros-exports.fx (whole)

(define atms-onto (->* (atms atms) atms)
  (lambda (xs as)
    (cond ((null? xs) as)
          ((atm-in? (car xs) as) (atms-onto (cdr xs) as))
          (else (atms-onto (cdr xs) (cons (car xs) as))))))
(define exports-onto (->* (levels atms mx-parser-types-names) mx-exports-exported)
  (lambda (ls as ns)
    (if (null? ls)
        (tuple as ns)
        (tagcase (car ls)
          (lv-rib (xs) (exports-onto (cdr ls) (atms-onto xs as) ns))
          (lv-earlier (xs) (exports-onto (cdr ls) (atms-onto xs as) ns))
          (lv-cage (n) (exports-onto (cdr ls) as (if (mx-syntax-name-in? n ns) ns (cons n ns))))
          (lv-shape (m) (exports-onto (cdr ls) as ns))))))
(define mx-exports-exports-of (->* (levels) mx-exports-exported)
  (lambda (env) (exports-onto (normal env) nil nil)))
(define atms-within? (->* (atms atms) bool)
  (lambda (xs ys) (or (null? xs) (and (atm-in? (car xs) ys) (atms-within? (cdr xs) ys)))))
(define names-within? (->* (mx-parser-types-names mx-parser-types-names) bool)
  (lambda (xs ys) (or (null? xs) (and (mx-syntax-name-in? (car xs) ys) (names-within? (cdr xs) ys)))))
(define exports=? (->* (mx-exports-exported mx-exports-exported) bool)
  (lambda (x y)
    (and (atms-within? (extract x 1) (extract y 1)) (atms-within? (extract y 1) (extract x 1))
         (names-within? (extract x 2) (extract y 2)) (names-within? (extract y 2) (extract x 2)))))
(define atm-strings (->* (atms) mx-nodes-strings)
  (lambda (as)
    (if (null? as)
        nil
        (let ((rest (atm-strings (cdr as))))
          (tagcase (car as)
            (atm-var (n d g) (cons (string-append "`" (string-append (symbol->string n) "`")) rest))
            (atm-rib-of (n) (cons (string-append "what `" (string-append (symbol->string n) "` defines")) rest))
            (atm-lit (t d) rest))))))
(define cage-strings (->* (mx-parser-types-names) mx-nodes-strings)
  (lambda (ns)
    (if (null? ns)
        nil
        (cons (string-append "what `" (string-append (symbol->string (car ns)) "` defines"))
              (cage-strings (cdr ns))))))
;; An export's names, as said, in order (`shown_exports`).
(define shown-exports (->* (mx-exports-exported) string)
  (lambda (e)
    (let ((ss (mx-nodes-sorted-strings (append (atm-strings (extract e 1)) (cage-strings (extract e 2)))
                              nil)))
      (if (null? ss) "nothing" (mx-nodes-joined ss)))))
(define-rec
(binders-under (->* (int symbol) nested)
  (lambda (at m)
    (tagcase (node-kids at)
      (k-pair (x y) (append (binders-under x m) (binders-under y m)))
      (k-form (x) (binders-under x m))
      (mx-nodes-k-seq (x) (binders-under x m))
      (k-desc (xs) (binders-under-each xs m))
      (k-binder (n d) (if d (the nested nil) (the nested (list (tuple n m)))))
      (else y (the nested nil)))))
(binders-under-each (->* ((listof int acyclic) symbol) nested)
  (lambda (xs m)
    (if (null? xs) nil (append (binders-under (car xs) m) (binders-under-each (cdr xs) m))))))
;; Those of nodes `i` to `j`, before `out`: from the last back, so that
;; they are in the nodes' order.
(define nested-from (->* (int int pvars nested) nested)
  (lambda (i j ps out)
    (if (< j i)
        out
        (let* ((items (proper-items (node-syn j)))
               (h (if (and (not (null? items)) (syn-symbol? (car items))) (list (car items)) nil))
               (here (tagcase (node-kids j)
                       (k-form (x)
                         (if (and (not (null? h)) (not (pvar? (syn-symbol (car h)) ps))
                                  (makes-defns? (syn-symbol (car h))))
                             (binders-under x (syn-symbol (car h)))
                             nil))
                       (else y nil))))
          (nested-from i (- j 1) ps (append here out))))))
(define nested-binders (->* (int int pvars) nested)
  (lambda (i n ps) (nested-from i (- n 1) ps nil)))
;; The macro whose use binds `n`, of `ns`, in a list of one.
(define nested-of (->* (symbol nested) (listof symbol acyclic))
  (lambda (n ns)
    (cond ((null? ns) nil)
          ((symbol=? n (extract (car ns) 1)) (list (extract (car ns) 2)))
          (else (nested-of n (cdr ns))))))
;; A name a nested use binds, `n`, which no binder of the template's binds
;; here (`b`), and the use is not said to define: refused, not read at the
;; top level, which the template would seem to mean by it.
(define unsaid-nested (->* (fails symbol (listof int acyclic) nested syn) unit)
  (lambda (mx-fail n b nested s)
    (let ((m (nested-of n nested)))
      (if (and (null? b) (not (null? m)))
          (let ((n (symbol->string n)))
            (fail-at mx-fail (string-append "`" (string-append n (string-append "` is bound in a use of `" (string-append (symbol->string (car m))
                                         (string-append "`, whose signature does not say its uses define it: "
                                         (string-append "say so (`(macro σ (defn (ℓ δ) …))`), or write `(with "
                                         (string-append "#%toplevel " (string-append n ")` for the top level's"))))))))
                     s))
          #u))))
;;; ------------------------------------------------------------ repeating

;; The pattern variables in `x` under more ellipses than `level`, in the
;; pattern's order.
(define drivers-of (->* (pvars syn int) mx-parser-types-names)
  (lambda (ps x level)
    (cond ((null? ps) nil)
          ((and (> (extract (extract (car ps) 2) 5) level)
                (mx-syntax-name-in? (extract (car ps) 1) (pattern-names x)))
           (cons (extract (car ps) 1) (drivers-of (cdr ps) x level)))
          (else (drivers-of (cdr ps) x level)))))
;; The renamings in `acts` at paths under `p`, of binders under more
;; ellipses than `level`: each with whether it is at the binder itself.
(define renames-under (->* (actions addr int) (listof (tupleof addr bool) acyclic))
  (lambda (acts p level)
    (if (null? acts)
        nil
        (let ((rest (renames-under (cdr acts) p level)) (at (extract (car acts) 1)))
          (tagcase (extract (car acts) 2)
            (act-rename (b d)
              (if (and (> d level) (addr-under? at p))
                  (cons (tuple b (mx-types-addr=? at b)) rest)
                  rest))
            (else y rest))))))
(define with-uses (ref mx-exports-uses mxp) (new nil))
;; For the checker: this parse's, in the order the expander met them.
(define with-uses-given (->* () mx-exports-uses) (lambda () (reverse (get with-uses))))
;; A template's `(with m name)`, `s`, of items `items` and head `hd`: noted,
;; for the checker to refuse if `m` gives no value `name`.
(define note-projection (->* (syn syns-a syn) unit)
  (lambda (s items hd)
    (if (and (not (= (syn-expansion hd) 0)) (= (len items) 3) (syn-symbol? (nth items 1))
             (syn-symbol? (nth items 2)) (not (string=? (syn-name (nth items 1)) "#%fx"))
             (not (string=? (syn-name (nth items 1)) "#%toplevel")))
        (set with-uses (cons (tuple (syn-start s) (syn-end s) (syn-symbol (nth items 2))
                                    (syn-start (nth items 2)) (syn-end (nth items 2)) 1)
                             (get with-uses)))
        #u)))
;; The uses of macro `n`, at `hd`, noted in each `with` around it.
(define note-uses (->* (symbol syn mx-exports-places) unit)
  (lambda (n hd withs)
    (if (null? withs)
        #u
        (begin
          (set with-uses (cons (tuple (extract (car withs) 1) (extract (car withs) 2) n
                                      (syn-start hd) (syn-end hd) 0)
                               (get with-uses)))
          (note-uses n hd (cdr withs))))))
(define path-refused (->* (fails syn) syn)
  (lambda (mx-fail s)
    (begin (fail-at mx-fail (string-append "`" (string-append (syn-name s) (string-append "` is not a path: a path is names joined "
                                        "by `::`, `module::name`")))
                    s)
           s)))
;; Name `n` at characters `a` to `b` of `s`, of its expansion.
(define named-at (->* (string int int syn) syn)
  (lambda (n a b s)
    (atom (string->symbol n) (+ (syn-start s) a) (+ (syn-start s) b) (syn-expansion s))))
;; Items `xs` as a list at the first `b` characters of `s`.
(define mx-exports-list-of (->* (syns-a int syn) syn)
  (lambda (xs b s) (mx-eager-reader-types-lst xs (syn-datums xs) (syn-start s) (+ (syn-start s) b) (syn-expansion s))))
;; `out`, the first `len` characters of path `s` read; the rest read on.
(define path-from (->* (fails syn bool syn int) syn)
  (lambda (mx-fail s d out len)
    (let* ((n (syn-name s)) (start (+ len 2)) (k (string-search n "::" start))
           (end (if (< k 0) (string-length n) k))
           (x (named-at (substring n start end) start end s)))
      (cond ((= start end) (path-refused mx-fail s))
            ((and (< k 0) d) (mx-exports-list-of (list (named-at "select" 0 end s) out x) end s))
            (else
             (let ((f (mx-exports-list-of (list (named-at "with" 0 end s) out x) end s)))
               (begin
                 (set with-uses (cons (tuple (syn-start f) (syn-end f)
                                             (string->symbol (substring n 0 end)) (syn-start x)
                                             (syn-end x) 2)
                                      (get with-uses)))
                 (if (< k 0) f (path-from mx-fail s d f end)))))))))
;; Path `s`, `a::b::c`, a symbol, where a value is (a description, if `d`):
;; `(with (with a b) c)`, or `(select (with a b) c)` (`TODO.md` §296), each
;; `with` at the part of `s` it reads and noted (2, its path to its name for
;; the name), for the checker to refuse if its module gives no value of the
;; name, as a template's: each name of a path is of the module before it,
;; never one bound around it (`Macros::path_written`).
(define path-written (->* (fails syn bool) syn)
  (lambda (mx-fail s d)
    (let ((k (string-search (syn-name s) "::" 0)))
      (if (= k 0)
          (path-refused mx-fail s)
          (path-from mx-fail s d (named-at (substring (syn-name s) 0 k) 0 k s) k)))))
(define binder-depths (->* (mx-nodes-found) depths)
  (lambda (fs)
    (cond ((null? fs) nil)
          ((tagcase (node-kids (extract (car fs) 1)) (k-binder (n d) #t) (else y #f))
           (cons (tuple (extract (car fs) 1) (extract (car fs) 4)) (binder-depths (cdr fs))))
          (else (binder-depths (cdr fs))))))
(define depth-of (->* (int depths) int)
  (lambda (b ds)
    (cond ((null? ds) 0) ((= b (extract (car ds) 1)) (extract (car ds) 2))
          (else (depth-of b (cdr ds))))))
;; A binder `b` (none or one) repeated, referred to under `depth`, fewer
;; ellipses than it is under: which repetition's is not said.
(define repeated-ref (->* (fails symbol (listof int acyclic) int depths syn) unit)
  (lambda (mx-fail n b depth bd s)
    (if (and (not (null? b)) (> (depth-of (car b) bd) depth))
        (fail-at mx-fail (string-append "`" (string-append (symbol->string n) (string-append "` is bound in each repetition of a "
                                     (string-append "part, and referred to under fewer ellipses than it is "
                                     "bound: which repetition's it is is not said"))))
                 s)
        #u)))
;;; ------------------------------------------------------------ heads read by scope

;; The pending lists' readings `rs` (`macros-scopes.fx`'s `aligned-heads`),
;; done: one where a definition is, read as no macro's use, refused as no
;; definition; each read as a use, checked (`heads-checked`), its head, a
;; template's binder, renamed with it (`clause`'s). `bd`, the binders'
;; depths; the actions to add.
(define heads-checked (ref (listof int acyclic) mxp) (new nil))
(define pending-defn? (->* (int (listof (tupleof int bool) acyclic)) bool)
  (lambda (at ps)
    (and (not (null? ps))
         (or (and (= at (extract (car ps) 1)) (extract (car ps) 2)) (pending-defn? at (cdr ps))))))
(define heads-done-each (->* (fails head-readings depths) actions)
  (lambda (mx-fail rs bd)
    (if (null? rs)
        nil
        (let* ((at (extract (car rs) 1)) (s (node-syn at)) (b (extract (car rs) 3)))
          (cond ((and (null? (extract (car rs) 2)) (pending-defn? at (get pending)))
                 (begin (not-definition mx-fail s (string-append "`define`, `define*`, `define-rec`, "
                                                              (string-append "a `begin` of them, or a use of "
                                                              "another such")))
                        nil))
                ((null? (extract (car rs) 2)) (heads-done-each mx-fail (cdr rs) bd))
                (else
                 (begin
                   (set heads-checked (cons (syn-start s) (get heads-checked)))
                   (let ((rest (heads-done-each mx-fail (cdr rs) bd)))
                     (if (null? b)
                         rest
                         (cons (tuple (append (node-path at) (the addr (list #t)))
                                      (act-rename (node-path (car b)) (depth-of (car b) bd)))
                               rest))))))))))
(define heads-done (->* (fails head-readings depths) actions)
  (lambda (mx-fail rs bd)
    (begin (set heads-checked nil) (heads-done-each mx-fail rs bd))))

;;; ============================================================ macros-facts.fx (whole)

;; The named shapes: name, definition, what each exports; those being
;; written, innermost first.
(define mx-facts-named (ref (listof (tupleof symbol syn int) acyclic) mxp) (new nil))
(define open (ref mx-parser-types-names mxp) (new nil))
(define mx-facts-head-is? (->* (syn string) bool)
  (lambda (s h)
    (let ((ps (split s)))
      (and (not (null? ps)) (syn-symbol? (car ps)) (string=? (syn-name (car ps)) h)))))
;; Whether signature `s` is one in shapes and facts: `(macro SHAPE …)`, its
;; shape `(_ …)` or `(letrec-shape …)`.
(define contract? (->* (syn) bool)
  (lambda (s)
    (let ((items (proper-items s)))
      (and (>= (len items) 3) (mx-facts-head-is? s "macro")
           (or (mx-facts-head-is? (nth items 1) "_") (mx-facts-head-is? (nth items 1) "letrec-shape"))))))
(define mx-facts-sym-is? (->* (syn string) bool)
  (lambda (s n) (and (syn-symbol? s) (string=? (syn-name s) n))))
;; A list's parts, and its tail (none, or one).
(define list-parts (->* (syn) (tupleof syns-a syns-a))
  (lambda (s)
    (tagcase s
      (mx-eager-reader-types-lst (items d a b x) (tuple items (the syns-a nil)))
      (dotted (items tail d a b x) (tuple items (list tail)))
      (else y (tuple (the syns-a nil) (the syns-a nil))))))
;; Items `xs` with tail `t` (none, or one) as one list: a tail that is a
;; list joined to it.
(define list-with (->* (syns-a syns-a int int) syn)
  (lambda (xs t a b)
    (cond ((null? t) (mk-list xs a b))
          ;; No items: the tail itself.
          ((null? xs) (car t))
          (else
           (tagcase (car t)
             (mx-eager-reader-types-lst (ys d c e x) (mk-list (append xs ys) a b))
             (dotted (ys u d c e x) (list-with (append xs ys) (list u) a b))
             (else y (if (syn-nil? (car t))
                         (mk-list xs a b)
                         (dotted xs (car t) (mx-datum-dotted (syn-datums xs) (syn->datum (car t)))
                                 a b 0))))))))
;;; ------------------------------------------------------------ shapes

;; `SHAPE`: a `letrec-shape`'s names noted, its top given back.
(define shape-usage string
  (string-append "`(letrec-shape ((name node) …) (_ part …))`, a node `(node-name tree fact "
                 "…)` or `(union node …)`"))
(define mx-facts-named-of (->* (symbol (listof (tupleof symbol syn int) acyclic))
                      (listof (tupleof symbol syn int) acyclic))
  (lambda (n xs)
    (cond ((null? xs) nil) ((symbol=? n (extract (car xs) 1)) (list (car xs)))
          (else (mx-facts-named-of n (cdr xs))))))
(define note-named (->* (fails syns-a) unit)
  (lambda (mx-fail ds)
    (if (null? ds)
        #u
        (let ((d (proper-items (car ds))))
          (cond ((or (not (= (len d) 2)) (not (syn-symbol? (car d))))
                 (fail-at mx-fail shape-usage (car ds)))
                ((not (null? (mx-facts-named-of (syn-symbol (car d)) (get mx-facts-named))))
                 (fail-at mx-fail (string-append "`" (string-append (syn-name (car d)) "` is a shape's name twice"))
                          (car d)))
                (else (begin (set mx-facts-named (cons (tuple (syn-symbol (car d)) (nth d 1) 0)
                                              (get mx-facts-named)))
                             (note-named mx-fail (cdr ds)))))))))
(define node-gives (->* (syns-a int) int)
  (lambda (fs g)
    (cond ((null? fs) g)
          ((mx-facts-head-is? (car fs) "export") (node-gives (cdr fs) 4))
          ((mx-facts-head-is? (car fs) "export*") (node-gives (cdr fs) 5))
          (else (node-gives (cdr fs) g)))))
(define-rec
;; What a node, or each member of a union, exports: 4 an `export`'s rib, 5
;; an `export*`'s ribcage, 0 none.
(def-gives (->* (fails syn) int)
  (lambda (mx-fail def)
    (let ((items (proper-items def)))
      (if (mx-facts-head-is? def "union")
          (members-gives mx-fail (cdr items) 0)
          (node-gives (if (> (len items) 2) (cdr (cdr items)) nil) 0)))))
(members-gives (->* (fails syns-a int) int)
  (lambda (mx-fail ms g)
    (if (null? ms)
        g
        (let ((mg (def-gives mx-fail (car ms))))
          (cond ((= mg 0) (members-gives mx-fail (cdr ms) g))
                ((and (not (= g 0)) (not (= g mg)))
                 (begin (fail-at mx-fail (string-append "a union's members export a rib and a "
                                                     "ribcage, `export` and `export*`: one kind")
                                 (car ms))
                        g))
                (else (members-gives mx-fail (cdr ms) mg))))))))
(define named-gives (->* (fails (listof (tupleof symbol syn int) acyclic))
                         (listof (tupleof symbol syn int) acyclic))
  (lambda (mx-fail xs)
    (if (null? xs)
        nil
        (cons (tuple (extract (car xs) 1) (extract (car xs) 2)
                     (def-gives mx-fail (extract (car xs) 2)))
              (named-gives mx-fail (cdr xs))))))
(define mx-facts-shape (->* (fails syn) syn)
  (lambda (mx-fail s)
    (if (not (mx-facts-head-is? s "letrec-shape"))
        s
        (let ((items (proper-items s)))
          (if (not (= (len items) 3))
              (begin (fail-at mx-fail shape-usage s) s)
              (begin (note-named mx-fail (proper-items (nth items 1)))
                     (set mx-facts-named (named-gives mx-fail (get mx-facts-named)))
                     (nth items 2)))))))
(define list-drop (->* (syns-a int) syns-a)
  (lambda (xs k) (if (or (= k 0) (null? xs)) xs (list-drop (cdr xs) (- k 1)))))
(define mx-facts-label-of (->* (symbol mx-facts-labels) mx-facts-labels)
  (lambda (n ls)
    (cond ((null? ls) nil) ((symbol=? n (extract (car ls) 1)) (list (car ls)))
          (else (mx-facts-label-of n (cdr ls))))))
(define labelled-name (->* (fails symbol (tupleof int mx-facts-steps mx-facts-steps int) mx-facts-labels syn) mx-facts-labels)
  (lambda (mx-fail n l ls at)
    (if (not (null? (mx-facts-label-of n ls)))
        (begin (fail-at mx-fail (string-append "`" (string-append (symbol->string n) "` labels two parts")) at) ls)
        (cons (tuple n (extract l 1) (extract l 2) (extract l 3) (extract l 4)) ls))))
;; Each label of a repeated element, the list's at `p`, onto `ls`.
(define repeated-labels (->* (fails mx-facts-labels mx-facts-steps mx-facts-labels syn) mx-facts-labels)
  (lambda (mx-fail inner p ls t)
    (cond ((null? inner) ls)
          ((= (extract (car inner) 2) 1)
           (begin (fail-at mx-fail "a repeated part in a repeated part: not yet" t) ls))
          (else
           (repeated-labels mx-fail (cdr inner) p
                            (labelled-name mx-fail (extract (car inner) 1)
                                           (tuple 1 p (extract (car inner) 3)
                                                  (extract (car inner) 5))
                                           ls t)
                            t)))))
;; Label `n` (a name), of kind, paths and gives `l`, onto `ls`.
(define mx-facts-labelled (->* (fails syn (tupleof int mx-facts-steps mx-facts-steps int) mx-facts-labels) mx-facts-labels)
  (lambda (mx-fail n l ls)
    (if (not (syn-symbol? n))
        (begin (fail-at mx-fail "a part's label is a name, `name @ σ`" n) ls)
        (labelled-name mx-fail (syn-symbol n) l ls n))))
;; What a part of type `t` gives: a binder, itself; a part of a named shape,
;; what its node exports.
(define gives (->* (syn) int)
  (lambda (t)
    (cond ((mx-facts-sym-is? t "bvar") 1) ((mx-facts-sym-is? t "dvar") 2) ((mx-facts-sym-is? t "place") 3)
          ((and (syn-symbol? t) (not (null? (mx-facts-named-of (syn-symbol t) (get mx-facts-named)))))
           (extract (car (mx-facts-named-of (syn-symbol t) (get mx-facts-named))) 3))
          ((mx-facts-head-is? t "bvar") 1)
          (else 0))))
;; Whether list `s` is a syntax type's form, not a list of parts.
(define form? (->* (syn) bool)
  (lambda (s)
    (let ((items (proper-items s)))
      (or (mx-facts-head-is? s "many") (mx-facts-head-is? s "literal") (mx-facts-head-is? s "trailmark") (mx-facts-head-is? s "oneof")
          (mx-facts-head-is? s "union")
          (and (or (mx-facts-head-is? s "expr") (mx-facts-head-is? s "bvar") (mx-facts-head-is? s "lambda"))
               (> (len items) 1) (syn-keyword? (nth items 1)))))))
(define mx-facts-name-in? (->* (symbol mx-parser-types-names) bool)
  (lambda (n ns) (and (not (null? ns)) (or (symbol=? n (car ns)) (mx-facts-name-in? n (cdr ns))))))
(define mx-facts-path-keys (->* (mx-facts-steps int int) syns-a)
  (lambda (p a b)
    (if (null? p)
        nil
        (cons (atom (mx-symbol->keyword (if (car p) 'a 'd)) a b 0) (mx-facts-path-keys (cdr p) a b)))))
;; `(path #:a #:d …)`, or `()`.
(define path-syn (->* (mx-facts-steps int int) syn)
  (lambda (p a b)
    (if (null? p) (mk-list nil a b) (mk-list (cons (mk-symbol "path" a b) (mx-facts-path-keys p a b)) a b))))
(define rib-entry (->* (fails int int mx-facts-steps int) syn)
  (lambda (mx-fail a b p g)
    (mk-list (list (path-syn p a b)
                   (mk-symbol (cond ((= g 1) "var") ((= g 2) "dvar") ((= g 3) "place")
                                    ((= g 4) "rib") (else "none"))
                              a b))
             a b)))
(define steps=? (->* (mx-facts-steps mx-facts-steps) bool)
  (lambda (p q)
    (cond ((null? p) (null? q)) ((null? q) #f)
          (else (and (mx-eq? (car p) (car q)) (steps=? (cdr p) (cdr q)))))))
;; A fact under `...`, `(bind v in e) ...`, each repetition's: `v` and `e`
;; labels of one list's element, each the repetition's own; the list's
;; path, `e`'s in the element, and what it imports there, addressed from the
;; element's root, in a list of one (`contract.rs`'s `each_bound`). One that
;; names what is outside its repetition needs an import from an outer
;; region's root: not yet (§289).
(define each-bound (->* (fails syn mx-facts-labels) eaches)
  (lambda (mx-fail f ls)
    (let ((items (proper-items f)) (a (syn-start f)) (b (syn-end f))
          (outside (string-append "a fact under `...` that names what is outside its "
                                  "repetition: not yet (TODO §289)")))
      (if (not (and (mx-facts-head-is? f "bind") (= (len items) 4) (mx-facts-sym-is? (nth items 2) "in")))
          (begin (fail-at mx-fail (string-append "a fact under `...` is a `bind`, each repetition's: "
                                              (string-append "what a list exports is every repetition's, `(export "
                                              "(x ...))`"))
                          f)
                 nil)
          (let* ((l (nth items 1)) (m (nth items 3))
                 (ll (if (syn-symbol? l) (mx-facts-label-of (syn-symbol l) ls) nil))
                 (ml (if (syn-symbol? m) (mx-facts-label-of (syn-symbol m) ls) nil)))
            (cond
              ((and (syn-symbol? l) (null? ll))
               (begin (fail-at mx-fail (string-append "`" (string-append (syn-name l) (string-append "` labels no part of this "
                                                   "node")))
                               l)
                      nil))
              ((or (null? ll) (= (extract (car ll) 2) 0)) (begin (fail-at mx-fail outside l) nil))
              ((and (syn-symbol? m) (null? ml))
               (begin (fail-at mx-fail (string-append "`" (string-append (syn-name m) (string-append "` labels no part of this "
                                                   "node")))
                               m)
                      nil))
              ((or (null? ml) (= (extract (car ml) 2) 0)) (begin (fail-at mx-fail outside m) nil))
              ((not (steps=? (extract (car ll) 3) (extract (car ml) 3)))
               (begin (fail-at mx-fail (string-append "a fact under `...` names one list's "
                                                   "repetition: these are of two lists")
                               f)
                      nil))
              ((= (extract (car ll) 5) 0)
               (begin (fail-at mx-fail (string-append "`" (string-append (syn-name l) (string-append "` labels a part that binds "
                                                   "nothing")))
                               l)
                      nil))
              (else
               (let* ((g (extract (car ll) 5)) (ep (extract (car ll) 4))
                      (bd (if (= g 5)
                              (mk-list (list (mk-symbol "cage" a b)
                                             (mk-list (list (mk-symbol "at" a b) (path-syn ep a b))
                                                      a b))
                                       a b)
                              (mk-list (list (mk-symbol "rho" a b) (rib-entry mx-fail a b ep g))
                                       a b))))
                 (list (tuple (extract (car ll) 3) (extract (car ml) 4) bd))))))))))
(define each-facts (->* (fails syns-a mx-facts-labels syns-a eaches) (tupleof syns-a eaches))
  (lambda (mx-fail fs ls out each)
    (cond ((null? fs) (tuple (reverse out) each))
          ((and (not (null? (cdr fs))) (mx-facts-sym-is? (car (cdr fs)) "..."))
           (begin
             (if (and (not (null? (cdr (cdr fs)))) (mx-facts-sym-is? (car (cdr (cdr fs))) "..."))
                 (fail-at mx-fail "a fact under more than one `...`: not yet" (car (cdr (cdr fs))))
                 #u)
             (let ((e (each-bound mx-fail (car fs) ls)))
               (each-facts mx-fail (cdr (cdr fs)) ls out (if (null? e) each (cons (car e) each))))))
          (else (each-facts mx-fail (cdr fs) ls (cons (car fs) out) each)))))
;; `(car . cdr)` at `s`'s place, a list's parts kept a list.
(define cons-syn (->* (syn syn syn) syn)
  (lambda (car-s cdr-s s) (list-with (list car-s) (list cdr-s) (syn-start s) (syn-end s))))
;; `sigma` with the part at `path` annotated, `(how part b)`.
(define annotate (->* (fails syn mx-facts-steps string syn) syn)
  (lambda (mx-fail sigma path how bd)
    (if (null? path)
        (mk-list (list (mk-symbol how (syn-start sigma) (syn-end sigma)) sigma bd)
                 (syn-start sigma) (syn-end sigma))
        (let ((ps (split sigma)))
          (if (null? ps)
              (begin (fail-at mx-fail "a label's part not found in its tree" sigma) sigma)
              (if (car path)
                  (cons-syn (annotate mx-fail (car ps) (cdr path) how bd) (nth ps 1) sigma)
                  (cons-syn (car ps) (annotate mx-fail (nth ps 1) (cdr path) how bd) sigma)))))))
(define in-element (->* (fails syn mx-facts-steps mx-facts-steps syn) syn)
  (lambda (mx-fail sigma lp p bd)
    (let ((ps (split sigma)))
      (cond ((null? ps) (begin (fail-at mx-fail "a repeated part not found in its tree" sigma) sigma))
            ((null? lp) (cons-syn (annotate mx-fail (car ps) p "import" bd) (nth ps 1) sigma))
            ((car lp) (cons-syn (in-element mx-fail (car ps) (cdr lp) p bd) (nth ps 1) sigma))
            (else (cons-syn (car ps) (in-element mx-fail (nth ps 1) (cdr lp) p bd) sigma))))))
;; `sigma` with each repetition's facts `xs` made: the part a fact names in
;; the repeated element of the list at its path imported its binding.
(define each-annotated (->* (fails syn eaches) syn)
  (lambda (mx-fail sigma xs)
    (if (null? xs)
        sigma
        (each-annotated mx-fail (in-element mx-fail sigma (extract (car xs) 1) (extract (car xs) 2)
                                         (extract (car xs) 3))
                        (cdr xs)))))
;; `L`, a label or `(label ...)`, as a rib's entry `(path δ)`, or, in a
;; ribcage (`ring`), a ring: `(rho (path δ))` or `(at path)`; and what each
;; repeated element must export for it, by the list's path.
(define mx-facts-entry (->* (fails syn mx-facts-labels bool) (tupleof syn adds))
  (lambda (mx-fail l ls ring)
    (let* ((items (proper-items l))
           (rep (and (= (len items) 2) (mx-facts-sym-is? (nth items 1) "...")))
           (n (if rep (car items) l))
           (a (syn-start l)) (b (syn-end l))
           (none (tuple (mk-list nil a b) (the adds nil))))
      (if (not (syn-symbol? n))
          (begin (fail-at mx-fail "a fact names a label, or `(label ...)`, every repetition's" l) none)
          (let ((found (mx-facts-label-of (syn-symbol n) ls)) (name (syn-name n)))
            (cond
              ((null? found)
               (begin (fail-at mx-fail (string-append "`" (string-append name "` labels no part of this node")) l)
                      none))
              ((and (= (extract (car found) 2) 0) rep)
               (begin (fail-at mx-fail (string-append "`" (string-append name (string-append "` is not repeated: named alone, not `("
                                                   (string-append name " ...)`"))))
                               l)
                      none))
              ((and (= (extract (car found) 2) 1) (not rep))
               (begin (fail-at mx-fail (string-append "`" (string-append name (string-append "` is a repeated part's: `(" (string-append name
                                                   (string-append " ...)`, every repetition's, or a fact under "
                                                   "`...`, each repetition's")))))
                               l)
                      none))
              (else
               (let* ((lab (car found)) (p (extract lab 3))
                      (g (if rep 4 (extract lab 5)))
                      (adds (if (and rep (not (= (extract lab 5) 0)))
                                (list (tuple p (rib-entry mx-fail a b (extract lab 4)
                                                          (extract lab 5))))
                                nil)))
                 (cond
                   ((= g 0)
                    (begin (fail-at mx-fail (string-append "`" (string-append name (string-append "` labels a part that binds "
                                                        "nothing")))
                                    l)
                           none))
                   ((= g 5) (tuple (mk-list (list (mk-symbol "at" a b) (path-syn p a b)) a b) adds))
                   (ring (tuple (mk-list (list (mk-symbol "rho" a b) (rib-entry mx-fail a b p g)) a b)
                                adds))
                   (else (tuple (rib-entry mx-fail a b p g) adds)))))))))))
;; Each of `L`s `xs` an entry (or a ring), onto `out` (the last first); and
;; what each repeated element must export for it.
(define mx-facts-entries-of (->* (fails syns-a mx-facts-labels bool syns-a adds) (tupleof syns-a adds))
  (lambda (mx-fail xs ls ring out elems)
    (if (null? xs)
        (tuple out elems)
        (let ((e (mx-facts-entry mx-fail (car xs) ls ring)))
          (mx-facts-entries-of mx-fail (cdr xs) ls ring (cons (extract e 1) out)
                      (append (extract e 2) elems))))))
;; What `(bind L in …)` imports: one level of `L`'s, a ribcage if it gives
;; one.
(define mx-facts-bound (->* (fails syn mx-facts-labels) (tupleof syn adds))
  (lambda (mx-fail l ls)
    (let* ((e (mx-facts-entry mx-fail l ls #f)) (a (syn-start l)) (b (syn-end l))
           (cage (mx-facts-head-is? (extract e 1) "at")))
      (tuple (mk-list (list (mk-symbol (if cage "cage" "rho") a b) (extract e 1)) a b)
             (extract e 2)))))
;; The facts read: imports (by path, the last first), a rib's entries (the
;; last first), a ribcage's rings (innermost first), whether there is one,
;; each repeated element's entries (by the list's path, the last first).
(define facts-each
  (->* (fails syns-a mx-facts-labels bool adds syns-a syns-a syns-a adds)
       (tupleof adds syns-a syns-a syns-a adds))
  (lambda (mx-fail fs ls top imports rib rings star elems)
    (if (null? fs)
        (tuple imports rib rings star elems)
        (let* ((f (car fs)) (items (proper-items f)))
          (cond
            ((and (mx-facts-head-is? f "bind") (= (len items) 4) (mx-facts-sym-is? (nth items 2) "in"))
             (let* ((m (nth items 3))
                    (target (if (syn-symbol? m) (mx-facts-label-of (syn-symbol m) ls) nil)))
               (cond ((null? target)
                      (begin (fail-at mx-fail "`(bind L in name)`: `name` a label of this node" m)
                             (tuple imports rib rings star elems)))
                     ((= (extract (car target) 2) 1)
                      (begin (fail-at mx-fail (string-append "a fact binding in a repeated part is "
                                                          "each repetition's: followed by `...`")
                                      m)
                             (tuple imports rib rings star elems)))
                     (else
                      (let ((e (mx-facts-bound mx-fail (nth items 1) ls)))
                        (facts-each mx-fail (cdr fs) ls top
                                    (cons (tuple (extract (car target) 3) (extract e 1)) imports)
                                    rib rings star (append (extract e 2) elems)))))))
            ((and (mx-facts-head-is? f "export") (not top))
             (let ((r (mx-facts-entries-of mx-fail (cdr items) ls #f nil elems)))
               (facts-each mx-fail (cdr fs) ls top imports (append (extract r 1) rib) rings star
                           (extract r 2))))
            ((and (mx-facts-head-is? f "export*") (not top))
             (let ((r (mx-facts-entries-of mx-fail (cdr items) ls #t nil elems)))
               ;; Innermost first: the last written.
               (facts-each mx-fail (cdr fs) ls top imports rib (extract r 1) (list f) (extract r 2))))
            ((or (mx-facts-head-is? f "export") (mx-facts-head-is? f "export*"))
             (begin (fail-at mx-fail (string-append "what a use defines is said in what its "
                                                 (string-append "signature makes, `(defn label …)`: the top node "
                                                 "exports nothing"))
                             f)
                    (tuple imports rib rings star elems)))
            (else
             (begin (fail-at mx-fail (string-append "a fact is `(bind L in name)`, `(export L …)` or "
                                                 "`(export* L …)`")
                             f)
                    (tuple imports rib rings star elems))))))))
(define imports-annotated (->* (fails syn adds) syn)
  (lambda (mx-fail sigma xs)
    (if (null? xs)
        sigma
        (imports-annotated mx-fail
                           (annotate mx-fail sigma (extract (car xs) 1) "import" (extract (car xs) 2))
                           (cdr xs)))))
(define syn-in? (->* (syn syns-a) bool)
  (lambda (x ys)
    (and (not (null? ys))
         (or (string=? (show-syn x) (show-syn (car ys))) (syn-in? x (cdr ys))))))
;; Entries `xs`, each once, the first kept, past those `seen`: however many
;; facts name `(v ...)`, an element exports `v` once.
(define distinct-from (->* (syns-a syns-a) syns-a)
  (lambda (xs seen)
    (cond ((null? xs) nil)
          ((syn-in? (car xs) seen) (distinct-from (cdr xs) seen))
          (else (cons (car xs) (distinct-from (cdr xs) (cons (car xs) seen)))))))
(define adds-at (->* (mx-facts-steps adds) syns-a)
  (lambda (p xs)
    (cond ((null? xs) nil)
          ((steps=? p (extract (car xs) 1)) (cons (extract (car xs) 2) (adds-at p (cdr xs))))
          (else (adds-at p (cdr xs))))))
(define adds-not-at (->* (mx-facts-steps adds) adds)
  (lambda (p xs)
    (cond ((null? xs) nil)
          ((steps=? p (extract (car xs) 1)) (adds-not-at p (cdr xs)))
          (else (cons (car xs) (adds-not-at p (cdr xs)))))))
(define element-annotated (->* (fails syn mx-facts-steps syns-a) syn)
  (lambda (mx-fail sigma path es)
    (let ((ps (split sigma)) (a (syn-start sigma)) (b (syn-end sigma)))
      (cond ((null? ps) (begin (fail-at mx-fail "a repeated part not found in its tree" sigma) sigma))
            ;; The list: `(elem ...)`.
            ((null? path)
             (cons-syn (mk-list (list (mk-symbol "export" a b) (car ps)
                                      (mk-list (cons (mk-symbol "rho" a b) es) a b))
                                a b)
                       (nth ps 1) sigma))
            ((car path) (cons-syn (element-annotated mx-fail (car ps) (cdr path) es) (nth ps 1) sigma))
            (else (cons-syn (car ps) (element-annotated mx-fail (nth ps 1) (cdr path) es) sigma))))))
;; `sigma` with each repeated element of the lists at the paths `xs` give
;; exporting their entries, one rib each.
(define elements-annotated (->* (fails syn adds) syn)
  (lambda (mx-fail sigma xs)
    (if (null? xs)
        sigma
        (let* ((p (extract (car xs) 1)) (same (distinct-from (adds-at p xs) nil))
               (rest (adds-not-at p xs)))
          (elements-annotated mx-fail (element-annotated mx-fail sigma p same) rest)))))
(define mx-facts-facts (->* (fails syn syns-a mx-facts-labels bool adds) syn)
  (lambda (mx-fail sigma fs ls top pre)
    (let* ((e (each-facts mx-fail fs ls nil nil))
           (r (facts-each mx-fail (extract e 1) ls top nil nil nil nil (reverse pre)))
           (imports (extract r 1)) (rib (extract r 2)) (rings (extract r 3)) (elems (extract r 5))
           (a (syn-start sigma)) (b (syn-end sigma))
           ;; Each repetition's facts, inside its element; then each element
           ;; exporting what a fact says of its labels.
           (s0 (each-annotated mx-fail sigma (reverse (extract e 2))))
           (s1 (elements-annotated mx-fail s0 (reverse elems)))
           (s2 (imports-annotated mx-fail s1 (reverse imports))))
      (cond ((and (not (null? rib)) (not (null? (extract r 4))))
             (begin (fail-at mx-fail (string-append "a node exports one level, `export`, or nested "
                                                 "ones, `export*`, not both")
                             sigma)
                    s2))
            ((not (null? rib))
             (annotate mx-fail s2 nil "export"
                       (mk-list (cons (mk-symbol "rho" a b) (reverse rib)) a b)))
            ((not (null? (extract r 4)))
             (annotate mx-fail s2 nil "export" (mk-list (cons (mk-symbol "cage" a b) rings) a b)))
            (else s2)))))
(define-rec
;;; ------------------------------------------------------------ trees

;; Tree `s` at `path` from its region's root, as a syntax type, with the
;; labels its labelled parts add to `ls` (`contract.rs`'s `tree`).
(tree (->* (fails syn mx-facts-steps mx-facts-labels) (tupleof syn mx-facts-labels))
  (lambda (mx-fail s path ls)
    (cond ((form? s) (mx-facts-form mx-fail s path ls))
          ((syn-nil? s) (tuple s ls))
          ((not (null? (split s)))
           (let* ((ps (list-parts s)) (r (mx-facts-parts mx-fail (extract ps 1) (extract ps 2) path ls nil)))
             (tuple (list-with (extract r 1) (extract r 2) (syn-start s) (syn-end s))
                    (extract r 3))))
          ((and (syn-symbol? s) (not (null? (mx-facts-named-of (syn-symbol s) (get mx-facts-named)))))
           (tuple (mx-facts-reference mx-fail s (syn-symbol s)) ls))
          ((mx-facts-sym-is? s "place") (tuple (mk-symbol "dvar" (syn-start s) (syn-end s)) ls))
          (else (tuple s ls)))))
;; A list's parts `xs` (its tail `t`) at `p`, onto `out` (the last first):
;; the syntax types, the tail, the labels.
(mx-facts-parts (->* (fails syns-a syns-a mx-facts-steps mx-facts-labels syns-a) (tupleof syns-a syns-a mx-facts-labels))
  (lambda (mx-fail xs t p ls out)
    (cond
      ((null? xs)
       (if (null? t)
           (tuple (reverse out) (the syns-a nil) ls)
           (let ((r (tree mx-fail (car t) p ls)))
             (tuple (reverse out) (list (extract r 1)) (extract r 2)))))
      ;; A tail labelled: `name @ . σ`.
      ((and (= (len xs) 2) (not (null? t)) (mx-facts-sym-is? (nth xs 1) "@"))
       (let* ((ls2 (mx-facts-labelled mx-fail (car xs) (tuple 0 p (the mx-facts-steps nil) (gives (car t))) ls))
              (r (tree mx-fail (car t) p ls2)))
         (tuple (reverse out) (list (extract r 1)) (extract r 2))))
      (else
       (let* ((lab (and (> (len xs) 2) (mx-facts-sym-is? (nth xs 1) "@")))
              (used (if lab 3 1))
              (t1 (if lab (nth xs 2) (car xs)))
              (rest (list-drop xs used))
              (ap (append p (the mx-facts-steps (list #t)))))
         (if (and (not (null? rest)) (mx-facts-sym-is? (car rest) "..."))
             ;; `part ...`: the element a region of its own, its labels the
             ;; list's, repeated, the list at `p`.
             (let* ((r (tree mx-fail t1 nil nil))
                    (inner (if lab
                               (mx-facts-labelled mx-fail (car xs) (tuple 0 (the mx-facts-steps nil) (the mx-facts-steps nil)
                                                              (gives t1))
                                         (extract r 2))
                               (extract r 2)))
                    (ls2 (repeated-labels mx-fail inner p ls t1)))
               (begin
                 (if (or (not (null? (cdr rest))) (not (null? t)))
                     (fail-at mx-fail "`...` comes last in a list" (car rest))
                     #u)
                 (tuple (reverse (cons (car rest) (cons (extract r 1) out))) (the syns-a nil) ls2)))
             (let* ((ls2 (if lab (mx-facts-labelled mx-fail (car xs) (tuple 0 ap (the mx-facts-steps nil) (gives t1)) ls)
                             ls))
                    (r (tree mx-fail t1 ap ls2)))
               (mx-facts-parts mx-fail rest t (append p (the mx-facts-steps (list #f))) (extract r 2)
                      (cons (extract r 1) out)))))))))
;; A syntax type's form: `(many σ)` its part a tree; a union's members each a
;; region with no labels; the rest as written.
(mx-facts-form (->* (fails syn mx-facts-steps mx-facts-labels) (tupleof syn mx-facts-labels))
  (lambda (mx-fail s path ls)
    (let ((items (proper-items s)) (a (syn-start s)) (b (syn-end s)))
      (cond ((and (mx-facts-head-is? s "many") (= (len items) 2))
             (let ((r (tree mx-fail (nth items 1) path ls)))
               (tuple (mk-list (list (car items) (extract r 1)) a b) (extract r 2))))
            ((or (mx-facts-head-is? s "union") (mx-facts-head-is? s "oneof"))
             (tuple (mk-list (cons (mk-symbol "oneof" a b) (unlabelled-members mx-fail (cdr items)))
                             a b)
                    ls))
            (else (tuple s ls))))))
(unlabelled-members (->* (fails syns-a) syns-a)
  (lambda (mx-fail ms)
    (if (null? ms)
        nil
        (let ((r (tree mx-fail (car ms) nil nil)))
          (begin
            (if (null? (extract r 2))
                #u
                (fail-at mx-fail (string-append "a label in a union's member that has no name: name "
                                             (string-append "the member, a node in a `letrec-shape`, whose facts "
                                             "may say it"))
                         (car ms)))
            (cons (extract r 1) (unlabelled-members mx-fail (cdr ms))))))))
;; A named shape referred to: its node, or union, written once, `(mu name
;; σ)`; inside its own writing, the `mu`'s variable.
(mx-facts-reference (->* (fails syn symbol) syn)
  (lambda (mx-fail s n)
    (if (mx-facts-name-in? n (get open))
        s
        (let* ((pushed (set open (cons n (get open))))
               (r (def-sigma mx-fail (extract (car (mx-facts-named-of n (get mx-facts-named))) 2)))
               (popped (set open (cdr (get open)))))
          (mk-list (list (mk-symbol "mu" (syn-start s) (syn-end s)) s r)
                   (syn-start s) (syn-end s))))))
;; A node's or a union's syntax type: each member a region of its own, its
;; facts its annotations; a union's members exporting one kind, one that
;; says none an empty one.
(def-sigma (->* (fails syn) syn)
  (lambda (mx-fail def)
    (if (mx-facts-head-is? def "union")
        (let ((kind (def-gives mx-fail def)) (a (syn-start def)) (b (syn-end def)))
          (mk-list (cons (mk-symbol "oneof" a b) (members-sigma mx-fail (cdr (proper-items def)) kind))
                   a b))
        (node-sigma mx-fail def))))
(members-sigma (->* (fails syns-a int) syns-a)
  (lambda (mx-fail ms kind)
    (if (null? ms)
        nil
        (let* ((t (node-sigma mx-fail (car ms))) (a (syn-start (car ms))) (b (syn-end (car ms)))
               (t2 (if (and (not (= kind 0)) (= (def-gives mx-fail (car ms)) 0))
                       (mk-list (list (mk-symbol "export" a b) t
                                      (mk-list (list (mk-symbol (if (= kind 5) "cage" "rho") a b))
                                               a b))
                                a b)
                       t)))
          (cons t2 (members-sigma mx-fail (cdr ms) kind))))))
(node-sigma (->* (fails syn) syn)
  (lambda (mx-fail node)
    (let ((items (proper-items node)))
      (if (< (len items) 2)
          (begin (fail-at mx-fail "a node is `(node-name tree fact …)`" node) node)
          (let ((r (tree mx-fail (nth items 1) nil nil)))
            (mx-facts-facts mx-fail (extract r 1) (cdr (cdr items)) (extract r 2) #f nil)))))))
;; Entries `ds` onto `out`, adds onto `adds`, the last first.
(define made-entries (->* (fails syns-a mx-facts-labels syns-a adds) (tupleof syns-a adds))
  (lambda (mx-fail ds ls out adds)
    (if (null? ds)
        (tuple (reverse out) (reverse adds))
        (let* ((d (car ds)) (two (proper-items d)) (pair (= (len two) 2))
               (every (and pair (mx-facts-sym-is? (nth two 1) "...")))
               (n (if pair (car two) d)) (a (syn-start d)) (b (syn-end d))
               (found (if (syn-symbol? n) (mx-facts-label-of (syn-symbol n) ls) nil)))
          (cond
            ((and every (not (syn-symbol? n)))
             (begin (fail-at mx-fail (string-append "every repetition's macro binder, `((label "
                                                 "(macro …)) ...)`: not yet")
                             d)
                    (made-entries mx-fail (cdr ds) ls out adds)))
            ;; `(label ...)`: every repetition's, as a fact names it.
            (every
             (let ((e (mx-facts-entry mx-fail d ls #f)))
               (made-entries mx-fail (cdr ds) ls (cons (extract e 1) out)
                             (append (reverse (extract e 2)) adds))))
            ((not (syn-symbol? n))
             (begin (fail-at mx-fail (string-append "what a use defines is a label, `(label ...)` "
                                                 "or `(label (macro …))`")
                             n)
                    (made-entries mx-fail (cdr ds) ls out adds)))
            ((null? found)
             (begin (fail-at mx-fail (string-append "`" (string-append (syn-name n) (string-append "` labels no part of this "
                                                 "signature's top")))
                             n)
                    (made-entries mx-fail (cdr ds) ls out adds)))
            ((= (extract (car found) 2) 1)
             (begin (fail-at mx-fail (string-append "`" (string-append (syn-name n) (string-append "` is a repeated part's: `("
                                                 (string-append (syn-name n) " ...)`, every repetition's"))))
                             n)
                    (made-entries mx-fail (cdr ds) ls out adds)))
            (pair
             ;; A macro's binder: its signature as written, read where it is.
             (made-entries mx-fail (cdr ds) ls
                           (cons (mk-list (list (path-syn (extract (car found) 3) a b) (nth two 1))
                                          a b)
                                 out)
                           adds))
            (else (made-entries mx-fail (cdr ds) ls
                                (cons (rib-entry mx-fail a b (extract (car found) 3)
                                                 (extract (car found) 5))
                                      out)
                                adds)))))))
;;; ------------------------------------------------------------ what is made

;; `MADE`: `(defn D …)` Herman's, each label its path, `(label ...)` every repetition's, the
;; list's rib; and what each repeated element must export for it, by the list's path.
(define mx-facts-made-of (->* (fails syn mx-facts-labels) (tupleof syn adds))
  (lambda (mx-fail m ls)
    (if (not (mx-facts-head-is? m "defn"))
        (tuple m (the adds nil))
        (let ((r (made-entries mx-fail (cdr (proper-items m)) ls nil nil)))
          (tuple (mk-list (cons (car (proper-items m)) (extract r 1)) (syn-start m) (syn-end m))
                 (extract r 2))))))
;; Signature `s`, in shapes and facts, as Herman's: `(macro/addressed σ made)`.
(define elaborated (->* (fails syn) syn)
  (lambda (mx-fail s)
    (let* ((items (proper-items s)) (a (syn-start s)) (b (syn-end s))
           (saved (get mx-facts-named))
           (top (begin (set mx-facts-named nil) (mx-facts-shape mx-fail (nth items 1))))
           (mx-facts-parts (list-parts top)))
      (if (or (null? (extract mx-facts-parts 1)) (not (mx-facts-sym-is? (car (extract mx-facts-parts 1)) "_")))
          (begin (fail-at mx-fail (string-append "a signature's shape is `(_ part …)`, as a "
                                              "`syntax-rules` pattern is")
                          top)
                 s)
          (let* ((operands (list-with (cdr (extract mx-facts-parts 1)) (extract mx-facts-parts 2) a b))
                 (r (tree mx-fail operands nil nil))
                 ;; What a use defines first: each element's export for a `(v ...)`
                 ;; there before the facts'.
                 (m (mx-facts-made-of mx-fail (nth items 2) (extract r 2)))
                 (sigma (mx-facts-facts mx-fail (extract r 1) (cdr (cdr (cdr items))) (extract r 2) #t
                               (extract m 2)))
                 (made (extract m 1))
                 (out (mk-list (list (mk-symbol "macro/addressed" a b) sigma made) a b)))
            (begin (set mx-facts-named saved) out))))))

;;; ============================================================ macros-files.fx (whole)

;; The name the importer has macro `n` by, of `m`'s, in a list of one.
(define internal-of
  (->* (symbol (listof (tupleof symbol symbol) acyclic)) (listof symbol acyclic))
  (lambda (n m)
    (cond ((null? m) nil)
          ((symbol=? n (extract (car m) 1)) (list (extract (car m) 2)))
          (else (internal-of n (cdr m))))))
(define proper (->* (syn) syns-a)
  (lambda (s) (tagcase s (mx-eager-reader-types-lst (items d a b x) items) (else y nil))))
(define mx-files-name-in? (->* (symbol mx-parser-types-names) bool)
  (lambda (n ns) (and (not (null? ns)) (or (symbol=? n (car ns)) (mx-files-name-in? n (cdr ns))))))
;; Whether `s` is a `define-syntax` or a `define-syntax-rec`.
(define defines-syntax? (->* (syn) bool)
  (lambda (s)
    (let ((items (proper s)))
      (and (not (null? items)) (syn-symbol? (car items))
           (or (string=? (syn-name (car items)) "define-syntax")
               (string=? (syn-name (car items)) "define-syntax-rec"))))))
;; Whether `s` is a `begin-for-syntax`, phase 1's forms (`macros-procedural.fx`).
(define for-syntax? (->* (syn) bool)
  (lambda (s)
    (let ((items (proper s)))
      (and (not (null? items)) (syn-symbol? (car items))
           (string=? (syn-name (car items)) "begin-for-syntax")))))
(define member-names (->* (syns-a) mx-parser-types-names)
  (lambda (ms)
    (cond ((null? ms) nil)
          ((and (not (null? (proper (car ms)))) (syn-symbol? (car (proper (car ms)))))
           (cons (syn-symbol (car (proper (car ms)))) (member-names (cdr ms))))
          (else (member-names (cdr ms))))))
;; The macros `s`, a `define-syntax` or a `define-syntax-rec`, defines.
(define syntax-names (->* (syn) mx-parser-types-names)
  (lambda (s)
    (let ((items (proper s)))
      (cond ((not (defines-syntax? s)) nil)
            ((string=? (syn-name (car items)) "define-syntax")
             (if (and (> (len items) 1) (syn-symbol? (nth items 1)))
                 (list (syn-symbol (nth items 1)))
                 nil))
            (else (member-names (cdr items)))))))
;; The name at the head of list `s`; "" if none.
(define mx-files-head-name (->* (syn) string)
  (lambda (s)
    (let ((items (proper s)))
      (if (and (not (null? items)) (syn-symbol? (car items))) (syn-name (car items)) ""))))
;; `(load-module "path")`'s path, in a list of one.
(define load-path (->* (syn) (listof string acyclic))
  (lambda (s)
    (let ((items (proper s)))
      (if (and (= (len items) 2) (syn-symbol? (car items))
               (string=? (syn-name (car items)) "load-module"))
          (tagcase (nth items 1) (atom (d a b x) (if (string? d) (list d) nil)) (else y nil))
          nil))))
;; An instance's, `((proj (load-module "path") arg …))`, in a list of one
;; (`binding-shapes.md`, step 4 (c); `Checker::load_def_of`).
(define instance-path (->* (syn) (listof string acyclic))
  (lambda (s)
    (let* ((items (proper s)) (applied (if (= (len items) 1) (proper (car items)) nil)))
      (if (and (>= (len applied) 2) (syn-symbol? (car applied))
               (string=? (syn-name (car applied)) "proj"))
          (load-path (nth applied 1))
          nil))))
;; A load's or an instance's path, in a list of one.
(define def-path (->* (syn) (listof string acyclic))
  (lambda (s) (let ((p (load-path s))) (if (null? p) (instance-path s) p))))
(define without-load (->* (symbol load-defs) load-defs)
  (lambda (n ls)
    (cond ((null? ls) nil)
          ((symbol=? n (extract (car ls) 1)) (without-load n (cdr ls)))
          (else (cons (car ls) (without-load n (cdr ls)))))))
(define mx-files-load-of (->* (symbol load-defs) (listof string acyclic))
  (lambda (n ls)
    (cond ((null? ls) nil)
          ((symbol=? n (extract (car ls) 1)) (list (extract (car ls) 2)))
          (else (mx-files-load-of n (cdr ls))))))
;; Whether `s` is an `import-with-syntax`.
(define imports-syntax? (->* (syn) bool)
  (lambda (s)
    (let ((items (proper s)))
      (and (not (null? items)) (syn-symbol? (car items))
           (string=? (syn-name (car items)) "import-with-syntax")))))
(define import-usage string
  (string-append "`(import-with-syntax name)`, the name a module loaded by `load-module`, or "
                 "`(import-with-syntax name (only name …))` or `(except name …)`"))
(define all-names (->* (fails syns-a) unit)
  (lambda (mx-fail xs)
    (cond ((null? xs) #u)
          ((syn-symbol? (car xs)) (all-names mx-fail (cdr xs)))
          (else (fail-at mx-fail import-usage (car xs))))))
(define name-filter (->* (fails syn) mx-files-filter)
  (lambda (mx-fail f)
    (let* ((items (proper f)) (head (if (and (not (null? items)) (syn-symbol? (car items)))
                                       (syn-name (car items))
                                       "")))
      (if (or (string=? head "only") (string=? head "except"))
          (begin (all-names mx-fail (cdr items)) (tuple (string=? head "only") (cdr items)))
          (begin (fail-at mx-fail import-usage f) (tuple #t (the syns-a nil)))))))
(define listed? (->* (symbol syns-a) bool)
  (lambda (n xs)
    (and (not (null? xs)) (or (symbol=? n (syn-symbol (car xs))) (listed? n (cdr xs))))))
(define not-macros (->* (syns-a mx-parser-types-names) syns-a)
  (lambda (xs ms)
    (cond ((null? xs) nil)
          ((mx-files-name-in? (syn-symbol (car xs)) ms) (not-macros (cdr xs) ms))
          (else (cons (car xs) (not-macros (cdr xs) ms))))))
;; The file read from `path`, in a list of one.
(define loaded-path (->* (string files) files)
  (lambda (path fs)
    (cond ((null? fs) nil)
          ((and (string=? (extract (car fs) 4) path) (> (extract (car fs) 2) 0))
           (the files (list (car fs))))
          (else (loaded-path path (cdr fs))))))
(define hidden-syntax (->* (syns-a) (listof file-macro acyclic))
  (lambda (xs)
    (cond ((null? xs) nil)
          ((defines-syntax? (car xs)) (cons (tuple (car xs) #t) (hidden-syntax (cdr xs))))
          (else (hidden-syntax (cdr xs))))))
(define hide-form? (->* (syn) bool)
  (lambda (s)
    (let ((items (proper s)))
      (string=? (mx-files-head-name s) "hide"))))
(define file-syntax (->* (syns-a) (listof file-macro acyclic))
  (lambda (forms)
    (cond ((null? forms) nil)
          ((or (defines-syntax? (car forms)) (for-syntax? (car forms)))
           (cons (tuple (car forms) #f) (file-syntax (cdr forms))))
          ((hide-form? (car forms))
           (append (hidden-syntax (cdr (proper (car forms)))) (file-syntax (cdr forms))))
          (else (file-syntax (cdr forms))))))
;; The file's forms its module shows: neither macros nor hidden.
(define file-items (->* (syns-a) syns-a)
  (lambda (forms)
    (cond ((null? forms) nil)
          ((or (defines-syntax? (car forms)) (for-syntax? (car forms)) (hide-form? (car forms)))
           (file-items (cdr forms)))
          ((datatype? (car forms))
           (append (expand-datatype (car forms)) (file-items (cdr forms))))
          (else (cons (car forms) (file-items (cdr forms)))))))
;; A definition's name: `name`, or `(name …)`'s.
(define defined-name (->* (syn) mx-parser-types-names)
  (lambda (s)
    (cond ((syn-symbol? s) (list (syn-symbol s)))
          ((and (not (null? (proper s))) (syn-symbol? (car (proper s))))
           (list (syn-symbol (car (proper s)))))
          (else nil))))
(define rec-names (->* (syns-a) mx-parser-types-names)
  (lambda (bs)
    (cond ((null? bs) nil)
          ((and (not (null? (proper (car bs)))) (syn-symbol? (car (proper (car bs)))))
           (cons (syn-symbol (car (proper (car bs)))) (rec-names (cdr bs))))
          (else (rec-names (cdr bs))))))
(define given-conversions (->* (syns-a) mx-parser-types-names)
  (lambda (rest)
    (if (or (null? rest) (null? (cdr rest)))
        nil
        (let ((more (given-conversions (cdr (cdr rest)))))
          (if (syn-symbol? (car (cdr rest))) (cons (syn-symbol (car (cdr rest))) more) more)))))
;; A generative type's conversions: the names `rest`, its form's parts
;; after its type, give (`#:seal mk #:unseal get`; the parser's
;; `generative-conversions` says what is wrong with them).
(define generative-values (->* (mx-parser-types-names syns-a) mx-parser-types-names)
  (lambda (ns rest) (if (null? ns) nil (given-conversions rest))))
(define mx-files-drop-n (->* (syns-a int) syns-a)
  (lambda (xs k) (if (or (= k 0) (null? xs)) xs (mx-files-drop-n (cdr xs) (- k 1)))))
;; The values and the types and effects `forms` define, onto `vs` and `ts`:
;; what the module shows. An `include`'s are known only once it is
;; checked: none (`Checker::names_shown`).
(define names-shown (->* (syns-a mx-parser-types-names mx-parser-types-names) (tupleof mx-parser-types-names mx-parser-types-names))
  (lambda (forms vs ts)
    (if (null? forms)
        (tuple vs ts)
        (let* ((items (proper (car forms)))
               (head (mx-files-head-name (car forms)))
               (named (if (< (len items) 2) nil (defined-name (nth items 1)))))
          (cond
            ((or (string=? head "define") (string=? head "define*"))
             (names-shown (cdr forms) (append named vs) ts))
            ((string=? head "define-rec")
             (names-shown (cdr forms) (append (rec-names (cdr items)) vs) ts))
            ((string=? head "define-generative")
             (names-shown (cdr forms) (append (generative-values named (mx-files-drop-n items 3)) vs)
                          (append named ts)))
            ((or (string=? head "define-type") (string=? head "define-effect"))
             (names-shown (cdr forms) vs (append named ts)))
            (else (names-shown (cdr forms) vs ts)))))))
(define macro-names (->* ((listof file-macro acyclic)) mx-parser-types-names)
  (lambda (ms)
    (if (null? ms)
        nil
        (append (syntax-names (extract (car ms) 1)) (macro-names (cdr ms))))))
;; Each macro's name the importer has it by, which no program writes.
(define internal-names (->* (string mx-parser-types-names) (listof (tupleof symbol symbol) acyclic))
  (lambda (key ns)
    (if (null? ns)
        nil
        (cons (tuple (car ns) (string->symbol (string-append "%syntax:" (string-append key (string-append ":"
                                                             (symbol->string (car ns)))))))
              (internal-names key (cdr ns))))))
(define taken? (->* (symbol (listof mx-files-filter acyclic)) bool)
  (lambda (n mx-files-filter)
    (or (null? mx-files-filter)
        (if (extract (car mx-files-filter) 1)
            (listed? n (extract (car mx-files-filter) 2))
            (not (listed? n (extract (car mx-files-filter) 2)))))))
(define shown-names (->* (mades) mx-parser-types-names)
  (lambda (made)
    (cond ((null? made) nil)
          ((extract (car made) 2) (shown-names (cdr made)))
          (else (cons (extract (car made) 1) (shown-names (cdr made)))))))
;; `ls` with what form `x` says of a load definition (`note-load-def!`).
(define load-noted (->* (syn load-defs) load-defs)
  (lambda (x ls)
    (let ((items (proper x)))
      (if (and (>= (len items) 3) (string=? (mx-files-head-name x) "define") (syn-symbol? (nth items 1)))
          (let* ((n (syn-symbol (nth items 1)))
                 (path (def-path (nth items (- (len items) 1))))
                 (kept (without-load n ls)))
            (if (and (not (null? path)) (<= (len items) 4)) (cons (tuple n (car path)) kept) kept))
          ls))))
(define mx-files-keyed (->* (string files) files)
  (lambda (key fs)
    (cond ((null? fs) nil)
          ((and (string=? (extract (car fs) 7) key) (> (extract (car fs) 2) 0))
           (the files (list (car fs))))
          (else (mx-files-keyed key (cdr fs))))))
(define read-in (->* (int files files) files)
  (lambda (at xs fs)
    (cond ((null? xs) nil)
          ((= (extract (car xs) 1) at)
           (if (> (extract (car xs) 2) 0)
               (the files (list (car xs)))
               (mx-files-keyed (extract (car xs) 7) fs)))
          (else (read-in at (cdr xs) fs)))))
;; The file the load at `at` read; one a form before made, which the
;; driver reads too at no form's place, by its key (`syn.rs`'s
;; `supply_loaded_in`).
(define mx-files-read-at (->* (int files) files)
  (lambda (at fs) (read-in at fs fs)))
;; The file the load named `lib` of `ls` read, in a list of one.
(define file-load-read (->* (symbol (listof (tupleof symbol int) acyclic) files) files)
  (lambda (lib ls fs)
    (cond ((null? ls) nil)
          ((symbol=? lib (extract (car ls) 1)) (mx-files-read-at (extract (car ls) 2) fs))
          (else (file-load-read lib (cdr ls) fs)))))
(define names-only (->* (syns-a) syns-a)
  (lambda (xs)
    (cond ((null? xs) nil)
          ((syn-symbol? (car xs)) (cons (car xs) (names-only (cdr xs))))
          (else (names-only (cdr xs))))))
;; `(only name …)` or `(except name …)`, its names those that are names.
(define lenient-filter (->* (syn) mx-files-filter)
  (lambda (f)
    (tuple (string=? (mx-files-head-name f) "only") (names-only (cdr (proper f))))))
;; Whether a file's forms, `xs`, as read (`fs` the files read, their forms
;; as read: `macros-imports.fx`'s `raw-loaded`), give macro `n` by its name:
;; a `define-syntax` of it, or an import that brings it, not hidden (`h`).
;; `ls`: the file's loads so far, each by where it starts. A macro its uses
;; make is not seen (`TODO.md` §287).
(define forms-give? (->* (symbol syns-a files (listof (tupleof symbol int) acyclic) bool) bool)
  (lambda (n xs fs ls h)
    (if (null? xs)
        #f
        (let* ((x (car xs))
               (items (proper x))
               (ls2 (if (and (>= (len items) 3) (string=? (mx-files-head-name x) "define")
                             (syn-symbol? (nth items 1))
                             (not (null? (load-path (nth items (- (len items) 1))))))
                        (cons (tuple (syn-symbol (nth items 1))
                                     (syn-start (nth items (- (len items) 1))))
                              ls)
                        ls)))
          (or (cond ((hide-form? x) (and (not h) (forms-give? n (cdr items) fs ls2 #t)))
                    (h #f)
                    ((defines-syntax? x) (mx-files-name-in? n (syntax-names x)))
                    ((imports-syntax? x)
                     (let ((f (if (and (>= (len items) 2) (syn-symbol? (nth items 1)))
                                  (file-load-read (syn-symbol (nth items 1)) ls2 fs)
                                  nil)))
                       (and (not (null? f))
                            (or (= (len items) 2) (taken? n (list (lenient-filter (nth items 2)))))
                            (forms-give? n (extract (car f) 5) fs nil #f))))
                    (else #f))
              (forms-give? n (cdr xs) fs ls2 h))))))
;; Whether `(import-with-syntax lib …)`, `x`, gives macro `n` its name.
(define brings? (->* (symbol syn files load-defs) bool)
  (lambda (n x fs ls)
    (let* ((items (proper x))
           (path (if (and (>= (len items) 2) (syn-symbol? (nth items 1)))
                     (mx-files-load-of (syn-symbol (nth items 1)) ls)
                     nil))
           (f (if (null? path) nil (loaded-path (car path) fs))))
      (and (not (null? f))
           (forms-give? n (extract (car f) 5) fs nil #f)
           (or (= (len items) 2) (taken? n (list (lenient-filter (nth items 2)))))))))
(define macro-sites (->* (symbol syns-a files load-defs mx-files-places) mx-files-places)
  (lambda (n xs fs ls found)
    (if (null? xs)
        found
        (let* ((x (car xs))
               (here (cond ((defines-syntax? x) (mx-files-name-in? n (syntax-names x)))
                           ((imports-syntax? x) (brings? n x fs ls))
                           (else #f)))
               (found2 (if here (the mx-files-places (list (tuple (syn-start x) (syn-end x)))) found)))
          (macro-sites n (cdr xs) fs (load-noted x ls) found2)))))
(define shown-syntax (->* ((listof file-macro acyclic)) (listof file-macro acyclic))
  (lambda (ms)
    (cond ((null? ms) nil)
          ((extract (car ms) 2) (shown-syntax (cdr ms)))
          (else (cons (car ms) (shown-syntax (cdr ms)))))))
;;; ------------------------------------------------------------ a macro written

;; A `syntax-rules`' trailmarks, `lits`, each a name the signature writes,
;; `(trailmark name)`, where the pattern has it; and no other: of the
;; pattern's nodes from `i`.
(define literals-said (->* (fails mx-parser-types-names int) unit)
  (lambda (mx-fail lits i)
    (if (= i (get nnodes))
        #u
        (let ((s (node-syn i)))
          (begin
            (if (syn-symbol? s)
                (let* ((k (mx-types-kind-of (node-core i))) (lit (or (= k 7) (= k 15)))
                       (n (syn-symbol s)))
                  (cond
                    ((and (= k 15) (not (mx-files-name-in? n lits)))
                     (fail-at mx-fail (string-append "`" (string-append (syn-name s) (string-append "` is the trailmark the "
                                                  (string-append "signature writes here, `(trailmark " (string-append (syn-name s)
                                                  (string-append ")`, and is not among the `syntax-rules`' "
                                                  "`#:trailmarks`"))))))
                              s))
                    ((and (not lit) (tagcase (node-kids i) (k-pvar (m) #t) (else y #f))
                          (mx-files-name-in? n lits))
                     (fail-at mx-fail (string-append "`" (string-append (syn-name s) (string-append "` is among the `syntax-rules`' "
                                                  (string-append "`#:trailmarks`, and the signature has a part "
                                                  (string-append "here, not `(trailmark " (string-append (syn-name s) ")`"))))))
                              s))
                    (else #u)))
                #u)
            (literals-said mx-fail lits (+ i 1)))))))
;; A macro's definition as written (`macros.rs`'s `written`).
(define define-usage string
  (string-append "`(define-syntax name [signature] (syntax-rules [#:trailmarks (name …)] "
                 "((_ . pattern) template) …))`"))
(define rec-usage string
  "`(define-syntax-rec (name [signature] (syntax-rules [#:trailmarks (name …)] clause …)) …)`")
(define rules-usage string
  "`(syntax-rules [#:trailmarks (name …)] ((_ . pattern) template) …)`")
(define written-in? (->* (symbol writtens) bool)
  (lambda (n ws)
   (and (not (null? ws)) (or (symbol=? n (extract (car ws) 1)) (written-in? n (cdr ws))))))
(define rules? (->* (syn) bool) (lambda (s) (string=? (mx-files-head-name s) "syntax-rules")))
(define names-only-unmarked (->* (mx-parser-types-names) mx-parser-types-names)
  (lambda (ns)
    (cond ((null? ns) nil)
          ((= (mark-now (car ns)) 0) (list (car ns)))
          (else (names-only-unmarked (cdr ns))))))
(define all-names? (->* (syns-a) bool)
  (lambda (xs) (or (null? xs) (and (syn-symbol? (car xs)) (all-names? (cdr xs))))))
(define names-of-syns (->* (syns-a) mx-parser-types-names)
  (lambda (xs) (if (null? xs) nil (cons (syn-symbol (car xs)) (names-of-syns (cdr xs))))))
;; A `syntax-rules`' parts after its head: the names its `#:trailmarks (name
;; …)` lists, if it has one, and its clauses; a `()` or a list of names
;; first, R7RS's literals, refused, saying how it is written now
;; (`macros.rs`'s `rules_parts`).
(define rules-parts (->* (fails syns-a syn) (tupleof mx-parser-types-names syns-a))
  (lambda (mx-fail parts rules)
    (let ((none (tuple (the mx-parser-types-names nil) (the syns-a nil))))
      (cond
        ((null? parts) (begin (fail-at mx-fail rules-usage rules) none))
        ((and (syn-keyword? (car parts)) (string=? (keyword-name (car parts)) "trailmarks")
              (not (null? (cdr parts))))
         (let ((ns (proper (nth parts 1))))
           (if (and (not (null? ns)) (all-names? ns))
               (tuple (names-of-syns ns) (cdr (cdr parts)))
               (begin (fail-at mx-fail (string-append "a `syntax-rules`' `#:trailmarks` are a list "
                                                   "of names, one or more")
                               (nth parts 1))
                      none))))
        ((syn-keyword? (car parts)) (begin (fail-at mx-fail rules-usage (car parts)) none))
        ((syn-nil? (car parts))
         (begin (fail-at mx-fail (string-append "a `syntax-rules` has no list of literals: drop the "
                                             (string-append "`()` (names its patterns match are written "
                                             "`#:trailmarks (name …)`)"))
                         (car parts))
                none))
        ((and (not (null? (proper (car parts)))) (all-names? (proper (car parts))))
         (begin (fail-at mx-fail (string-append "a `syntax-rules`' names its patterns match are its "
                                             (string-append "trailmarks: `#:trailmarks " (string-append (show-syn (car parts))
                                             "`, each defined `(define-syntax name (trailmark))`")))
                         (car parts))
                none))
        (else (tuple (the mx-parser-types-names nil) parts))))))
(define rules-written (->* (fails symbol syns-a syn syn) mx-files-written)
  (lambda (mx-fail n sig rules at)
    (let* ((rp (rules-parts mx-fail (cdr (proper rules)) rules)) (lits (extract rp 1))
           (unbound (names-only-unmarked lits)))
      (begin
        ;; Each of its `#:trailmarks` bound to one where it is defined.
        (if (null? unbound)
            #u
            (fail-at mx-fail (no-trailmark (symbol->string (car unbound))) at))
        (tuple n sig lits (extract rp 2) at)))))
;; A trailmark's definition, `(define-syntax name (trailmark ["where it
;; belongs"]))`: its transformer, in a list of one (`macros.rs`'s `Mark`).
(define trailmark? (->* (syn) bool)
  (lambda (t)
    (let ((items (proper t)))
      (and (not (null? items)) (syn-symbol? (car items))
           (string=? (syn-name (car items)) "trailmark")))))
(define written-one (->* (fails syn syns-a syn string) mx-files-written)
  (lambda (mx-fail name rest at usage)
    (let ((none (tuple (string->symbol "") (the syns-a nil) (the mx-parser-types-names nil) (the syns-a nil) at)))
      (cond
        ((or (not (syn-symbol? name)) (reserved-syn? name))
         (begin (fail-at mx-fail (string-append usage ": a macro's name, not a form's") name) none))
        ((and (= (len rest) 1) (trailmark? (car rest)))
         (begin (fail-at mx-fail "a trailmark is defined alone, by `define-syntax`" at) none))
        ((and (= (len rest) 1) (rules? (car rest)))
         (rules-written mx-fail (syn-symbol name) nil (car rest) at))
        ((and (= (len rest) 2) (rules? (nth rest 1)))
         (rules-written mx-fail (syn-symbol name) (list (car rest)) (nth rest 1) at))
        ((and (>= (len rest) 2) (= (len (proper (nth rest 1))) 2))
         (begin (fail-at mx-fail (string-append "a macro's clauses are a transformer's: "
                                             (string-append rules-usage ", after its signature if it is written"))
                         (nth rest 1))
                none))
        (else (begin (fail-at mx-fail usage at) none))))))
(define written-members (->* (fails syns-a writtens) writtens)
  (lambda (mx-fail ms out)
    (if (null? ms)
        (reverse out)
        (let ((parts (proper (car ms))))
          (if (null? parts)
              (begin (fail-at mx-fail rec-usage (car ms)) nil)
              (let ((w (written-one mx-fail (car parts) (cdr parts) (car ms) rec-usage)))
                (if (written-in? (extract w 1) out)
                    (begin (fail-at mx-fail (string-append "`" (string-append (symbol->string (extract w 1))
                                                        (string-append "` is defined twice in this "
                                                        "`define-syntax-rec`")))
                                    (car parts))
                           nil)
                    (written-members mx-fail (cdr ms) (cons w out)))))))))
(define mx-files-written-of (->* (fails syn) writtens)
  (lambda (mx-fail form)
    (let ((items (proper form)))
      (cond
        ((< (len items) 2)
         (begin (fail-at mx-fail (if (string=? (mx-files-head-name form) "define-syntax-rec") rec-usage
                                  define-usage)
                         form)
                nil))
        ((string=? (mx-files-head-name form) "define-syntax-rec") (written-members mx-fail (cdr items) nil))
        (else (list (written-one mx-fail (nth items 1) (cdr (cdr items)) form define-usage)))))))
(define trailmark-form (->* (syn) syns-a)
  (lambda (form)
    (let ((items (proper form)))
      (if (and (= (len items) 3) (string=? (mx-files-head-name form) "define-syntax")
               (trailmark? (nth items 2)))
          (list (nth items 2))
          nil))))
(define macros-but (->* (symbol macro-env) macro-env)
  (lambda (n ms)
    (cond ((null? ms) nil)
          ((symbol=? n (extract (car ms) 1)) (macros-but n (cdr ms)))
          (else (cons (car ms) (macros-but n (cdr ms)))))))
;; Trailmark `form` defined, apart from the macros, which no longer have its
;; name: its name.
(define define-trailmark (->* (fails syn) mx-parser-types-names)
  (lambda (mx-fail form)
    (let* ((name (nth (proper form) 1)) (t (nth (proper form) 2)) (said (cdr (proper t))))
      (cond
        ((or (not (syn-symbol? name)) (reserved-syn? name))
         (begin (fail-at mx-fail (string-append define-usage ": a macro's name, not a form's") name)
                nil))
        ((not (or (null? said) (and (null? (cdr said)) (syn-string? (car said)))))
         (begin (fail-at mx-fail "`(trailmark)`, or `(trailmark \"where it belongs\")`" t) nil))
        (else
         (begin
           (set defined (macros-but (syn-symbol name) (get defined)))
           (set mark-count (+ (get mark-count) 1))
           (set marks (cons (tuple (syn-symbol name) (get mark-count)) (get marks)))
           (list (syn-symbol name))))))))
;; Macros `ns` defined: a trailmark of a name of theirs no longer is one.
(define hide-marks! (->* (mx-parser-types-names) unit)
  (lambda (ns)
    (if (null? ns)
        #u
        (begin (set marks (cons (tuple (car ns) 0) (get marks))) (hide-marks! (cdr ns))))))
;; A signature's `(poly ((name kind) …) σ)`: its binders and `σ`; else none
;; and itself.
(define signature-poly (->* (fails syn) (tupleof syns-a syn))
  (lambda (mx-fail s)
    (let ((items (proper s)))
      (if (and (= (len items) 3) (syn-symbol? (car items)) (string=? (syn-name (car items)) "poly"))
          (if (or (syn-nil? (nth items 1)) (not (null? (proper (nth items 1)))))
              (tuple (proper (nth items 1)) (nth items 2))
              (begin (fail-at mx-fail "a signature's `(poly ((name kind) …) σ)`" (nth items 1))
                     (tuple (the syns-a nil) s)))
          (tuple (the syns-a nil) s)))))
(define letters-from? (->* (string int) bool)
  (lambda (n i)
    (or (= i (string-length n))
        (and (or (char=? (string-ref n i) #\A) (char=? (string-ref n i) #\D))
             (letters-from? n (+ i 1))))))
(define letters? (->* (syn) bool)
  (lambda (a) (and (syn-symbol? a) (letters-from? (syn-name a) 0))))
(define lettered-entries (->* (syns-a) syns-a)
  (lambda (es)
    (cond ((null? es) nil)
          ((and (pair? (proper-items (car es))) (letters? (car (proper-items (car es)))))
           (list (car (proper-items (car es)))))
          (else (lettered-entries (cdr es))))))
(define-rec
;; An address of signature `s` written as Herman's letters, `AAD`, in a list
;; of one: the kernel's own signatures keep them, a program's write `(path
;; …)` (`macros.rs`'s `lettered`).
(lettered-in (->* (syn) syns-a)
  (lambda (s)
    (let* ((items (proper-items s))
           (head (if (and (pair? items) (syn-symbol? (car items))) (syn-name (car items)) ""))
           (here (cond ((or (string=? head "rho") (string=? head "defn"))
                        (lettered-entries (cdr items)))
                       ((and (or (string=? head "at") (string=? head "shape")) (= (len items) 2)
                             (letters? (nth items 1)))
                        (list (nth items 1)))
                       (else nil))))
      (if (null? here) (lettered-each items) here))))
(lettered-each (->* (syns-a) syns-a)
  (lambda (xs)
    (if (null? xs)
        nil
        (let ((r (lettered-in (car xs)))) (if (null? r) (lettered-each (cdr xs)) r))))))
;; Operands `body` of a signature whose result is `made`, `(defn entry …)`:
;; `body`, defining `entries`, `(export body (rho entry …))`.
(define defined-in (->* (fails syn syn syns-a) syn)
  (lambda (mx-fail body made entries)
    (let ((bs (proper-items body)) (a (syn-start made)) (b (syn-end made)))
      (cond ((null? entries) body)
            ((and (pair? bs) (syn-symbol? (car bs)) (string=? (syn-name (car bs)) "export"))
             (begin (fail-at mx-fail (string-append "what a use defines is said once, in what it "
                                                 (string-append "makes, `(defn (ℓ δ) …)`, not by an `export` "
                                                 "around its parts too"))
                             body)
                    body))
            (else (let ((rho (mk-list (cons (mk-symbol "rho" a b) entries) a b)))
                    (mk-list (list (mk-symbol "export" a b) body rho) a b)))))))
;; A signature in Herman's form, `(macro/addressed σ made)`, as one in shapes
;; and facts is elaborated to: `σ`, and what it makes, 0 an expression,
;; `expr`, 1 definitions, `defn`, 2 a description, `desc`. `(macro/addressed
;; σ (defn (ℓ δ) …))`: definitions, `(export σ (rho (ℓ δ) …))`
;; (`macros.rs`'s `signature`).
(define signature-made (->* (fails syn) (tupleof syn int))
  (lambda (mx-fail s)
    (let ((items (proper-items s)))
      (if (and (= (len items) 3) (syn-symbol? (car items))
               (string=? (syn-name (car items)) "macro/addressed"))
          (let* ((m (nth items 2)) (ms (proper-items m))
                 (made (cond ((syn-symbol? m) (syn-name m))
                             ((and (pair? ms) (syn-symbol? (car ms))
                                   (string=? (syn-name (car ms)) "defn"))
                              "(defn")
                             (else ""))))
            (cond ((string=? made "defn") (tuple (nth items 1) 1))
                  ((string=? made "(defn") (tuple (defined-in mx-fail (nth items 1) m (cdr ms)) 1))
                  ((string=? made "desc") (tuple (nth items 1) 2))
                  ((string=? made "expr") (tuple (nth items 1) 0))
                  (else (begin (fail-at mx-fail (string-append "what a signature's uses make is "
                                                            (string-append "`expr`, `defn`, `(defn D …)` or "
                                                            "`desc`"))
                                        (nth items 2))
                               (tuple s 0)))))
          (let ((h (if (and (pair? items) (syn-symbol? (car items))) (syn-name (car items)) "")))
            (begin (fail-at mx-fail (cond ((string=? h "->")
                                        (string-append "a macro's signature is `(macro (_ part …) "
                                                       "made fact …)`: `->` is a procedure's type"))
                                       ((string=? h "macro")
                                        (string-append "a signature in shapes and facts is `(macro "
                                                       (string-append "(_ part …) made fact …)`; Herman's, "
                                                       "`(macro/addressed σ made)`")))
                                       (else
                                        (string-append "a signature is `(macro (_ part …) made "
                                                       (string-append "fact …)`, what its uses make written: "
                                                       "`expr`, `defn`, `(defn D …)` or `desc`"))))
                            s)
                   (tuple s 0)))))))
(define-rec
(signature-of (->* (fails syn) signed)
  (lambda (mx-fail s0)
    (let* ((poly0 (signature-poly mx-fail s0))
           ;; In shapes and facts: elaborated to Herman's form, which is read.
           (poly (if (contract? (extract poly0 2))
                     (tuple (extract poly0 1) (elaborated mx-fail (extract poly0 2)))
                     poly0))
           (sig (extract poly 2)) (sigs (proper-items sig))
           (lettered (if (and (pair? sigs) (syn-symbol? (car sigs))
                              (string=? (syn-name (car sigs)) "macro/addressed"))
                         (lettered-in sig)
                         nil))
           (said (if (null? lettered)
                     #u
                     (fail-at mx-fail (string-append "`" (string-append (show-syn (car lettered)) (string-append "` is an address "
                                                  (string-append "in Herman's letters: " path-usage))))
                              (car lettered))))
           (made (signature-made mx-fail (extract poly 2)))
           (t (read-sty mx-fail (extract made 1) nil nil)))
      (begin
        ;; Each macro it says a use defines, its signature read now.
        (tagcase t
          (s-export (b x) (tagcase x (bind-rib (es) (syntax-sigs-read! mx-fail es)) (else y #u)))
          (else y #u))
        (tuple (extract poly 1) t (extract made 2))))))
(syntax-sigs-read! (->* (fails entries) unit)
  (lambda (mx-fail es)
    (cond ((null? es) #u)
          ((and (>= (extract (car es) 2) 100) (null? (syntax-sig-read (extract (car es) 2))))
           (let ((r (signature-of mx-fail (syntax-sig-of (extract (car es) 2)))))
             (begin (if (null? (extract r 1))
                        (syntax-read! (extract (car es) 2) (extract r 2) (extract r 3))
                        (fail-at mx-fail "a `(macro σ made)` entry's signature has no `poly`, not yet"
                                 (syntax-sig-of (extract (car es) 2))))
                    (syntax-sigs-read! mx-fail (cdr es)))))
          (else (syntax-sigs-read! mx-fail (cdr es)))))))
(define written-names (->* (writtens) mx-parser-types-names)
  (lambda (ws) (if (null? ws) nil (cons (extract (car ws) 1) (written-names (cdr ws))))))
;; The names `poly`'s binders bind.
(define binder-names (->* (syns-a) mx-parser-types-names)
  (lambda (bs)
    (cond ((null? bs) nil)
          ((and (not (null? (proper (car bs)))) (syn-symbol? (car (proper (car bs)))))
           (cons (syn-symbol (car (proper (car bs)))) (binder-names (cdr bs))))
          (else (binder-names (cdr bs))))))
;; Whether syntax `s` names one of `ns`.
(define names-any? (->* (syn mx-parser-types-names) bool)
  (lambda (s ns)
    (cond ((syn-symbol? s) (mx-files-name-in? (syn-symbol s) ns))
          ((null? (split s)) #f)
          (else (or (names-any? (car (split s)) ns) (names-any? (nth (split s) 1) ns))))))
(define signed-each (->* (fails writtens) (listof signed-opt acyclic))
  (lambda (mx-fail ws)
    (if (null? ws)
        nil
        (let ((w (car ws)))
          (if (null? (extract w 2))
              (cons (the signed-opt nil) (signed-each mx-fail (cdr ws)))
              (let ((s (signature-of mx-fail (car (extract w 2)))))
                (begin
                  (set defined (cons (tuple (extract w 1) (tuple (extract s 2) (the clauses nil)
                                                                 (extract s 1) (extract s 3)))
                                     (get defined)))
                  (cons (the signed-opt (list s)) (signed-each mx-fail (cdr ws))))))))))
;; How macro `n`'s refusal is said, its signature inferred, `shown`, not
;; fitting its clauses: refused, said so on a line of its own, then what was
;; inferred and how one is written (`macros.rs`'s `refused_inferred`).
(define said-fail (->* (fails string string) fails)
  (lambda (mx-fail n shown)
    (lambda ((m string) (a int) (b int))
      (mx-fail (string-append "`" (string-append n (string-append "` is refused: no signature is written, and the one inferred "
                           (string-append "from its clauses does not fit them: " (string-append m (string-append "\n  inferred: " (string-append shown
                           (string-append "\n  write one: (define-syntax " (string-append n " SIGNATURE (syntax-rules …))")))))))))
            a b))))

;;; ============================================================ macros-infer.fx (whole)

(define mx-infer-parts (ref (arrayof mx-infer-part mxp) mxp) (new (make-array 64 (tuple (sh-leaf) (the addr nil) 0))))
(define nparts (ref int mxp) (new 0))
(define found-vars (ref vars mxp) (new nil))
;; Each region's root (a list's element, a `union`'s member), and the part
;; whose region it is; the clauses each member is of; each list by
;; ellipsis that took in patterns written one by one, and the place of the
;; first.
(define boundaries (ref (listof (tupleof int int) acyclic) mxp) (new nil))
(define mx-infer-members (ref (listof (tupleof int mx-infer-ints) acyclic) mxp) (new nil))
(define mx-infer-absorbed (ref (listof (tupleof int int int) acyclic) mxp) (new nil))
(define mx-infer-roots (ref mx-infer-ints mxp) (new nil))
(define parts-reset! (->* () unit)
  (lambda ()
    (begin (set nparts 0) (set found-vars nil) (set boundaries nil) (set mx-infer-members nil)
           (set mx-infer-absorbed nil) (set mx-infer-roots nil))))
(define mx-infer-part-at (->* (int) mx-infer-part) (lambda (i) (array-ref (get mx-infer-parts) i)))
(define mx-infer-shape-of (->* (int) mx-infer-shape) (lambda (i) (extract (mx-infer-part-at i) 1)))
(define path-of (->* (int) addr) (lambda (i) (extract (mx-infer-part-at i) 2)))
(define mx-infer-region (->* (int) int) (lambda (i) (extract (mx-infer-part-at i) 3)))
(define shape-set! (->* (int mx-infer-shape) unit)
  (lambda (i sh)
    (array-set! (get mx-infer-parts) i (tuple sh (path-of i) (mx-infer-region i)))))
(define copy-parts (->* ((arrayof mx-infer-part mxp) (arrayof mx-infer-part mxp) int) unit)
  (lambda (from to i)
    (if (= i (array-length from))
        #u
        (begin (array-set! to i (array-ref from i)) (copy-parts from to (+ i 1))))))
;; A new part at `path` from `root` (-1: a region of its own): its number.
(define part-new (->* (addr int) int)
  (lambda (path root)
    (let ((i (get nparts)))
      (begin
        (if (= i (array-length (get mx-infer-parts)))
            (let ((bigger (the (arrayof mx-infer-part mxp) (make-array (* 2 i) (mx-infer-part-at 0)))))
              (begin (copy-parts (get mx-infer-parts) bigger 0) (set mx-infer-parts bigger)))
            #u)
        (set nparts (+ i 1))
        (array-set! (get mx-infer-parts) i (tuple (sh-leaf) (if (< root 0) (the addr nil) path)
                                         (if (< root 0) i root)))
        i))))
(define leaf? (->* (int) bool) (lambda (p) (tagcase (mx-infer-shape-of p) (sh-leaf () #t) (else y #f))))
(define mx-infer-union? (->* (int) bool)
  (lambda (p) (tagcase (mx-infer-shape-of p) (sh-union (ms) #t) (else y #f))))
(define int-assoc (->* (int (listof (tupleof int int) acyclic)) mx-infer-ints)
  (lambda (k xs)
    (cond ((null? xs) nil)
          ((= k (extract (car xs) 1)) (list (extract (car xs) 2)))
          (else (int-assoc k (cdr xs))))))
(define mx-infer-int-in? (->* (int mx-infer-ints) bool)
  (lambda (i is) (and (not (null? is)) (or (= i (car is)) (mx-infer-int-in? i (cdr is))))))
;; The part whose region `r` is, if any.
(define boundary-of (->* (int) mx-infer-ints) (lambda (r) (int-assoc r (get boundaries))))
(define var-in (->* (int symbol vars) (listof (tupleof int int) acyclic))
  (lambda (cid n vs)
    (cond ((null? vs) nil)
          ((and (= cid (extract (car vs) 1)) (symbol=? n (extract (car vs) 2)))
           (list (tuple (extract (car vs) 3) (extract (car vs) 4))))
          (else (var-in cid n (cdr vs))))))
;; Pattern variable `n` of clause `cid`'s part, and its depth, in a list of
;; one.
(define var-of (->* (int symbol) (listof (tupleof int int) acyclic))
  (lambda (cid n) (var-in cid n (get found-vars))))
(define mx-infer-var-part (->* (int symbol) mx-infer-ints)
  (lambda (cid n) (let ((v (var-of cid n))) (if (null? v) nil (list (extract (car v) 1))))))
(define var-replaced (->* (int symbol int int vars) vars)
  (lambda (cid n p depth vs)
    (cond ((null? vs) nil)
          ((and (= cid (extract (car vs) 1)) (symbol=? n (extract (car vs) 2)))
           (cons (tuple cid n p depth) (cdr vs)))
          (else (cons (car vs) (var-replaced cid n p depth (cdr vs)))))))
(define var! (->* (int symbol int int) unit)
  (lambda (cid n p depth)
    (set found-vars (if (null? (var-of cid n))
                        (append (get found-vars) (the vars (list (tuple cid n p depth))))
                        (var-replaced cid n p depth (get found-vars))))))
(define regions-from (->* (int) mx-infer-ints)
  (lambda (r)
    (let ((b (boundary-of r))) (if (null? b) (list r) (cons r (regions-from (mx-infer-region (car b))))))))
;; The regions part `p` is in, innermost first.
(define mx-infer-regions (->* (int) mx-infer-ints)
  (lambda (p) (regions-from (mx-infer-region p))))
(define via-from (->* (int int) mx-infer-ints)
  (lambda (q r)
    (let ((b (boundary-of q)))
      (cond ((null? b) nil)
            ((= (mx-infer-region (car b)) r) (list (car b)))
            (else (via-from (mx-infer-region (car b)) r))))))
;; How part `x` is reached from region `r`: directly (-1), or through the
;; part in `r` whose region it is in; in a list of one, none if `r` is not
;; around it.
(define via (->* (int int) mx-infer-ints)
  (lambda (x r) (if (= (mx-infer-region x) r) (list -1) (via-from (mx-infer-region x) r))))
(define mx-infer-members-of (->* (int (listof (tupleof int mx-infer-ints) acyclic)) mx-infer-ints)
  (lambda (m xs)
    (cond ((null? xs) nil)
          ((= m (extract (car xs) 1)) (extract (car xs) 2))
          (else (mx-infer-members-of m (cdr xs))))))
(define mx-infer-member-of (->* (mx-infer-ints int) mx-infer-ints)
  (lambda (ms cid)
    (cond ((null? ms) nil)
          ((let ((cs (mx-infer-members-of (car ms) (get mx-infer-members)))) (mx-infer-int-in? cid cs)) (list (car ms)))
          (else (mx-infer-member-of (cdr ms) cid)))))
;; The region inside part `b` clause `cid` is in, in a list of one.
(define inner (->* (int int) mx-infer-ints)
  (lambda (b cid)
    (tagcase (mx-infer-shape-of b)
      (sh-seq (e) (list e))
      (sh-union (ms) (mx-infer-member-of ms cid))
      (else y nil))))
(define nth-names (->* (lits-of int) mx-parser-types-names)
  (lambda (xs i) (if (= i 0) (car xs) (nth-names (cdr xs) (- i 1)))))
(define class-of (->* (fails mx-infer-item lits-of) class)
  (lambda (mx-fail it lits)
    (let* ((s (extract it 2)) (x (ellipsis-of s)))
      (cond
        ((not (null? x)) (cl-seq (car x)))
        ((syn-symbol? s)
         (let ((name (syn-name s)))
           (cond ((string=? name "_") (cl-wild))
                 ((string=? name "...")
                  (begin (fail-at mx-fail "`...` follows the part it repeats, last in a list" s)
                         (cl-wild)))
                 ((or (mx-syntax-name-in? (syn-symbol s) (nth-names lits (extract it 1))) (reserved? name))
                  (cl-lit (syn-symbol s)))
                 (else (cl-var (syn-symbol s))))))
        ((syn-nil? s) (cl-nil))
        ((null? (split s)) (cl-wild))
        ;; A datum matches itself: its part takes anything.
        (else (tagcase s
                (vec (i d a b y) (cl-wild))
                (else z (cl-pair (car (split s)) (nth (split s) 1)))))))))
(define classes-of (->* (fails mx-infer-items lits-of) classes)
  (lambda (mx-fail its lits)
    (if (null? its) nil (cons (class-of mx-fail (car its) lits) (classes-of mx-fail (cdr its) lits)))))
(define leafy? (->* (class) bool)
  (lambda (c) (tagcase c (cl-var (n) #t) (cl-wild () #t) (else y #f))))
(define all-leafy? (->* (classes) bool)
  (lambda (cs) (or (null? cs) (and (leafy? (car cs)) (all-leafy? (cdr cs))))))
(define vars-here! (->* (mx-infer-items classes int) unit)
  (lambda (its cs id)
    (if (null? its)
        #u
        (begin (tagcase (car cs)
                 (cl-var (n) (var! (extract (car its) 1) n id (extract (car its) 3)))
                 (else y #u))
               (vars-here! (cdr its) (cdr cs) id)))))
(define seq-vars! (->* (vars int) unit)
  (lambda (vs id)
    (if (null? vs)
        #u
        (begin (var! (extract (car vs) 1) (extract (car vs) 2) id (extract (car vs) 4))
               (seq-vars! (cdr vs) id)))))
(define same-group? (->* (class class) bool)
  (lambda (a b)
    (tagcase a
      (cl-nil () (tagcase b (cl-nil () #t) (else y #f)))
      (cl-pair (x y) (tagcase b (cl-pair (u v) #t) (else z #f)))
      (cl-seq (x) (tagcase b (cl-seq (u) #t) (else z #f)))
      (cl-lit (n) (tagcase b (cl-lit (m) (symbol=? n m)) (else z #f)))
      (else y (leafy? b)))))
(define into-group (->* (class int mx-infer-groups) mx-infer-groups)
  (lambda (c k gs)
    (cond ((null? gs) (list (tuple c (the mx-infer-ints (list k)))))
          ((same-group? (extract (car gs) 1) c)
           (cons (tuple (extract (car gs) 1) (append (extract (car gs) 2) (the mx-infer-ints (list k))))
                 (cdr gs)))
          (else (cons (car gs) (into-group c k (cdr gs)))))))
(define mx-infer-grouped (->* (classes int mx-infer-groups) mx-infer-groups)
  (lambda (cs k out)
    (if (null? cs)
        out
        (mx-infer-grouped (cdr cs) (+ k 1) (into-group (car cs) k out)))))
(define nth-item (->* (mx-infer-items int) mx-infer-item)
  (lambda (xs i) (if (= i 0) (car xs) (nth-item (cdr xs) (- i 1)))))
(define items-at (->* (mx-infer-items mx-infer-ints) mx-infer-items)
  (lambda (its ks) (if (null? ks) nil (cons (nth-item its (car ks)) (items-at its (cdr ks))))))
(define dedup-cids (->* (mx-infer-items) mx-infer-ints)
  (lambda (its)
    (cond ((null? its) nil)
          ((and (not (null? (cdr its))) (= (extract (car its) 1) (extract (car (cdr its)) 1)))
           (dedup-cids (cdr its)))
          (else (cons (extract (car its) 1) (dedup-cids (cdr its)))))))
(define mx-infer-halves (->* (mx-infer-items classes bool) mx-infer-items)
  (lambda (its cs car?)
    (if (null? its)
        nil
        (tagcase (car cs)
          (cl-pair (a d)
            (cons (tuple (extract (car its) 1) (if car? a d) (extract (car its) 3))
                  (mx-infer-halves (cdr its) (cdr cs) car?)))
          (else y (mx-infer-halves (cdr its) (cdr cs) car?))))))
(define any-list-end? (->* (classes) bool)
  (lambda (cs)
    (and (not (null? cs))
         (or (tagcase (car cs) (cl-nil () #t) (cl-seq (x) #t) (else y #f))
             (any-list-end? (cdr cs))))))
(define all-nil? (->* (classes) bool)
  (lambda (cs)
    (or (null? cs) (and (tagcase (car cs) (cl-nil () #t) (else y #f)) (all-nil? (cdr cs))))))
(define span-in? (->* (syn spans) bool)
  (lambda (s xs)
    (and (not (null? xs))
         (or (and (= (syn-start s) (extract (car xs) 1)) (= (syn-end s) (extract (car xs) 2)))
             (span-in? s (cdr xs))))))
(define take (->* (fails mx-infer-item mx-infer-cx took) (listof took acyclic))
  (lambda (mx-fail it c acc)
    (let ((cid (extract it 1)) (depth (extract it 3))
          (es (extract acc 1)) (vs (extract acc 2)) (tk (extract acc 3)) (sq (extract acc 4)))
      (tagcase (class-of mx-fail it (extract c 1))
        (cl-nil () (list acc))
        (cl-wild () (list acc))
        (cl-seq (x)
         (list (tuple (append es (the mx-infer-items (list (tuple cid x (+ depth 1))))) vs tk #t)))
        (cl-var (n) (list (tuple es (append vs (the vars (list (tuple cid n 0 depth)))) tk sq)))
        (cl-pair (a d)
          (take mx-fail (tuple cid d depth) c
                (tuple (append es (the mx-infer-items (list (tuple cid a depth)))) vs #t sq)))
        (cl-lit (n) nil)))))
(define take-each (->* (fails mx-infer-items mx-infer-cx took) (listof took acyclic))
  (lambda (mx-fail its c acc)
    (if (null? its)
        (list acc)
        (let ((t (take mx-fail (car its) c acc)))
          (if (null? t) nil (take-each mx-fail (cdr its) c (car t)))))))
(define pkind-of (->* (int symbol pkinds) kds)
  (lambda (cid n pk)
    (cond ((null? pk) nil)
          ((and (= cid (extract (car pk) 1)) (symbol=? n (extract (car pk) 2)))
           (list (extract (car pk) 3)))
          (else (pkind-of cid n (cdr pk))))))
(define known-kinds (->* (mx-infer-items pkinds) kds)
  (lambda (es pk)
    (if (null? es)
        nil
        (let* ((s (extract (car es) 2))
               (k (if (syn-symbol? s) (pkind-of (extract (car es) 1) (syn-symbol s) pk) nil))
               (rest (known-kinds (cdr es) pk)))
          (if (and (not (null? k)) (tagcase (car k) (kd-ty (t) #t) (else y #f)))
              (cons (car k) rest)
              rest)))))
(define kd=? (->* (kd kd) bool)
  (lambda (a b)
    (tagcase a
      (kd-ref (p) (tagcase b (kd-ref (q) (= p q)) (else y #f)))
      (kd-ty (t) (tagcase b (kd-ty (u) (sty=? t u)) (else y #f))))))
;; Whether a part of syntax type `a` may be placed where `b` is too.
(define reconcile? (->* (kd kd) bool)
  (lambda (a b)
    (or (kd=? a b)
        (tagcase a
          (kd-ty (t)
            (tagcase b
              (kd-ty (u)
                (let ((x (mx-types-kind-of t)) (y (mx-types-kind-of u)))
                  (or (and (= x 1) (or (= y 0) (= y 2) (= y 10)))
                      (and (= x 12) (or (= y 4) (= y 0) (= y 2)))
                      (and (= x 5) (= y 0)) (and (= x 2) (= y 0)) (and (= x 0) (= y 10)))))
              (else z #f)))
          (else y #f)))))
(define disagrees-any? (->* (kd kds) bool)
  (lambda (a bs)
    (and (not (null? bs))
         (or (and (not (reconcile? a (car bs))) (not (reconcile? (car bs) a)))
             (disagrees-any? a (cdr bs))))))
(define any-disagree? (->* (kds kds) bool)
  (lambda (as all)
    (and (not (null? as))
         (or (disagrees-any? (car as) all) (any-disagree? (cdr as) all)))))
;; Whether elements `es`' pattern variables are of syntax types known
;; that do not agree.
(define unknit? (->* (mx-infer-items pkinds) bool)
  (lambda (es pk)
    (let ((known (known-kinds es pk)))
      (any-disagree? known known))))
(define absorbable (->* (fails mx-infer-items classes mx-infer-cx) (listof took acyclic))
  (lambda (mx-fail its cs c)
    (if (or (not (any-list-end? cs)) (all-nil? cs))
        nil
        (let ((t (take-each mx-fail its c (tuple (the mx-infer-items nil) (the vars nil) #f #f))))
          (cond
            ((null? t) nil)
            ((not (extract (car t) 4)) nil)
            ((and (extract (car t) 3)
                  (or (span-in? (extract (car its) 2) (extract c 3))
                      (unknit? (extract (car t) 1) (extract c 2))))
             nil)
            (else t))))))
(define-rec
;; The part for `its`, at `path` from region `root` (-1: its own).
(merge (->* (fails mx-infer-items addr int mx-infer-cx) int)
  (lambda (mx-fail its path root c)
    (let* ((id (part-new path root)) (cs (classes-of mx-fail its (extract c 1))))
      (if (all-leafy? cs)
          (begin (vars-here! its cs id) id)
          (let ((ab (absorbable mx-fail its cs c)))
            (if (not (null? ab))
                (let ((a (car ab)))
                  (begin
                    (seq-vars! (extract a 2) id)
                    (let ((e (merge mx-fail (extract a 1) nil -1 c)))
                      (begin
                        (set boundaries (cons (tuple e id) (get boundaries)))
                        (shape-set! id (sh-seq e))
                        (if (extract a 3)
                            (let ((s (extract (car its) 2)))
                              (set mx-infer-absorbed (cons (tuple id (syn-start s) (syn-end s))
                                                  (get mx-infer-absorbed))))
                            #u)
                        id))))
                (let ((mx-infer-groups (mx-infer-grouped cs 0 nil)))
                  (begin
                    (if (null? (cdr mx-infer-groups))
                        (one! mx-fail id its cs c)
                        (shape-set! id (sh-union (members! mx-fail its mx-infer-groups id c))))
                    id))))))))
(members! (->* (fails mx-infer-items mx-infer-groups int mx-infer-cx) mx-infer-ints)
  (lambda (mx-fail its gs id c)
    (if (null? gs)
        nil
        (let* ((sub (items-at its (extract (car gs) 2)))
               (m (merge mx-fail sub nil -1 c)))
          (begin
            (set boundaries (cons (tuple m id) (get boundaries)))
            (set mx-infer-members (cons (tuple m (dedup-cids sub)) (get mx-infer-members)))
            (cons m (members! mx-fail its (cdr gs) id c)))))))
;; Part `id`, of `its` all of one shape.
(one! (->* (fails int mx-infer-items classes mx-infer-cx) unit)
  (lambda (mx-fail id its cs c)
    (tagcase (car cs)
      (cl-nil () (shape-set! id (sh-nil)))
      (cl-lit (n) (shape-set! id (sh-lit n)))
      (cl-pair (x y)
        (let* ((root (mx-infer-region id)) (base (path-of id))
               (a (merge mx-fail (mx-infer-halves its cs #t) (append base (the addr (list #t))) root c))
               (d (merge mx-fail (mx-infer-halves its cs #f) (append base (the addr (list #f))) root c)))
          (shape-set! id (sh-pair a d))))
      (else y #u)))))
(define ints-text (->* (mx-infer-ints) string)
  (lambda (is) (if (null? is) "." (string-append (int->string (car is)) (string-append " " (ints-text (cdr is)))))))
(define shape-text (->* (mx-infer-shape) string)
  (lambda (sh)
    (tagcase sh
      (sh-nil () "n") (sh-pair (a d) (string-append "p" (string-append (int->string a) (string-append "," (int->string d)))))
      (sh-seq (e) (string-append "s" (int->string e))) (sh-lit (n) (symbol->string n))
      (sh-leaf () "l") (sh-union (ms) (string-append "u" (ints-text ms))))))
(define addr-text (->* (addr) string)
  (lambda (a) (if (null? a) "/" (string-append (if (car a) "A" "D") (addr-text (cdr a))))))
;; Parts `i` and those before it, onto `out`.
(define parts-text (->* (int string) string)
  (lambda (i out)
    (if (< i 0)
        out
        (parts-text (- i 1)
                    (string-append (shape-text (mx-infer-shape-of i)) (string-append (addr-text (path-of i))
                                   (string-append (int->string (mx-infer-region i)) (string-append ";" out))))))))
;; The shape, as compared from one iteration to the next.
(define fingerprint (->* () string)
  (lambda () (string-append (ints-text (get mx-infer-roots)) (parts-text (- (get nparts) 1) ""))))
(define kind-at (->* (int kinds) kds)
  (lambda (p ks)
    (cond ((null? ks) nil)
          ((= p (extract (car ks) 1)) (list (extract (car ks) 2)))
          (else (kind-at p (cdr ks))))))
;; How specific a syntax type is, where a part is placed as more than one
;; (`infer.rs`'s `rank`).
(define rank (->* (kd) int)
  (lambda (k)
    (tagcase k
      (kd-ref (p) 4)
      (kd-ty (t)
        (let ((n (mx-types-kind-of t)))
          (cond ((= n 1) 0) ((= n 12) 1) ((= n 5) 2) ((= n 2) 3) ((= n 0) 6) ((= n 10) 7)
                ((= n 3) 8) (else 4)))))))
(define least-ranked (->* (kds kd) kd)
  (lambda (ks best)
    (cond ((null? ks) best)
          ((< (rank (car ks)) (rank best)) (least-ranked (cdr ks) (car ks)))
          (else (least-ranked (cdr ks) best)))))
(define all-reconciled? (->* (kd kds) bool)
  (lambda (c ks) (or (null? ks) (and (reconcile? c (car ks)) (all-reconciled? c (cdr ks))))))
;; One syntax type for a part placed as each of `ks` (none if none), and
;; whether some of them do not agree with it.
(define combine (->* (kds) (tupleof kds bool))
  (lambda (ks)
    (if (null? ks)
        (tuple (the kds nil) #f)
        (let ((chosen (least-ranked (cdr ks) (car ks))))
          (tuple (the kds (list chosen)) (not (all-reconciled? chosen ks)))))))
;; Whether a part of syntax type `k` is placed in a scope its signature
;; says; whether one with an expression in it runs each time it is placed.
(define scoped? (->* (kd) bool)
  (lambda (k)
    (tagcase k
      (kd-ref (p) #t)
      (kd-ty (t)
        (let ((n (mx-types-kind-of t)))
          (not (or (= n 1) (= n 12) (= n 3) (= n 4) (= n 5) (= n 13) (= n 11))))))))
(define affine? (->* (kd) bool)
  (lambda (k)
    (tagcase k
      (kd-ref (p) #t)
      (kd-ty (t)
        (let ((n (mx-types-kind-of t)))
          (not (or (= n 1) (= n 12) (= n 2) (= n 5) (= n 3) (= n 4) (= n 13))))))))
;; Whether `t` has an annotation whose addresses are from outside it.
(define open? (->* (sty) bool)
  (lambda (t)
    (tagcase t
      (s-import (b x) #t) (s-export (b x) #t)
      (s-pair (a d) (or (open? a) (open? d)))
      (s-many (b) (open? b)) (s-typed (b h) (open? b))
      (else y #f))))
(define compound? (->* (sty) bool)
  (lambda (t)
    (tagcase t
      (s-pair (a d) #t) (s-mu (x r p) #t) (s-union (ms) #t) (s-import (b x) #t) (s-export (b x) #t)
      (s-many (b) #t) (s-typed (b h) #t) (else y #f))))
(define len-us (->* (unsigneds) int)
  (lambda (us) (if (null? us) 0 (+ 1 (len-us (cdr us))))))
(define nones-onto (->* (int makes-known) makes-known)
  (lambda (n out) (if (= n 0) out (nones-onto (- n 1) (cons (the mx-infer-ints nil) out)))))
(define nones (->* (int) makes-known) (lambda (n) (nones-onto n nil)))
(define made-or-expr (->* (makes-known) mx-infer-ints)
  (lambda (ks)
   (if (null? ks) nil (cons (if (null? (car ks)) 0 (car (car ks))) (made-or-expr (cdr ks))))))
(define known=? (->* (makes-known makes-known) bool)
  (lambda (a b)
    (or (and (null? a) (null? b))
        (and (not (null? a)) (not (null? b))
             (if (null? (car a))
                 (null? (car b))
                 (and (not (null? (car b))) (= (car (car a)) (car (car b)))))
             (known=? (cdr a) (cdr b))))))
(define made-said (->* (int) string)
  (lambda (k) (cond ((= k 1) "definitions") ((= k 2) "a description") (else "an expression"))))
(define nth-unsigned (->* (unsigneds int) unsigned)
  (lambda (us i) (if (= i 0) (car us) (nth-unsigned (cdr us) (- i 1)))))
(define nth-known (->* (makes-known int) mx-infer-ints)
  (lambda (ks i) (if (= i 0) (car ks) (nth-known (cdr ks) (- i 1)))))
(define set-known (->* (makes-known int mx-infer-ints) makes-known)
  (lambda (ks i k) (if (= i 0) (cons k (cdr ks)) (cons (car ks) (set-known (cdr ks) (- i 1) k)))))
;; A pattern's names but `_`, `...`, the forms' and the literals.
(define pattern-vars-of (->* (syn mx-parser-types-names) mx-parser-types-names)
  (lambda (p lits)
    (cond ((syn-symbol? p)
           (let ((n (syn-name p)))
             (if (or (string=? n "_") (string=? n "...") (reserved? n)
                     (mx-syntax-name-in? (syn-symbol p) lits))
                 nil
                 (list (syn-symbol p)))))
          ((null? (split p)) nil)
          (else (append (pattern-vars-of (car (split p)) lits)
                        (pattern-vars-of (nth (split p) 1) lits))))))
(define unsigned-index (->* (symbol unsigneds int) int)
  (lambda (n us i)
    (cond ((null? us) -1)
          ((symbol=? n (extract (car us) 1)) i)
          (else (unsigned-index n (cdr us) (+ i 1))))))
(define proper-or-dotted (->* (syn) syns-a)
  (lambda (s)
   (tagcase s (mx-eager-reader-types-lst (mx-infer-items d a b x) mx-infer-items) (dotted (mx-infer-items t d a b x) mx-infer-items) (else y nil))))
(define mx-infer-named-in? (->* (symbol (listof (tupleof symbol sty) acyclic)) bool)
  (lambda (n xs)
   (and (not (null? xs)) (or (symbol=? n (extract (car xs) 1)) (mx-infer-named-in? n (cdr xs))))))
(define kernel-form? (->* (string) bool)
  (lambda (n) (mx-infer-named-in? (string->symbol n) (kernel-sigs pfail-at))))
(define-rec
(makes-of (->* (syn mx-parser-types-names unsigneds makes-known) mx-infer-ints)
  (lambda (t pv us known)
    (cond
      ((syn-symbol? t)
       (if (or (string=? (syn-name t) "#t") (string=? (syn-name t) "#f")) (list 0) nil))
      ((syn-nil? t) nil)
      ((null? (split t)) (list 0))
      (else
       (let ((h (car (split t))))
         (cond
           ((syn-keyword? h) (list 0))
           ((not (syn-symbol? h)) nil)
           ((mx-syntax-name-in? (syn-symbol h) pv) nil)
           (else
            (let ((i (unsigned-index (syn-symbol h) us 0)))
              (if (>= i 0)
                  (nth-known known i)
                  (let ((m (macro-of (syn-symbol h) (get defined))))
                    (if (not (null? m))
                        ;; A procedural macro (`3`) is read as an expression's.
                        (list (if (= (extract (car m) 4) 3) 0 (extract (car m) 4)))
                        (head-makes (syn-name h) (proper-or-dotted t) pv us known))))))))))))
(head-makes (->* (string syns-a mx-parser-types-names unsigneds makes-known) mx-infer-ints)
  (lambda (n mx-infer-items pv us known)
    (cond
      ((or (string=? n "define") (string=? n "define*") (string=? n "define-rec")
           (string=? n "define-type") (string=? n "define-effect"))
       (list 1))
      ((string=? n "begin")
       (let ((each (each-makes (cdr mx-infer-items) pv us known)))
         (cond ((mx-infer-int-in? 1 each) (list 1)) ((null? each) nil) (else (list (car each))))))
      ((kernel-form? n) (list 0))
      ((type-form? n) (list 2))
      (else nil))))
(each-makes (->* (syns-a mx-parser-types-names unsigneds makes-known) mx-infer-ints)
  (lambda (xs pv us known)
    (if (null? xs)
        nil
        (append (makes-of (car xs) pv us known) (each-makes (cdr xs) pv us known))))))
(define makes-round (->* (fails unsigneds rules makes-known makes-known) makes-known)
  (lambda (mx-fail us rs known next)
    (if (null? rs)
        next
        (let* ((mi (extract (car rs) 1)) (t (extract (car rs) 3))
               (u (nth-unsigned us mi))
               (pv (pattern-vars-of (extract (car rs) 2) (extract u 2)))
               (k (makes-of t pv us known)))
          (if (null? k)
              (makes-round mx-fail us (cdr rs) known next)
              (let ((j (nth-known next mi)))
                (if (and (not (null? j)) (not (= (car j) (car k))))
                    (begin
                      (fail-at mx-fail (string-append "this template makes " (string-append (made-said (car k))
                                                   (string-append ", and another of `"
                                                   (string-append (symbol->string (extract u 1)) (string-append "`'s "
                                                   (made-said (car j)))))))
                               t)
                      next)
                    (makes-round mx-fail us (cdr rs) known (set-known next mi k)))))))))
(define makes-rounds (->* (fails unsigneds rules makes-known int) makes-known)
  (lambda (mx-fail us rs known n)
    (if (< n 0)
        known
        (let ((next (makes-round mx-fail us rs known (nones (len-us us)))))
          (if (known=? next known) known (makes-rounds mx-fail us rs next (- n 1)))))))
(define makes-all (->* (fails unsigneds rules) mx-infer-ints)
  (lambda (mx-fail us rs)
    (let ((known (makes-rounds mx-fail us rs (nones (len-us us)) (len-us us))))
      (made-or-expr known))))
(define sol-empty (->* () sol)
  (lambda () (tuple (the kinds nil) (the pkinds nil) (the mx-infer-ints nil) (the binds-at nil)
                    (the binds-at nil) (the binds-at nil) (the spans nil))))
(define bind-at (->* (int binds-at) (listof mx-types-bind acyclic))
  (lambda (p bs)
    (cond ((null? bs) nil)
          ((= p (extract (car bs) 1)) (list (extract (car bs) 2)))
          (else (bind-at p (cdr bs))))))
;; Each macro's signature so far, from `s`, in round `n`: its syntax types,
;; each `mu` tagged with the round and its part, so that where a template
;; places it says of what part it is (`solve`'s `classify`; `infer.rs`'s
;; `build`).
(define built-round (ref int mxp) (new 0))
;; Whether part `p` is a `mu` of its own: a list by ellipsis, a `union`, or a
;; part not known yet.
(define has-mu? (->* (int sol) bool)
  (lambda (p s)
    (or (tagcase (mx-infer-shape-of p) (sh-seq (e) #t) (sh-union (ms) #t) (else y #f))
        (and (leaf? p) (null? (kind-at p (extract s 1)))))))
(define mx-infer-position-of (->* (int mx-infer-ints int) int)
  (lambda (q xs i)
    (cond ((null? xs) -1) ((= q (car xs)) i) (else (mx-infer-position-of q (cdr xs) (+ i 1))))))
(define-rec
;; Part `p`'s type, under the `mu`s of parts `around`, the innermost first:
;; one of them referred to by its variable, another part's `mu` written
;; where it is referred to.
(part-ty (->* (int sol mx-infer-ints) sty)
  (lambda (p s around)
    (let* ((base
            (if (has-mu? p s)
                (mu-ty p s around)
                (tagcase (mx-infer-shape-of p)
                  (sh-nil () (s-nil))
                  (sh-lit (n) (s-literal n))
                  (sh-pair (a d) (s-pair (part-ty a s around) (part-ty d s around)))
                  (else y
                    (tagcase (car (kind-at p (extract s 1)))
                      (kd-ref (q)
                        (let ((i (mx-infer-position-of q around 0)))
                          (if (< i 0) (mu-ty q s around) (s-var i))))
                      ;; Its annotations' addresses from its own root.
                      (kd-ty (t) (if (and (compound? t) (open? t)) (s-mu t -1 -1) t)))))))
           (t (if (mx-infer-int-in? p (extract s 3)) (s-many base) base))
           (i (bind-at p (extract s 4))))
      (if (null? i) t (s-import t (car i))))))
;; The `mu` of part `p`, `has-mu?`, under those of `around`.
(mu-ty (->* (int sol mx-infer-ints) sty)
  (lambda (p s around)
    (let ((in (cons p around)))
      (s-mu (tagcase (mx-infer-shape-of p)
              (sh-seq (e) (seq-ty e s in))
              (sh-union (ms) (s-union (members-ty ms s in)))
              ;; Not known: anything, by a `mu` of its own, which says
              ;; where it is placed is this part.
              (else y (s-data)))
            (get built-round) p))))
(members-ty (->* (mx-infer-ints sol mx-infer-ints) stys)
  (lambda (ms s around)
    (if (null? ms)
        nil
        (let* ((t (part-ty (car ms) s around)) (e (bind-at (car ms) (extract s 5))))
          (cons (if (null? e) t (s-export t (car e))) (members-ty (cdr ms) s around))))))
;; As `seq-type` makes `(elem ...)`, its body.
(seq-ty (->* (int sol mx-infer-ints) sty)
  (lambda (e s around)
    (let* ((t (part-ty e s around)) (x (bind-at e (extract s 5)))
           (elem (cond ((not (null? x)) (s-export t (car x)))
                       ((= (mx-types-kind-of t) 1) (s-export t (bind-rib (list (tuple (the addr nil) 1)))))
                       ((= (mx-types-kind-of t) 12) (s-export t (bind-rib (list (tuple (the addr nil) 4)))))
                       (else t))))
      (s-union (seq-body elem))))))
(define build-roots (->* (mx-infer-ints int sol) stys)
  (lambda (rs mi s)
    (if (null? rs)
        nil
        (let* ((t (part-ty (car rs) s nil)) (e (bind-at mi (extract s 6))))
          (cons (if (null? e) t (s-export t (car e))) (build-roots (cdr rs) (+ mi 1) s))))))
(define build (->* (sol int) stys)
  (lambda (s n) (begin (set built-round n) (build-roots (get mx-infer-roots) 0 s))))
(define mx-infer-got (ref (listof mx-infer-placed acyclic) mxp) (new nil))
(define pvars-for (->* (int vars sol bool) pvars)
  (lambda (cid vs s mx-parser-types-names)
    (cond
      ((null? vs) nil)
      ((= cid (extract (car vs) 1))
       (let* ((p (extract (car vs) 3)) (k (kind-at p (extract s 1)))
              (core (cond ((not (leaf? p)) (s-data))
                          ((and (not (null? k)) (tagcase (car k) (kd-ty (t) #t) (else y #f)))
                           (tagcase (car k) (kd-ty (t) t) (else y (s-data))))
                          ((and (null? k) mx-parser-types-names) (s-id))
                          (else (s-data)))))
         (cons (tuple (extract (car vs) 2)
                      (tuple core (the levels nil) #f nil (extract (car vs) 4)))
               (pvars-for cid (cdr vs) s mx-parser-types-names))))
      (else (pvars-for cid (cdr vs) s mx-parser-types-names)))))
(define placements-in (->* (mx-nodes-found) placements)
  (lambda (fs)
    (if (null? fs)
        nil
        (let ((at (extract (car fs) 1)))
          (tagcase (node-kids at)
            (k-pvar (n)
              (cons (tuple n (node-core at) (normal (env-levels (extract (car fs) 2)))
                           (extract (car fs) 3) (extract (car fs) 4))
                    (placements-in (cdr fs))))
            (else y (placements-in (cdr fs))))))))
;; With each pattern variable whose syntax type is not known yet taken for
;; a name (`names`), where a form takes a name or a list; else anything.
(define placed-as (->* (fails int syn int sol bool) mx-infer-placed)
  (lambda (mx-fail cid template made s mx-parser-types-names)
    (begin
      (nodes-reset!)
      (let* ((ps (pvars-for cid (get found-vars) s mx-parser-types-names))
             (ty (cond ((= made 1) (s-defn)) ((= made 2) (s-type)) (else (s-expr))))
             (root (align mx-fail 1 ps ty template -1 nil))
             (fs (mx-scopes-walk mx-fail root env-empty #f 0)))
        (tuple (placements-in fs)
               (if (= made 1)
                   (the (listof mx-exports-exported acyclic)
                        (list (mx-exports-exports-of (levels-of mx-fail root template))))
                   nil))))))
(define try-placing (->* (int syn int sol bool) (listof mx-infer-placed acyclic))
  (lambda (cid template made s mx-parser-types-names)
    (begin
      (set mx-infer-got nil)
      (let ((r (prompt parse-tag
                       (begin (set mx-infer-got (list (placed-as pfail-at cid template made s mx-parser-types-names)))
                              (p-ok nil))
                       (lambda (r) r))))
        (tagcase r (p-err (m x y) (the (listof mx-infer-placed acyclic) nil)) (p-ok (ts) (get mx-infer-got)))))))
(define placements-of (->* (int syn int sol) (listof mx-infer-placed acyclic))
  (lambda (cid template made s)
    (let ((as-names (try-placing cid template made s #t)))
      (if (null? as-names) (try-placing cid template made s #f) as-names))))

;;; ============================================================ macros-show.fx (whole)

;;; ------------------------------------------------------------ written

;; Syntax type `t` as a signature writes it, at `a`..`b` (`infer.rs`'s
;; `shown_sty`): a `mu` whose variable is used, or whose addresses are
;; from it; a list by ellipsis, `(σ ...)`. `shown`, the `mu`s being
;; written, the innermost first, each its variable and whether it is used.
(define shown (ref (listof (tupleof string bool) acyclic) mxp) (new nil))
;; Items `xs` and tail `t` as one list: a tail that is a list joined to it.
(define joined-list (->* (syns-a syn int int) syn)
  (lambda (xs t a b)
    (tagcase t
      (mx-eager-reader-types-lst (more d c e y) (mk-list (append xs more) a b))
      (dotted (more tl d c e y)
        (dotted (append xs more) tl (mx-datum-dotted (syn-datums (append xs more)) (syn->datum tl))
                a b 0))
      (else y
        (if (null? xs) t (dotted xs t (mx-datum-dotted (syn-datums xs) (syn->datum t)) a b 0))))))
(define tail-of? (->* (sty) bool) (lambda (d) (tagcase d (s-var (j) (= j 0)) (else y #f))))
(define addr-same? (->* (addr addr) bool)
  (lambda (x y)
    (if (null? x)
        (null? y)
        (and (not (null? y)) (mx-eq? (car x) (car y)) (addr-same? (cdr x) (cdr y))))))
(define entry-is? (->* ((tupleof addr int) addr int) bool)
  (lambda (e a delta) (and (addr-same? (extract e 1) a) (= (extract e 2) delta))))
(define own-of (->* (entries) (listof entries acyclic))
  (lambda (es)
    (cond ((null? es) nil)
          ((null? (cdr es)) (if (entry-is? (car es) (list #f) 2) (list (the entries nil)) nil))
          ((and (not (null? (extract (car es) 1))) (car (extract (car es) 1)))
           (let ((r (own-of (cdr es))))
             (if (null? r)
                 nil
                 (list (cons (tuple (cdr (extract (car es) 1)) (extract (car es) 2)) (car r))))))
          (else nil))))
(define len-entries (->* (entries) int)
  (lambda (es) (if (null? es) 0 (+ 1 (len-entries (cdr es))))))
(define self-entry? (->* (mx-types-bind int) bool)
  (lambda (bd delta)
    (tagcase bd
      (bind-rib (es) (and (= (len-entries es) 1) (entry-is? (car es) nil delta)))
      (else y #f))))
;; Element `e` of a list whose pair exports `both`, the element's entries from the pair's root,
;; then the rest's, `(D rib)` (`seq-body`), in a list of one (`infer.rs`'s `seq_elem`).
(define rebased-elem (->* (sty mx-types-bind) stys)
  (lambda (e both)
    (tagcase both
      (bind-rib (es)
        (let ((own (own-of es)))
          (cond ((null? own) nil)
                ((and (= (mx-types-kind-of e) 1) (self-entry? (bind-rib (car own)) 1)) (list e))
                ((and (= (mx-types-kind-of e) 12) (self-entry? (bind-rib (car own)) 4)) (list e))
                (else (list (s-export e (bind-rib (car own))))))))
      (else y nil))))
(define no-entries? (->* (mx-types-bind) bool)
  (lambda (bd) (tagcase bd (bind-rib (es) (null? es)) (else y #f))))
(define both-ribs? (->* (mx-types-bind) bool)
  (lambda (bd)
    (tagcase bd
      (bind-rib (es)
        (and (= (len-entries es) 2)
             (entry-is? (car es) (list #t) 2) (entry-is? (car (cdr es)) (list #f) 2)))
      (else y #f))))
(define listed-of (->* (sty sty) stys)
  (lambda (nil-m pair-m)
    (tagcase nil-m
      (s-nil ()
        (tagcase pair-m
          (s-pair (e d)
            (if (and (tail-of? d)
                     (tagcase e (s-bvar () #f) (s-dvar () #f) (s-export (x bd) #f) (else y #t)))
                (list e)
                nil))
          (else y nil)))
      (s-export (n none)
        (tagcase pair-m
          (s-export (p both)
            (if (and (= (mx-types-kind-of n) 6) (no-entries? none))
                (tagcase p
                  (s-pair (e d)
                    (if (tail-of? d)
                        (tagcase e
                          ;; An element's own export not a rib's: the pair's the element's.
                          (s-export (x bd) (if (both-ribs? both) (list e) nil))
                          (else y (rebased-elem e both)))
                        nil))
                  (else y nil))
                nil))
          (else y nil)))
      (else y nil))))
;; The element of `mu` `t`, in a list of one, if it is a list as `(σ ...)`
;; writes one (`seq-type`) and its element does not refer to it: its
;; element as written, outside the list's `mu`.
(define listed-elem (->* (sty) stys)
  (lambda (t)
    (let ((e (tagcase t
               (s-mu (x r p)
                 (tagcase x
                   (s-union (ms)
                     (if (and (not (null? ms)) (not (null? (cdr ms))) (null? (cdr (cdr ms))))
                         (listed-of (car ms) (car (cdr ms)))
                         nil))
                   (else y nil)))
               (else y nil))))
      (if (null? e) nil (lowered (car e))))))
(define nth-of (->* (int (listof (tupleof string bool) acyclic)) (tupleof string bool))
  (lambda (j xs) (if (= j 0) (car xs) (nth-of (- j 1) (cdr xs)))))
(define nth-shown (->* (int) (tupleof string bool))
  (lambda (j) (nth-of j (get shown))))
(define used! (->* (int (listof (tupleof string bool) acyclic))
                   (listof (tupleof string bool) acyclic))
  (lambda (j xs)
    (if (= j 0)
        (cons (tuple (extract (car xs) 1) #t) (cdr xs))
        (cons (car xs) (used! (- j 1) (cdr xs))))))
(define len-shown (->* ((listof (tupleof string bool) acyclic)) int)
  (lambda (xs) (if (null? xs) 0 (+ 1 (len-shown (cdr xs))))))
(define delta-name (->* (int) string)
  (lambda (d)
    (cond ((= d 0) "none") ((= d 1) "var") ((= d 2) "rib") ((= d 3) "env") ((= d 4) "dvar")
          (else "place"))))
(define mx-show-path-keys (->* (addr int int) syns-a)
  (lambda (x a b)
    (if (null? x)
        nil
        (cons (atom (mx-symbol->keyword (if (car x) 'a 'd)) a b 0) (mx-show-path-keys (cdr x) a b)))))
;; `(path #:a #:d …)`, its steps from the root, first first.
(define show-addr (->* (addr int int) syn)
  (lambda (x a b)
    (if (null? x) (mk-list nil a b) (mk-list (cons (mk-symbol "path" a b) (mx-show-path-keys x a b)) a b))))
(define show-entries (->* (entries int int) syns-a)
  (lambda (es a b)
    (if (null? es)
        nil
        (cons (mk-list (list (show-addr (extract (car es) 1) a b)
                             (let ((d (extract (car es) 2)))
                               (if (>= d 100) (syntax-sig-of d) (mk-symbol (delta-name d) a b))))
                       a b)
              (show-entries (cdr es) a b)))))
(define show-rib (->* (entries int int) syn)
  (lambda (es a b) (mk-list (cons (mk-symbol "rho" a b) (show-entries es a b)) a b)))
(define show-rings (->* (rings int int) syns-a)
  (lambda (rs a b)
    (if (null? rs)
        nil
        (cons (tagcase (car rs)
                (ring-rib (es) (show-rib es a b))
                (ring-at (x) (mk-list (list (mk-symbol "at" a b) (show-addr x a b)) a b))
                (ring-shape (x) (mk-list (list (mk-symbol "shape" a b) (show-addr x a b)) a b)))
              (show-rings (cdr rs) a b)))))
(define show-bind (->* (mx-types-bind int int) syn)
  (lambda (bd a b)
    (tagcase bd
      (bind-rib (es) (show-rib es a b))
      (bind-cage (rs) (mk-list (cons (mk-symbol "cage" a b) (show-rings rs a b)) a b))
      (else y (show-rib nil a b)))))
;; Whether `n` is a kernel form's word, as a signature shows it: reserved,
;; and not one of the standard trailmarks, `else` and `=>`.
(define word? (->* (symbol) bool)
  (lambda (n) (and (reserved-sym? n) (not (symbol=? n 'else)) (not (symbol=? n '=>)))))
(define-rec
(show-sty (->* (sty int int) syn)
  (lambda (t a b)
    (let ((sym (lambda ((s string)) (mk-symbol s a b))))
      (tagcase t
        (s-expr () (sym "expr")) (s-lambda () (sym "lambda")) (s-bvar () (sym "bvar"))
        (s-id () (sym "id")) (s-data () (sym "data")) (s-type () (sym "desc"))
        (s-label () (sym "label")) (s-dvar () (sym "dvar")) (s-kind () (sym "kind"))
        (s-defn () (sym "defn")) (s-item () (sym "item")) (s-rules () (sym "rules"))
        (s-nil () (mk-list nil a b))
        ;; A `syntax-rules`' trailmark, inferred as a word its pattern
        ;; writes; a reserved name but the standard trailmarks, a kernel
        ;; form's word.
        (s-literal (n)
          (mk-list (list (sym (if (word? n) "literal" "trailmark")) (atom n a b 0)) a b))
        (s-mark (n i) (mk-list (list (sym "trailmark") (atom n a b 0)) a b))
        (s-pair (x y) (show-pair t nil a b))
        (s-import (x bd) (mk-list (list (sym "import") (show-sty x a b) (show-bind bd a b)) a b))
        (s-export (x bd) (mk-list (list (sym "export") (show-sty x a b) (show-bind bd a b)) a b))
        (s-many (x) (mk-list (list (sym "many") (show-sty x a b)) a b))
        (s-typed (x h) (show-sty x a b))
        (s-union (ms) (mk-list (cons (sym "oneof") (mx-show-show-each ms a b)) a b))
        (s-var (j) (begin (set shown (used! j (get shown))) (sym (extract (nth-shown j) 1))))
        (s-mu (x r p) (show-mu t a b))))))
(mx-show-show-each (->* (stys int int) syns-a)
  (lambda (ms a b) (if (null? ms) nil (cons (show-sty (car ms) a b) (mx-show-show-each (cdr ms) a b)))))
(show-pair (->* (sty syns-a int int) syn)
  (lambda (cur items a b)
    (tagcase cur
      (s-pair (x y) (show-pair y (append items (the syns-a (list (show-sty x a b)))) a b))
      (s-nil () (mk-list items a b))
      (s-mu (x r p)
        (let ((e (listed-elem cur)))
          (if (null? e)
              (joined-list items (show-sty cur a b) a b)
              (mk-list (append items
                               (the syns-a (list (show-sty (car e) a b) (mk-symbol "..." a b))))
                       a b))))
      (else y (joined-list items (show-sty cur a b) a b)))))
(show-mu (->* (sty int int) syn)
  (lambda (t a b)
    (let ((e (listed-elem t)) (body (tagcase t (s-mu (x r p) x) (else y t))))
      (if (not (null? e))
          (mk-list (list (show-sty (car e) a b) (mk-symbol "..." a b)) a b)
          (let ((name (string-append "m" (int->string (+ (len-shown (get shown)) 1)))))
            (begin
              (set shown (cons (tuple name #f) (get shown)))
              (let* ((shown-body (show-sty body a b)) (used (extract (car (get shown)) 2)))
                (begin
                  (set shown (cdr (get shown)))
                  (if (or used (open? body))
                      (mk-list (list (mk-symbol "mu" a b) (mk-symbol name a b) shown-body) a b)
                      shown-body))))))))))
(define sig-word? (->* (syn) bool)
  (lambda (s)
    (and (syn-symbol? s)
         (let ((n (syn-name s)))
           (or (string=? n "import") (string=? n "export") (string=? n "union") (string=? n "oneof")
               (string=? n "mu") (string=? n "trailmark")
               (string=? n "many") (string=? n "literal"))))))
(define-rec
;; Signature `s` as a message says it (`infer.rs`'s `sig_text`): each
;; annotation, `union`, `mu`, `many` or literal that is a list's rest after
;; a ` . `.
(sig-text (->* (syn) string)
  (lambda (s)
    (tagcase s
      (mx-eager-reader-types-lst (items d a b x) (if (null? items) "()" (string-append "(" (string-append (sig-items items #t a b) ")"))))
      (dotted (items tl d a b x)
        (string-append "(" (string-append (sig-items items #f a b) (string-append " . " (string-append (sig-text tl) ")")))))
      (else y (show-syn s)))))
;; Items `xs`, the first first (`first`); a word of `sig-word?` past the
;; first, and a proper list's, starts its rest.
(sig-items (->* (syns-a bool int int) string)
  (lambda (xs proper a b) (sig-items-from xs #t proper a b)))
(sig-items-from (->* (syns-a bool bool int int) string)
  (lambda (xs first proper a b)
    (cond ((null? xs) "")
          ((and (not first) proper (sig-word? (car xs)))
           (string-append " . " (sig-text (mk-list xs a b))))
          (else (string-append (if first "" " ") (string-append (sig-text (car xs))
                               (sig-items-from (cdr xs) #f proper a b))))))))

;;; ============================================================ macros-shown.fx (whole)

(define mx-shown-regions (ref (listof zone acyclic) mxp) (new nil))
(define mx-shown-imported (ref mx-shown-imports mxp) (new nil))
(define wanted (ref spots mxp) (new nil))
(define mx-shown-given (ref givens mxp) (new nil))
(define mx-shown-places (ref spots mxp) (new nil))
(define used (ref (listof string acyclic) mxp) (new nil))
(define counter (ref int mxp) (new 0))
;; The `letrec-shape`'s shapes, each `mu`'s, the newest first; its node or union once written.
(define mx-shown-shapes (ref (listof (tupleof symbol syns-a) acyclic) mxp) (new nil))
(define shown-result (ref syns-a mxp) (new nil))
;; Not said in shapes and facts: back to `in-shapes`.
(define cannot (->* (syn) void)
  (lambda (s) (pfail-at "not in shapes and facts" (syn-start s) (syn-end s))))
(define pattern-of (->* (syns-a) syns-a)
  (lambda (clause)
    (let ((c (if (null? clause) nil (split (car clause)))))
      (let ((p (if (null? c) nil (split (car c))))) (if (null? p) nil (list (nth p 1)))))))
(define name-syn (->* (symbol int int) syn)
  (lambda (n a b) (mk-symbol (symbol->string n) a b)))
(define shape-defs (->* ((listof (tupleof symbol syns-a) acyclic) int int) syns-a)
  (lambda (xs a b)
    (if (null? xs)
        nil
        (cons (mk-list (list (name-syn (extract (car xs) 1) a b) (car (extract (car xs) 2))) a b)
              (shape-defs (cdr xs) a b)))))
(define-rec
;; `s` written, each list's tail that is a list its items: the same structure, however built.
(flat (->* (syn) string)
  (lambda (s)
    (tagcase s
      (mx-eager-reader-types-lst (items d a b x) (string-append "(" (string-append (flat-items items #t) ")")))
      (dotted (items tl d a b x)
        (if (or (syn-nil? tl) (not (null? (split tl))))
            (flat (list-with items (list tl) a b))
            (string-append "(" (string-append (flat-items items #t) (string-append " . " (string-append (flat tl) ")"))))))
      (else y (show-syn s)))))
(flat-items (->* (syns-a bool) string)
  (lambda (xs first)
    (if (null? xs)
        ""
        (string-append (if first "" " ") (string-append (flat (car xs)) (flat-items (cdr xs) #f)))))))
;;; ------------------------------------------------------------ nodes

(define node-state (->* () nstate)
  (lambda ()
    (tuple (get mx-shown-regions) (get mx-shown-imported) (get wanted) (get mx-shown-given) (get mx-shown-places) (get used)
           (get counter))))
(define node-restore! (->* (nstate) unit)
  (lambda (s)
    (begin (set mx-shown-regions (extract s 1)) (set mx-shown-imported (extract s 2)) (set wanted (extract s 3))
           (set mx-shown-given (extract s 4)) (set mx-shown-places (extract s 5)) (set used (extract s 6))
           (set counter (extract s 7)) #u)))
(define node-reset! (->* (syns-a) unit)
  (lambda (pattern)
    (node-restore! (tuple (list (tuple 0 -1 (the addr nil) pattern (the syns-a nil)))
                          (the mx-shown-imports nil) (the spots nil) (the givens nil) (the spots nil)
                          (the (listof string acyclic) nil) 0))))
(define mx-shown-region-of (->* (int (listof zone acyclic)) zone)
  (lambda (r gs) (if (= (extract (car gs) 1) r) (car gs) (mx-shown-region-of r (cdr gs)))))
(define region-export (->* (int) syns-a)
  (lambda (r) (extract (mx-shown-region-of r (get mx-shown-regions)) 5)))
(define element-in (->* (int addr (listof zone acyclic)) int)
  (lambda (r p gs)
    (cond ((null? gs) -1)
          ((and (= (extract (car gs) 2) r) (steps=? (extract (car gs) 3) p)) (extract (car gs) 1))
          (else (element-in r p (cdr gs))))))
;; The element region of the list at `p` of region `r`, -1 if none.
(define element-of (->* (int addr) int)
  (lambda (r p) (element-in r p (get mx-shown-regions))))
(define spot-in? (->* (int addr spots) bool)
  (lambda (r p xs)
    (and (not (null? xs))
         (or (and (= (extract (car xs) 1) r) (steps=? (extract (car xs) 2) p))
             (spot-in? r p (cdr xs))))))
(define want! (->* (int addr) unit)
  (lambda (r p) (if (spot-in? r p (get wanted)) #u (set wanted (cons (tuple r p) (get wanted))))))
;; A rib's entry `(path δ)` of region `r`: its part, or, a list's rib of what its element
;; exports, one entry, `(v ...)`.
(define mx-shown-entry (->* (syn int) mx-shown-label)
  (lambda (e r)
    (let* ((parts (proper-items e))
           (ok (if (and (= (len parts) 2) (syn-symbol? (nth parts 1))) #u (cannot e)))
           (p (read-addr pfail-at (car parts))) (d (syn-name (nth parts 1)))
           (er (element-of r p)))
      (if (and (string=? d "rib") (>= er 0))
          (let* ((ex (region-export er))
                 (items (if (null? ex) (begin (cannot e) nil) (proper-items (car ex))))
                 (inner (if (and (= (len items) 2) (mx-facts-sym-is? (car items) "rho"))
                            (mx-shown-entry (nth items 1) er)
                            (begin (cannot e) (tuple 0 er p)))))
            (if (= (extract inner 1) 0) (tuple 1 er (extract inner 3)) (begin (cannot e) inner)))
          (begin (want! r p)
                 (if (string=? d "place") (set mx-shown-places (cons (tuple r p) (get mx-shown-places))) #u)
                 (tuple 0 r p))))))
;; What binding `b` of region `r` says, one label: `(rho entry)`, `(cage (at path))`, or a
;; ring, `(at path)`.
(define mx-shown-bound (->* (syn int) mx-shown-label)
  (lambda (b r)
    (let ((items (proper-items b)))
      (cond ((and (mx-facts-head-is? b "rho") (= (len items) 2)) (mx-shown-entry (nth items 1) r))
            ((and (mx-facts-head-is? b "cage") (= (len items) 2) (mx-facts-head-is? (nth items 1) "at"))
             (mx-shown-bound (nth items 1) r))
            ((and (mx-facts-head-is? b "at") (= (len items) 2))
             (let* ((p (read-addr pfail-at (nth items 1))) (w (want! r p))) (tuple 0 r p)))
            (else (begin (cannot b) (tuple 0 r (the addr nil))))))))
(define binds-of (->* (mx-shown-imports) (listof (tupleof int addr mx-shown-label) acyclic))
  (lambda (xs)
    (if (null? xs)
        nil
        (let* ((r (extract (car xs) 1)) (tp (extract (car xs) 2))
               (w (want! r tp)) (l (mx-shown-bound (extract (car xs) 3) r)))
          (cons (tuple r tp l) (binds-of (cdr xs)))))))
(define mx-shown-entries-of (->* (syns-a int) mx-shown-labels)
  (lambda (es r)
    (if (null? es) nil (let ((l (mx-shown-entry (car es) r))) (cons l (mx-shown-entries-of (cdr es) r))))))
(define mx-shown-rings-of (->* (syns-a) mx-shown-labels)
  (lambda (rs) (if (null? rs) nil (let ((l (mx-shown-bound (car rs) 0))) (cons l (mx-shown-rings-of (cdr rs)))))))
;; A node's export: a rib's entries, or a ribcage's rings, innermost last.
(define mx-shown-exports-of (->* (syn) mx-shown-labels)
  (lambda (b)
    (let ((items (proper-items b)))
      (cond ((mx-facts-head-is? b "rho") (mx-shown-entries-of (cdr items) 0))
            ((mx-facts-head-is? b "cage") (mx-shown-rings-of (reverse (cdr items))))
            (else (begin (cannot b) nil))))))
;; What a use defines: each a label, and its signature if a macro's (none, or one).
(define defined-of (->* (syns-a) (listof (tupleof mx-shown-label syns-a) acyclic))
  (lambda (es)
    (if (null? es)
        nil
        (let* ((parts (proper-items (car es)))
               (d (if (= (len parts) 2) (nth parts 1) (begin (cannot (car es)) (car es))))
               (one (if (syn-symbol? d)
                        (tuple (mx-shown-entry (car es) 0) (the syns-a nil))
                        (let* ((p (read-addr pfail-at (car parts))) (w (want! 0 p)))
                          (tuple (tuple 0 0 p) (list d))))))
          (cons one (defined-of (cdr es)))))))
(define name-in (->* (int addr givens) symbol)
  (lambda (r p xs)
    (cond ((null? xs) (begin (pfail-at "a part not named" 0 0) (string->symbol "_")))
          ((and (= (extract (car xs) 1) r) (steps=? (extract (car xs) 2) p)) (extract (car xs) 3))
          (else (name-in r p (cdr xs))))))
(define mx-shown-name-of (->* (int addr) symbol)
  (lambda (r p) (name-in r p (get mx-shown-given))))
;; A label said: its name, or `(name ...)`.
(define mx-shown-said (->* (mx-shown-label int int) syn)
  (lambda (l a b)
    (let ((n (name-syn (mx-shown-name-of (extract l 2) (extract l 3)) a b)))
      (if (= (extract l 1) 0) n (mk-list (list n (mk-symbol "..." a b)) a b)))))
(define defined-said (->* ((listof (tupleof mx-shown-label syns-a) acyclic) int int) syns-a)
  (lambda (ds a b)
    (if (null? ds)
        nil
        (let ((l (mx-shown-said (extract (car ds) 1) a b)))
          (cons (if (null? (extract (car ds) 2))
                    l
                    (mk-list (list l (car (extract (car ds) 2))) a b))
                (defined-said (cdr ds) a b))))))
(define bind-facts (->* ((listof (tupleof int addr mx-shown-label) acyclic) int int) syns-a)
  (lambda (bs a b)
    (if (null? bs)
        nil
        (let* ((r (extract (car bs) 1))
               (f (mk-list (list (mk-symbol "bind" a b) (mx-shown-said (extract (car bs) 3) a b)
                                 (mk-symbol "in" a b)
                                 (name-syn (mx-shown-name-of r (extract (car bs) 2)) a b))
                           a b))
               (rest (bind-facts (cdr bs) a b)))
          (if (= r 0) (cons f rest) (cons f (cons (mk-symbol "..." a b) rest)))))))
(define said-each (->* (mx-shown-labels int int) syns-a)
  (lambda (ls a b) (if (null? ls) nil (cons (mx-shown-said (car ls) a b) (said-each (cdr ls) a b)))))
(define named-at? (->* (int addr givens) bool)
  (lambda (r p xs)
    (and (not (null? xs))
         (or (and (= (extract (car xs) 1) r) (steps=? (extract (car xs) 2) p))
             (named-at? r p (cdr xs))))))
(define all-named? (->* (spots) bool)
  (lambda (xs)
    (or (null? xs)
        (and (named-at? (extract (car xs) 1) (extract (car xs) 2) (get mx-shown-given))
             (all-named? (cdr xs))))))
;;; ------------------------------------------------------------ trees

;; Whether `s` is a word that starts a list's rest, `(bvar . (import …))`, as `sig-text` shows.
(define rest-word? (->* (syn) bool)
  (lambda (s)
    (or (mx-facts-sym-is? s "import") (mx-facts-sym-is? s "export") (mx-facts-sym-is? s "mu") (mx-facts-sym-is? s "oneof")
        (mx-facts-sym-is? s "union") (mx-facts-sym-is? s "many") (mx-facts-sym-is? s "literal") (mx-facts-sym-is? s "trailmark"))))
(define parts-split (->* (syns-a syns-a syns-a int int) (tupleof syns-a syns-a))
  (lambda (xs t out a b)
    (cond ((null? xs) (tuple (reverse out) t))
          ((rest-word? (car xs)) (tuple (reverse out) (list (list-with xs t a b))))
          (else (parts-split (cdr xs) t (cons (car xs) out) a b)))))
;; List `s`'s parts, and its tail (none, or one), a rest that is a form its tail.
(define parts-of (->* (syn) (tupleof syns-a syns-a))
  (lambda (s)
    (let ((ps (list-parts s)))
      (if (null? (extract ps 1))
          ps
          (parts-split (cdr (extract ps 1)) (extract ps 2) (list (car (extract ps 1)))
                       (syn-start s) (syn-end s))))))
;;; ------------------------------------------------------------ regions and names

;; How many regions `gs` are.
(define zones-len (->* ((listof zone acyclic)) int)
  (lambda (gs) (if (null? gs) 0 (+ 1 (zones-len (cdr gs))))))
(define region-pattern (->* (int) syns-a)
  (lambda (r) (extract (mx-shown-region-of r (get mx-shown-regions)) 4)))
(define export-set (->* ((listof zone acyclic) int syn) (listof zone acyclic))
  (lambda (gs r ex)
    (cond ((null? gs) nil)
          ((= (extract (car gs) 1) r)
           (cons (tuple r (extract (car gs) 2) (extract (car gs) 3) (extract (car gs) 4) (list ex))
                 (cdr gs)))
          (else (cons (car gs) (export-set (cdr gs) r ex))))))
;; Pattern `p`'s part at `path`, if it has one (none, or one).
(define at-path (->* (syns-a addr) syns-a)
  (lambda (p path)
    (cond ((or (null? p) (null? path)) p)
          (else (let ((ps (split (car p))))
                  (if (null? ps)
                      nil
                      (at-path (list (if (car path) (car ps) (nth ps 1))) (cdr path))))))))
(define no-exports? (->* ((listof zone acyclic)) bool)
  (lambda (gs) (or (null? gs) (and (null? (extract (car gs) 5)) (no-exports? (cdr gs))))))
(define shape-named? (->* (string (listof (tupleof symbol syns-a) acyclic)) bool)
  (lambda (s xs)
    (and (not (null? xs))
         (or (string=? s (symbol->string (extract (car xs) 1))) (shape-named? s (cdr xs))))))
(define shape-set (->* ((listof (tupleof symbol syns-a) acyclic) symbol syn)
                       (listof (tupleof symbol syns-a) acyclic))
  (lambda (xs n def)
    (cond ((null? xs) nil)
          ((symbol=? (extract (car xs) 1) n) (cons (tuple n (list def)) (cdr xs)))
          (else (cons (car xs) (shape-set (cdr xs) n def))))))
(define string-in? (->* (string (listof string acyclic)) bool)
  (lambda (s xs) (and (not (null? xs)) (or (string=? s (car xs)) (string-in? s (cdr xs))))))
(define free? (->* (string) bool)
  (lambda (s)
    (not (or (string=? s "_") (string=? s "...") (string=? s "@") (string=? s ".")
             (string=? s "in") (string-in? s (get used)) (shape-named? s (get mx-shown-shapes))))))
(define next-free (->* () string)
  (lambda ()
    (begin (set counter (+ (get counter) 1))
           (let ((v (string-append "p" (int->string (get counter)))))
             (if (free? v) v (next-free))))))
;; A name for the part at `path` of region `r`, named: its pattern variable there, if free,
;; else the next `pN` free.
(define fresh! (->* (int addr) symbol)
  (lambda (r path)
    (let* ((v (at-path (region-pattern r) path))
           (name (if (and (pair? v) (syn-symbol? (car v)) (free? (syn-name (car v))))
                     (syn-name (car v))
                     (next-free)))
           (n (string->symbol name)))
      (begin (set used (cons name (get used)))
             (set mx-shown-given (cons (tuple r path n) (get mx-shown-given)))
             n))))
;; Part `t` at `p` of region `r`, written `place` if its binder is a place's.
(define mx-shown-typed (->* (syn int addr) syn)
  (lambda (t r p)
    (if (and (spot-in? r p (get mx-shown-places)) (mx-facts-sym-is? t "dvar"))
        (mk-symbol "place" (syn-start t) (syn-end t))
        t)))
(define-rec
;; Bare tree `s`, at `path` of region `r`, written: each part named, `name @ σ`, a tail `name @
;; . σ`, an element `name @ σ ...`.
(write-tree (->* (syn int addr) syn)
  (lambda (s r path)
    (if (or (form? s) (null? (split s)))
        s
        (let ((ps (parts-of s)))
          (write-parts (extract ps 1) (extract ps 2) r path nil (syn-start s) (syn-end s))))))
(write-parts (->* (syns-a syns-a int addr syns-a int int) syn)
  (lambda (xs t r p out a b)
    (cond
      ((null? xs)
       (if (null? t)
           (mk-list (reverse out) a b)
           (let ((w (mx-shown-typed (write-tree (car t) r p) r p)))
             (if (spot-in? r p (get wanted))
                 (if (syn-symbol? w)
                     (let ((n (fresh! r p)))
                       (list-with (reverse (cons (mk-symbol "@" a b) (cons (name-syn n a b) out)))
                                  (list w) a b))
                     (begin (cannot w) w))
                 (list-with (reverse out) (list w) a b)))))
      ((and (= (len xs) 2) (null? t) (mx-facts-sym-is? (nth xs 1) "..."))
       (let* ((er (element-of r p))
              (checked (if (< er 0) (cannot (car xs)) #u))
              (out2 (if (spot-in? er nil (get wanted))
                        (let ((n (fresh! er nil)))
                          (cons (mk-symbol "@" a b) (cons (name-syn n a b) out)))
                        out))
              (elem (mx-shown-typed (write-tree (car xs) er nil) er nil)))
         (mk-list (reverse (cons (nth xs 1) (cons elem out2))) a b)))
      (else
       (let* ((ap (append p (the addr (list #t))))
              (out2 (if (spot-in? r ap (get wanted))
                        (let ((n (fresh! r ap)))
                          (cons (mk-symbol "@" a b) (cons (name-syn n a b) out)))
                        out))
              (part (mx-shown-typed (write-tree (car xs) r ap) r ap)))
         (write-parts (cdr xs) t r (append p (the addr (list #f))) (cons part out2) a b)))))))
(define-rec
;; Node `sigma`, its labels from `pattern` (the top's): its tree, its facts, and at the top
;; (`made` given) what its uses make, in a list of one.
(mx-shown-node (->* (syn syns-a syns-a) (tupleof syn syns-a syns-a))
  (lambda (sigma0 pattern made)
    (let* ((saved (node-state))
           (reset (node-reset! pattern))
           (ex (and (null? made) (mx-facts-head-is? sigma0 "export")))
           (sigma (if ex (nth (proper-items sigma0) 1) sigma0))
           (export (if ex (list (nth (proper-items sigma0) 2)) nil))
           (bare (strip sigma 0 nil))
           (binds (binds-of (reverse (get mx-shown-imported))))
           (cage (and ex (mx-facts-head-is? (car export) "cage")))
           (exports (if ex (mx-shown-exports-of (car export)) nil))
           (defined (if (and (pair? made) (mx-facts-head-is? (car made) "defn"))
                        (defined-of (cdr (proper-items (car made))))
                        nil))
           (tree (write-tree bare 0 nil))
           (checked (if (all-named? (get wanted)) #u (cannot sigma)))
           (a (syn-start sigma)) (b (syn-end sigma))
           (facts (append (bind-facts binds a b)
                          (if (null? exports)
                              nil
                              (list (mk-list (cons (mk-symbol (if cage "export*" "export") a b)
                                                   (said-each exports a b))
                                             a b)))))
           (made2 (cond ((null? made) nil)
                        ((mx-facts-head-is? (car made) "defn")
                         (list (mk-list (cons (mk-symbol "defn" a b) (defined-said defined a b))
                                        (syn-start (car made)) (syn-end (car made)))))
                        (else made))))
      (begin (node-restore! saved) (tuple tree facts made2)))))
;; `s`, at `path` of region `r`, its imports, `mu`s and elements' exports taken out and noted;
;; back to `in-shapes` if one is where a fact cannot say it.
(strip (->* (syn int addr) syn)
  (lambda (s r path)
    (let ((items (proper-items s)) (a (syn-start s)) (b (syn-end s)))
      (cond ((mx-facts-head-is? s "import")
             (if (= (len items) 3)
                 (let ((bare (strip (nth items 1) r path)))
                   (begin (set mx-shown-imported (cons (tuple r path (nth items 2)) (get mx-shown-imported))) bare))
                 (begin (cannot s) s)))
            ((or (mx-facts-head-is? s "export") (mx-facts-head-is? s "path") (mx-facts-head-is? s "rho") (mx-facts-head-is? s "cage"))
             (begin (cannot s) s))
            ((mx-facts-head-is? s "mu")
             (if (and (= (len items) 3) (syn-symbol? (nth items 1)))
                 (begin (shape! (syn-symbol (nth items 1)) (nth items 2)) (nth items 1))
                 (begin (cannot s) s)))
            ((or (mx-facts-head-is? s "oneof") (mx-facts-head-is? s "union"))
             (mk-list (cons (mk-symbol "oneof" a b) (plain-each (cdr items))) a b))
            ((mx-facts-head-is? s "many")
             (if (= (len items) 2)
                 (mk-list (list (car items) (plain (nth items 1))) a b)
                 (begin (cannot s) s)))
            ((form? s) s)
            ((null? (split s)) s)
            (else (let ((ps (parts-of s)))
                    (strip-parts (extract ps 1) (extract ps 2) r path nil a b)))))))
(strip-parts (->* (syns-a syns-a int addr syns-a int int) syn)
  (lambda (xs t r p out a b)
    (cond
      ((null? xs)
       (list-with (reverse out) (if (null? t) nil (list (strip (car t) r p))) a b))
      ;; `elem ...`: the element a region of its own.
      ((and (= (len xs) 2) (null? t) (mx-facts-sym-is? (nth xs 1) "..."))
       (let* ((pc (at-path (region-pattern r) p))
              (pe (let ((ps (if (null? pc) nil (split (car pc)))))
                    (if (null? ps) nil (list (car ps)))))
              (er (zones-len (get mx-shown-regions)))
              (pushed (set mx-shown-regions (cons (tuple er r p pe (the syns-a nil)) (get mx-shown-regions))))
              (ex (mx-facts-head-is? (car xs) "export"))
              (elem (if ex (nth (proper-items (car xs)) 1) (car xs)))
              (noted (if ex
                         (set mx-shown-regions (export-set (get mx-shown-regions) er (nth (proper-items (car xs)) 2)))
                         #u)))
         (mk-list (reverse (cons (nth xs 1) (cons (strip elem er nil) out))) a b)))
      (else
       (let ((part (strip (car xs) r (append p (the addr (list #t))))))
         (strip-parts (cdr xs) t r (append p (the addr (list #f))) (cons part out) a b))))))
(plain-each (->* (syns-a) syns-a)
  (lambda (ms) (if (null? ms) nil (let ((m (plain (car ms)))) (cons m (plain-each (cdr ms)))))))
;; A tree that says no fact: a union's member, a `many`'s part.
(plain (->* (syn) syn)
  (lambda (s)
    (let* ((saved (node-state)) (reset (node-reset! nil)) (bare (strip s 0 nil))
           (ok (and (null? (get mx-shown-imported)) (no-exports? (get mx-shown-regions)))))
      (begin (node-restore! saved) (if ok bare (begin (cannot s) bare))))))
;; `mu` `n`'s shape, `body`, written once: a union's members each a node, or one node.
(shape! (->* (symbol syn) unit)
  (lambda (n body)
    (if (shape-named? (symbol->string n) (get mx-shown-shapes))
        #u
        (let* ((pushed (set mx-shown-shapes (cons (tuple n (the syns-a nil)) (get mx-shown-shapes))))
               (a (syn-start body)) (b (syn-end body))
               (def (if (or (mx-facts-head-is? body "oneof") (mx-facts-head-is? body "union"))
                        (mk-list (cons (mk-symbol "union" a b)
                                       (mx-shown-members (cdr (proper-items body)) 0))
                                 a b)
                        (let ((r (mx-shown-node body nil nil)))
                          (mk-list (cons (name-syn n a b) (cons (extract r 1) (extract r 2)))
                                   a b)))))
          (set mx-shown-shapes (shape-set (get mx-shown-shapes) n def))))))
(mx-shown-members (->* (syns-a int) syns-a)
  (lambda (ms more)
    (if (null? ms)
        nil
        (let* ((r (mx-shown-node (car ms) nil nil)) (a (syn-start (car ms))) (b (syn-end (car ms)))
               (end (syn-nil? (extract r 1)))
               (k (if end more (+ more 1)))
               (mx-shown-label (cond (end "end") ((= k 1) "more")
                            (else (string-append "more" (int->string k)))))
               (m (mk-list (cons (mk-symbol mx-shown-label a b) (cons (extract r 1) (extract r 2))) a b)))
          (cons m (mx-shown-members (cdr ms) k)))))))
(define shown-of (->* (syn syns-a) syn)
  (lambda (sig clause)
    (let ((items (proper-items sig)) (a (syn-start sig)) (b (syn-end sig)))
      (if (not (and (= (len items) 3) (mx-facts-sym-is? (car items) "macro/addressed")))
          (begin (cannot sig) sig)
          (let* ((reset (set mx-shown-shapes nil))
                 (r (mx-shown-node (nth items 1) (pattern-of clause) (list (nth items 2))))
                 (tree (extract r 1)) (under (mk-symbol "_" a b))
                 (top (cond ((syn-nil? tree) (mk-list (list under) a b))
                            ((not (null? (split tree))) (list-with (list under) (list tree) a b))
                            (else (begin (cannot tree) tree))))
                 (defs (shape-defs (reverse (get mx-shown-shapes)) a b))
                 (shape (if (null? defs)
                            top
                            (mk-list (list (mk-symbol "letrec-shape" a b) (mk-list defs a b) top)
                                     a b)))
                 (shown (mk-list (cons (mk-symbol "macro" a b)
                                       (cons shape (cons (car (extract r 3)) (extract r 2))))
                                 a b))
                 (back (elaborated pfail-at shown)))
            (if (string=? (flat back) (flat sig)) shown (begin (cannot sig) shown)))))))
;; Signature `sig`, Herman's, in shapes and facts, its labels from `clause`'s pattern (none, or
;; `((_ . pattern) template)`): in a list of one, none if that does not say the same.
(define in-shapes (->* (syn syns-a) syns-a)
  (lambda (sig clause)
    (let* ((saved-named (get mx-facts-named)) (saved-open (get open))
           (r (prompt parse-tag (begin (set shown-result (list (shown-of sig clause))) (p-ok nil))
                      (lambda (r) r))))
      (begin (set mx-facts-named saved-named)
             (set open saved-open)
             (tagcase r (p-err (m x y) (the syns-a nil)) (p-ok (ts) (get shown-result)))))))

;;; ============================================================ macros-solve.fx (whole)

;; Signature `sig`, inferred for a macro whose clauses are `cs`, as a message says it: in shapes
;; and facts, labelled by its first clause's pattern, if one says it; else in Herman's form
;; (`macros.rs`'s `shown_inferred`).
(define sig-shown (->* (syns-a syn) string)
  (lambda (cs sig)
    (let ((c (in-shapes sig (if (null? cs) nil (list (car cs))))))
      (sig-text (if (null? c) sig (car c))))))
;; What the regions must export, as said, the last said first.
(define demands (ref (listof (tupleof int demand) acyclic) mxp) (new nil))
(define demand! (->* (int demand) unit)
  (lambda (p d) (set demands (cons (tuple p d) (get demands)))))
(define pents=? (->* (pents pents) bool)
  (lambda (a b)
    (if (null? a)
        (null? b)
        (and (not (null? b)) (= (extract (car a) 1) (extract (car b) 1))
             (= (extract (car a) 2) (extract (car b) 2)) (pents=? (cdr a) (cdr b))))))
(define pent-in? (->* ((tupleof int int) pents) bool)
  (lambda (e es)
    (and (not (null? es))
         (or (and (= (extract e 1) (extract (car es) 1)) (= (extract e 2) (extract (car es) 2)))
             (pent-in? e (cdr es))))))
(define rd=? (->* (rd rd) bool)
  (lambda (a b)
    (tagcase a
      (rd-at (x) (tagcase b (rd-at (y) (= x y)) (else z #f)))
      (rd-rib (es) (tagcase b (rd-rib (fs) (pents=? es fs)) (else z #f))))))
(define pent-into (->* ((tupleof int int) pents) pents)
  (lambda (e out)
    (cond ((null? out) (list e))
          ((< (extract e 1) (extract (car out) 1)) (cons e out))
          (else (cons (car out) (pent-into e (cdr out)))))))
;; Entries `es` sorted by part.
(define pents-sorted (->* (pents pents) pents)
  (lambda (es out) (if (null? es) out (pents-sorted (cdr es) (pent-into (car es) out)))))
(define int-into (->* (int mx-infer-ints) mx-infer-ints)
  (lambda (x out)
    (cond ((null? out) (list x)) ((< x (car out)) (cons x out))
          (else (cons (car out) (int-into x (cdr out)))))))
(define ints-sorted (->* (mx-infer-ints mx-infer-ints) mx-infer-ints)
  (lambda (xs out) (if (null? xs) out (ints-sorted (cdr xs) (int-into (car xs) out)))))
(define gent-merged (->* (int int int gents) gents)
  (lambda (p d g es)
    (cond ((null? es) (list (tuple p d g)))
          ((and (= p (extract (car es) 1)) (= g (extract (car es) 3)))
           (let ((e (extract (car es) 2)))
             (if (and (or (= e 1) (= e 4)) (not (= e d))) (cons (tuple p 5 g) (cdr es)) es)))
          (else (cons (car es) (gent-merged p d g (cdr es)))))))
(define rib-gents (->* (int atms gents) (listof gents acyclic))
  (lambda (cid as es)
    (if (null? as)
        (list es)
        (let ((nd (tagcase (car as)
                    (atm-var (n d g) (list (tuple n (if d 4 1) g)))
                    (atm-rib-of (n) (list (tuple n 2 0)))
                    (atm-lit (t d) (the (listof (tupleof symbol int int) acyclic) nil)))))
          (if (null? nd)
              (rib-gents cid (cdr as) es)
              (let ((p (mx-infer-var-part cid (extract (car nd) 1))))
                (if (null? p)
                    nil
                    (rib-gents cid (cdr as)
                               (gent-merged (car p) (extract (car nd) 2) (extract (car nd) 3)
                                            es)))))))))
(define gent-into (->* ((tupleof int int int) gents) gents)
  (lambda (e out)
    (cond ((null? out) (list e))
          ((or (< (extract e 1) (extract (car out) 1))
               (and (= (extract e 1) (extract (car out) 1))
                    (< (extract e 3) (extract (car out) 3))))
           (cons e out))
          (else (cons (car out) (gent-into e (cdr out)))))))
;; Entries `es` sorted by part, then by how many lists gathered each.
(define gents-sorted (->* (gents gents) gents)
  (lambda (es out) (if (null? es) out (gents-sorted (cdr es) (gent-into (car es) out)))))
;; Environment `env` of clause `cid`, its atoms by part, in a list of one;
;; none if it has a level an inferred signature does not say.
(define lvs-of (->* (int levels) (listof lvs acyclic))
  (lambda (cid env)
    (if (null? env)
        (list (the lvs nil))
        (let ((rest (lvs-of cid (cdr env))))
          (if (null? rest)
              nil
              (tagcase (car env)
                (lv-rib (as)
                  (let ((es (rib-gents cid as nil)))
                    (if (null? es)
                        nil
                        (list (cons (iv-rib (gents-sorted (car es) nil)) (car rest))))))
                (lv-cage (n)
                  (let ((p (mx-infer-var-part cid n)))
                    (if (null? p) nil (list (cons (iv-cage (car p)) (car rest))))))
                (lv-earlier (as) nil)
                (lv-shape (m) nil)))))))
(define gent-parts (->* (gents) mx-infer-ints)
  (lambda (es) (if (null? es) nil (cons (extract (car es) 1) (gent-parts (cdr es))))))
(define atoms-of (->* (lv) mx-infer-ints)
  (lambda (l)
    (tagcase l
      (iv-rib (es) (gent-parts es))
      (iv-cage (p) (list p)))))
(define seqs-from (->* (int int int) int)
  (lambda (q r n)
    (if (= q r)
        n
        (let ((b (boundary-of q)))
          (if (null? b)
              n
              (seqs-from (mx-infer-region (car b)) r
                         (+ n (tagcase (mx-infer-shape-of (car b)) (sh-seq (e) 1) (else y 0)))))))))
;; How many lists by ellipsis are between part `x` and region `r` around it.
(define seqs-between (->* (int int) int)
  (lambda (x r) (seqs-from (mx-infer-region x) r 0)))
(define nth-int (->* (mx-infer-ints int) int)
  (lambda (xs i) (if (= i 0) (car xs) (nth-int (cdr xs) (- i 1)))))
(define all-gathered? (->* (gents int) bool)
  (lambda (es r)
    (or (null? es)
        (and (not (null? (via (extract (car es) 1) r)))
             (= (seqs-between (extract (car es) 1) r) (extract (car es) 3))
             (all-gathered? (cdr es) r)))))
(define mx-solve-len-ints (->* (mx-infer-ints) int) (lambda (xs) (if (null? xs) 0 (+ 1 (mx-solve-len-ints (cdr xs))))))
(define all-via? (->* (mx-infer-ints int) bool)
  (lambda (xs r) (or (null? xs) (and (not (null? (via (car xs) r))) (all-via? (cdr xs) r)))))
;; The first of regions `rs` from `i` around each of level `l`'s binders,
;; through as many lists as gathered each; -1 if none is.
(define first-around (->* (mx-infer-ints int lv) int)
  (lambda (rs i l)
    (cond ((>= i (mx-solve-len-ints rs)) -1)
          ((tagcase l
             (iv-rib (es) (all-gathered? es (nth-int rs i)))
             (iv-cage (y) (all-via? (list y) (nth-int rs i))))
           i)
          (else (first-around rs (+ i 1) l)))))
(define sites-from (->* (int mx-infer-ints int lvs sites) (listof sites acyclic))
  (lambda (p rs i ls out)
    (if (null? ls)
        (list (reverse out))
        (let ((j (first-around rs i (car ls))))
          (if (< j 0)
              nil
              (let* ((site (if (= j 0) p (car (boundary-of (nth-int rs (- j 1))))))
                     (out2 (if (and (not (null? out)) (= site (extract (car out) 1)))
                               (cons (tuple site (extract (car out) 2)
                                            (append (extract (car out) 3)
                                                    (the lvs (list (car ls)))))
                                     (cdr out))
                               (cons (tuple site (nth-int rs j) (the lvs (list (car ls)))) out))))
                (sites-from p rs j (cdr ls) out2)))))))
(define sites-of (->* (int lvs) (listof sites acyclic))
  (lambda (p ls) (sites-from p (mx-infer-regions p) 0 ls nil)))
(define all-through? (->* (mx-infer-ints int int) bool)
  (lambda (xs r u)
    (or (null? xs)
        (let ((v (via (car xs) r)))
          (and (not (null? v)) (= (car v) u) (all-through? (cdr xs) r u))))))
;; The `union` in `r` all of level `l`'s binders are inside; -1 if none.
(define through (->* (lv int) int)
  (lambda (l r)
    (let ((xs (atoms-of l)))
      (if (null? xs)
          -1
          (let ((v (via (car xs) r)))
            (if (and (not (null? v)) (>= (car v) 0) (mx-infer-union? (car v)) (all-through? xs r (car v)))
                (car v)
                -1))))))
;; The levels from the first of `ls` all through `u`, and the rest.
(define run-through (->* (lvs int int) (tupleof lvs lvs))
  (lambda (ls r u)
    (if (and (not (null? ls)) (= (through (car ls) r) u))
        (let ((more (run-through (cdr ls) r u)))
          (tuple (cons (car ls) (extract more 1)) (extract more 2)))
        (tuple (the lvs nil) ls))))
(define thru-add (->* (int (tupleof int int int) thru) thru)
  (lambda (b e th)
    (cond ((null? th) (list (tuple b (the gents (list e)))))
          ((= b (extract (car th) 1))
           (cons (tuple b (append (extract (car th) 2) (the gents (list e)))) (cdr th)))
          (else (cons (car th) (thru-add b e (cdr th)))))))
(define entries-by (->* (gents int pents thru) (listof (tupleof pents thru) acyclic))
  (lambda (es r out th)
    (if (null? es)
        (list (tuple out th))
        (let* ((e (car es)) (v (via (extract e 1) r)) (g (extract e 3)))
          (cond ((null? v) nil)
                ((< (car v) 0)
                 (if (= g 0)
                     (entries-by (cdr es) r
                                 (append out (the pents (list (tuple (extract e 1) (extract e 2)))))
                                 th)
                     nil))
                (else
                 (let ((seq (tagcase (mx-infer-shape-of (car v)) (sh-seq (x) #t) (else y #f))))
                   (if (and seq (= g 0))
                       nil
                       (entries-by (cdr es) r out
                                   (thru-add (car v)
                                             (tuple (extract e 1) (extract e 2) (if seq (- g 1) g))
                                             th))))))))))
(define-rec
(rib-entries (->* (gents int int) (listof pents acyclic))
  (lambda (es r cid)
    (let ((split (entries-by es r nil nil)))
      (if (null? split)
          nil
          (let ((rest (through-entries (extract (car split) 2) r cid)))
            (if (null? rest)
                nil
                (list (pents-sorted (append (extract (car split) 1) (car rest)) nil))))))))
(through-entries (->* (thru int int) (listof pents acyclic))
  (lambda (th r cid)
    (if (null? th)
        (list (the pents nil))
        (let* ((b (extract (car th) 1)) (m (inner b cid)))
          (if (null? m)
              nil
              (let ((sub (rib-entries (extract (car th) 2) (car m) cid)))
                (if (null? sub)
                    nil
                    (begin
                      (demand! (car m) (dm-rib (car sub)))
                      (let ((rest (through-entries (cdr th) r cid)))
                        (if (null? rest) nil (list (cons (tuple b 2) (car rest))))))))))))))
;; Levels `ls`, said from region `r` for clause `cid` (`infer.rs`'s
;; `rings`), in a list of one; what each region exports noted.
(define mx-solve-rings-of (->* (lvs int int) (listof rds acyclic))
  (lambda (ls r cid)
    (if (null? ls)
        (list (the rds nil))
        (let ((u (through (car ls) r)))
          (if (>= u 0)
              (let* ((run (run-through ls r u)) (m (inner u cid)))
                (if (null? m)
                    nil
                    (let ((sub (mx-solve-rings-of (extract run 1) (car m) cid)))
                      (if (null? sub)
                          nil
                          (begin
                            (demand! (car m) (dm-cage (car sub)))
                            (let ((rest (mx-solve-rings-of (extract run 2) r cid)))
                              (if (null? rest) nil (list (cons (rd-at u) (car rest))))))))))
              (let ((one (tagcase (car ls)
                           (iv-cage (y)
                             (let ((v (via y r)))
                               (if (and (not (null? v)) (< (car v) 0))
                                   (the (listof rd acyclic) (list (rd-at y)))
                                   nil)))
                           (iv-rib (es)
                             (let ((e (rib-entries es r cid)))
                               (if (null? e) nil (list (rd-rib (car e)))))))))
                (if (null? one)
                    nil
                    (let ((rest (mx-solve-rings-of (cdr ls) r cid)))
                      (if (null? rest) nil (list (cons (car one) (car rest))))))))))))
(define len-rds (->* (rds) int) (lambda (xs) (if (null? xs) 0 (+ 1 (len-rds (cdr xs))))))
(define longest-rings (->* (site-rings rds) rds)
  (lambda (xs best)
    (cond ((null? xs) best)
          ((>= (len-rds (extract (car xs) 2)) (len-rds best))
           (longest-rings (cdr xs) (extract (car xs) 2)))
          (else (longest-rings (cdr xs) best)))))
(define all-in? (->* (pents pents) bool)
  (lambda (a b) (or (null? a) (and (pent-in? (car a) b) (all-in? (cdr a) b)))))
(define all-in-or-union? (->* (pents pents) bool)
  (lambda (b a)
    (or (null? b)
        (and (or (pent-in? (car b) a) (and (= (extract (car b) 2) 2) (mx-infer-union? (extract (car b) 1))))
             (all-in-or-union? (cdr b) a)))))
(define rib-fits? (->* (pents pents) bool)
  (lambda (a b) (and (all-in? a b) (all-in-or-union? b a))))
(define empty-ribs! (->* (int pents pents) unit)
  (lambda (cid a b)
    (if (null? b)
        #u
        (begin
          (if (pent-in? (car b) a)
              #u
              (let ((m (inner (extract (car b) 1) cid)))
                (if (null? m) #u (demand! (car m) (dm-rib nil)))))
          (empty-ribs! cid a (cdr b))))))
(define fit (->* (int rds rds) unit)
  (lambda (cid rs ls)
    (if (null? ls)
        #u
        (let ((l (car ls)))
          (cond
            ((and (not (null? rs)) (rd=? (car rs) l)) (fit cid (cdr rs) (cdr ls)))
            ((and (not (null? rs))
                  (tagcase (car rs)
                    (rd-rib (a) (tagcase l (rd-rib (b) (rib-fits? a b)) (else y #f)))
                    (else y #f)))
             (begin
               (tagcase (car rs)
                 (rd-rib (a) (tagcase l (rd-rib (b) (empty-ribs! cid a b)) (else y #u)))
                 (else y #u))
               (fit cid (cdr rs) (cdr ls))))
            ((tagcase l (rd-at (u) (mx-infer-union? u)) (else y #f))
             (begin
               (tagcase l
                 (rd-at (u) (let ((m (inner u cid)))
                              (if (null? m) #u (demand! (car m) (dm-cage nil)))))
                 (else y #u))
               (fit cid rs (cdr ls))))
            (else (fit cid rs (cdr ls))))))))
(define fit-each (->* (site-rings rds) unit)
  (lambda (xs longest)
    (if (null? xs)
        #u
        (begin (fit (extract (car xs) 1) (extract (car xs) 2) longest)
               (fit-each (cdr xs) longest)))))
(define merge-rings (->* (site-rings) rds)
  (lambda (mx-list)
    (if (null? mx-list)
        nil
        (let ((longest (longest-rings (cdr mx-list) (extract (car mx-list) 2))))
          (begin (fit-each mx-list longest) longest)))))
(define mx-solve-entries-of (->* (pents) entries)
  (lambda (es)
    (if (null? es) nil (cons (tuple (path-of (extract (car es) 1)) (extract (car es) 2))
                             (mx-solve-entries-of (cdr es))))))
(define ring-of (->* (rd) ring)
  (lambda (r)
    (tagcase r
      (rd-at (x) (ring-at (path-of x)))
      (rd-rib (es) (ring-rib (mx-solve-entries-of es))))))
(define rings-each (->* (rds) rings)
  (lambda (rs) (if (null? rs) nil (cons (ring-of (car rs)) (rings-each (cdr rs))))))
(define rings-bind (->* (rds) mx-types-bind)
  (lambda (rs)
    (if (and (not (null? rs)) (null? (cdr rs)) (tagcase (car rs) (rd-rib (es) #t) (else y #f)))
        (tagcase (car rs) (rd-rib (es) (bind-rib (mx-solve-entries-of es))) (else y (bind-rib nil)))
        (bind-cage (rings-each rs)))))
;; A part of the signature so far: a list by ellipsis or a `union`,
;; itself; one whose syntax type is not known yet, none.
(define classify (->* (sty) kds)
  (lambda (core)
    (tagcase core
      (s-mu (b r p)
        (cond ((not (= r (get built-round))) (list (kd-ty core)))
              ((leaf? p) nil)
              (else (list (kd-ref p)))))
      (else y (list (kd-ty core))))))
(define name-add (->* (symbol kds named-kds) named-kds)
  (lambda (n ks out)
    (cond ((null? out) (list (tuple n ks)))
          ((symbol=? n (extract (car out) 1))
           (cons (tuple n (append (extract (car out) 2) ks)) (cdr out)))
          (else (cons (car out) (name-add n ks (cdr out)))))))
(define by-name (->* (placements named-kds) named-kds)
  (lambda (pls out)
    (if (null? pls)
        out
        (by-name (cdr pls) (name-add (extract (car pls) 1) (classify (extract (car pls) 2)) out)))))
(define pkinds-of (->* (int named-kds pkinds) pkinds)
  (lambda (cid xs out)
    (if (null? xs)
        out
        (let ((c (extract (combine (extract (car xs) 2)) 1)))
          (pkinds-of cid (cdr xs)
                     (if (null? c) out (cons (tuple cid (extract (car xs) 1) (car c)) out)))))))
(define pkinds-each (->* (by-clause int pkinds) pkinds)
  (lambda (ps cid out)
    (if (null? ps)
        out
        (pkinds-each (cdr ps) (+ cid 1)
                     (if (null? (car ps))
                         out
                         (pkinds-of cid (by-name (extract (car (car ps)) 1) nil) out))))))
(define nth-placed (->* (by-clause int) (listof mx-infer-placed acyclic))
  (lambda (ps i) (if (= i 0) (car ps) (nth-placed (cdr ps) (- i 1)))))
;; Kept from before where a clause's template is not aligned, yet.
(define keep-failed (->* (by-clause pkinds pkinds) pkinds)
  (lambda (ps prev out)
    (cond ((null? prev) out)
          ((and (null? (nth-placed ps (extract (car prev) 1)))
                (null? (pkind-of (extract (car prev) 1) (extract (car prev) 2) out))
                (tagcase (extract (car prev) 3) (kd-ty (t) #t) (else y #f)))
           (keep-failed ps (cdr prev) (cons (car prev) out)))
          (else (keep-failed ps (cdr prev) out)))))
(define kinds-of-part (->* (int (listof (tupleof int symbol int int) acyclic) pkinds) kds)
  (lambda (p vs pk)
    (cond ((null? vs) nil)
          ((= p (extract (car vs) 3))
           (append (pkind-of (extract (car vs) 1) (extract (car vs) 2) pk)
                   (kinds-of-part p (cdr vs) pk)))
          (else (kinds-of-part p (cdr vs) pk)))))
(define absorbed-span (->* (int (listof (tupleof int int int) acyclic)) spans)
  (lambda (s xs)
    (cond ((null? xs) nil)
          ((= s (extract (car xs) 1)) (list (tuple (extract (car xs) 2) (extract (car xs) 3))))
          (else (absorbed-span s (cdr xs))))))
;; Each leaf's syntax type, of its pattern variables'; and the lists by
;; ellipsis whose element turns out to be of more than one.
(define part-kinds (->* ((listof (tupleof int symbol int int) acyclic) pkinds mx-infer-ints)
                        (tupleof kinds spans))
  (lambda (vs pk seen)
    (cond
      ((null? vs) (tuple (the kinds nil) (the spans nil)))
      ((or (mx-infer-int-in? (extract (car vs) 3) seen) (not (leaf? (extract (car vs) 3))))
       (part-kinds (cdr vs) pk seen))
      (else
       (let* ((p (extract (car vs) 3))
              (c (combine (kinds-of-part p (get found-vars) pk)))
              (rest (part-kinds (cdr vs) pk (cons p seen)))
              (b (boundary-of p))
              (span (if (null? b) nil (absorbed-span (car b) (get mx-infer-absorbed)))))
         (tuple (if (null? (extract c 1))
                    (extract rest 1)
                    (cons (tuple p (car (extract c 1))) (extract rest 1)))
                (if (and (extract c 2) (not (null? span)))
                    (cons (car span) (extract rest 2))
                    (extract rest 2))))))))
(define kind-of-part (->* (int kinds) kds)
  (lambda (p ks) (if (leaf? p) (kind-at p ks) (list (kd-ref p)))))
(define count-named (->* (symbol placements) int)
  (lambda (n pls)
    (cond ((null? pls) 0)
          ((symbol=? n (extract (car pls) 1)) (+ 1 (count-named n (cdr pls))))
          (else (count-named n (cdr pls))))))
(define many-in (->* (int placements placements kinds mx-infer-ints) mx-infer-ints)
  (lambda (cid pls all ks out)
    (if (null? pls)
        out
        (let* ((n (extract (car pls) 1)) (v (var-of cid n))
               (k (if (null? v) nil (kind-of-part (extract (car v) 1) ks)))
               (more (and (not (null? k)) (affine? (car k))
                          (or (> (count-named n all) 1) (extract (car pls) 4)
                              (> (extract (car pls) 5) (extract (car v) 2))))))
          (many-in cid (cdr pls) all ks
                   (if (and more (not (mx-infer-int-in? (extract (car v) 1) out)))
                       (cons (extract (car v) 1) out)
                       out))))))
;; `many`: placed more than once, in a part another macro's signature says
;; is, or under more ellipses than the pattern has it.
(define many-each (->* (by-clause int kinds mx-infer-ints) mx-infer-ints)
  (lambda (ps cid ks out)
    (if (null? ps)
        out
        (many-each (cdr ps) (+ cid 1) ks
                   (if (null? (car ps))
                       out
                       (let ((pls (extract (car (car ps)) 1))) (many-in cid pls pls ks out)))))))
(define mx-solve-name-seen? (->* (symbol mx-parser-types-names) bool)
  (lambda (n ns) (and (not (null? ns)) (or (symbol=? n (car ns)) (mx-solve-name-seen? n (cdr ns))))))
(define site-add (->* (int int rds site-list) site-list)
  (lambda (s cid rs xs)
    (cond ((null? xs) (list (tuple s (the site-rings (list (tuple cid rs))))))
          ((= s (extract (car xs) 1))
           (cons (tuple s (append (extract (car xs) 2) (the site-rings (list (tuple cid rs)))))
                 (cdr xs)))
          (else (cons (car xs) (site-add s cid rs (cdr xs)))))))
(define site-rings-each (->* (int (listof site acyclic) site-list) site-list)
  (lambda (cid at out)
    (if (null? at)
        out
        (let ((rs (mx-solve-rings-of (extract (car at) 3) (extract (car at) 2) cid)))
          (site-rings-each cid (cdr at)
                           (if (null? rs) out (site-add (extract (car at) 1) cid (car rs) out)))))))
(define sites-of-env (->* (int int levels site-list) site-list)
  (lambda (cid p env out)
    (let ((ls (lvs-of cid env)))
      (if (null? ls)
          out
          (let ((at (sites-of p (car ls))))
            (if (null? at)
                out
                (let ((out2 (if (or (null? (car at)) (not (= (extract (car (car at)) 1) p)))
                                (site-add p cid nil out)
                                out)))
                  (site-rings-each cid (car at) out2))))))))
(define sites-in (->* (int placements mx-parser-types-names kinds site-list) site-list)
  (lambda (cid pls seen ks out)
    (if (null? pls)
        out
        (let* ((n (extract (car pls) 1)) (p (mx-infer-var-part cid n))
               (k (if (null? p) nil (kind-of-part (car p) ks))))
          (if (or (null? p) (mx-solve-name-seen? n seen) (null? k) (not (scoped? (car k))))
              (sites-in cid (cdr pls) (if (null? p) seen (cons n seen)) ks out)
              (sites-in cid (cdr pls) (cons n seen) ks
                        (sites-of-env cid (car p) (extract (car pls) 3) out)))))))
(define sites-each (->* (by-clause int kinds site-list) site-list)
  (lambda (ps cid ks out)
    (if (null? ps)
        out
        (sites-each (cdr ps) (+ cid 1) ks
                    (if (null? (car ps))
                        out
                        (sites-in cid (extract (car (car ps)) 1) nil ks out))))))
(define site-into (->* ((tupleof int site-rings) site-list) site-list)
  (lambda (x out)
    (cond ((null? out) (list x))
          ((< (extract x 1) (extract (car out) 1)) (cons x out))
          (else (cons (car out) (site-into x (cdr out)))))))
(define sorted-sites (->* (site-list site-list) site-list)
  (lambda (xs out) (if (null? xs) out (sorted-sites (cdr xs) (site-into (car xs) out)))))
(define imports-each (->* (site-list) binds-at)
  (lambda (xs)
    (if (null? xs)
        nil
        (let* ((rs (merge-rings (extract (car xs) 2))) (rest (imports-each (cdr xs))))
          (if (null? rs) rest (cons (tuple (extract (car xs) 1) (rings-bind rs)) rest))))))
(define cage-parts (->* (int mx-parser-types-names) mx-infer-ints)
  (lambda (cid ns)
    (if (null? ns)
        nil
        (let ((p (mx-infer-var-part cid (car ns))))
          (append p (cage-parts cid (cdr ns)))))))
(define cage-lvs (->* (mx-infer-ints) lvs)
  (lambda (ps) (if (null? ps) nil (cons (iv-cage (car ps)) (cage-lvs (cdr ps))))))
(define root-rings (->* (int int mx-infer-ints by-clause int) site-rings)
  (lambda (root mi of ps cid)
    (cond
      ((null? ps) nil)
      ((or (null? (car ps)) (not (= (car of) mi)) (null? (extract (car (car ps)) 2)))
       (root-rings root mi (cdr of) (cdr ps) (+ cid 1)))
      (else
       (let* ((e (car (extract (car (car ps)) 2)))
              (ribs (if (null? (extract e 1))
                        nil
                        (let ((l (lvs-of cid (the levels (list (lv-rib (extract e 1)))))))
                          (if (and (not (null? l)) (not (null? (car l))))
                              (list (car (car l)))
                              nil))))
              (cages (cage-lvs (ints-sorted (cage-parts cid (extract e 2)) nil)))
              (rs (mx-solve-rings-of (append ribs cages) root cid))
              (rest (root-rings root mi (cdr of) (cdr ps) (+ cid 1))))
         (if (null? rs) rest (cons (tuple cid (car rs)) rest)))))))
;; What a macro that makes definitions defines of what a use gives.
(define root-exports (->* (mx-infer-ints int mx-infer-ints mx-infer-ints by-clause) binds-at)
  (lambda (rs mi makes of ps)
    (if (null? rs)
        nil
        (let* ((rest-now (if (= (nth-int makes mi) 1)
                             (let ((rings (merge-rings (root-rings (car rs) mi of ps 0))))
                               (if (null? rings) nil (list (tuple mi (rings-bind rings)))))
                             nil)))
          (append rest-now (root-exports (cdr rs) (+ mi 1) makes of ps))))))
(define empty-demand? (->* (demand) bool)
  (lambda (d) (tagcase d (dm-rib (es) (null? es)) (dm-cage (rs) (null? rs)))))
(define demand-at (->* (int got-demands) (listof demand acyclic))
  (lambda (p xs)
    (cond ((null? xs) nil)
          ((= p (extract (car xs) 1)) (list (extract (car xs) 2)))
          (else (demand-at p (cdr xs))))))
(define demand-set (->* (int demand got-demands) got-demands)
  (lambda (p d xs)
    (cond ((null? xs) (list (tuple p d)))
          ((= p (extract (car xs) 1)) (cons (tuple p d) (cdr xs)))
          (else (cons (car xs) (demand-set p d (cdr xs)))))))
(define consolidated (->* (got-demands got-demands) got-demands)
  (lambda (ds got)
    (if (null? ds)
        got
        (let* ((p (extract (car ds) 1)) (d (extract (car ds) 2)) (old (demand-at p got)))
          (consolidated (cdr ds)
                        (if (or (null? old) (and (empty-demand? (car old)) (not (empty-demand? d))))
                            (demand-set p d got)
                            got))))))
(define first-demand (->* (mx-infer-ints got-demands) (listof demand acyclic))
  (lambda (ms got)
    (cond ((null? ms) nil)
          ((not (null? (demand-at (car ms) got))) (demand-at (car ms) got))
          (else (first-demand (cdr ms) got)))))
(define fill-members (->* (mx-infer-ints demand got-demands) got-demands)
  (lambda (ms d got)
    (cond ((null? ms) got)
          ((not (null? (demand-at (car ms) got))) (fill-members (cdr ms) d got))
          (else (fill-members (cdr ms) d
                              (demand-set (car ms)
                                          (tagcase d (dm-rib (es) (dm-rib nil))
                                                   (dm-cage (rs) (dm-cage nil)))
                                          got))))))
(define unions-filled (->* (int got-demands) got-demands)
  (lambda (p got)
    (if (= p (get nparts))
        got
        (unions-filled (+ p 1)
                       (tagcase (mx-infer-shape-of p)
                         (sh-union (ms)
                           (let ((d (first-demand ms got)))
                             (if (null? d) got (fill-members ms (car d) got))))
                         (else y got))))))
(define demand-binds (->* (got-demands) binds-at)
  (lambda (xs)
    (if (null? xs)
        nil
        (cons (tuple (extract (car xs) 1)
                     (tagcase (extract (car xs) 2)
                       (dm-rib (es) (bind-rib (mx-solve-entries-of es)))
                       (dm-cage (rs) (bind-cage (rings-each rs)))))
              (demand-binds (cdr xs))))))
(define exports-of-demands (->* () binds-at)
  (lambda ()
    (let ((got (unions-filled 0 (consolidated (reverse (get demands)) nil))))
      (demand-binds got))))
(define mx-solve-solve (->* (by-clause sol mx-infer-ints mx-infer-ints) sol)
  (lambda (ps prev makes of)
    (let* ((pk (keep-failed ps (extract prev 2) (pkinds-each ps 0 nil)))
           (ks (part-kinds (get found-vars) pk nil))
           (kinds (extract ks 1)) (sticky (extract ks 2))
           (many (many-each ps 0 kinds nil)))
      (begin
        (set demands nil)
        (let* ((sites (sites-each ps 0 kinds nil))
               (imports (imports-each (sorted-sites sites nil)))
               (roots-ex (root-exports (get mx-infer-roots) 0 makes of ps))
               (exports (exports-of-demands)))
          (tuple kinds pk many imports exports roots-ex sticky))))))
(define mx-solve-head-is? (->* (syn string) bool)
  (lambda (s h)
    (let ((ps (proper-items s)))
      (and (pair? ps) (syn-symbol? (car ps)) (string=? (syn-name (car ps)) h)))))
;; The signature of operands `s` making definitions, as written: `(macro σ (defn
;; (ℓ δ) …))` of `(export σ (rho (ℓ δ) …))`, else `(-> s defn)` (`macros.rs`'s
;; `defn_arrow`).
(define defn-arrow (->* (syn int int) syn)
  (lambda (s a b)
    (let* ((ps (proper-items s))
           (rho (if (and (= (len ps) 3) (mx-solve-head-is? s "export") (mx-solve-head-is? (nth ps 2) "rho"))
                    (proper-items (nth ps 2))
                    nil)))
      (if (null? rho)
          (mk-list (list (mk-symbol "macro/addressed" a b) s (mk-symbol "defn" a b)) a b)
          (let ((made (mk-list (cons (mk-symbol "defn" a b) (cdr rho)) a b)))
            (mk-list (list (mk-symbol "macro/addressed" a b) (nth ps 1) made) a b))))))
(define made-sigs (->* (syns-a mx-infer-ints unsigneds) syns-a)
  (lambda (sigs makes us)
    (if (null? sigs)
        nil
        (let* ((u (car us)) (a (syn-start (extract u 4))) (b (syn-end (extract u 4))))
          (cons (cond ((= (car makes) 1) (defn-arrow (car sigs) a b))
                      (else (mk-list (list (mk-symbol "macro/addressed" a b) (car sigs)
                                           (mk-symbol (if (= (car makes) 0) "expr" "desc") a b))
                                     a b)))
                (made-sigs (cdr sigs) (cdr makes) (cdr us)))))))
(define mx-solve-clause-usage string "a clause is `((_ . pattern) template)`")
(define rules-in (->* (fails syns-a int) rules)
  (lambda (mx-fail cs mi)
    (if (null? cs)
        nil
        (let ((two (proper-items (car cs))))
          (if (not (= (len two) 2))
              (begin (fail-at mx-fail mx-solve-clause-usage (car cs)) nil)
              (let ((whole (split (car two))))
                (if (null? whole)
                    (begin (fail-at mx-fail mx-solve-clause-usage (car two)) nil)
                    (cons (tuple mi (nth whole 1) (nth two 1)) (rules-in mx-fail (cdr cs) mi)))))))))
(define rules-of (->* (fails unsigneds int) rules)
  (lambda (mx-fail us mi)
    (if (null? us)
        nil
        (append (rules-in mx-fail (extract (car us) 3) mi) (rules-of mx-fail (cdr us) (+ mi 1))))))
(define typed-pkinds (->* (pkinds) pkinds)
  (lambda (pk)
    (cond ((null? pk) nil)
          ((tagcase (extract (car pk) 3) (kd-ty (t) #t) (else y #f))
           (cons (car pk) (typed-pkinds (cdr pk))))
          (else (typed-pkinds (cdr pk))))))
(define fresh-sol (->* (pkinds) sol)
  (lambda (pk)
    (let ((e (sol-empty)))
      (tuple (extract e 1) (typed-pkinds pk) (extract e 3) (extract e 4) (extract e 5)
             (extract e 6) (extract e 7)))))
(define nth-u (->* (unsigneds int) unsigned)
  (lambda (us i) (if (= i 0) (car us) (nth-u (cdr us) (- i 1)))))
(define lits-each (->* (rules unsigneds) (listof mx-parser-types-names acyclic))
  (lambda (rs us)
    (if (null? rs) nil (cons (extract (nth-u us (extract (car rs) 1)) 2) (lits-each (cdr rs) us)))))
(define rule-macros (->* (rules) mx-infer-ints)
  (lambda (rs) (if (null? rs) nil (cons (extract (car rs) 1) (rule-macros (cdr rs))))))
(define items-of (->* (rules int int) mx-infer-items)
  (lambda (rs mi cid)
    (cond ((null? rs) nil)
          ((= (extract (car rs) 1) mi)
           (cons (tuple cid (extract (car rs) 2) 0) (items-of (cdr rs) mi (+ cid 1))))
          (else (items-of (cdr rs) mi (+ cid 1))))))
(define roots-each (->* (fails unsigneds rules int mx-infer-cx) mx-infer-ints)
  (lambda (mx-fail us rs mi c)
    (if (null? us)
        nil
        (let ((r (merge mx-fail (items-of rs mi 0) nil -1 c)))
          (cons r (roots-each mx-fail (cdr us) rs (+ mi 1) c))))))
(define show-each-sig (->* (stys unsigneds) syns-a)
  (lambda (tys us)
    (if (null? tys)
        nil
        (let ((u (extract (car us) 4)))
          (begin
            (set shown nil)
            (cons (show-sty (car tys) (syn-start u) (syn-end u))
                  (show-each-sig (cdr tys) (cdr us))))))))
(define texts (->* (syns-a) mx-solve-strings)
  (lambda (xs) (if (null? xs) nil (cons (show-syn (car xs)) (texts (cdr xs))))))
(define mx-solve-strings=? (->* (mx-solve-strings mx-solve-strings) bool)
  (lambda (a b)
    (if (null? a)
        (null? b)
        (and (not (null? b)) (string=? (car a) (car b)) (mx-solve-strings=? (cdr a) (cdr b))))))
(define provisional (->* (unsigneds stys mx-infer-ints) macro-env)
  (lambda (us tys makes)
    (if (null? us)
        nil
        (cons (tuple (extract (car us) 1) (tuple (car tys) nil nil (car makes)))
              (provisional (cdr us) (cdr tys) (cdr makes))))))
(define placed-each (->* (rules mx-infer-ints int sol) by-clause)
  (lambda (rs makes cid s)
    (if (null? rs)
        nil
        (cons (placements-of cid (extract (car rs) 3) (nth-int makes (extract (car rs) 1)) s)
              (placed-each (cdr rs) makes (+ cid 1) s)))))
(define infer-loop
  (->* (fails unsigneds rules mx-infer-ints sol spans string (listof (tupleof mx-solve-strings syns-a) acyclic) int
              macro-env)
       syns-a)
  (lambda (mx-fail us rs makes s0 sticky shape-was last n saved)
    (if (= n 0)
        (if (null? last) nil (extract (car last) 2))
        (begin
          (parts-reset!)
          (let* ((c (tuple (lits-each rs us) (extract s0 2) sticky))
                 (made-roots (begin (set mx-infer-roots (roots-each mx-fail us rs 0 c)) #u))
                 (mx-infer-shape (fingerprint))
                 (s (if (string=? mx-infer-shape shape-was) s0 (fresh-sol (extract s0 2))))
                 (tys (build s n))
                 (shown (show-each-sig tys us))
                 (mx-solve-strings (texts shown)))
            (if (and (not (null? last)) (mx-solve-strings=? (extract (car last) 1) mx-solve-strings))
                shown
                (begin
                  (set defined (append (provisional us tys makes) saved))
                  (let* ((ps (placed-each rs makes 0 s))
                         (next (mx-solve-solve ps s makes (rule-macros rs))))
                    (infer-loop mx-fail us rs makes next (append (extract next 7) sticky) mx-infer-shape
                                (list (tuple mx-solve-strings shown)) (- n 1) saved)))))))))
;;; ------------------------------------------------------------ inferring

;; The signatures of macros `us`, defined together, inferred: each as
;; written, `(macro σ defn)` and `(macro σ desc)` for what makes definitions or a
;; description (`infer.rs`'s `infer`).
(define infer (->* (fails unsigneds) syns-a)
  (lambda (mx-fail us)
    (let* ((rs (rules-of mx-fail us 0)) (makes (makes-all mx-fail us rs)) (saved (get defined))
           (sigs (infer-loop mx-fail us rs makes (sol-empty) nil "" nil 16 saved)))
      (begin
        (set defined saved)
        (made-sigs sigs makes us)))))
(define caught-sigs (ref syns-a mxp) (new nil))
;; `infer` of `us`, none if it fails: the macros defined as they were.
(define infer-caught (->* (unsigneds) syns-a)
  (lambda (us)
    (let* ((saved (get defined))
           (r (prompt parse-tag (begin (set caught-sigs (infer pfail-at us)) (p-ok nil))
                      (lambda (r) r))))
      (begin (set defined saved)
             (tagcase r (p-err (m x y) (the syns-a nil)) (p-ok (ts) (get caught-sigs)))))))
;; Each of clauses `cs`'s signature, inferred alone; none if one is not.
(define clause-sigs (->* (unsigned syns-a) syns-a)
  (lambda (u cs)
    (if (null? cs)
        nil
        (let ((one (infer-caught (list (tuple (extract u 1) (extract u 2) (list (car cs))
                                              (extract u 4))))))
          (if (null? one)
              nil
              (let ((rest (clause-sigs u (cdr cs))))
                (if (and (null? rest) (not (null? (cdr cs)))) nil (cons (car one) rest))))))))
;; What a signature says its macro makes: `defn`, `type`, or "" (an
;; expression); and its operands' part.
(define arrow-parts (->* (syn) syns-a)
  (lambda (s)
    (let ((ps (proper-items s)))
      (if (and (= (len ps) 3) (syn-symbol? (car ps))
               (string=? (syn-name (car ps)) "macro/addressed"))
          ps
          nil))))
(define mx-solve-made-of (->* (syn) string)
  (lambda (s) (let ((ps (arrow-parts s))) (if (null? ps) "" (syn-name (nth ps 2))))))
(define same-made? (->* (syns-a string) bool)
  (lambda (ms k) (or (null? ms) (and (string=? (mx-solve-made-of (car ms)) k) (same-made? (cdr ms) k)))))
(define sig-bodies (->* (syns-a) syns-a)
  (lambda (ms)
    (if (null? ms)
        nil
        (cons (let ((ps (arrow-parts (car ms)))) (if (null? ps) (car ms) (nth ps 1)))
              (sig-bodies (cdr ms))))))
;; A signature of unsigned macro `u`'s clauses' shapes whole, `(union σ₁
;; …)`, each clause's inferred alone (`macros.rs`'s `by_clauses`): for when
;; the one inferred of them together, position by position, does not fit
;; them. None for one clause, clauses that make different things, or one
;; not inferred alone.
(define by-clauses (->* (unsigned) syns-a)
  (lambda (u)
    (let ((cs (extract u 3)) (a (syn-start (extract u 4))) (b (syn-end (extract u 4))))
      (if (or (null? cs) (null? (cdr cs)))
          nil
          (let ((ms (clause-sigs u cs)))
            (if (or (null? ms) (not (same-made? (cdr ms) (mx-solve-made-of (car ms)))))
                nil
                (let ((union (mk-list (cons (mk-symbol "oneof" a b) (sig-bodies ms)) a b)))
                  ;; What it makes a word: `expr`, `defn` or `desc`.
                  (if (string=? (mx-solve-made-of (car ms)) "")
                      nil
                      (list (mk-list (list (mk-symbol "macro/addressed" a b) union
                                           (mk-symbol (mx-solve-made-of (car ms)) a b))
                                     a b))))))))))

;;; ============================================================ macros-fresh.fx (whole)

;; The form's key, the names it made, the binders the resolution renamed
;; (`%x~K.N`, `macros-shapes.fx`), its expansions, each by its number with
;; its tag (`expansion-begun!`), and each key's hash with how many forms had
;; it; the prefix of a module file's forms' keys, "" for a program's.
(define form-key (ref string mxp) (new ""))
(define fresh-count (ref int mxp) (new 0))
(define captors-made (ref int mxp) (new 0))
(define mx-fresh-expanded (ref int mxp) (new 0))
(define form-tags (ref tags mxp) (new nil))
(define keys-seen (ref seen-keys mxp) (new nil))
(define module-prefix (ref string mxp) (new ""))
;; A hash of `s`'s atoms and shape, as the Rust one's (`macros.rs`'s
;; `syn_hash`): a number by its value (a real by its text), a character by
;; its code, so `(check 1)` and `(check 2)` have keys apart (Kayak's
;; INCR_IMPED.md §5).
(define hash-p int 2147483647)
(define mix (->* (int int) int) (lambda (h x) (remainder (+ (* h 65599) x) hash-p)))
(define text-hash (->* (string) int) (lambda (t) (remainder (string-hash t) hash-p)))
(define-rec
(syn-hash (->* (syn) int)
  (lambda (s)
    (tagcase s
      (atom (d a b x)
        (cond ((symbol? d) (mix 1 (text-hash (symbol->string d))))
              ((string? d) (mix 2 (text-hash d)))
              ((int? d) (mix 7 (modulo d hash-p)))
              ((f64? d) (mix 8 (text-hash (f64->string d))))
              ((char? d) (mix 9 (char->integer d)))
              ((bool? d) (mix 10 (if d 1 0)))
              ((mx-keyword? d) (mix 11 (text-hash (symbol->string (mx-keyword->symbol d)))))
              (else 3)))
      (mx-eager-reader-types-lst (xs d a b x) (syns-hash 4 xs))
      (dotted (xs t d a b x) (mix (syns-hash 5 xs) (syn-hash t)))
      (vec (xs d a b x) (syns-hash 6 xs)))))
(syns-hash (->* (int syns-a) int)
  (lambda (h xs) (if (null? xs) h (syns-hash (mix h (syn-hash (car xs))) (cdr xs))))))
(define seen-count (->* (int seen-keys) int)
  (lambda (h ks)
    (cond ((null? ks) 0)
          ((= h (extract (car ks) 1)) (extract (car ks) 2))
          (else (seen-count h (cdr ks))))))
;; Form `s` begun: its key, and its names counted from none.
(define form-begin! (->* (syn) unit)
  (lambda (s)
    (let* ((h (syn-hash s)) (n (seen-count h (get keys-seen))))
      (begin (set keys-seen (cons (tuple h (+ n 1)) (get keys-seen)))
             (set form-key (string-append (get module-prefix) (string-append (int->string h)
                                          (if (= n 0) "" (string-append "-" (int->string n))))))
             (set fresh-count 0)
             (set captors-made 0)
             (set mx-fresh-expanded 0)
             (set form-tags nil)))))
;; The next name's tag, `K.N`.
(define fresh-tag (->* () string)
  (lambda ()
    (begin (set fresh-count (+ (get fresh-count) 1))
           (string-append (get form-key) (string-append "." (int->string (get fresh-count)))))))
;; The next renamed binder's tag, counted apart.
(define captor-tag (->* () string)
  (lambda ()
    (begin (set captors-made (+ (get captors-made) 1))
           (string-append (get form-key) (string-append "." (int->string (get captors-made)))))))
;; Expansion `made` begun: its tag, `K.N`, counted apart from the names'
;; (`macros.rs`'s `expansion_begun`), so that its aliases, `%~K.N:x`, are of
;; the form and the macros it uses alone.
(define expansion-begun! (->* (int) string)
  (lambda (made)
    (begin (set mx-fresh-expanded (+ (get mx-fresh-expanded) 1))
           (let ((t (string-append (get form-key) (string-append "." (int->string (get mx-fresh-expanded))))))
             (begin (set form-tags (cons (tuple made t) (get form-tags))) t)))))
(define tag-in (->* (int tags) string)
  (lambda (made ts)
    (cond ((null? ts) "0")
          ((= made (extract (car ts) 1)) (extract (car ts) 2))
          (else (tag-in made (cdr ts))))))
;; Expansion `made`'s tag, one of this form's; a template's check's (`made`
;; 0), `0`.
(define mx-fresh-tag-of (->* (int) string) (lambda (made) (tag-in made (get form-tags))))
;; Whether `c` may be in an expansion's tag: a form's key, `H`, `H-1`,
;; `m{F}mH` or `m{F}mmH`, and a count.
(define tag-char? (->* (char) bool)
  (lambda (c) (or (char-numeric? c) (char=? c #\.) (char=? c #\-) (char=? c #\m))))
(define tag-end (->* (string int) int)
  (lambda (s i)
    (if (and (< i (string-length s)) (tag-char? (string-ref s i))) (tag-end s (+ i 1)) i)))
;; Name `n` from `i`, each `%~:` at its start `%~TAG:`, after `out`; "" if it
;; has none (`any`). As `procedural::tagged_name`.
(define tagged-name (->* (string int string string bool) string)
  (lambda (n i tag out any)
    (let ((c (if (and (<= (+ i 2) (string-length n)) (string=? (substring n i (+ i 2)) "%~"))
                 (tag-end n (+ i 2))
                 -1)))
      (if (and (>= c 0) (< c (string-length n)) (char=? (string-ref n c) #\:))
          (let* ((t (substring n (+ i 2) c))
                 (seg (string-append "%~" (string-append (if (string=? t "") tag t) ":"))))
            (tagged-name n (+ c 1) tag (string-append out seg) (or any (string=? t ""))))
          (if any (string-append out (substring n i (string-length n))) "")))))
(define-rec
;; What an explicit-renaming transformer gave, `s`, each name its `rename`
;; made (`%~:x`, the prelude's `%er-rename`) an alias of expansion `made`,
;; `%~K.N:x`: the transformer is given nothing of the expansion that calls
;; it, so what it gives is the same whatever expansion calls it, and the
;; expander makes its aliases, as a syntactic closure's are made where it is
;; opened. A quoted datum's too; a vector's left as it is. As
;; `procedural::tagged`.
(tagged (->* (syn int) syn)
  (lambda (s made)
    (tagcase s
      (atom (d a b x)
        (if (symbol? d)
            (let ((n (tagged-name (symbol->string d) 0 (mx-fresh-tag-of made) "" #f)))
              (if (string=? n "") s (atom (string->symbol n) a b x)))
            s))
      (mx-eager-reader-types-lst (items d a b x)
        (let ((xs (tagged-each items made))) (mx-eager-reader-types-lst xs (syn-datums xs) a b x)))
      (dotted (items tail d a b x)
        (let ((xs (tagged-each items made)) (t (tagged tail made)))
          (dotted xs t (mx-datum-dotted (syn-datums xs) (syn->datum t)) a b x)))
      (vec (items d a b x) s))))
(tagged-each (->* (syns-a int) syns-a)
  (lambda (xs made) (if (null? xs) nil (cons (tagged (car xs) made) (tagged-each (cdr xs) made))))))
;; The prefix of the keys of the forms of the module file of key `key`
;; (`input:` or not): `m{F}m` for its own expansion, `m{F}mm` for the uses
;; an importer expands (`imported`), `F` a hash of the key as a string's
;; (`macros.rs`'s `file_prefix`): no two files' forms, nor a file's and an
;; importer's, have one key, nor so their expansions' tags, which their
;; definition environments are kept by.
(define file-prefix (->* (string bool) string)
  (lambda (key imported)
    (let* ((path (if (and (> (string-length key) 6) (string=? (substring key 0 6) "input:"))
                     (substring key 6 (string-length key))
                     key))
           (f (mix 2 (remainder (string-hash path) hash-p))))
      (string-append "m" (string-append (int->string f) (if imported "mm" "m"))))))
;; The prefix of the keys of the uses an importer of an instance, `lib`, of a
;; file expands, of the file's, `prefix`: `m{F}mm{G}m`, `G` a hash of `lib`
;; as a symbol's, so that two instances' are apart (`instance_prefix`).
(define instance-prefix (->* (string symbol) string)
  (lambda (prefix lib)
    (string-append prefix (string-append (int->string (mix 1 (text-hash (symbol->string lib)))) "m"))))
(define fresh-scope-begin! (->* (string) fresh-saved)
  (lambda (prefix)
    (let ((s (tuple (get form-key) (get fresh-count) (get captors-made) (get keys-seen)
                    (get module-prefix) (get mx-fresh-expanded) (get form-tags))))
      (begin (set keys-seen nil) (set module-prefix prefix) s))))
(define fresh-scope-end! (->* (fresh-saved) unit)
  (lambda (s)
    (begin (set form-key (extract s 1)) (set fresh-count (extract s 2))
           (set captors-made (extract s 3)) (set keys-seen (extract s 4))
           (set module-prefix (extract s 5)) (set mx-fresh-expanded (extract s 6))
           (set form-tags (extract s 7)))))
;; An importer's expansions of a file's uses begun (`macros-imports.fx`'s
;; `define-file`): their keys of `prefix`, counted apart; the importer's
;; count and prefix given back, for `file-keys-end!`.
(define file-keys-begin! (->* (string) (tupleof seen-keys string))
  (lambda (prefix)
    (let ((s (tuple (get keys-seen) (get module-prefix))))
      (begin (set keys-seen nil) (set module-prefix prefix) s))))
(define file-keys-end! (->* ((tupleof seen-keys string)) unit)
  (lambda (s) (begin (set keys-seen (extract s 1)) (set module-prefix (extract s 2)))))
;; One use of the file's expanded as a form of its own: the form's state
;; before, for `use-end!`, which puts it back but for the keys counted.
(define use-begin! (->* () fresh-saved)
  (lambda ()
    (tuple (get form-key) (get fresh-count) (get captors-made) (get keys-seen)
           (get module-prefix) (get mx-fresh-expanded) (get form-tags))))
(define use-end! (->* (fresh-saved) unit)
  (lambda (s)
    (begin (set form-key (extract s 1)) (set fresh-count (extract s 2))
           (set captors-made (extract s 3)) (set mx-fresh-expanded (extract s 6))
           (set form-tags (extract s 7)))))
;; The state a session keeps after a form, and puts back before the next.
(define seen-now (->* () seen-keys) (lambda () (get keys-seen)))
(define seen! (->* (seen-keys) unit) (lambda (ks) (set keys-seen ks)))

;;; ============================================================ macros-contracts.fx (whole)

(define leaf-sty? (->* (sty) bool)
  (lambda (t)
    (tagcase t
      (s-expr () #t) (s-lambda () #t) (s-bvar () #t) (s-dvar () #t) (s-id () #t) (s-data () #t)
      (s-type () #t) (s-label () #t) (s-kind () #t) (s-defn () #t) (s-item () #t)
      (else y #f))))
;; The places of the nodes from `i` to `n` at a leaf of the signature.
(define leaves-of (->* (fails int int mx-contracts-places) mx-contracts-places)
  (lambda (mx-fail i n out)
    (if (= i n)
        out
        (begin
          (if (null? (node-hole i))
              #u
              (fail-at mx-fail "a procedural macro's contract of typed parts: not yet" (node-syn i)))
          (leaves-of mx-fail (+ i 1) n
                     (if (leaf-sty? (node-core i))
                         (cons (tuple (syn-start (node-syn i)) (syn-end (node-syn i))) out)
                         out))))))
(define pseudo-parts (ref syns-a mxp) (new nil))
(define place-in? (->* (int int mx-contracts-places) bool)
  (lambda (a b ps)
    (and (not (null? ps))
         (or (and (= a (extract (car ps) 1)) (= b (extract (car ps) 2)))
             (place-in? a b (cdr ps))))))
(define-rec
;; `s`, each part at a place in `leaves` the pattern variable `%part:k`,
;; that part `pseudo-parts`' `k`th.
(pseudo-pattern (->* (syn mx-contracts-places) syn)
  (lambda (s leaves)
    (if (place-in? (syn-start s) (syn-end s) leaves)
        (let ((k (len (get pseudo-parts))))
          (begin (set pseudo-parts (cons s (get pseudo-parts)))
                 (atom (string->symbol (string-append "%part:" (int->string k)))
                       (syn-start s) (syn-end s) (syn-expansion s))))
        (tagcase s
          (mx-eager-reader-types-lst (items d a b x)
            (let ((xs (pseudo-each items leaves))) (mx-eager-reader-types-lst xs (syn-datums xs) a b x)))
          (dotted (items tail d a b x)
            (let ((xs (pseudo-each items leaves)) (tl (pseudo-pattern tail leaves)))
              (dotted xs tl (mx-datum-dotted (syn-datums xs) (syn->datum tl)) a b x)))
          (else y s)))))
(pseudo-each (->* (syns-a mx-contracts-places) syns-a)
  (lambda (xs leaves)
    (if (null? xs)
        nil
        (let ((x (pseudo-pattern (car xs) leaves))) (cons x (pseudo-each (cdr xs) leaves)))))))
;; Part `k` of a pattern variable `%part:k`; -1 for another name.
(define part-number (->* (symbol) int)
  (lambda (n)
    (let ((s (symbol->string n)))
      (if (and (> (string-length s) 6) (string=? (substring s 0 6) "%part:"))
          (mx-parse-nat (substring s 6 (string-length s)) 10)
          -1))))
(define atoms-binders (->* (fails atms syn (listof symbol acyclic)) (listof symbol acyclic))
  (lambda (mx-fail as part out)
    (if (null? as)
        out
        (tagcase (car as)
          (atm-var (b d g) (atoms-binders mx-fail (cdr as) part (cons b out)))
          (else y (begin (fail-at mx-fail (string-append "a part in the scope of the binders another "
                                                      (string-append "part exports: not yet in a procedural "
                                                      "macro's contract"))
                                  part)
                         out))))))
(define scope-binders (->* (fails levels syn (listof symbol acyclic)) (listof symbol acyclic))
  (lambda (mx-fail ls part out)
    (if (null? ls)
        (the (listof symbol acyclic) (reverse out))
        (tagcase (car ls)
          (lv-rib (as) (scope-binders mx-fail (cdr ls) part (atoms-binders mx-fail as part out)))
          (lv-earlier (as) (scope-binders mx-fail (cdr ls) part (atoms-binders mx-fail as part out)))
          (else y (begin (fail-at mx-fail (string-append "a part in the scope of a ribcage or a "
                                                      (string-append "module: not yet in a procedural macro's "
                                                      "contract"))
                                  part)
                         out))))))
(define part-vars (->* (fails mx-nodes-found syns-a (listof part-var acyclic)) (listof part-var acyclic))
  (lambda (mx-fail fs parts out)
    (if (null? fs)
        out
        (let* ((at (extract (car fs) 1))
               (v (tagcase (node-kids at)
                    (k-pvar (n)
                      (let ((k (part-number n)))
                        (if (< k 0)
                            nil
                            (list (tuple k (tagcase (node-core at) (s-bvar () #t) (s-dvar () #t)
                                             (else y #f))
                                         (scope-binders mx-fail (env-levels (extract (car fs) 2))
                                                        (nth parts k) nil)
                                         (node-core at))))))
                    (else y nil))))
          (part-vars mx-fail (cdr fs) parts (if (null? v) out (cons (car v) out)))))))
;; Each binder: its part's number, and its name.
(define binder-parts
  (->* ((listof part-var acyclic) syns-a mx-contracts-numbered)
       mx-contracts-numbered)
  (lambda (vs parts out)
    (cond ((null? vs) out)
          ((and (extract (car vs) 2) (syn-symbol? (nth parts (extract (car vs) 1))))
           (binder-parts (cdr vs) parts
                         (cons (tuple (extract (car vs) 1)
                                      (syn-symbol (nth parts (extract (car vs) 1))))
                               out)))
          (else (binder-parts (cdr vs) parts out)))))
;; Each binder's fresh name, by its part, as `Macros::contract_renames` makes it.
(define fresh-names (->* (mx-contracts-numbered) mx-contracts-numbered)
  (lambda (bs)
    (if (null? bs)
        nil
        (cons (tuple (extract (car bs) 1)
                     (string->symbol (string-append "%contract:" (string-append (int->string (extract (car bs) 1))
                                                    (string-append ":" (symbol->string (extract (car bs) 2)))))))
              (fresh-names (cdr bs))))))
(define fresh-originals (->* (mx-contracts-numbered mx-contracts-numbered) mx-contracts-renames)
  (lambda (bs fs)
    (if (null? bs)
        nil
        (cons (tuple (extract (car fs) 2) (extract (car bs) 2))
              (fresh-originals (cdr bs) (cdr fs))))))
(define part-name (->* (int mx-contracts-numbered) (listof symbol acyclic))
  (lambda (k bs)
    (cond ((null? bs) nil) ((= k (extract (car bs) 1)) (list (extract (car bs) 2)))
          (else (part-name k (cdr bs))))))
(define mx-contracts-var-part (->* (symbol) int) (lambda (b) (part-number b)))
(define renamed-to (->* (symbol mx-contracts-renames) (listof symbol acyclic))
  (lambda (n rs)
    (cond ((null? rs) nil) ((symbol=? n (extract (car rs) 1)) (list (extract (car rs) 2)))
          (else (renamed-to n (cdr rs))))))
(define scope-renames
  (->* ((listof symbol acyclic) mx-contracts-numbered
        mx-contracts-numbered mx-contracts-renames)
       mx-contracts-renames)
  (lambda (in-scope bs fs out)
    (if (null? in-scope)
        out
        (let* ((k (mx-contracts-var-part (car in-scope))) (x (part-name k bs)) (to (part-name k fs)))
          (scope-renames (cdr in-scope) bs fs
                         (if (or (null? x) (not (null? (renamed-to (car x) out))))
                             out
                             (cons (tuple (car x) (car to)) out)))))))
;; Each part a binder's scope reaches, by its place, and what is renamed in it.
(define part-renames
  (->* ((listof part-var acyclic) syns-a mx-contracts-numbered
        mx-contracts-numbered renamings)
       renamings)
  (lambda (vs parts bs fs out)
    (if (null? vs)
        out
        (let* ((k (extract (car vs) 1))
               (own (if (extract (car vs) 2)
                        (the mx-contracts-renames (list (tuple (car (part-name k bs)) (car (part-name k fs)))))
                        nil))
               (rs (scope-renames (extract (car vs) 3) bs fs own))
               (p (nth parts k)))
          (part-renames (cdr vs) parts bs fs
                        (if (null? rs)
                            out
                            (cons (tuple (syn-start p) (syn-end p) rs (extract (car vs) 4))
                                  out)))))))
;; The parts of use operands `operands`, aligned with contract `sig` (just
;; now: the nodes are the use's), that a binder's scope reaches: each by its
;; place, with what is renamed in it, its binders' names each to a fresh
;; one, the innermost first; and each fresh name with the name it stands
;; for (`Macros::contract_renames`). Found as a pattern's are: each of the
;; use's parts at a leaf of the signature made a pattern variable, the
;; pattern aligned and walked.
(define contract-renames
  (->* (fails sty syn) (tupleof renamings mx-contracts-renames))
  (lambda (mx-fail sig operands)
    (let* ((leaves (leaves-of mx-fail 0 (get nnodes) nil))
           (made (begin (set pseudo-parts nil) (pseudo-pattern operands leaves)))
           (parts (the syns-a (reverse (get pseudo-parts))))
           (root (begin (nodes-reset!) (align mx-fail 0 (the pvars nil) sig made -1 nil)))
           (mx-nodes-found (mx-scopes-walk mx-fail root env-empty #f 0))
           (vars (part-vars mx-fail mx-nodes-found parts nil))
           (binders (binder-parts vars parts nil))
           (fresh (fresh-names binders)))
      (tuple (part-renames vars parts binders fresh nil)
             (the mx-contracts-renames (fresh-originals binders fresh))))))
(define-rec
(syn-renamed (->* (syn mx-contracts-renames) syn)
  (lambda (s rs)
    (tagcase s
      (atom (d a b x)
        (if (syn-symbol? s)
            (let ((to (renamed-to (syn-symbol s) rs))) (if (null? to) s (atom (car to) a b x)))
            s))
      (mx-eager-reader-types-lst (items d a b x) (let ((xs (renamed-each items rs))) (mx-eager-reader-types-lst xs (syn-datums xs) a b x)))
      (dotted (items tail d a b x)
        (let ((xs (renamed-each items rs)) (tl (syn-renamed tail rs)))
          (dotted xs tl (mx-datum-dotted (syn-datums xs) (syn->datum tl)) a b x)))
      (vec (items d a b x)
        (let ((xs (renamed-each items rs))) (vec xs (datum-list->vector (syn-datums xs)) a b x))))))
(renamed-each (->* (syns-a mx-contracts-renames) syns-a)
  (lambda (xs rs) (if (null? xs) nil (cons (syn-renamed (car xs) rs) (renamed-each (cdr xs) rs))))))
(define part-renames-at (->* (int int renamings) (listof (tupleof mx-contracts-renames sty) acyclic))
  (lambda (a b ps)
    (cond ((null? ps) nil)
          ((and (= a (extract (car ps) 1)) (= b (extract (car ps) 2)))
           (list (tuple (extract (car ps) 3) (extract (car ps) 4))))
          (else (part-renames-at a b (cdr ps))))))
(define bound-in (->* (atms symbol bool) int)
  (lambda (as n d)
    (if (null? as)
        -1
        (let ((b (tagcase (car as)
                   (atm-lit (i e)
                     (tagcase (node-kids i)
                       (k-binder (m e2) (if (and (symbol=? m n) (mx-eq? e2 d)) i -1))
                       (else y -1)))
                   (atm-var (m e g) (if (and (symbol=? m n) (mx-eq? e d)) -2 -1))
                   (else y -1))))
          (if (= b -1) (bound-in (cdr as) n d) b)))))
;; The binder node a reference to `n` (of a description's if `d`) is bound
;; by in `ls`, the innermost; -2 for a pattern's binder, -1 for none.
(define bound-by (->* (levels symbol bool) int)
  (lambda (ls n d)
    (if (null? ls)
        -1
        (let ((b (tagcase (car ls)
                   (lv-rib (as) (bound-in as n d))
                   (lv-earlier (as) (bound-in as n d))
                   (else y -1))))
          (if (= b -1) (bound-by (cdr ls) n d) b)))))
(define free-refs (->* (mx-nodes-found mx-contracts-renames placed-renames) placed-renames)
  (lambda (fs rs out)
    (if (null? fs)
        out
        (free-refs (cdr fs) rs
                   (tagcase (node-kids (extract (car fs) 1))
                     (k-ref (n d f)
                       (let ((to (renamed-to n rs)) (s (node-syn (extract (car fs) 1))))
                         (if (or (null? to)
                                 (not (= (bound-by (env-levels (extract (car fs) 2)) n d) -1)))
                             out
                             (cons (tuple (syn-start s) (syn-end s) (car to)) out))))
                     (else y out))))))
(define placed-at (->* (int int placed-renames) (listof symbol acyclic))
  (lambda (a b at)
    (cond ((null? at) nil)
          ((and (= a (extract (car at) 1)) (= b (extract (car at) 2))) (list (extract (car at) 3)))
          (else (placed-at a b (cdr at))))))
(define-rec
;; `s`, each symbol at a place `at` has made the name it says.
(syn-renamed-at (->* (syn placed-renames) syn)
  (lambda (s at)
    (tagcase s
      (atom (d a b x)
        (let ((to (placed-at a b at)))
          (if (and (syn-symbol? s) (not (null? to))) (atom (car to) a b x) s)))
      (mx-eager-reader-types-lst (items d a b x) (let ((xs (renamed-at-each items at))) (mx-eager-reader-types-lst xs (syn-datums xs) a b x)))
      (dotted (items tail d a b x)
        (let ((xs (renamed-at-each items at)) (tl (syn-renamed-at tail at)))
          (dotted xs tl (mx-datum-dotted (syn-datums xs) (syn->datum tl)) a b x)))
      (vec (items d a b x)
        (let ((xs (renamed-at-each items at)))
          (vec xs (datum-list->vector (syn-datums xs)) a b x))))))
(renamed-at-each (->* (syns-a placed-renames) syns-a)
  (lambda (xs at)
    (if (null? xs) nil (cons (syn-renamed-at (car xs) at) (renamed-at-each (cdr xs) at))))))
;; Part `part`, of syntax type `core`, each free reference in it to a name
;; `rs` names renamed, as its own binding structure finds them (aligned as
;; a program's form is, its macros' uses by their signatures); its binders,
;; and what it quotes, as they are (`Macros::references_renamed`). By
;; place: two references at one place are one template's part, alike.
(define references-renamed (->* (fails syn sty mx-contracts-renames) syn)
  (lambda (mx-fail part core rs)
    (let* ((mx-nodes-found (extract (program-scoped mx-fail core part #f nil) 2))
           (at (free-refs mx-nodes-found rs nil)))
      (syn-renamed-at part at))))
(define-rec
;; Use `s`, each part `ps` has by its place renamed as it says.
;; Use `s` α-renamed: each part `ps` has, by its place, a binder's its fresh
;; name, any other's references to the binders in whose scope it is, as
;; the part's own binding structure finds them (`references-renamed`).
(renamed-use (->* (fails syn renamings) syn)
  (lambda (mx-fail s ps)
    (let ((r (part-renames-at (syn-start s) (syn-end s) ps)))
      (if (not (null? r))
          (tagcase (extract (car r) 2)
            (s-bvar () (syn-renamed s (extract (car r) 1)))
            (s-dvar () (syn-renamed s (extract (car r) 1)))
            (else y (references-renamed mx-fail s (extract (car r) 2) (extract (car r) 1))))
          (tagcase s
            (mx-eager-reader-types-lst (items d a b x)
              (let ((xs (renamed-use-each mx-fail items ps))) (mx-eager-reader-types-lst xs (syn-datums xs) a b x)))
            (dotted (items tail d a b x)
              (let ((xs (renamed-use-each mx-fail items ps)) (tl (renamed-use mx-fail tail ps)))
                (dotted xs tl (mx-datum-dotted (syn-datums xs) (syn->datum tl)) a b x)))
            (else y s))))))
(renamed-use-each (->* (fails syns-a renamings) syns-a)
  (lambda (mx-fail xs ps)
    (if (null? xs) nil (cons (renamed-use mx-fail (car xs) ps) (renamed-use-each mx-fail (cdr xs) ps))))))
(define env-at (->* (int mx-nodes-found) mx-env-indexed-env)
  (lambda (i fs)
    (cond ((null? fs) env-empty)
          ((= i (extract (car fs) 1)) (extract (car fs) 2))
          (else (env-at i (cdr fs))))))
(define anode-of (->* (int mx-nodes-found) anode)
  (lambda (i fs)
    (let ((none (the (listof symbol acyclic) nil)) (no (the (listof syn acyclic) nil))
          (bit (lambda ((d bool)) (if d 1 0))))
      (tagcase (node-kids i)
        (k-pair (x y) (tuple 0 (list x y) none no))
        (k-nil () (tuple 1 nil none no))
        (k-binder (m d) (tuple 2 (list (bit d)) none no))
        (k-ref (m d f)
          (let ((b (bound-by (env-levels (env-at i fs)) m d)))
            (if (= b -1) (tuple 4 (list (bit d)) (list m) no) (tuple 3 (list b) none no))))
        (k-desc (xs) (tuple 5 xs none no))
        (mx-nodes-k-seq (x) (tuple 6 (list x) none no))
        (k-form (x) (tuple 7 (list x) none no))
        (else y (tuple 8 nil none (list (node-syn i))))))))
(define anodes-from (->* (int int mx-nodes-found anodes) anodes)
  (lambda (i n fs out)
    (if (= i n)
        (the anodes (reverse out))
        (anodes-from (+ i 1) n fs (cons (anode-of i fs) out)))))
(define alpha-nodes (->* (fails syn sty) anodes)
  (lambda (mx-fail s made)
    (let ((mx-nodes-found (extract (program-scoped mx-fail made s #f nil) 2)))
      (anodes-from 0 (get nnodes) mx-nodes-found nil))))
(define ints-same? (->* ((listof int acyclic) (listof int acyclic)) bool)
  (lambda (xs ys)
    (cond ((null? xs) (null? ys)) ((null? ys) #f)
          (else (and (= (car xs) (car ys)) (ints-same? (cdr xs) (cdr ys)))))))
(define names-same? (->* ((listof symbol acyclic) (listof symbol acyclic)) bool)
  (lambda (xs ys)
    (cond ((null? xs) (null? ys)) ((null? ys) #f)
          (else (and (symbol=? (car xs) (car ys)) (names-same? (cdr xs) (cdr ys)))))))
(define-rec
;; Whether syntax `a` and `b` are the same, part by part, wherever each is.
(syn-same? (->* (syn syn) bool)
  (lambda (a b)
    (tagcase a
      (mx-eager-reader-types-lst (xs d c e x) (tagcase b (mx-eager-reader-types-lst (ys d2 c2 e2 x2) (all-same? xs ys)) (else y #f)))
      (dotted (xs t d c e x)
        (tagcase b
          (dotted (ys u d2 c2 e2 x2) (and (all-same? xs ys) (syn-same? t u)))
          (else y #f)))
      (atom (d c e x)
        (if (syn-symbol? a)
            (and (syn-symbol? b) (symbol=? (syn-symbol a) (syn-symbol b)))
            (mx-match-same-atom? a b)))
      (else y #f))))
(all-same? (->* (syns-a syns-a) bool)
  (lambda (xs ys)
    (if (null? xs)
        (null? ys)
        (and (not (null? ys)) (syn-same? (car xs) (car ys)) (all-same? (cdr xs) (cdr ys)))))))
(define anode-same? (->* (anode anode) bool)
  (lambda (a b)
    (and (= (extract a 1) (extract b 1)) (ints-same? (extract a 2) (extract b 2))
         (names-same? (extract a 3) (extract b 3))
         (or (null? (extract a 4))
             (and (not (null? (extract b 4)))
                  (syn-same? (car (extract a 4)) (car (extract b 4))))))))
(define alpha-same? (->* (anodes anodes) bool)
  (lambda (as bs)
    (cond ((null? as) (null? bs))
          ((null? bs) #f)
          (else (and (anode-same? (car as) (car bs)) (alpha-same? (cdr as) (cdr bs)))))))
(define mx-contracts-fresh-of (->* (symbol mx-contracts-renames int) (listof symbol acyclic))
  (lambda (n fresh made)
    (cond ((null? fresh) nil)
          ((or (symbol=? n (extract (car fresh) 1))
               (string=? (symbol->string n)
                         (string-append "%~" (string-append (mx-fresh-tag-of made) (string-append ":"
                                        (symbol->string (extract (car fresh) 1)))))))
           (list (extract (car fresh) 2)))
          (else (mx-contracts-fresh-of n (cdr fresh) made)))))
(define mx-contracts-escaped (->* (anodes mx-contracts-renames int) (listof symbol acyclic))
  (lambda (ns fresh made)
    (if (null? ns)
        nil
        (let ((x (if (and (= (extract (car ns) 1) 4) (not (null? (extract (car ns) 3))))
                     (mx-contracts-fresh-of (car (extract (car ns) 3)) fresh made)
                     nil)))
          (if (null? x) (mx-contracts-escaped (cdr ns) fresh made) x)))))
;; Output `out2`, the transformer's of the α-renamed use, held to output
;; `out`, its of the use: the same up to their binders' names; else refused
;; at use `s`, by `in-use`: a fresh name of `fresh`, or its alias of
;; expansion `made` (`%~K.N:x′`), free in `out2`, placed outside its binder's
;; scope; else a name read (`Macros::contract_expanded`).
(define contract-held (->* (fails syn syn syn mx-contracts-renames int sty) unit)
  (lambda (in-use s out out2 fresh made ty)
    (let* ((ours (lambda ((m string) (a int) (b int))
                   (in-use (string-append "its expansion: " m) (syn-start s) (syn-end s))))
           (a (alpha-nodes ours out ty)) (b (alpha-nodes ours out2 ty)))
      (if (alpha-same? a b)
          #u
          (let ((x (mx-contracts-escaped b fresh made)))
            (in-use (if (null? x)
                        "its expansion depends on how its use's bound names are spelled"
                        (string-append "its expansion places `" (string-append (symbol->string (car x))
                                       (string-append "` where its contract does not put it in `"
                                       (string-append (symbol->string (car x)) "`'s scope")))))
                    (syn-start s) (syn-end s)))))))
(define mx-contracts-named-in? (->* ((tupleof symbol bool) named-ns) bool)
  (lambda (x ys)
    (and (not (null? ys))
         (or (and (symbol=? (extract x 1) (extract (car ys) 1))
                  (mx-eq? (extract x 2) (extract (car ys) 2)))
             (mx-contracts-named-in? x (cdr ys))))))
(define own-name? (->* (symbol string) bool)
  (lambda (n own)
    (let ((s (symbol->string n)) (k (string-length own)))
      (and (> k 0) (>= (string-length s) k) (string=? (substring s 0 k) own)))))
;; The first of `xs` that `ys` has not, nor, if `own` is not empty, is
;; spelled from it; in a list of one.
(define named-not-in (->* (named-ns named-ns string) (listof symbol acyclic))
  (lambda (xs ys own)
    (cond ((null? xs) nil)
          ((or (mx-contracts-named-in? (car xs) ys) (own-name? (extract (car xs) 1) own))
           (named-not-in (cdr xs) ys own))
          (else (list (extract (car xs) 1))))))
(define said-not-yet (->* (fails syn) unit)
  (lambda (mx-fail at)
    (fail-at mx-fail "a contract whose uses define what a part of theirs defines: not yet" at)))
(define said-names (->* (fails atms syns-a syn) named-ns)
  (lambda (mx-fail as parts at)
    (if (null? as)
        nil
        (tagcase (car as)
          (atm-var (b d g)
            (let ((p (nth parts (part-number b))))
              (if (syn-symbol? p)
                  (cons (tuple (syn-symbol p) d) (said-names mx-fail (cdr as) parts at))
                  (begin (fail-at mx-fail "a binder that is not a name" p) nil))))
          (atm-rib-of (n) (begin (said-not-yet mx-fail at) nil))
          (atm-lit (i d) (said-names mx-fail (cdr as) parts at))))))
;; What contract `sig` says a use of operands `operands` defines, of the
;; use's names: the use aligned, then made a pattern, each leaf a part, whose
;; export is evaluated.
(define contract-said (->* (fails sty syn) named-ns)
  (lambda (mx-fail sig operands)
    (let* ((use (begin (nodes-reset!) (align mx-fail 2 (the pvars nil) sig operands -1 nil)))
           (leaves (leaves-of mx-fail 0 (get nnodes) nil))
           (made (begin (set pseudo-parts nil) (pseudo-pattern operands leaves)))
           (parts (the syns-a (reverse (get pseudo-parts))))
           (root (begin (nodes-reset!) (align mx-fail 0 (the pvars nil) sig made -1 nil)))
           (mx-nodes-found (mx-scopes-walk mx-fail root env-empty #f 0))
           (e (node-export root)))
      (if (null? e)
          nil
          (let ((x (mx-exports-exports-of (evaluate mx-fail (extract (car e) 1) (extract (car e) 2) made))))
            (if (null? (extract x 2))
                (said-names mx-fail (extract x 1) parts operands)
                (begin (said-not-yet mx-fail operands) nil)))))))
(define atm-defines (->* (atms named-ns) named-ns)
  (lambda (as out)
    (if (null? as)
        out
        (atm-defines (cdr as)
                     (tagcase (car as)
                       (atm-lit (b d)
                         (tagcase (node-kids b)
                           (k-binder (m e)
                             (if (mx-contracts-named-in? (tuple m d) out) out (cons (tuple m d) out)))
                           (else y out)))
                       (else y out))))))
(define level-defines (->* (levels named-ns) named-ns)
  (lambda (ls out)
    (if (null? ls)
        out
        (level-defines (cdr ls)
                       (tagcase (car ls) (lv-rib (as) (atm-defines as out)) (else y out))))))
(define defines-from (->* (fails int int named-ns) named-ns)
  (lambda (mx-fail i n out)
    (if (= i n)
        out
        (defines-from mx-fail (+ i 1) n
          (let ((k (mx-types-kind-of (node-core i))))
            (if (or (= k 14) (= k 11))
                (tagcase (node-kids i)
                  (k-form (x)
                    (let ((ex (node-export x)))
                      (if (null? ex)
                          out
                          (level-defines (evaluate mx-fail (extract (car ex) 1) (extract (car ex) 2)
                                                   (node-syn i))
                                         out))))
                  (else y out))
                out))))))
;; What definitions `out` define, aligned as a program's are: each binder a
;; definition's form exports, by its name.
(define expansion-defines (->* (fails syn) named-ns)
  (lambda (mx-fail out)
    (let* ((scoped (program-scoped mx-fail (s-defn) out #f nil))
           (ds (begin (set program-walk #t) (defines-from mx-fail 0 (get nnodes) nil))))
      (begin (set program-walk #f) ds))))
;; Definitions `out`, what a transformer made of use operands `operands` in
;; expansion `made`, held to contract `sig` (`Macros::defined_held`):
;; definitions, aligned as a program's are, that define of the use's names
;; exactly those the contract says its uses define; any other name they
;; define their own, an alias of the expansion (`%~K.N:x`). Refused at `out`.
(define defined-held (->* (fails sty syn syn int) unit)
  (lambda (mx-fail sig operands out made)
    (let* ((said (contract-said mx-fail sig operands))
           (ours (lambda ((m string) (a int) (b int))
                   (mx-fail (string-append "its expansion: " m) a b)))
           (defines (expansion-defines ours out))
           (own (string-append "%~" (string-append (mx-fresh-tag-of made) ":")))
           (extra (named-not-in defines said own))
           (missing (named-not-in said defines "")))
      (cond ((not (null? extra))
             (fail-at mx-fail (string-append "its expansion defines `" (string-append (symbol->string (car extra))
                                          (string-append "`, which its contract does not say a use defines, "
                                          "nor is it its own (an alias, as `rename` makes one)")))
                      out))
            ((not (null? missing))
             (fail-at mx-fail (string-append "its contract says a use defines `"
                                          (string-append (symbol->string (car missing))
                                          "`, and its expansion does not"))
                      out))
            (else #u)))))
;;; ------------------------------------------------------------ macros an expansion defines

;; The macros the last top-level form's expansion defined, in order
;; (`Withs::made_syntax`): for the parser, which holds each to the globals
;; and checks its templates (`made-syntax-taken!`).
(define made-syntax (ref mx-parser-types-names mxp) (new nil))
(define made-syntax-reset! (->* () unit) (lambda () (set made-syntax nil)))
(define made-syntax-add! (->* (mx-parser-types-names) unit)
  (lambda (ns) (set made-syntax (append (get made-syntax) ns))))
(define made-syntax-count (->* () int) (lambda () (length (get made-syntax))))
(define made-syntax-taken! (->* () mx-parser-types-names)
  (lambda () (let ((ns (get made-syntax))) (begin (set made-syntax nil) ns))))
;; The part of `s` at address `a`, in a list of one; none if there is none.
(define syn-at-addr (->* (syn addr) syns-a)
  (lambda (s a)
    (cond ((null? a) (list s))
          ((null? (split s)) nil)
          (else (syn-at-addr (if (car a) (car (split s)) (nth (split s) 1)) (cdr a))))))
;; Whether `n` is among `ns` from the `k`th on.
(define name-in-from? (->* (symbol mx-parser-types-names int) bool)
  (lambda (n ns k)
    (cond ((null? ns) #f)
          ((> k 0) (name-in-from? n (cdr ns) (- k 1)))
          (else (or (symbol=? n (car ns)) (name-in-from? n (cdr ns) 0))))))
;; Whether macro `n`'s signature is the one `said` writes: the same syntax
;; type, each a subtype of the other, of the same made, none `poly`, and no
;; procedural macro's of no contract.
(define same-signature? (->* (fails symbol syn) bool)
  (lambda (mx-fail n said)
    (let ((have (car (macro-of n (get defined)))) (want (signature-of mx-fail said)))
      (and (null? (extract want 1)) (null? (extract have 3))
           (= (extract have 4) (extract want 3))
           (sub? (extract have 1) (extract want 2) nil)
           (sub? (extract want 2) (extract have 1) nil)))))
(define syntax-held-each (->* (fails symbol syn int entries) unit)
  (lambda (mx-fail name s from es)
    (if (null? es)
        #u
        (let ((d (extract (car es) 2)))
          (if (< d 100)
              (syntax-held-each mx-fail name s from (cdr es))
              (let* ((said (syntax-sig-of d)) (shown (show-syn said))
                     (part (syn-at-addr (nth (split s) 1) (extract (car es) 1)))
                     (x (if (and (not (null? part)) (syn-symbol? (car part)))
                            (syn-name (car part))
                            ""))
                     (refused (lambda ((m string))
                                (mx-fail (string-append "a use of `" (string-append (symbol->string name) (string-append "`: " m)))
                                      (syn-start s) (syn-end s)))))
                (cond ((string=? x "")
                       (refused (string-append "its signature says a macro of `" (string-append shown (string-append "` is "
                                               "defined at a part that is not a name")))))
                      ((not (name-in-from? (string->symbol x) (get made-syntax) from))
                       (refused (string-append "its signature says it defines `" (string-append x (string-append "`, a macro "
                                               (string-append "of `" (string-append shown "`, and its expansion does not")))))))
                      ((not (same-signature? mx-fail (string->symbol x) said))
                       (refused (string-append "its signature says `" (string-append x (string-append "` is a macro of `"
                                               (string-append shown (string-append "`, and its expansion defines one of "
                                               "another signature")))))))
                      (else (syntax-held-each mx-fail name s from (cdr es))))))))))
;; Use `s` of macro `name`, its expansion done: each macro its signature
;; says a use defines, `(ℓ (macro σ made))` in what it makes, defined by the
;; expansion (`made-syntax` from `from` on), and of signature `σ`
;; (`Macros::syntax_held`).
(define syntax-held (->* (fails symbol syn int) unit)
  (lambda (mx-fail name s from)
    (let ((m (macro-of name (get defined))))
      (if (null? m)
          #u
          (tagcase (extract (car m) 1)
            (s-export (body b)
              (tagcase b (bind-rib (es) (syntax-held-each mx-fail name s from es)) (else y #u)))
            (else y #u))))))

;;; ============================================================ macros-procedural.fx (whole)

;;; ------------------------------------------------------------ the programs

;; A style, by its form's name: 0 explicit renaming, 1 syntactic closures,
;; 2 reversed; -1 none.
(define style-named (->* (string) int)
  (lambda (n)
    (cond ((string=? n "er-macro-transformer") 0)
          ((string=? n "sc-macro-transformer") 1)
          ((string=? n "rsc-macro-transformer") 2)
          (else -1))))
;; Phase-1 forms `fs`, each its length in characters on a line of its own
;; and its text, as a request gives them (`procedural::asked`).
(define forms-given (->* ((listof string acyclic)) string)
  (lambda (fs)
    (if (null? fs)
        ""
        (string-append (int->string (string-length (car fs))) (string-append "\n" (string-append (car fs)
                       (forms-given (cdr fs))))))))
;; The request that defines a transformer, `code`, where its macro is
;; defined: checked at its style's type and compiled, once, for its uses to
;; call (`procedural::define`, which the Rust expander calls itself).
(define checked-text (->* (int string (listof string acyclic)) string)
  (lambda (s code fs)
    (string-append "%define " (string-append (int->string s) (string-append " " (string-append (int->string (length fs)) (string-append "\n"
                   (string-append (forms-given fs) code))))))))
;; The request that checks phase-1 forms `fs`, in force from a
;; `begin-for-syntax` on.
(define forms-text (->* ((listof string acyclic)) string)
  (lambda (fs) (string-append "%forms " (string-append (int->string (length fs)) (string-append "\n" (forms-given fs))))))
;; The request that runs one on `form` (as text), in expansion `made`: the
;; transformer checked and compiled at its first use, then called
;; (`procedural::apply`, which the Rust expander calls itself). Its tag field
;; `0`: no transformer is given the expansion's (`tagged`), so a request is
;; the same whatever expansion makes it.
(define applied-text (->* (int string string int (listof string acyclic)) string)
  (lambda (s code form made fs)
    (string-append "%apply " (string-append (int->string s) (string-append " 0 "
                   (string-append (int->string (string-length code)) (string-append " " (string-append (int->string (length fs)) (string-append "\n"
                   (string-append (forms-given fs) (string-append code form)))))))))))
;; The runtime's answer to `code`: what it gave (`=`), or why not (`!`),
;; in a sum.
(define mx-procedural-run (->* (string) (sumof (ok string) (failed string)))
  (lambda (code)
    (let ((a (mx-expand-procedural code)))
      (if (and (> (string-length a) 0) (char=? (string-ref a 0) #\=))
          (sum ok (substring a 1 (string-length a)))
          (sum failed (if (> (string-length a) 0) (substring a 1 (string-length a)) a))))))
(define mx-procedural-escaped (->* (string int string) string)
  (lambda (s i out)
    (if (>= i (string-length s))
        out
        (let ((c (string-ref s i)))
          (mx-procedural-escaped s (+ i 1)
                   (string-append out (cond ((char=? c #\") "\\\"")
                                            ((char=? c #\\) "\\\\")
                                            ((char=? c (integer->char 10)) "\\n")
                                            (else (char->string c)))))))))
(define char-text (->* (char) string)
  (lambda (c)
    (cond ((char=? c #\space) "#\\space")
          ((char=? c (integer->char 10)) "#\\newline")
          ((char=? c (integer->char 9)) "#\\tab")
          (else (string-append "#\\" (char->string c))))))
(define datum-text (->* (datum) string)
  (lambda (d)
    (cond ((null? d) "()")
          ((symbol? d) (symbol->string d))
          ((mx-keyword? d) (string-append "#:" (symbol->string (mx-keyword->symbol d))))
          ((string? d) (string-append "\"" (string-append (mx-procedural-escaped d 0 "") "\"")))
          ((char? d) (char-text d))
          ((int? d) (int->string d))
          ((f64? d) (f64->string d))
          ((bool? d) (if d "#t" "#f"))
          (else "#u"))))
(define-rec
;;; ------------------------------------------------------------ written

;; `s` as text the reader reads back.
(syn-text (->* (syn) string)
  (lambda (s)
    (tagcase s
      (atom (d a b x) (datum-text d))
      (mx-eager-reader-types-lst (items d a b x) (string-append "(" (string-append (syns-text items) ")")))
      (dotted (items tail d a b x)
        (string-append "(" (string-append (syns-text items) (string-append " . " (string-append (syn-text tail) ")")))))
      (vec (items d a b x) (string-append "#(" (string-append (syns-text items) ")"))))))
(syns-text (->* (syns-a) string)
  (lambda (xs)
    (cond ((null? xs) "")
          ((null? (cdr xs)) (syn-text (car xs)))
          (else (string-append (syn-text (car xs)) (string-append " " (syns-text (cdr xs)))))))))
;;; ------------------------------------------------------------ places

;; Where a part a template made is: a key of its own, which no other part
;; and no text has, a place past every text's (`source-base`, above the
;; positions of every file loaded, `load-base` apart); and, by its key,
;; where the template wrote it and which expansion made it, for what is
;; said of it (`source-of`; the driver's `fx26_span`), as the Rust front
;; end's `Span` has a template's place and its `expansion`. Provenance
;; only: what the checkers find of an expression is keyed by its place in
;; the parser's walk (`docs/research/loads-and-places.md`), so parts that
;; share a key, or a part placed twice, are told apart. Keys are made
;; in order, a list's start before its parts' and its end after, so that
;; a part is within its list. One table for the session, so that no key is
;; given twice.
(define source-base int 1125899906842624)
(define sources (ref (arrayof (tupleof int int) mxp) mxp) (new (make-array 1024 (tuple 0 0))))
(define nsources (ref int mxp) (new 0))
(define copy-sources (->* ((arrayof (tupleof int int) mxp) (arrayof (tupleof int int) mxp) int) unit)
  (lambda (from to i)
    (if (= i (array-length from))
        #u
        (begin (array-set! to i (array-ref from i)) (copy-sources from to (+ i 1))))))
(define new-source-key (->* (int int) int)
  (lambda (p made)
    (let ((i (get nsources)))
      (begin
        (if (= i (array-length (get sources)))
            (let ((bigger (the (arrayof (tupleof int int) mxp) (make-array (* 2 i) (tuple 0 0)))))
              (begin (copy-sources (get sources) bigger 0) (set sources bigger)))
            #u)
        (array-set! (get sources) i (tuple p made))
        (set nsources (+ i 1))
        (+ source-base i)))))
;; A new key, for template place `p`, of expansion `made`; none in a
;; template's check (`made` 0), whose places are the template's.
(define source-key (->* (int int) int)
  (lambda (p made)
    (if (= made 0) p (new-source-key p made))))
(define macro-of-defined (->* (symbol macro-env) (listof macro acyclic))
  (lambda (n ms)
    (cond ((null? ms) nil)
          ((symbol=? n (extract (car ms) 1)) (list (extract (car ms) 2)))
          (else (macro-of-defined n (cdr ms))))))
(define procedurals (ref (listof procedural acyclic) mxp) (new nil))
(define procedural-in
  (->* (symbol (listof procedural acyclic))
       (listof (tupleof int string def-env bool (listof string acyclic)) acyclic))
  (lambda (n ps)
    (cond ((null? ps) nil)
          ((symbol=? n (extract (car ps) 1))
           (list (tuple (extract (car ps) 2) (extract (car ps) 3) (extract (car ps) 4)
                        (extract (car ps) 5) (extract (car ps) 6))))
          (else (procedural-in n (cdr ps))))))
(define procedural-of
  (->* (symbol) (listof (tupleof int string def-env bool (listof string acyclic)) acyclic))
  (lambda (n)
    ;; Only while `defined` has it so (it, not `procedurals`, is what a
    ;; state saves and gives back, `macros-at!`): one that makes `3`, or one
    ;; of a contract, which has no clauses; a `syntax-rules` macro since
    ;; defined of the name is not one.
    (let ((m (macro-of-defined n (get defined))))
      (if (and (not (null? m)) (or (= (extract (car m) 4) 3) (null? (extract (car m) 2))))
          (procedural-in n (get procedurals))
          nil))))
(define mx-procedural-skip (->* (string int) int)
  (lambda (s i)
    (if (and (< i (string-length s)) (char-whitespace? (string-ref s i))) (mx-procedural-skip s (+ i 1)) i)))
(define mx-procedural-delimiter? (->* (char) bool)
  (lambda (c) (or (char-whitespace? c) (char=? c #\() (char=? c #\)) (char=? c #\"))))
(define token-end (->* (string int) int)
  (lambda (s i)
    (cond ((>= i (string-length s)) i)
          ;; `#\(`: a character, not a list.
          ((and (char=? (string-ref s i) #\\) (> i 0) (char=? (string-ref s (- i 1)) #\#))
           (token-end s (+ i 2)))
          ((mx-procedural-delimiter? (string-ref s i)) i)
          (else (token-end s (+ i 1))))))
(define mx-procedural-char-named (->* (string) datum)
  (lambda (n)
    (cond ((string=? n "space") #\space)
          ((string=? n "newline") (integer->char 10))
          ((string=? n "tab") (integer->char 9))
          (else (string-ref n 0)))))
(define read-token (->* (string int int int int int) mx-procedural-read-one)
  (lambda (s i j a b x)
    (let* ((t (substring s i j)) (n (mx-parse-number t 10)))
      (list (tuple (atom (cond ((not (null? n)) (car n))
                               ((string=? t "#t") #t)
                               ((string=? t "#f") #f)
                               ((and (> (string-length t) 2) (string=? (substring t 0 2) "#\\"))
                                (mx-procedural-char-named (substring t 2 (string-length t))))
                               ((and (> (string-length t) 2) (string=? (substring t 0 2) "#:"))
                                (mx-symbol->keyword
                                 (string->symbol (substring t 2 (string-length t)))))
                               (else (string->symbol t)))
                         a b x)
                   j)))))
(define read-string-at (->* (string int string int int int) mx-procedural-read-one)
  (lambda (s i out a b x)
    (cond ((>= i (string-length s)) nil)
          ((char=? (string-ref s i) #\") (list (tuple (atom out a b x) (+ i 1))))
          ((and (char=? (string-ref s i) #\\) (< (+ i 1) (string-length s)))
           (let ((c (string-ref s (+ i 1))))
             (read-string-at s (+ i 2)
                             (string-append out (if (char=? c #\n)
                                                    (char->string (integer->char 10))
                                                    (char->string c)))
                             a b x)))
          (else (read-string-at s (+ i 1) (string-append out (char->string (string-ref s i)))
                                a b x)))))
(define-rec
;; The datum at `i`, and where it ends.
(mx-procedural-read-at (->* (string int int int int) mx-procedural-read-one)
  (lambda (s i a b x)
    (if (>= i (string-length s))
        nil
        (let ((c (string-ref s i)))
          (cond ((char=? c #\() (mx-procedural-read-items s (mx-procedural-skip s (+ i 1)) a b x nil))
                ((char=? c #\") (read-string-at s (+ i 1) "" a b x))
                ((char=? c #\')
                 (let ((r (mx-procedural-read-at s (mx-procedural-skip s (+ i 1)) a b x)))
                   (if (null? r)
                       nil
                       (let ((q (list (atom 'quote a b x) (extract (car r) 1))))
                         (list (tuple (mx-eager-reader-types-lst q (syn-datums q) a b x) (extract (car r) 2)))))))
                ((and (char=? c #\#) (< (+ i 1) (string-length s))
                      (char=? (string-ref s (+ i 1)) #\())
                 (let ((r (mx-procedural-read-items s (mx-procedural-skip s (+ i 2)) a b x nil)))
                   (if (null? r)
                       nil
                       (tagcase (extract (car r) 1)
                         (mx-eager-reader-types-lst (items d a2 b2 x2)
                           (list (tuple (vec items (datum-list->vector (syn-datums items)) a b x)
                                        (extract (car r) 2))))
                         (else y nil)))))
                (else (read-token s i (token-end s i) a b x)))))))
;; A list's items from `i`, `out` those read, the last first.
(mx-procedural-read-items (->* (string int int int int syns-a) mx-procedural-read-one)
  (lambda (s i a b x out)
    (cond ((>= i (string-length s)) nil)
          ((char=? (string-ref s i) #\))
           (let ((items (reverse out)))
             (list (tuple (mx-eager-reader-types-lst items (syn-datums items) a b x) (+ i 1)))))
          ((and (char=? (string-ref s i) #\.) (< (+ i 1) (string-length s))
                (mx-procedural-delimiter? (string-ref s (+ i 1))) (not (null? out)))
           (let ((r (mx-procedural-read-at s (mx-procedural-skip s (+ i 1)) a b x)))
             (if (null? r)
                 nil
                 (let ((e (mx-procedural-skip s (extract (car r) 2))) (items (reverse out))
                       (t (extract (car r) 1)))
                   (if (and (< e (string-length s)) (char=? (string-ref s e) #\)))
                       (list (tuple (dotted items t (mx-datum-dotted (syn-datums items) (syn->datum t))
                                            a b x)
                                    (+ e 1)))
                       nil)))))
          (else
           (let ((r (mx-procedural-read-at s i a b x)))
             (if (null? r)
                 nil
                 (mx-procedural-read-items s (mx-procedural-skip s (extract (car r) 2)) a b x
                             (cons (extract (car r) 1) out)))))))))
(define read-answer (->* (string int int int) (listof syn acyclic))
  (lambda (text a b x)
    (let ((r (mx-procedural-read-at text (mx-procedural-skip text 0) a b x)))
      (if (and (not (null? r)) (= (mx-procedural-skip text (extract (car r) 2)) (string-length text)))
          (list (extract (car r) 1))
          nil))))
;; `n`'s alias, of expansion `made`, by its tag.
(define mx-procedural-alias (->* (string int) string)
  (lambda (n made) (string-append "%~" (string-append (mx-fresh-tag-of made) (string-append ":" n)))))
(define mx-procedural-head-of (->* (syns-a) string)
  (lambda (items)
    (if (and (not (null? items)) (syn-symbol? (car items))) (syn-name (car items)) "")))
(define mx-procedural-symbols-of (->* (syns-a) (listof symbol acyclic))
  (lambda (xs)
    (cond ((null? xs) nil)
          ((syn-symbol? (car xs)) (cons (syn-symbol (car xs)) (mx-procedural-symbols-of (cdr xs))))
          (else (mx-procedural-symbols-of (cdr xs))))))
(define free-names (->* (syn) (listof symbol acyclic))
  (lambda (s)
    (tagcase s
      (mx-eager-reader-types-lst (items d a b x) (mx-procedural-symbols-of items))
      (else y nil))))
(define in-names? (->* (symbol (listof symbol acyclic)) bool)
  (lambda (n ns) (and (not (null? ns)) (or (symbol=? n (car ns)) (in-names? n (cdr ns))))))
(define-rec
;; What a syntactic-closure transformer gave, `s`, its closures opened: in
;; the macro's environment (`macro-env`), each name an alias of expansion
;; `made`, else as written; a closure's form in its own environment, its
;; free names in the one around it; a quoted datum as it is
;; (`procedural::closed`).
(closed (->* (fails syn bool int) syn)
  (lambda (mx-fail s macro-env made)
    (tagcase s
      (atom (d a b x)
        (if (and macro-env (symbol? d))
            (atom (string->symbol (mx-procedural-alias (symbol->string d) made)) a b x)
            s))
      (mx-eager-reader-types-lst (items d a b x)
        (let ((head (mx-procedural-head-of items)))
          (cond ((string=? head "quote") s)
                ((string=? head "%sc") (opened-closure mx-fail s items macro-env made))
                (else (let ((xs (mx-procedural-closed-each mx-fail items macro-env made)))
                        (mx-eager-reader-types-lst xs (syn-datums xs) a b x))))))
      (dotted (items tail d a b x)
        (let ((xs (mx-procedural-closed-each mx-fail items macro-env made)) (t (closed mx-fail tail macro-env made)))
          (dotted xs t (mx-datum-dotted (syn-datums xs) (syn->datum t)) a b x)))
      (vec (items d a b x) s))))
(mx-procedural-closed-each (->* (fails syns-a bool int) syns-a)
  (lambda (mx-fail xs macro-env made)
    (if (null? xs) nil (cons (closed mx-fail (car xs) macro-env made)
                             (mx-procedural-closed-each mx-fail (cdr xs) macro-env made)))))
;; A closure, `(%sc env (free …) form)`, placed where names are in
;; `outer`'s environment.
(opened-closure (->* (fails syn syns-a bool int) syn)
  (lambda (mx-fail s items outer made)
    (if (not (= (len items) 4))
        (begin (mx-fail  "a syntactic closure is `(%sc env (free …) form)`"
                     (syn-start s) (syn-end s))
               s)
        (let* ((env (car (cdr items))) (free (car (cdr (cdr items))))
               (form (car (cdr (cdr (cdr items)))))
               (e (if (syn-symbol? env) (syn-name env) "")))
          (cond ((not (or (string=? e "%env:use") (string=? e "%env:macro")))
                 (begin (mx-fail "a syntactic closure's environment is one a transformer was given"
                              (syn-start env) (syn-end env))
                        s))
                (else (opened mx-fail form (string=? e "%env:macro") outer (free-names free)
                              made)))))))
;; A closure's form, `s`, closed in `inner`, its `free` names in `outer`.
(opened (->* (fails syn bool bool (listof symbol acyclic) int) syn)
  (lambda (mx-fail s inner outer free made)
    (tagcase s
      (atom (d a b x)
        (if (and (symbol? d) (in-names? d free))
            (closed mx-fail s outer made)
            (closed mx-fail s inner made)))
      (mx-eager-reader-types-lst (items d a b x)
        (let ((head (mx-procedural-head-of items)))
          (if (or (string=? head "quote") (string=? head "%sc"))
              (closed mx-fail s inner made)
              (let ((xs (opened-each mx-fail items inner outer free made)))
                (mx-eager-reader-types-lst xs (syn-datums xs) a b x)))))
      (dotted (items tail d a b x)
        (let ((xs (opened-each mx-fail items inner outer free made))
              (t (opened mx-fail tail inner outer free made)))
          (dotted xs t (mx-datum-dotted (syn-datums xs) (syn->datum t)) a b x)))
      (vec (items d a b x) s))))
(opened-each (->* (fails syns-a bool bool (listof symbol acyclic) int) syns-a)
  (lambda (mx-fail xs inner outer free made)
    (if (null? xs)
        nil
        (cons (opened mx-fail (car xs) inner outer free made)
              (opened-each mx-fail (cdr xs) inner outer free made))))))
(define mx-procedural-digits-end (->* (string int) int)
  (lambda (s i)
    (if (and (< i (string-length s)) (tag-char? (string-ref s i))) (mx-procedural-digits-end s (+ i 1)) i)))
(define mx-procedural-alias-base-from (->* (string bool) string)
  (lambda (n any)
    (let ((c (if (and (> (string-length n) 2) (string=? (substring n 0 2) "%~"))
                 (mx-procedural-digits-end n 2)
                 -1)))
      (if (and (> c 2) (< c (string-length n)) (char=? (string-ref n c) #\:))
          (mx-procedural-alias-base-from (substring n (+ c 1) (string-length n)) #t)
          (if any n "")))))
;;; ------------------------------------------------------------ aliases

;; `x`'s name as where its alias's macro was defined, if it is an alias
;; (`%~T:x`, `T` its expansion's tag): every `%~T:` taken off; else "".
(define mx-procedural-alias-base (->* (string) string)
  (lambda (n) (mx-procedural-alias-base-from n #f)))
(define named? (->* (string (listof (tupleof symbol sty) acyclic)) bool)
  (lambda (n ns)
    (and (not (null? ns))
         (or (string=? n (symbol->string (extract (car ns) 1))) (named? n (cdr ns))))))
;; Whether `n` is a keyword's name, which nothing binds: a form's, a
;; description's word, a macro's definition's, a macro's
;; (`Macros::procedural_opened`'s).
(define mx-procedural-keyword-name? (->* (fails string) bool)
  (lambda (mx-fail n)
    (or (reserved? n) (desc-word? n #t) (type-form? n)
        (named? n (kernel-sigs mx-fail)) (named? n (kernel-definitions mx-fail))
        (string=? n "else") (string=? n "=>") (string=? n "quote") (string=? n "quasiquote")
        (string=? n "unquote") (string=? n "unquote-splicing")
        ;; A macro's definition's words (`transformer_word`).
        (string=? n "syntax-rules") (string=? n "trailmark") (>= (style-named n) 0)
        (macro? (string->symbol n)))))
(define-rec
;; `s` with every alias its name.
(unaliased (->* (syn) syn)
  (lambda (s)
    (tagcase s
      (atom (d a b x)
        (if (symbol? d)
            (let ((base (mx-procedural-alias-base (symbol->string d))))
              (if (string=? base "") s (atom (string->symbol base) a b x)))
            s))
      (mx-eager-reader-types-lst (items d a b x) (let ((xs (unaliased-each items))) (mx-eager-reader-types-lst xs (syn-datums xs) a b x)))
      (dotted (items tail d a b x)
        (let ((xs (unaliased-each items)) (t (unaliased tail)))
          (dotted xs t (mx-datum-dotted (syn-datums xs) (syn->datum t)) a b x)))
      (vec (items d a b x)
        (let ((xs (unaliased-each items))) (vec xs (datum-list->vector (syn-datums xs)) a b x))))))
(unaliased-each (->* (syns-a) syns-a)
  (lambda (xs) (if (null? xs) nil (cons (unaliased (car xs)) (unaliased-each (cdr xs)))))))
(define-rec
;; What a transformer gave, `s`, each alias that is a keyword's name made
;; the keyword, and every alias quoted made its name (`procedural::
;; keywords`).
(keywords (->* (fails syn) syn)
  (lambda (mx-fail s)
    (tagcase s
      (atom (d a b x)
        (if (symbol? d)
            (let ((base (mx-procedural-alias-base (symbol->string d))))
              (if (and (not (string=? base "")) (mx-procedural-keyword-name? mx-fail base))
                  (atom (string->symbol base) a b x)
                  s))
            s))
      (mx-eager-reader-types-lst (items d a b x)
        (let* ((head (mx-procedural-head-of items)) (base (mx-procedural-alias-base head))
               (h (if (string=? base "") head base)))
          (if (or (string=? h "quote") (string=? h "quasiquote"))
              (unaliased s)
              (let ((xs (keywords-each mx-fail items))) (mx-eager-reader-types-lst xs (syn-datums xs) a b x)))))
      (dotted (items tail d a b x)
        (let ((xs (keywords-each mx-fail items)) (t (keywords mx-fail tail)))
          (dotted xs t (mx-datum-dotted (syn-datums xs) (syn->datum t)) a b x)))
      (vec (items d a b x)
        (let ((xs (keywords-each mx-fail items)))
          (vec xs (datum-list->vector (syn-datums xs)) a b x))))))
(keywords-each (->* (fails syns-a) syns-a)
  (lambda (mx-fail xs)
    (if (null? xs) nil (cons (keywords mx-fail (car xs)) (keywords-each mx-fail (cdr xs)))))))
(define-rec
;; Datum `d` as syntax, each part at `a`..`b` of expansion `x`: what
;; `read-answer` of its text makes, with no text between (`heap_syn`'s
;; `syn_of_datum`, the Rust expander's).
(syn-of-datum (->* (datum int int int) syn)
  (lambda (d a b x)
    (cond ((vector? d)
           (vec (syns-of-datum (mx-datum-vector->list d) a b x) d a b x))
          ((pair? d) (syn-of-list d nil a b x))
          ((null? d) (mx-eager-reader-types-lst nil d a b x))
          (else (atom d a b x)))))
(syn-of-list (->* (datum syns-a int int int) syn)
  (lambda (whole out a b x) (syn-of-rest whole whole out a b x)))
;; The list `whole`, its parts before `l` made, newest first, `out`.
(syn-of-rest (->* (datum datum syns-a int int int) syn)
  (lambda (whole l out a b x)
    (if (pair? l)
        (syn-of-rest whole (cdr l) (cons (syn-of-datum (car l) a b x) out) a b x)
        (if (null? l)
            (mx-eager-reader-types-lst (reverse out) whole a b x)
            (dotted (reverse out) (syn-of-datum l a b x) whole a b x)))))
(syns-of-datum (->* (datum int int int) syns-a)
  (lambda (l a b x)
    (if (pair? l) (cons (syn-of-datum (car l) a b x) (syns-of-datum (cdr l) a b x)) nil))))
;; A use of procedural macro `name`, `s`, in expansion `made`: what its
;; transformer gave, at the use's place, its closures opened and its
;; keywords' aliases the keywords (`Macros::expand_procedural`).
(define expand-procedural (->* (fails symbol syn int) syn)
  (lambda (mx-fail name s made)
    (let* ((p (car (procedural-of name))) (style (extract p 1))
           (a (syn-start s)) (b (syn-end s))
           (in-use (lambda ((m string))
                    (mx-fail (string-append "a use of `" (string-append (symbol->string name) (string-append "`: " m))) a b)))
           (opened (lambda ((at syn))
                     (keywords mx-fail (cond ((= style 1) (closed mx-fail at #t made))
                                          ((= style 2) (closed mx-fail at #f made))
                                          (else (tagged at made)))))))
      ;; In the expansion's phase-1 namespace, on the use's datum, the one
      ;; the Rust expander calls it in (`phase1.rs`): what it gives made
      ;; syntax at the use's place, no text between; apart, by text, where
      ;; the runtime has none.
      (tagcase (mx-call-procedural style (extract p 2) (syn->datum s) (mx-fresh-tag-of made)
                                        (extract p 5))
        (ok d (opened (syn-of-datum d (new-source-key a made) (new-source-key b made) made)))
        (failed why (begin (in-use (string-append "its transformer: " why)) s))
        (apart u
          (tagcase (mx-procedural-run (applied-text style (extract p 2) (syn-text s) made (extract p 5)))
            (failed why (begin (in-use (string-append "its transformer: " why)) s))
            (ok out
              (let ((read (read-answer out (new-source-key a made) (new-source-key b made) made)))
                (if (null? read)
                    (begin (in-use (string-append "its transformer gave `" (string-append out "`, not one form")))
                           s)
                    (opened (car read)))))))))))
(define mx-procedural-contracts (ref int mxp) (new 1))
;;; ------------------------------------------------------------ contracts

;; A use `s` of procedural macro `name`, in expansion `made`: expanded, and
;; held to its contract if it has one (`Macros::contract_expanded`;
;; `typed-macros.md`, "Decided: procedural macros checked by their
;; contracts"): the use aligned with it; then, unless
;; `FIXPT_MACRO_CONTRACTS=0`, the transformer run on it and on it with each
;; binder's name fresh throughout its scope, the second's fresh names where
;; their binders' scopes are (placement), and, named back, the first
;; (commutation). The first is the expansion. Of a contract that makes
;; definitions, the expansion is definitions, and defines of the use's names
;; exactly those it says (`defined-held`); a name it defines is a binder,
;; renamed as any other.
(define procedural-expanded (->* (fails symbol syn int) syn)
  (lambda (mx-fail name s made)
    (let* ((shown (string-append "a use of `" (string-append (symbol->string name) "`: ")))
           (in-use (lambda ((m string) (a int) (b int)) (mx-fail (string-append shown m) a b)))
           (at-use (lambda ((m string) (a int) (b int))
                     (mx-fail (string-append shown m) (syn-start s) (syn-end s))))
           (m (car (macro-of-defined name (get defined))))
           (sig (extract m 1))
           (defn (= (extract m 4) 1))
           (contract (extract (car (procedural-of name)) 4))
           (operands (nth (split s) 1)))
      (if (not contract)
          (expand-procedural mx-fail name s made)
          (begin
            (nodes-reset!)
            (align in-use 2 (the pvars nil) sig operands -1 nil)
            (let ((out (expand-procedural mx-fail name s made)))
              (if (= (get mx-procedural-contracts) 0)
                  out
                  (let* ((held (if defn
                                   (begin (defined-held at-use sig operands out made)
                                          (nodes-reset!)
                                          (align in-use 2 (the pvars nil) sig operands -1 nil))
                                   0))
                         (r (contract-renames in-use sig operands)) (fresh (extract r 2)))
                    (if (null? fresh)
                        out
                        (let ((out2 (expand-procedural mx-fail name
                                                       (renamed-use in-use s (extract r 1)) made)))
                          (begin (contract-held in-use s out out2 fresh made
                                                (if defn (s-defn) (s-expr)))
                                 out)))))))))))
(define pfail-apart (->* (string int int) void)
  (lambda (message a b) (abort-current-continuation parse-tag (p-err message a b))))
(define fits-apart? (->* (sty syn pvars) bool)
  (lambda (sig operands ps)
    (tagcase (prompt parse-tag (begin (align pfail-apart 1 ps sig operands -1 nil) (p-ok nil))
                     (lambda ((r presult)) r))
      (p-err (why a b) #f)
      (else y #t))))
(define checked-from (->* (int int symbol sty pvars (listof int acyclic)) (listof int acyclic))
  (lambda (i n name sig ps out)
    (if (= i n)
        out
        (let* ((s (node-syn i)) (parts (split s))
               (h (if (or (null? parts) (not (syn-symbol? (car parts)))
                          (pvar? (syn-symbol (car parts)) ps)
                          (tagcase (node-kids i) (k-form (x) #f) (else y #t)))
                      nil
                      (list (syn-symbol (car parts)))))
               (checked (and (not (null? h))
                             (if (macro? (car h))
                                 (or (= (makes (car h)) 0) (= (makes (car h)) 1))
                                 (and (symbol=? (car h) name)
                                      (fits-apart? sig (nth parts 1) ps))))))
          (checked-from (+ i 1) n name sig ps (if checked (cons (syn-start s) out) out))))))
;;; ------------------------------------------------------------ uses checked once

;; A template's checked uses (`Clause::checked_uses`), by where each starts:
;; each list its alignment (`template-actions`, just made) met as a use of a
;; macro, its operands aligned with that one's signature; and each use of
;; `name`, the macro being defined, whose operands fit its signature `sig`,
;; tried apart, the pattern's variables `ps` as theirs, so that no
;; definition is refused for one (`typed-macros.md`, "Decided: a use a
;; template made is not aligned again").
(define checked-uses (->* (symbol sty pvars) (listof int acyclic))
  (lambda (name sig ps) (append (get heads-checked) (checked-from 0 (get nnodes) name sig ps nil))))
;; Each key's part made at a template's checked use, by key, from
;; `source-base`: what it makes fits its macro's signature.
(define trusts (ref (arrayof bool mxp) mxp) (new (make-array 1024 #f)))
(define copy-trusts (->* ((arrayof bool mxp) (arrayof bool mxp) int) unit)
  (lambda (from to i)
    (if (= i (array-length from))
        #u
        (begin (array-set! to i (array-ref from i)) (copy-trusts from to (+ i 1))))))
(define mx-procedural-int-in? (->* (int (listof int acyclic)) bool)
  (lambda (x xs) (and (not (null? xs)) (or (= x (car xs)) (mx-procedural-int-in? x (cdr xs))))))
;; Key `a`, made for template place `ta` of clause `c` by expansion `made`:
;; trusted if `ta` is a checked use's.
(define trust-if! (->* (clause int int int) unit)
  (lambda (c ta a made)
    (if (and (> made 0) (mx-procedural-int-in? ta (extract c 8)))
        (let ((i (- a source-base)))
          (begin
            (if (>= i (array-length (get trusts)))
                (let ((bigger (the (arrayof bool mxp) (make-array (* 2 (+ i 1)) #f))))
                  (begin (copy-trusts (get trusts) bigger 0) (set trusts bigger)))
                #u)
            (array-set! (get trusts) i #t)))
        #u)))
(define trusted-key? (->* (int) bool)
  (lambda (k)
    (let ((i (- k source-base)))
      (and (>= i 0) (< i (array-length (get trusts))) (array-ref (get trusts) i)))))
;; Whether a use a template made is aligned again, 1, or not, 0
;; (`FIXPT_MACRO_RECHECK`; `use-aligned!`); whether a procedural macro's
;; expansions are held to its contract (`FIXPT_MACRO_CONTRACTS`).
(define recheck (ref int mxp) (new 0))
;; A use `s`'s operands aligned with its macro's signature `sig`, failing by
;; `in-use`; not where the use is trusted (`trusted-key?`), unless
;; `FIXPT_MACRO_RECHECK` asks, and then a refusal is the binding-shape
;; check's fault, said so.
(define use-aligned! (->* (fails syn sty syn) unit)
  (lambda (in-use s sig operands)
    (let ((trusted (trusted-key? (syn-start s))))
      (if (and trusted (= (get recheck) 0))
          #u
          (begin
            (align (if trusted
                       (lambda ((msg string) (a int) (b int))
                         (in-use (string-append msg (string-append " (a use a template made, which its macro's "
                                                (string-append "definition found fits: a fault in the "
                                                "binding-shape check, not in the program)")))
                                 a b))
                       in-use)
                   2 (the pvars nil) sig operands -1 nil)
            #u)))))
;; How many keys have been made: the next one's number, from `source-base`.
(define sources-made (->* () int) (lambda () (get nsources)))
;; Key `k`'s template place and expansion; a place that is no key, itself
;; and 0.
(define mx-procedural-source-of (->* (int) (tupleof int int))
  (lambda (k)
    (let ((i (- k source-base)))
      (if (and (>= i 0) (< i (get nsources))) (array-ref (get sources) i) (tuple k 0)))))
;; The phase-1 forms in force (`begin-for-syntax`'s texts, in order), each
;; transformer defined now running where they are (`phase1.rs`): its own,
;; the sixth of its `procedural`.
(define phase1-forms (ref (listof string acyclic) mxp) (new nil))
(define phase1-forms-now (->* () (listof string acyclic)) (lambda () (get phase1-forms)))
(define phase1-forms! (->* ((listof string acyclic)) unit) (lambda (fs) (set phase1-forms fs)))
(define phase1-state-now (->* () phase1-state)
  (lambda () (tuple (get phase1-forms) (get procedurals))))
(define phase1-state! (->* (phase1-state) unit)
  (lambda (s) (begin (set phase1-forms (extract s 1)) (set procedurals (extract s 2)))))
(define phase1-none? (->* () bool)
  (lambda () (and (null? (get phase1-forms)) (null? (get procedurals)))))
(define syns-texts (->* (syns-a) (listof string acyclic))
  (lambda (xs) (if (null? xs) nil (cons (syn-text (car xs)) (syns-texts (cdr xs))))))
;; `(begin-for-syntax form …)`, `s`: its forms in force from now on, checked,
;; compiled and run in their namespace now, so that one refused is refused
;; here, as the Rust expander refuses it (`Macros::for_syntax`).
(define for-syntax! (->* (fails syn) unit)
  (lambda (mx-fail s)
    (let ((fs (append (get phase1-forms)
                      (syns-texts (tagcase s (mx-eager-reader-types-lst (items d a b x) (cdr items)) (else y nil))))))
      (tagcase (mx-procedural-run (forms-text fs))
        (ok v (set phase1-forms fs))
        (failed why
          (mx-fail (string-append "`begin-for-syntax`: " why) (syn-start s) (syn-end s)))))))
;; `(define-syntax name [σ] (STYLE proc))`, its parts: the name, the style,
;; the procedure, and its contract if written; in a list of one; none if
;; `form` is not one.
(define procedural-form (->* (syn) (listof (tupleof syn int syn syns-a) acyclic))
  (lambda (form)
    (tagcase form
      (mx-eager-reader-types-lst (items d a b x)
        (if (and (or (= (len items) 3) (= (len items) 4))
                 (string=? (mx-procedural-head-of items) "define-syntax"))
            (let ((t (if (= (len items) 4) (nth items 3) (nth items 2)))
                  (sig (if (= (len items) 4) (list (nth items 2)) nil)))
              (tagcase t
                (mx-eager-reader-types-lst (parts d2 a2 b2 x2)
                  (let ((s (style-named (mx-procedural-head-of parts))))
                    (if (and (>= s 0) (= (length parts) 2))
                        (list (tuple (car (cdr items)) s (car (cdr parts)) sig))
                        nil)))
                (else y nil)))
            nil))
      (else y nil))))
;; `(define-syntax name (STYLE proc))`, `form`, defined, its transformer
;; checked; its name (`macros.fx`'s `define-macro`). As `defined` has it, a
;; procedural macro has no signature nor clauses, and makes `3`
;; (`macros-align.fx`'s `makes`); its transformer is `procedurals`'.
(define define-procedural (->* (fails syn def-env) (listof symbol acyclic))
  (lambda (mx-fail form e)
    (let* ((p (car (procedural-form form))) (name (extract p 1)) (code (extract p 3))
           (text (syn-text code))
           ;; Its contract: an expression's or definitions', of untyped parts, yet.
           (signed (if (null? (extract p 4)) nil (list (signature-of mx-fail (car (extract p 4))))))
           (sig (if (null? signed) (s-expr) (extract (car signed) 2))))
      (cond
        ((or (not (syn-symbol? name)) (reserved? (syn-name name)))
         (begin (mx-fail "a macro's name, not a form's" (syn-start name) (syn-end name)) nil))
        ((and (not (null? signed))
              (or (not (null? (extract (car signed) 1))) (= (extract (car signed) 3) 2)))
         (begin (fail-at mx-fail (string-append "`" (string-append (syn-name name) (string-append "`'s contract: a procedural "
                                             (string-append "macro's is an expression's or definitions', of "
                                             "untyped parts, not yet another"))))
                         (car (extract p 4)))
                nil))
        (else
         (tagcase (mx-procedural-run (checked-text (extract p 2) text (get phase1-forms)))
           (ok v
             (begin
               (set procedurals
                    (cons (tuple (syn-symbol name) (extract p 2) text e (not (null? signed))
                                 (get phase1-forms))
                          (get procedurals)))
               ;; With a contract, a macro by its signature (an expression's or definitions'),
               ;; which a template's alignment takes as a `syntax-rules`
               ;; one's; with none, one that makes `3`, whose binding
               ;; structure nothing says.
               (set defined
                    (cons (tuple (syn-symbol name)
                                 (the macro (tuple sig (the clauses nil) (the syns-a nil)
                                                   (if (null? signed) 3 (extract (car signed) 3)))))
                          (get defined)))
               (list (syn-symbol name))))
           (failed why
             (begin (mx-fail (string-append "`" (string-append (syn-name name) (string-append "`'s transformer: " why)))
                          (syn-start code) (syn-end code))
                    nil))))))))
(define def-env-now (ref def-env mxp) (new (def-env-top)))
;;; ------------------------------------------------------------ definition environments

;; Where a macro defined now is defined: in a file's scope, if `file` has
;; one, as its importer has it; else where macros are now (`def-env-now`).
(define def-env-of-file (->* (scopes) def-env)
  (lambda (file)
    (if (null? file)
        (get def-env-now)
        (tuple (if (= (extract (get def-env-now) 1) 3) 3 1) (extract (car file) 1)
               (extract (car file) 2)))))
(define def-env-is (->* () def-env) (lambda () (get def-env-now)))
(define def-env-now! (->* (def-env) unit) (lambda (h) (set def-env-now h)))
(define file-scope-begin! (->* () scope-saved)
  (lambda ()
    (let ((saved (tuple (get defined) (get procedurals) (get def-env-now) (get marks)
                        (get phase1-forms))))
      (begin (set defined nil) (set procedurals nil) (set def-env-now (tuple 2 'within nil))
             (set phase1-forms nil)
             (set marks nil)
             saved))))
(define file-scope-end! (->* (scope-saved) unit)
  (lambda (s)
    (begin (set defined (extract s 1)) (set procedurals (extract s 2))
           (set def-env-now (extract s 3)) (set marks (extract s 4))
           (set phase1-forms (extract s 5)))))
;; Where the macro each expansion's aliases are of was defined, by the
;; expansion's tag (`tag-of`), the last first; a template's check's is `0`
;; (`macros.rs`'s `Withs::def_envs`). Given to the resolution after expansion
;; (`macros-shapes.fx`'s `aliases-resolved`), which another instance of the
;; expander's files reads.
(define mx-procedural-def-envs (ref mx-nodes-def-envs mxp) (new nil))
(define def-env! (->* (int def-env) unit)
  (lambda (k h) (set mx-procedural-def-envs (cons (tuple (mx-fresh-tag-of k) h) (get mx-procedural-def-envs)))))
(define def-envs-given (->* () mx-nodes-def-envs) (lambda () (get mx-procedural-def-envs)))
(define-rec
;; `s`, each part at `a`..`b` of expansion `x`: a hole's call, where the
;; template has the hole.
(respanned (->* (syn int int int) syn)
  (lambda (s a b x)
    (tagcase s
      (mx-eager-reader-types-lst (items d c e y) (mx-eager-reader-types-lst (respanned-each items a b x) d a b x))
      (dotted (items tail d c e y)
        (dotted (respanned-each items a b x) (respanned tail a b x) d a b x))
      (vec (items d c e y) (vec (respanned-each items a b x) d a b x))
      (atom (d c e y) (atom d a b x)))))
(respanned-each (->* (syns-a int int int) syns-a)
  (lambda (xs a b x)
    (if (null? xs) nil (cons (respanned (car xs) a b x) (respanned-each (cdr xs) a b x))))))
(define-rec
;; `s`, each part of it at a key of its own for its place, in expansion
;; `made`.
(in-expansion (->* (syn int) syn)
  (lambda (s made)
    (let ((a (source-key (syn-start s) made)))
      (tagcase s
        (mx-eager-reader-types-lst (items d c e x)
          (let* ((xs (each-in items made)) (b (source-key e made))) (mx-eager-reader-types-lst xs d a b made)))
        (dotted (items tail d c e x)
          (let* ((xs (each-in items made)) (tl (in-expansion tail made)) (b (source-key e made)))
            (dotted xs tl d a b made)))
        (vec (items d c e x)
          (let* ((xs (each-in items made)) (b (source-key e made))) (vec xs d a b made)))
        (atom (d c e x) (atom d a (source-key e made) made))))))
(each-in (->* (syns-a int) syns-a)
  (lambda (xs made)
    (if (null? xs) nil (cons (in-expansion (car xs) made) (each-in (cdr xs) made))))))
(define chain (ref (listof link acyclic) mxp) (new nil))
(define chain-depth (ref int mxp) (new 0))
;; The uses one form's expansion may expand, and how deep, each in the
;; expansion of the one before (`macros.rs`'s `limits`).
;; The limits of one form's expansion, read as it begins (`macros.rs`'s
;; `limits`): 10000 and 1000, or as `FIXPT_MACRO_USES` and
;; `FIXPT_MACRO_DEPTH` set them.
(define max-uses (ref int mxp) (new 10000))
(define max-depth (ref int mxp) (new 1000))
;; Limit `name`, `USES` or `DEPTH`, as the runtime reads it; -1 where the
;; value set is none, said at `s`.
(define limit-of (->* (fails syn string) int)
  (lambda (mx-fail s name)
    (let* ((a (mx-limit name)) (rest (substring a 1 (string-length a))))
      (if (char=? (string-ref a 0) #\=)
          (mx-parse-nat rest 10)
          (begin (fail-at mx-fail rest s) -1)))))
;; One form's expansion begun, at `s`: its chain empty, its limits read.
;; The uses it may expand, or -1 where a limit set is none.
(define chain-begun! (->* (fails syn) int)
  (lambda (mx-fail s)
    (let* ((u (limit-of mx-fail s "USES")) (d (if (< u 0) -1 (limit-of mx-fail s "DEPTH")))
           (r (if (< d 0) -1 (limit-of mx-fail s "RECHECK")))
           (c (if (< r 0) -1 (limit-of mx-fail s "CONTRACTS"))))
      (begin (set chain nil) (set chain-depth 0)
             (set max-uses u) (set max-depth d) (set recheck r) (set mx-procedural-contracts c)
             (if (< c 0) -1 u)))))
(define use-left! (->* () unit)
  (lambda () (begin (set chain (cdr (get chain))) (set chain-depth (- (get chain-depth) 1)))))
(define link-of? (->* (symbol (listof link acyclic)) bool)
  (lambda (n ls)
    (and (not (null? ls)) (or (symbol=? n (extract (car ls) 1)) (link-of? n (cdr ls))))))
(define-rec
;; How many parts syntax `s` has, each list, pair's tail and atom one.
(syn-size (->* (syn) int)
  (lambda (s)
    (tagcase s
      (mx-eager-reader-types-lst (items d a b x) (+ 1 (sizes items)))
      (dotted (items tail d a b x) (+ 1 (+ (sizes items) (syn-size tail))))
      (else y 1))))
(sizes (->* (syns-a) int)
  (lambda (xs) (if (null? xs) 0 (+ (syn-size (car xs)) (sizes (cdr xs)))))))
;; `ls` (innermost first), each use's parts counted once where a use of its
;; macro is around it, -1 where none is.
(define sized (->* ((listof link acyclic)) (listof link acyclic))
  (lambda (ls)
    (if (null? ls)
        nil
        (let ((n (extract (car ls) 1)) (s (extract (car ls) 2)))
          (cons (tuple n s (if (link-of? n (cdr ls)) (syn-size s) -1)) (sized (cdr ls)))))))
;; The macros of the uses inside the one in `ls` that `s` is, of `n` and
;; `size` parts, innermost first, in a list of one; none if it is none.
(define came-back
  (->* (symbol syn int (listof link acyclic) mx-parser-types-names) (listof mx-parser-types-names acyclic))
  (lambda (n s size ls passed)
    (cond ((null? ls) nil)
          ((and (symbol=? n (extract (car ls) 1))
                (= size (if (< (extract (car ls) 3) 0) (syn-size (extract (car ls) 2))
                            (extract (car ls) 3)))
                (syn-same? (extract (car ls) 2) s))
           (list passed))
          (else (came-back n s size (cdr ls) (cons (extract (car ls) 1) passed))))))
;; The outermost use of `ls` (innermost first, each sized) that is one of
;; those around it: it, its macro and the macros passed, in a list of one.
(define first-repeat (->* ((listof link acyclic)) (listof (tupleof syn symbol mx-parser-types-names) acyclic))
  (lambda (ls)
    (if (null? ls)
        nil
        (let ((outer (first-repeat (cdr ls))))
          (if (not (null? outer))
              outer
              (let* ((n (extract (car ls) 1)) (s (extract (car ls) 2)) (size (extract (car ls) 3))
                     (back (if (< size 0) nil (came-back n s size (cdr ls) nil))))
                (if (null? back) nil (list (tuple s n (car back))))))))))
(define mx-procedural-name-seen? (->* (symbol mx-parser-types-names) bool)
  (lambda (n ns) (and (not (null? ns)) (or (symbol=? n (car ns)) (mx-procedural-name-seen? n (cdr ns))))))
(define each-once (->* (mx-parser-types-names mx-parser-types-names) mx-parser-types-names)
  (lambda (ns seen)
    (cond ((null? ns) seen)
          ((mx-procedural-name-seen? (car ns) seen) (each-once (cdr ns) seen))
          (else (each-once (cdr ns) (cons (car ns) seen))))))
(define names-said (->* (mx-parser-types-names) string)
  (lambda (ns)
    (if (null? (cdr ns))
        (string-append "`" (string-append (symbol->string (car ns)) "`"))
        (string-append "`" (string-append (symbol->string (car ns)) (string-append "`, " (names-said (cdr ns))))))))
;; `, through `a`, `b``: the macros passed, outermost first, each once.
(define through-said (->* (mx-parser-types-names) string)
  (lambda (ns)
    (if (null? ns) "" (string-append ", through " (names-said (reverse (each-once ns nil)))))))
;; A limit reached at use `s` of `n`, inside `ls`: `why`, unless the chain
;; says better, its outermost use that is, part by part, one of the same
;; macro it is inside: "this use of `m` is one it was expanded from,
;; unchanged[, through …]", at that use (`Macros::limit_error`). #f.
(define limit-failed (->* (fails symbol syn (listof link acyclic) string) bool)
  (lambda (mx-fail n s ls why)
    (let ((r (first-repeat (sized (cons (tuple n s -1) ls)))))
      (begin
        (if (null? r)
            (fail-at mx-fail why s)
            (fail-at mx-fail (string-append "this use of `" (string-append (symbol->string (extract (car r) 2))
                                         (string-append "` is one it was expanded from, unchanged"
                                         (string-append (through-said (extract (car r) 3))
                                         ": its expansion would never end"))))
                     (extract (car r) 1)))
        #f))))
;; `, inside `a`, `b``: the macros of the `k` innermost uses of `ls`.
(define inside-said (->* ((listof link acyclic) int string) string)
  (lambda (ls k out)
    (if (or (null? ls) (= k 0))
        out
        (inside-said (cdr ls) (- k 1)
                     (string-append out (string-append (if (string=? out "") ", inside " ", ") (string-append "`"
                                    (string-append (symbol->string (extract (car ls) 1)) "`"))))))))
;; A use of `n`, `s`, about to be expanded, `fuel` uses left: entered in the
;; chain; or refused where a limit is reached, the fuel spent or the depth,
;; saying why (`limit-failed`). Nothing more is looked at here
;; (`Macros::enter`, `TODO.md` §231). Whether it is entered.
(define use-entered? (->* (fails symbol syn int) bool)
  (lambda (mx-fail n s fuel)
    (let ((ls (get chain)) (shown (string-append "`" (string-append (symbol->string n) "`"))))
      (cond
        ((= fuel 0)
         (limit-failed mx-fail n s ls (string-append "this macro's expansion does not end: "
                                                  (string-append (int->string (get max-uses))
                                                  (string-append " uses expanded in one form, the last of "
                                                  (string-append shown (inside-said ls 4 "")))))))
        ((= (get chain-depth) (get max-depth))
         (limit-failed mx-fail n s ls
                       (string-append "this macro's expansion does not end: its uses are "
                                      (string-append (int->string (get max-depth)) (string-append " deep, each in the "
                                      (string-append "expansion of the one before, the last of " (string-append shown
                                      (inside-said ls 4 ""))))))))
        (else
         (begin (set chain (cons (tuple n s -1) ls))
                (set chain-depth (+ (get chain-depth) 1))
                #t))))))

;;; ============================================================ macros.fx (whole)

;;; ------------------------------------------------------------ clauses

(define mx-top-clause-usage string "a clause is `((_ . pattern) template)`")
(define expr-kind? (->* (sty) bool)
  (lambda (t) (= (mx-types-kind-of t) 0)))
;; Each typed hole's contract, its type read as a use reads the template's
;; types; none where it names the macro's own binders, which no use can
;; write.
(define contracts-of (->* (pvars mx-parser-types-names scopes) mx-nodes-contracts)
  (lambda (ps binders file)
    (if (null? ps)
        nil
        (let* ((n (extract (car ps) 1)) (info (extract (car ps) 2)) (h (extract info 4))
               (rest (contracts-of (cdr ps) binders file)))
          (if (or (null? h) (null? (extract (car h) 1)) (not (expr-kind? (extract info 1)))
                  (names-any? (car (extract (car h) 1)) binders))
              rest
              (let* ((ty (car (extract (car h) 1))) (a (syn-start ty)) (b (syn-end ty))
                     (m (cond ((null? file) "#%toplevel")
                              ((and (syn-symbol? ty)
                                    (not (mx-files-name-in? (syn-symbol ty) (extract (car file) 3))))
                               "#%fx")
                              (else (symbol->string (extract (car file) 1))))))
                (cons (tuple n (mk-list (list (mk-symbol "select" a b) (mk-symbol m a b) ty) a b))
                      rest)))))))
;; Where clause `c`'s macro is defined (`def-env-of-file`): refused where a
;; module file uses its own macro with a typed hole, whose contract is read
;; by a qualifier, which names no file's own items.
(define clause-def-env (->* (fails syn pvars syns-a scopes) def-env)
  (lambda (mx-fail c ps poly file)
    (let ((e (def-env-of-file file)) (cs (contracts-of ps (binder-names poly) file)))
      (begin
        (if (and (= (extract e 1) 2) (not (null? cs)))
            (fail-at mx-fail (string-append "`" (string-append (symbol->string (extract (car cs) 1)) (string-append "`: a typed "
                                         "hole in a macro a file uses itself, not yet")))
                     c)
            #u)
        e))))
(define stub (->* (syn) clause)
  (lambda (c)
    (tuple c c (the actions nil) (the pvars nil) (the mx-nodes-contracts nil) (def-env-top)
           (the marked nil) (the (listof int acyclic) nil))))
;; What pattern variable `at` is, as the walk found it (`f`).
(define pinfo-at (->* (int (tupleof int mx-env-indexed-env bool int)) pinfo)
  (lambda (at f)
    (tuple (node-core at) (env-levels (extract f 2)) (extract f 3) (node-hole at) (extract f 4))))
(define pvars-of (->* (fails mx-nodes-found pvars int) pvars)
  (lambda (mx-fail fs ps defn)
    (if (null? fs)
        ps
        (let ((at (extract (car fs) 1)))
          (tagcase (node-kids at)
            (k-pvar (n)
              (cond
                ((and (> (extract (car fs) 4) 0) (not (null? (node-hole at))))
                 (begin (fail-at mx-fail "a typed part under an ellipsis: not yet" (node-syn at)) ps))
                ((pvar? n ps)
                  (begin (fail-at mx-fail (string-append "`" (string-append (symbol->string n) (string-append "` is a pattern "
                                                      "variable twice")))
                                  (node-syn at))
                         ps))
                (else
                  (cond
                    ((and (= defn 1) (not (null? (node-hole at))))
                      (begin (fail-at mx-fail (string-append "a macro that makes definitions has no "
                                                          (string-append "typed parts, not yet: what each use "
                                                          "defines is checked where it is"))
                                      (node-syn at))
                             ps))
                    ((and (= defn 2) (not (null? (node-hole at))))
                     (begin (fail-at mx-fail "a macro that makes a description has no typed parts"
                                     (node-syn at))
                            ps))
                    (else
                      (pvars-of mx-fail (cdr fs)
                                (append ps (the pvars (list (tuple n (pinfo-at at (car fs))))))
                                defn))))))
            (else y (pvars-of mx-fail (cdr fs) ps defn)))))))
;; The pattern, aligned with the signature: each variable's type and
;; declared environment. A macro that makes definitions (`defn`) has no
;; typed parts: what each use defines is checked where it is. And what
;; such a macro's uses define, if its signature says (`(macro (export σ β)
;; defn)`), in a list of one: what a template using it sees, and what its
;; own template is held to.
(define pattern-vars (->* (fails sty syn int mx-parser-types-names) (tupleof pvars (listof mx-exports-exported acyclic)))
  (lambda (mx-fail sig pattern defn lits)
    (begin
      (nodes-reset!)
      (let* ((root (align mx-fail 0 (the pvars nil) sig pattern -1 nil))
             (mx-top-said (literals-said mx-fail lits 0))
             (mx-nodes-found (mx-scopes-walk mx-fail root env-empty #f 0))
             (e (node-export root))
             (declared (if (and (= defn 1) (not (null? e)))
                           (the (listof mx-exports-exported acyclic)
                                (list (mx-exports-exports-of (evaluate mx-fail (extract (car e) 1)
                                                            (extract (car e) 2) pattern))))
                           nil)))
        (tuple (pvars-of mx-fail mx-nodes-found nil defn) declared)))))
(define action-at (->* (addr actions) (listof action acyclic))
  (lambda (path as)
    (cond ((null? as) nil)
          ((mx-types-addr=? path (extract (car as) 1)) (list (extract (car as) 2)))
          (else (action-at path (cdr as))))))
(define says-times? (->* ((listof int acyclic) int pvars actions) bool)
  (lambda (ks depth ps acts)
    (and (not (null? ks))
         (or (tagcase (node-kids (car ks))
               (k-pvar (n) (>= (extract (car (pvar-of n ps)) 5) depth))
               (k-ref (n d free)
                 (let ((a (action-at (node-path (car ks)) acts)))
                   (and (not (null? a)) (tagcase (car a) (act-rename (p e) (>= e depth))
                                                 (else z #f)))))
               (else y #f))
             (says-times? (cdr ks) depth ps acts)))))
;; Each part a template repeats, `rs`, with how many ellipses it is under:
;; refused if nothing in it says how many times (`actions`, what is done to
;; its names).
(define mx-top-repeats-said (->* (fails (listof (tupleof int int) acyclic) pvars actions) unit)
  (lambda (mx-fail rs ps acts)
    (cond ((null? rs) #u)
          ((says-times? (under (extract (car rs) 1)) (extract (car rs) 2) ps acts)
           (mx-top-repeats-said mx-fail (cdr rs) ps acts))
          (else (fail-at mx-fail (string-append "`...` follows a part with no pattern variable in "
                                             (string-append "it under as many ellipses in the pattern, nor a "
                                             "binder repeated as deep, to say how many times"))
                         (node-syn (extract (car rs) 1)))))))
;; A reference to `n`: the template's binder of it, if one binds it in
;; `env` (its node, in a list of one), made fresh with it; else, as `free`
;; says (`k-ref`'s), the top level's, a file's own or a standard name, the
;; top level's qualified, or none (written as it is). In a list of one.
(define ref-action
  (->* (fails symbol (listof int acyclic) int scopes syn depths) (listof action acyclic))
  (lambda (mx-fail n b free file s bd)
    (cond ((not (null? b)) (list (act-rename (node-path (car b)) (depth-of (car b) bd))))
          ((= free 2) nil)
          ;; A name a macro's template defined this one with, an alias
          ;; already: what it names where that macro is defined, kept.
          ((and (not (= free 3)) (not (string=? (mx-procedural-alias-base (symbol->string n)) ""))) nil)
          ;; A name in a macro the template defines (`s-rules`): its alias.
          ((= free 4) (list (act-alias)))
          ((= free 3)
           (begin
             (fail-at mx-fail (string-append "`" (string-append (symbol->string n) (string-append "`, the module a template's "
                                          (string-append "`(with module name)` takes from, is bound by the "
                                          "template or given by a use: a top level's, not yet"))))
                      s)
             nil))
          ;; A file's macro's description names each type it writes free as
          ;; the file has it, one name at a time: through the file's module,
          ;; of a type it shows, or the standard environment (`macros.rs`'s).
          ((= free 1) (cond ((null? file) (list (act-alias)))
                            ((mx-files-name-in? n (extract (car file) 3))
                             (list (if (= (extract (def-env-of-file file) 1) 3)
                                       (act-instance-type (extract (car file) 1))
                                       (act-module-type (extract (car file) 1)))))
                            (else (list (act-standard-type)))))
          ((or (null? file) (mx-files-name-in? n (extract (car file) 2)) (mx-files-name-in? n (extract (car file) 5)))
           (list (act-alias)))
          (else
           (begin
             (fail-at mx-fail (string-append "`" (string-append (symbol->string n) (string-append "` is neither a value the file's "
                                          (string-append "module shows nor a standard binding: a file's macro "
                                          "names only those, as they are where it is imported"))))
                      s)
             (list (act-alias)))))))
;; The actions for what the walk found, onto `out`; `uses`, the affine
;; pattern variables placed so far; `bd`, the binders' depths.
(define name-actions (->* (fails pvars mx-nodes-found scopes actions mx-parser-types-names nested depths) actions)
  (lambda (mx-fail ps fs file out mx-scopes-uses nested bd)
    (if (null? fs)
        out
        (let* ((at (extract (car fs) 1)) (mx-env-indexed-env (env-levels (extract (car fs) 2)))
               (many (extract (car fs) 3))
               (depth (extract (car fs) 4)) (s (node-syn at)) (path (node-path at)))
          (tagcase (node-kids at)
            (k-pvar (n)
              (let* ((info (car (pvar-of n ps))) (name (symbol->string n))
                     ;; Anything with an expression in it, placed twice, runs
                     ;; twice: placed once, unless its signature says `many`,
                     ;; or it is a typed hole of no effect, `pure`.
                     (affine (and (not (extract info 3)) (not (pure-hole? (extract info 4)))
                                  (affine-kind? (extract info 1)))))
                (begin
                  (mx-scopes-placed mx-fail n info (node-core at) mx-env-indexed-env s ps)
                  (if (< depth (extract info 5))
                      (fail-at mx-fail (string-append "`" (string-append name (string-append "` is placed under " (string-append (int->string depth)
                                                   (string-append " ellipses, and the pattern has it under "
                                                   (int->string (extract info 5)))))))
                               s)
                      #u)
                  (if (and affine (> depth (extract info 5)))
                      (fail-at mx-fail (string-append "`" (string-append name (string-append "` is placed under more ellipses than "
                                                   (string-append "the pattern has it, so in each repetition, and "
                                                   (string-append "an expression in it may do something each "
                                                   (string-append "time it runs: its signature says `(many …)` "
                                                   "if it may"))))))
                               s)
                      #u)
                  (if (and affine many)
                      (fail-at mx-fail (string-append "`" (string-append name (string-append "` is placed where it may be placed "
                                                   (string-append "more than once (a `many` part), and an "
                                                   (string-append "expression in it may do something each time "
                                                   (string-append "it runs: its signature says `(many …)` if it "
                                                   "may"))))))
                               s)
                      #u)
                  (if (and affine (mx-files-name-in? n mx-scopes-uses))
                      (fail-at mx-fail (string-append "`" (string-append name (string-append "` is placed twice, and an expression "
                                                   (string-append "in it may do something each time it runs: its "
                                                   "signature says `(many …)` if it may"))))
                               s)
                      #u)
                  (name-actions mx-fail ps (cdr fs) file out
                                (if affine (the mx-parser-types-names (cons n mx-scopes-uses)) mx-scopes-uses) nested bd))))
            (k-binder (n d)
              (name-actions mx-fail ps (cdr fs) file (cons (tuple path (act-rename path depth)) out)
                            mx-scopes-uses nested bd))
            (k-ref (n d free)
              (let ((b (mx-scopes-binder-of mx-fail n d mx-env-indexed-env s)))
                (begin
                  (if d #u (unsaid-nested mx-fail n b nested s))
                  (repeated-ref mx-fail n b depth bd s)
                  (let ((act (ref-action mx-fail n b free file s bd)))
                    (name-actions mx-fail ps (cdr fs) file
                                  (if (null? act) out (cons (tuple path (car act)) out)) mx-scopes-uses
                                  nested bd)))))
            (else y (name-actions mx-fail ps (cdr fs) file out mx-scopes-uses nested bd)))))))
(define form-head-of (->* (int) (listof symbol acyclic))
  (lambda (i)
    (let ((items (proper (node-syn i))))
      (if (and (not (null? items)) (syn-symbol? (car items)))
          (list (syn-symbol (car items)))
          nil))))
;; From node `i` to `n`, a file's macro's use of another of its own: that
;; macro, by the name the importer has it by.
(define type-actions (->* (fails int int scopes actions) actions)
  (lambda (mx-fail i n file out)
    (if (= i n)
        out
        (let* ((path (node-path i)) (mx-nodes-kids (node-kids i)) (head (form-head-of i))
               (s (node-syn i))
               ;; A trailmark of the file's: by the name the importer has it by.
               (out2 (if (and (not (null? file)) (syn-symbol? s)
                              (or (= (mx-types-kind-of (node-core i)) 15) (= (mx-types-kind-of (node-core i)) 16))
                              (not (null? (internal-of (syn-symbol s) (extract (car file) 4)))))
                         (cons (tuple path (act-mark (car (internal-of (syn-symbol s)
                                                                       (extract (car file) 4)))))
                               out)
                         out)))
          (type-actions mx-fail (+ i 1) n file
                        (if (and (not (null? file)) (tagcase mx-nodes-kids (k-form (x) #t) (else y #f))
                                 (not (null? head))
                                 (not (null? (internal-of (car head) (extract (car file) 4)))))
                            (let ((h (car (internal-of (car head) (extract (car file) 4)))))
                              (cons (tuple path (act-macro-head h)) out2))
                            out2))))))
;; The template, an expression or definitions (`defn`), aligned with the
;; kernel's forms' signatures and the macros', its pattern variables placed
;; by the rules, and what is done to each of its names where it is
;; transcribed.
(define template-actions
  (->* (fails pvars syn int (listof mx-exports-exported acyclic) scopes) actions)
  (lambda (mx-fail ps template defn declared file)
    (begin
      (set heads nil)
      (set heads-on (uses-syntax-macro? template))
      (let* ((made (cond ((= defn 1) (s-defn)) ((= defn 2) (s-type)) (else (s-expr))))
             (r (aligned-heads mx-fail ps made template)) (root (extract r 1)) (mx-nodes-found (extract r 2))
             (done (begin (set heads-on #f) (heads-done mx-fail (extract r 3) (binder-depths mx-nodes-found))))
             (nested (nested-binders 0 (get nnodes) ps))
             (named (append done (name-actions mx-fail ps mx-nodes-found file nil nil nested
                                               (binder-depths mx-nodes-found)))))
        (begin
          ;; Each part a template repeats says how many times: a pattern
          ;; variable in it that the pattern has under as many ellipses, or
          ;; a reference to a binder of the template's repeated as deep.
          (mx-top-repeats-said mx-fail (the (listof (tupleof int int) acyclic) (reverse (get seq-depths)))
                        ps named)
          ;; What the template defines of what its uses give, what its
          ;; signature says.
          (if (null? declared)
              #u
              (let ((made (mx-exports-exports-of (levels-of mx-fail root template))))
                (if (exports=? made (car declared))
                    #u
                    (fail-at mx-fail (string-append "this template defines " (string-append (shown-exports made)
                                                 (string-append ", and its signature says its uses define "
                                                 (shown-exports (car declared)))))
                             template))))
          (type-actions mx-fail 0 (get nnodes) file named))))))
;; Clause `c` checked against signature `sig` (Herman's rules), in a
;; file's scope if `file` has one: its pattern fits the signature, each of
;; the pattern's variables is placed in the template where the signature
;; says what is in scope there, and a reference to a pattern's binder is
;; where no other pattern binder may shadow it. `lits`, its `syntax-rules`'
;; literals.
(define check-clause (->* (fails syn sty int syns-a mx-parser-types-names scopes symbol) clause)
  (lambda (mx-fail c sig defn poly lits file name)
    (let ((two (proper c)))
      (if (not (= (len two) 2))
          (begin (fail-at mx-fail mx-top-clause-usage c) (stub c))
          (let ((whole (split (car two))))
            (if (null? whole)
                (begin (fail-at mx-fail mx-top-clause-usage (car two)) (stub c))
                (let* ((pattern (nth whole 1))
                       (pv (pattern-vars mx-fail sig pattern defn lits)) (ps (extract pv 1))
                       (template (nth two 1))
                       (acts (template-actions mx-fail ps template defn (extract pv 2) file)))
                  (tuple pattern template acts ps (contracts-of ps (binder-names poly) file)
                         (clause-def-env mx-fail c ps poly file) (mx-match-marks-of lits)
                         (checked-uses name sig ps)))))))))
(define unsigned-of (->* (writtens) unsigneds)
  (lambda (ws)
    (cond ((null? ws) nil)
          ((null? (extract (car ws) 2))
           (cons (tuple (extract (car ws) 1) (extract (car ws) 3) (extract (car ws) 4)
                        (extract (car ws) 5))
                 (unsigned-of (cdr ws))))
          (else (unsigned-of (cdr ws))))))
;; `s` read as a signature, none if it is not one.
(define tried-sigs (ref (listof signed acyclic) mxp) (new nil))
(define tried-sig (->* (syn) (listof signed acyclic))
  (lambda (s)
    (let ((r (prompt parse-tag (begin (set tried-sigs (list (signature-of pfail-at s))) (p-ok nil))
                     (lambda (r) r))))
      (tagcase r (p-err (m x y) (the (listof signed acyclic) nil)) (p-ok (ts) (get tried-sigs))))))
;; Unsigned macro `w`'s clauses' shapes whole, as a signature that is one
;; (`by-clauses`); none if not.
(define alt-sig (->* (mx-files-written) syns-a)
  (lambda (w)
    (let ((alt (by-clauses (tuple (extract w 1) (extract w 3) (extract w 4) (extract w 5)))))
      (if (or (null? alt) (null? (tried-sig (car alt)))) nil alt))))
;; Each signature, the written one's, or the inferred one read, with how
;; it is said where its macro is refused.
(define inferred-each
  (->* (fails writtens (listof signed-opt acyclic) syns-a) (listof mx-top-said acyclic))
  (lambda (mx-fail ws mx-files-written inferred)
    (cond
      ((null? ws) nil)
      ((not (null? (car mx-files-written)))
       (cons (tuple (car mx-files-written) (the (listof string acyclic) nil))
             (inferred-each mx-fail (cdr ws) (cdr mx-files-written) inferred)))
      (else
       (let* ((n (symbol->string (extract (car ws) 1))) (cs (extract (car ws) 4))
              ;; Not a signature: the union of its clauses' shapes whole,
              ;; if that is one (`macros.rs`'s).
              (alt (if (null? (tried-sig (car inferred))) (alt-sig (car ws)) nil))
              (sig (if (null? alt) (car inferred) (car alt)))
              (shown (sig-shown cs sig))
              (s (signature-of (said-fail mx-fail n (sig-shown cs (car inferred))) sig)))
         (begin
           (set defined (cons (tuple (extract (car ws) 1) (tuple (extract s 2) (the clauses nil)
                                                                  (the syns-a nil) (extract s 3)))
                              (get defined)))
           (cons (tuple (the signed-opt (list s)) (the (listof string acyclic) (list shown)))
                 (inferred-each mx-fail (cdr ws) (cdr mx-files-written) (cdr inferred)))))))))
;; `w`'s clauses checked against signature `s`, none if one does not fit.
(define tried-checks (ref (listof clauses acyclic) mxp) (new nil))
(define check-each (->* (fails syns-a sty int syns-a mx-parser-types-names scopes symbol) clauses)
  (lambda (mx-fail cs sig defn poly lits file name)
    (if (null? cs)
        nil
        (let ((c (check-clause mx-fail (car cs) sig defn poly lits file name)))
          (cons c (check-each mx-fail (cdr cs) sig defn poly lits file name))))))
(define tried-check (->* (mx-files-written signed scopes) (listof clauses acyclic))
  (lambda (w s file)
    (let ((r (prompt parse-tag
                     (begin (set tried-checks (list (check-each pfail-at (extract w 4) (extract s 2)
                                                               (extract s 3) (extract s 1)
                                                               (extract w 3) file (extract w 1))))
                            (p-ok nil))
                     (lambda (r) r))))
      (tagcase r (p-err (m x y) (the (listof clauses acyclic) nil))
        (p-ok (ts) (get tried-checks))))))
;; Unsigned `w` defined anew by its clauses' shapes whole, if its clauses
;; fit that signature; none, and as it was, if not.
(define refit (->* (mx-files-written scopes) (listof signed acyclic))
  (lambda (w file)
    (let* ((alt (alt-sig w)) (s (if (null? alt) nil (tried-sig (car alt)))) (was (get defined)))
      (if (null? s)
          nil
          (begin
            (set defined (cons (tuple (extract w 1) (tuple (extract (car s) 2) (the clauses nil)
                                                           (the syns-a nil) (extract (car s) 3)))
                               (get defined)))
            (if (null? (tried-check w (car s) file)) (begin (set defined was) nil) s))))))
(define checked-each (->* (fails writtens (listof mx-top-said acyclic) scopes) macro-env)
  (lambda (mx-fail ws sigs file)
    (if (null? ws)
        nil
        (let* ((w (car ws)) (s0 (car (extract (car sigs) 1))) (shown (extract (car sigs) 2))
               (f (if (null? shown) mx-fail (said-fail mx-fail (symbol->string (extract w 1))
                                                    (car shown))))
               (empty (if (null? (extract w 4))
                          (fail-at mx-fail "a macro has at least one clause" (extract w 5))
                          #u))
               ;; Inferred, and its clauses not fitting it: the union of
               ;; their shapes whole, if they fit that (`macros.rs`'s).
               (alt (if (or (null? shown) (not (null? (tried-check w s0 file)))) nil
                        (refit w file)))
               (s (if (null? alt) s0 (car alt)))
               (cs (check-each f (extract w 4) (extract s 2) (extract s 3) (extract s 1)
                               (extract w 3) file (extract w 1))))
          (cons (tuple (extract w 1) (tuple (extract s 2) cs (extract s 1) (extract s 3)))
                (checked-each mx-fail (cdr ws) (cdr sigs) file))))))
;; Each signature, written, or inferred together for those not written,
;; each defined first; then each macro's clauses checked, and defined.
(define define-written (->* (fails writtens scopes macro-env) mx-parser-types-names)
  (lambda (mx-fail ws file before)
    (let* ((mx-files-written (signed-each mx-fail ws))
           (us (unsigned-of ws))
           (inferred (if (null? us) nil (infer mx-fail us)))
           (sigs (inferred-each mx-fail ws mx-files-written inferred))
           (made (checked-each mx-fail ws sigs file)))
      (begin
        (set defined (append made before))
        (hide-marks! (written-names ws))
        (written-names ws)))))
;;; ------------------------------------------------------------ defining

;; `(define-syntax name [σ] (syntax-rules (literal …) ((_ . pattern)
;; template) …))`, its signature written or inferred, or a
;; `define-syntax-rec` of them, checked, in a file's scope if `file` has
;; one, and defined: their names. A template's free name is the top level's
;; where the macro is used (`act-toplevel`). Each defined first, with its
;; signature, so that a template may use it or another of its
;; `define-syntax-rec`; what was defined before put back if one fails.
(define define-macro (->* (fails syn scopes) mx-parser-types-names)
  (lambda (mx-fail form file)
    (let* ((before (get defined))
           (undo (lambda ((m string) (a int) (b int))
                   (begin (set defined before) (mx-fail m a b)))))
      (cond ((not (null? (trailmark-form form))) (define-trailmark mx-fail form))
            ((null? (procedural-form form))
             (define-written undo (mx-files-written-of undo form) file before))
            (else (define-procedural mx-fail form (def-env-of-file file)))))))
;;; ------------------------------------------------------------ the session

(define loads (ref load-defs mxp) (new nil))
(define by-state (ref by-states mxp) (new nil))
;; The globals the forms read so far define, and those the forms before
;; this parse did (`Checker::global_type` as a program is expanded, before
;; any of its own is checked).
(define mx-top-globals (ref mx-parser-types-names mxp) (new nil))
(define globals-before (ref mx-parser-types-names mxp) (new nil))
(define state-at (->* (int by-states) by-states)
  (lambda (s xs)
    (cond ((null? xs) nil)
          ((= s (extract (car xs) 1)) (the by-states (list (car xs))))
          (else (state-at s (cdr xs))))))
(define macros-at! (->* (int) unit)
  (lambda (s)
    (let ((at (state-at s (get by-state))))
      (begin
        (set with-uses nil)
        (if (null? at)
            (begin (set defined nil) (set loads nil) (set mx-top-globals nil) (seen! nil)
                   (set marks nil) (phase1-state! (tuple nil nil)))
            (begin (set defined (extract (car at) 2)) (set loads (extract (car at) 3))
                   (set mx-top-globals (extract (car at) 4)) (seen! (extract (car at) 5))
                   (set marks (extract (car at) 6)) (phase1-state! (extract (car at) 7))))
        (set globals-before (get mx-top-globals))))))
(define macros-after! (->* (int) unit)
  (lambda (s)
    (if (and (null? (get defined)) (null? (get loads)) (null? (get mx-top-globals))
             (null? (seen-now)) (null? (get marks)) (phase1-none?))
        #u
        (set by-state (cons (tuple s (get defined) (get loads) (get mx-top-globals) (seen-now)
                                   (get marks) (phase1-state-now))
                            (get by-state))))))
;; Names `ns` defined as globals.
(define globals! (->* (mx-parser-types-names) unit) (lambda (ns) (set mx-top-globals (append ns (get mx-top-globals)))))
;; Whether `n` was a global before this parse; or is one, of those so far.
(define global-before? (->* (symbol) bool) (lambda (n) (mx-files-name-in? n (get globals-before))))
(define global-now? (->* (symbol) bool)
  (lambda (n) (or (global-before? n) (mx-files-name-in? n (get mx-top-globals)))))
(define expansions (ref expansions-made mxp) (new nil))
(define nexpansions (ref int mxp) (new 0))
(define record-expansion (->* (symbol syn bool) int)
  (lambda (name s typed)
    (let ((n (+ (get nexpansions) 1)))
      (begin
        (set nexpansions n)
        (set expansions (cons (tuple n name (syn-start s) (syn-end s) typed) (get expansions)))
        (expansion-begun! n)
        n))))
;; Whether a clause of pattern variables `ps` has a typed hole: checked once.
(define holed? (->* (pvars) bool)
  (lambda (ps)
    (and (not (null? ps)) (or (not (null? (extract (extract (car ps) 2) 4))) (holed? (cdr ps))))))
;; For the driver, which says an error in an expansion at each use.
(define expansions-given (->* () expansions-made) (lambda () (get expansions)))
;; The expansions left before one does not end.
(define budget (ref int mxp) (new 0))
;; The pattern variables of `ps` that are names (`bvar`, `dvar`, `id`),
;; which match a name alone (a `label` may be a keyword).
(define name-pvars (->* (pvars) mx-parser-types-names)
  (lambda (ps)
    (cond ((null? ps) nil)
          ((let ((k (mx-types-kind-of (extract (extract (car ps) 2) 1))))
             (and (name-kind? k) (not (= k 5))))
           (cons (extract (car ps) 1) (name-pvars (cdr ps))))
          (else (name-pvars (cdr ps))))))
(define mx-top-fresh (ref freshes mxp) (new nil))
(define mx-top-counts (ref mx-top-counts mxp) (new nil))
(define iters (ref (listof int acyclic) mxp) (new nil))
(define-rec
;; Type `t`, which a template writes, read in module `m` (`#%toplevel`,
;; `#%fx` or a file's), in expansion `made`: `(select m t)`; of a
;; definition's `(->* (param …) result)`, which is no type, each of its
;; types so (`macros.rs`'s `selected`).
(selected (->* (symbol syn int) syn)
  (lambda (m t made)
    (let ((items (proper t)) (a (source-key (syn-start t) made)))
      (if (and (= (len items) 3) (syn-symbol? (car items)) (string=? (syn-name (car items)) "->*")
               (or (syn-nil? (nth items 1)) (not (null? (proper (nth items 1))))))
          (let* ((h (in-expansion (car items) made)) (params (nth items 1))
                 (pa (source-key (syn-start params) made))
                 (ps (selected-each m (proper params) made))
                 (pb (source-key (syn-end params) made))
                 (r (selected m (nth items 2) made)) (b (source-key (syn-end t) made)))
            (list-at a b made (list h (list-at pa pb made ps nil) r) nil))
          (let* ((ty (in-expansion t made)) (b (source-key (syn-end t) made)))
            (list-at a b made (list (atom 'select a b made) (atom m a b made) ty) nil))))))
(selected-each (->* (symbol syns-a int) syns-a)
  (lambda (m ts made)
    (if (null? ts) nil (cons (selected m (car ts) made) (selected-each m (cdr ts) made))))))
;; `n` `#f`s, onto `out`.
(define mx-top-falses (->* (int addr) addr)
  (lambda (n out) (if (= n 0) out (mx-top-falses (- n 1) (cons #f out)))))
(define bind-nth (->* (mx-parser-types-names (listof syns-a acyclic) int mx-match-binds) mx-match-binds)
  (lambda (ns ls r bs)
    (if (null? ns)
        bs
        (bind-nth (cdr ns) (cdr ls) r (cons (tuple (car ns) (nth (car ls) r)) bs)))))
(define driver-lists (->* (mx-parser-types-names mx-match-binds) (listof syns-a acyclic))
  (lambda (ns bs)
    (if (null? ns)
        nil
        (let ((b (car (bound-of (car ns) bs))))
          (cons (if (syn-nil? b) (the syns-a nil) (proper-items b)) (driver-lists (cdr ns) bs))))))
(define count-of (->* (addr (listof int acyclic) mx-top-counts) int)
  (lambda (b is cs)
    (cond ((null? cs) -1)
          ((and (mx-types-addr=? b (extract (car cs) 1))
                (ints=? is (extract (car cs) 2)))
           (extract (car cs) 3))
          (else (count-of b is (cdr cs))))))
;; The counts known of the binders `under`, as repeated in `is`.
(define known-counts (->* ((listof (tupleof addr bool) acyclic) (listof int acyclic))
                          (listof int acyclic))
  (lambda (under is)
    (if (null? under)
        nil
        (let ((k (count-of (extract (car under) 1) is (get mx-top-counts))))
          (if (< k 0) (known-counts (cdr under) is) (cons k (known-counts (cdr under) is)))))))
;; Each binder of `under` that is the binder itself repeated `n` times here.
(define note-counts (->* ((listof (tupleof addr bool) acyclic) int) unit)
  (lambda (under n)
    (if (null? under)
        #u
        (begin
          (if (extract (car under) 2)
              (set mx-top-counts (cons (tuple (extract (car under) 1) (get iters) n) (get mx-top-counts)))
              #u)
          (note-counts (cdr under) n)))))
(define mx-top-fresh-of (->* (addr (listof int acyclic) freshes) (listof symbol acyclic))
  (lambda (p is fs)
    (cond ((null? fs) nil)
          ((and (mx-types-addr=? p (extract (car fs) 1))
                (ints=? is (extract (car fs) 2)))
           (list (extract (car fs) 3)))
          (else (mx-top-fresh-of p is (cdr fs))))))
;; The fresh name of the binder at `binder`, under `d` ellipses, in the
;; repetitions it is in, made the first time.
(define fresh-name (->* (symbol addr int) symbol)
  (lambda (n binder d)
    (let* ((is (ints-prefix (get iters) d)) (known (mx-top-fresh-of binder is (get mx-top-fresh))))
      (if (null? known)
          (let ((f (string->symbol (string-append "%" (string-append (symbol->string n) (string-append ":" (fresh-tag)))))))
            (begin (set mx-top-fresh (cons (tuple binder is f) (get mx-top-fresh))) f))
          (car known)))))
;; Each typed hole's contract, by its pattern variable, in a list of one.
(define contract-of (->* (symbol mx-nodes-contracts) syns-a)
  (lambda (n cs)
    (cond ((null? cs) nil)
          ((symbol=? n (extract (car cs) 1)) (list (extract (car cs) 2)))
          (else (contract-of n (cdr cs))))))
;; Whether a template is being transcribed for its check, its holes' calls
;; in place of a use's expressions (`template-checks`).
(define checking (ref bool mxp) (new #f))
(define transcribe-name
  (->* (clause syn symbol int int (listof action acyclic) mx-match-binds int) syn)
  (lambda (c t n a b act bs made)
    (let ((given (bound-of n bs)) (contract (contract-of n (extract c 5))))
      (if (not (null? given))
          ;; A typed hole's expression, held to its type; in a template's
          ;; check, its call, where the template has it.
          (cond ((get checking) (respanned (car given) a b made))
                ((null? contract) (car given))
                (else
                 (let* ((g (car given)) (ga (syn-start g)) (gb (syn-end g)) (gx (syn-expansion g)))
                   (list-at ga gb gx
                            (list (atom 'the ga gb gx) (in-expansion (car contract) made) g) nil))))
          (if (null? act)
              (atom n a b made)
              (tagcase (car act)
                (act-rename (binder d) (atom (fresh-name n binder d) a b made))
                (act-alias ()
                  (atom (string->symbol (mx-procedural-alias (symbol->string n) made)) a b made))
                (act-mark (i) (atom i a b made))
                (else y (atom n a b made))))))))
(define-rec
(transcribe-at (->* (fails clause syn addr mx-match-binds int) syn)
  (lambda (mx-fail c t path bs made)
    (let* ((ta (syn-start t)) (tb (syn-end t)) (act (action-at path (extract c 3)))
           (typed (if (null? act)
                      ""
                      (tagcase (car act)
                        (act-standard-type () "#%fx")
                        (act-module-type (m) (symbol->string m))
                        (act-instance-type (m) (mx-procedural-alias (symbol->string m) made))
                        (else y "")))))
      (cond
        ((not (string=? typed "")) (selected (string->symbol typed) t made))
        ((syn-symbol? t)
         (let* ((a (source-key ta made)) (b (source-key tb made)))
           (transcribe-name c t (syn-symbol t) a b act bs made)))
        ;; `(... ...)`, the ellipsis of a macro the template defines.
        ((tagcase t
           (mx-eager-reader-types-lst (xs d a b x) (and (= (len xs) 2) (ellipsis? (car xs)) (ellipsis? (nth xs 1))))
           (else y #f))
         (atom (string->symbol "...") (source-key ta made) (source-key tb made) made))
        ((not (null? (list-items t)))
         (let* ((a (source-key ta made)) (tr (trust-if! c ta a made))
                (xs (transcribe-items mx-fail c (list-items t) path bs made))
                (rest (tagcase t
                        (dotted (items tail d c2 e x)
                          (let ((at (append path (mx-top-falses (len items) nil))))
                            (list (transcribe-at mx-fail c tail at bs made))))
                        (else y nil)))
                (head (if (null? act)
                          nil
                          (tagcase (car act) (act-macro-head (h) (list h)) (else y nil))))
                (xs2 (if (null? head)
                         xs
                         (cons (atom (car head) (syn-start (car xs)) (syn-end (car xs))
                                     (syn-expansion (car xs)))
                               (cdr xs))))
                (b (source-key tb made)))
           (list-at a b made xs2 rest)))
        (else (let* ((a (source-key ta made)) (b (source-key tb made))) (at-place t a b made)))))))
;; The `r`th to the `n`th repetitions of `x`, `drivers` bound to each's
;; element of `lists`, onto `out` (the last made first).
(repeat-each
  (->* (fails clause syn addr mx-match-binds int mx-parser-types-names (listof syns-a acyclic) int int syns-a) syns-a)
  (lambda (mx-fail c x p bs made drivers lists r n out)
    (if (= r n)
        (the syns-a (reverse out))
        (let* ((each (bind-nth drivers lists r bs))
               (was (get iters))
               (one (begin (set iters (append was (the (listof int acyclic) (list r))))
                           (transcribe-at mx-fail c x p each made))))
          (begin (set iters was)
                 (repeat-each mx-fail c x p bs made drivers lists (+ r 1) n (cons one out)))))))
;; The items of a template's list, the `k`th at `path`, `k` `#f`s and a `#t`
;; past it.
(transcribe-items (->* (fails clause syns-a addr mx-match-binds int) syns-a)
  (lambda (mx-fail c xs path bs made)
    (cond
      ((null? xs) nil)
      ;; `p ...`: `p` once for each element of what the use gave of the
      ;; pattern variables in it under more ellipses than this, together.
      ((and (not (null? (cdr xs))) (ellipsis? (car (cdr xs))))
       (let ((reps (mx-top-repeated mx-fail c (car xs) (append path (the addr (list #t))) bs made)))
         (append reps (transcribe-items mx-fail c (cdr (cdr xs)) (append path (the addr (list #f #f)))
                                        bs made))))
      (else
       (let ((first (transcribe-at mx-fail c (car xs) (append path (the addr (list #t))) bs made)))
         (cons first (transcribe-items mx-fail c (cdr xs) (append path (the addr (list #f))) bs
                                       made)))))))
;; `x`, at `p`, repeated: as many times as every count it is repeated by
;; says, the pattern variables' and the repeated binders' it refers to.
(mx-top-repeated (->* (fails clause syn addr mx-match-binds int) syns-a)
  (lambda (mx-fail c x p bs made)
    (let* ((level (mx-match-len-ints (get iters)))
           (drivers (drivers-of (extract c 4) x level))
           (lists (driver-lists drivers bs))
           (odd (odd-length lists 0))
           (under (renames-under (extract c 3) p level))
           (known (append (known-counts under (get iters))
                          (if (null? lists) (the (listof int acyclic) nil)
                              (list (len (car lists)))))))
      (cond
        ((>= odd 0)
         (begin (fail-at mx-fail (string-append "`" (string-append (symbol->string (car drivers)) (string-append "` and `"
                                             (string-append (symbol->string (mx-match-nth-name drivers odd))
                                             (string-append "`, repeated together, are given "
                                             (string-append (int->string (len (car lists))) (string-append " and "
                                             (string-append (int->string (len (nth-list lists odd))) " times"))))))))
                         x)
                nil))
        ((null? known)
         (begin (fail-at mx-fail (string-append "this part is repeated before the repetition that "
                                             "binds what it refers to")
                         x)
                nil))
        ((not (all-ints? known (car known)))
         (begin (fail-at mx-fail (string-append "this part repeats binders and parts that were "
                                             "repeated, or given, different numbers of times")
                         x)
                nil))
        (else
         (begin (note-counts under (car known))
                (repeat-each mx-fail c x p bs made drivers lists 0 (car known) nil))))))))
;; Template `t` of clause `c`, at `path` in it, transcribed by expansion
;; `made`: each part the template makes at its own place in the
;; `define-syntax`, of that expansion, each pattern variable's what the
;; use gave.
(define transcribe (->* (fails clause syn addr mx-match-binds int) syn)
  (lambda (mx-fail c t path bs made)
    (begin
      (if (null? path) (begin (set mx-top-fresh nil) (set mx-top-counts nil) (set iters nil)) #u)
      (transcribe-at mx-fail c t path bs made))))
(define first-match (->* (fails symbol syn syn clauses) syn)
  (lambda (mx-fail name s operands cs)
    (if (null? cs)
        (begin (fail-at mx-fail (string-append "a use of `" (string-append (symbol->string name) (string-append "` that no clause "
                                            "of it matches")))
                        s)
               s)
        (let ((mx-match-binds (matched (extract (car cs) 1) operands (name-pvars (extract (car cs) 4))
                              (extract (car cs) 7) nil)))
          (if (null? mx-match-binds)
              (first-match mx-fail name s operands (cdr cs))
              (let* ((made (record-expansion name s (holed? (extract (car cs) 4))))
                     (shown (symbol->string name))
                     (noted (def-env! made (extract (car cs) 6)))
                     (in-use (lambda ((msg string) (a int) (b int))
                               (mx-fail (string-append "a use of `" (string-append shown (string-append "`: " msg)))
                                     (syn-start s) (syn-end s)))))
                (transcribe in-use (car cs) (extract (car cs) 2) nil (car mx-match-binds) made)))))))
;; A use of macro `name`, `s`, parsed by its signature, then the first
;; clause whose pattern matches it transcribed.
(define expand-use (->* (fails symbol syn) syn)
  (lambda (mx-fail name s)
    (let* ((m (car (macro-of name (get defined))))
           (operands (nth (split s) 1))
           (shown (symbol->string name))
           (in-use (lambda ((msg string) (a int) (b int))
                     (mx-fail (string-append "a use of `" (string-append shown (string-append "`: " msg))) a b))))
      (if (not (null? (procedural-of name)))
          (let ((made (record-expansion name s #f)))
            (begin (def-env! made (extract (car (procedural-of name)) 3))
                   (procedural-expanded mx-fail name s made)))
          (begin
            (nodes-reset!)
            (use-aligned! in-use s (extract m 1) operands)
            (first-match mx-fail name s operands (extract m 2)))))))
(define-rec
;; `s`, of syntax type `core`, expanded: each use of a macro in it, as its
;; walk finds them, expanded, and what that makes in its turn, each where
;; it is, in what is bound around it (`macros.rs`'s `expand_region`,
;; `TODO.md` §295): a list a macro's name heads is no use where the program
;; wrote the head and a binder around it binds the name. `bound`: the
;; macros' names bound around `s`; `withs`, the `with`s it is in.
(expand-region (->* (fails syn sty mx-parser-types-names mx-files-places) syn)
  (lambda (mx-fail s core bound withs)
    (expand-at mx-fail s (if (names-macro? s) (uses-found mx-fail core s bound) nil) withs)))
;; `s`, where uses `us` are found from (`uses-found`): each use expanded,
;; each path written (`path-written`).
(expand-at (->* (fails syn mx-scopes-uses mx-files-places) syn)
  (lambda (mx-fail s us withs)
    (let* ((items (list-items s)) (hd (if (null? items) s (car items)))
           (head (if (syn-symbol? hd) (syn-name hd) ""))
           (noted (if (string=? head "with") (note-projection s items hd) #u)))
      (cond
        ((not (null? (part-here us)))
         (tagcase (car (part-here us))
           (pt-use (bound) (expand-one mx-fail s (syn-symbol hd) bound withs))
           (pt-path (d) (path-written mx-fail s d))))
        ((null? items) s)
        ((string=? head "quote") s)
        ;; A definition of a macro's name: refused, as a form's is.
        ((and (or (string=? head "define") (string=? head "define*"))
              (not (null? (cdr items))) (syn-symbol? (nth items 1))
              (macro? (syn-symbol (nth items 1))))
         (begin (fail-at mx-fail (string-append "`" (string-append (syn-name (nth items 1)) (string-append "` is a macro's name, "
                                             "which is not defined again")))
                         (nth items 1))
                s))
        (else (rebuilt s (expand-items mx-fail items 0 us (string=? head "with") s withs)
                       (expand-tail mx-fail s (len items) us withs)))))))
;; Use `s` of macro `h` expanded, and what it makes in its turn, `bound` the
;; macros' names bound around it.
(expand-one (->* (fails syn symbol mx-parser-types-names mx-files-places) syn)
  (lambda (mx-fail s h bound withs)
    (let ((hd (car (list-items s))))
      (cond
        ((makes-defns? h)
         (begin (fail-at mx-fail (string-append "`" (string-append (syn-name hd) (string-append "` makes definitions, which are "
                                             "made at the top level, not where an expression is")))
                         s)
                s))
        ((use-entered? mx-fail h s (get budget))
         (begin
           (set budget (- (get budget) 1))
           ;; A name the program wrote (a template's is the macro, by
           ;; hygiene), in a `with`'s body.
           (if (= (syn-expansion hd) 0) (note-uses h hd withs) #u)
           (let ((made (expand-region mx-fail (expand-use mx-fail h s)
                                      (if (makes-desc? h) (s-type) (s-expr)) bound withs)))
             (begin (use-left!) made))))
        (else s)))))
;; Items from the `k`th, each expanded, `us` the uses in them; in a `with`'s
;; body (from the third) noted as in it.
(expand-items (->* (fails syns-a int mx-scopes-uses bool syn mx-files-places) syns-a)
  (lambda (mx-fail xs k us with s withs)
    (if (null? xs)
        nil
        (let ((x (expand-at mx-fail (car xs) (uses-in k us)
                            (if (and with (>= k 2))
                                (cons (tuple (syn-start s) (syn-end s)) withs)
                                withs))))
          (cons x (expand-items mx-fail (cdr xs) (+ k 1) (uses-past k us) with s withs))))))
(expand-tail (->* (fails syn int mx-scopes-uses mx-files-places) syns-a)
  (lambda (mx-fail s k us withs)
    (tagcase s
      (dotted (items t d a b x) (list (expand-at mx-fail t (uses-in (- -1 k) us) withs)))
      (else y nil)))))
(define expand-in (->* (fails syn mx-files-places) syn)
  (lambda (mx-fail s withs) (expand-region mx-fail s (s-item) nil withs)))
;; `form` with each use of a macro in it expanded, as each expands, until
;; none is left; quoted data left as it is. A form of the program's is at
;; the place written for it, whatever a use made of it.
(define expand (->* (fails syn) syn)
  (lambda (mx-fail form)
    (if (and (null? (get defined)) (not (names-macro? form)))
        form
        (let ((n (chain-begun! mx-fail form)))
          (if (< n 0)
              form
              (begin (set budget n)
                     (at-place (expand-in mx-fail form nil) (syn-start form) (syn-end form) 0)))))))
(define-rec
;; `s`, where a definition is: a use of a macro that makes definitions, what
;; it makes; a `begin`, each of its; a definition, as `expand-in` expands
;; it.
(expand-defn (->* (fails syn) syns-a)
  (lambda (mx-fail s)
    (let* ((items (list-items s)) (hd (if (null? items) s (car items)))
           (head (if (syn-symbol? hd) (syn-name hd) "")))
      (cond
        ((and (syn-symbol? hd) (macro? (syn-symbol hd)))
         (if (use-entered? mx-fail (syn-symbol hd) s (get budget))
             (begin
               (set budget (- (get budget) 1))
               (let* ((from (made-syntax-count))
                      (made (expand-defn mx-fail (expand-use mx-fail (syn-symbol hd) s))))
                 (begin (syntax-held mx-fail (syn-symbol hd) s from) (use-left!) made)))
             nil))
        ((string=? head "begin") (expand-defns mx-fail (cdr items)))
        ;; A macro an expansion defines: defined now, for what comes after;
        ;; its names noted for the parser (`made-syntax-taken!`).
        ((or (string=? head "define-syntax") (string=? head "define-syntax-rec"))
         (begin (made-syntax-add! (define-macro mx-fail s nil)) nil))
        (else (list (expand-in mx-fail s nil)))))))
(expand-defns (->* (fails syns-a) syns-a)
  (lambda (mx-fail xs)
    (if (null? xs) nil (append (expand-defn mx-fail (car xs)) (expand-defns mx-fail (cdr xs)))))))
;; Top-level form `form` expanded: a use of a macro that makes definitions,
;; the definitions it makes, in order, a `begin` of them spliced (as R7RS's
;; at the top level); each at `form`'s place, as `expand`'s, as a top-level
;; `include`'s forms are (`Macros::expand_top`). Else `expand`'s, alone.
(define expand-top (->* (fails syn) syns-a)
  (lambda (mx-fail form)
    (let ((items (list-items form)))
      (if (and (not (null? items)) (syn-symbol? (car items))
               (makes-top-defns? (syn-symbol (car items))))
          (let ((n (begin (made-syntax-reset!) (chain-begun! mx-fail form))))
            (if (< n 0)
                (list form)
                (begin (set budget n)
                       (at-places (expand-defn mx-fail form) (syn-start form) (syn-end form)))))
          (list (expand mx-fail form))))))
;;; ------------------------------------------------------------ macros' names

;; Where the last of forms `xs` after this one makes `n` a macro, by
;; `define-syntax` or `import-with-syntax`, in a list of one: as the Rust
;; checker sees it, every macro of the program defined before any
;; definition is checked (`Macros::defined_before`). `fs`, the files read.
(define later-macro-site (->* (symbol syns-a files) mx-files-places)
  (lambda (n xs fs) (macro-sites n xs fs (get loads) nil)))

;;; ============================================================ macros-templates.fx (whole)

(define mx-templates-untyped? (->* (pvars) bool)
  (lambda (ps) (or (null? ps) (and (null? (extract (extract (car ps) 2) 4)) (mx-templates-untyped? (cdr ps))))))
;; A name made fresh: `prefix` and the count.
(define fresh-symbol (->* (string) symbol)
  (lambda (prefix)
    (string->symbol (string-append prefix (fresh-tag)))))
(define symbol-in (->* (syn symbol) syns-a)
  (lambda (s n)
    (cond ((syn-symbol? s) (if (symbol=? (syn-symbol s) n) (list s) nil))
          ((null? (split s)) nil)
          (else (let ((first (symbol-in (car (split s)) n)))
                  (if (null? first) (symbol-in (nth (split s) 1) n) first))))))
;; Where pattern variable `n` is in clause `c`'s pattern (else the pattern).
(define pattern-place (->* (clause symbol) syn)
  (lambda (c n)
    (let ((found (symbol-in (extract c 1) n))) (if (null? found) (extract c 1) (car found)))))
;; Each binder of the clause a fresh name, for the holes in its scope to be
;; given; any part but an `expr` or a `bvar`, refused.
(define bvar-binds (->* (fails clause pvars mx-match-binds) mx-match-binds)
  (lambda (mx-fail c ps out)
    (if (null? ps)
        out
        (let* ((n (extract (car ps) 1))
               (k (mx-types-kind-of (extract (extract (car ps) 2) 1))))
          (cond ((or (= k 0) (= k 10)) (bvar-binds mx-fail c (cdr ps) out))
                ((= k 1)
                 (let ((f (fresh-symbol (string-append "%" (string-append (symbol->string n) ":")))))
                   (bvar-binds mx-fail c (cdr ps)
                               (append out (the mx-match-binds (list (tuple n (mk-symbol (symbol->string f)
                                                                                0 0))))))))
                (else
                 (begin
                   (fail-at mx-fail (string-append "`" (string-append (symbol->string n) (string-append "` is not an `expr`, a "
                                                (string-append "`lambda` or a `bvar`: a macro whose parts are "
                                                "typed is checked once, of those alone, not yet"))))
                            (pattern-place c n))
                   out)))))))
(define proper-of (->* (syn) syns-a)
  (lambda (s) (tagcase s (mx-eager-reader-types-lst (items d a b x) items) (else y nil))))
;; A `lambda` part's type, written `(subr E (A …) R)`, in a list of one.
(define written-subr (->* (fails clause symbol syns-a) syns-a)
  (lambda (mx-fail c n ty)
    (let ((items (if (null? ty) nil (proper-of (car ty)))))
      (if (and (= (len items) 4) (syn-symbol? (car items))
               (string=? (syn-name (car items)) "subr")
               (or (not (null? (proper-of (nth items 2))))
                   (syn-nil? (nth items 2))))
          (list (car ty))
          (begin (fail-at mx-fail (string-append "`" (string-append (symbol->string n) (string-append "` is a `lambda` part: its "
                                              (string-append "type is written, `(lambda #:type (subr E (A …) "
                                              "R))`, which its stand-in in the check is made of"))))
                          (pattern-place c n))
                 nil)))))
(define arg-names (->* (syns-a) (listof symbol acyclic))
  (lambda (ts) (if (null? ts) nil (let ((v (fresh-symbol "%a:"))) (cons v (arg-names (cdr ts)))))))
(define arg-params (->* ((listof symbol acyclic) syns-a int int) syns-a)
  (lambda (ns ts a b)
    (if (null? ns)
        nil
        (cons (mk-list (list (mk-symbol (symbol->string (car ns)) a b) (car ts)) a b)
              (arg-params (cdr ns) (cdr ts) a b)))))
(define arg-refs (->* ((listof symbol acyclic) int int) syns-a)
  (lambda (ns a b)
    (if (null? ns) nil (cons (mk-symbol (symbol->string (car ns)) a b) (arg-refs (cdr ns) a b)))))
;; `(lambda ((a A) …) (call a …))`, of the argument types of `ty`, each `a`
;; a fresh name.
(define lambda-stand-in (->* (syn syn int int) syn)
  (lambda (call ty a b)
    (let* ((names (arg-names (proper-of (nth (proper-of ty) 2)))))
      (mk-list (list (mk-symbol "lambda" a b)
                     (list-at a b 0 (arg-params names (proper-of (nth (proper-of ty) 2)) a b) nil)
                     (list-at a b 0 (cons call (arg-refs names a b)) nil))
               a b))))
;; A hole's type or effect: as written, or a binder of `kind` named for the
;; hole `n` (`%type-of-e`), what a message about it says, in a list with
;; that binder.
(define hole-part (->* (syns-a string string int int) syns-a)
  (lambda (given kind n a b)
    (if (not (null? given))
        (list (car given))
        (let ((v (mk-symbol (string-append "%" (string-append kind (string-append "-of-" n))) a b)))
          (list v (mk-list (list v (mk-symbol kind a b)) a b))))))
(define rib-scope (->* (fails clause symbol atms pvars) scope-vars)
  (lambda (mx-fail c n as ps)
    (if (null? as)
        nil
        (tagcase (car as)
          (atm-var (v e g)
            (let* ((info (pvar-of v ps)) (h (if (null? info) nil (extract (car info) 4))))
              (if (or (null? h) (null? (extract (car h) 1)))
                  (begin (fail-at mx-fail (string-append "`" (string-append (symbol->string v)
                                                      (string-append "` binds a variable `" (string-append (symbol->string n)
                                                      (string-append "` is in the scope of: its type is written, "
                                                      (string-append "`(bvar #:type T)`, where a macro's parts "
                                                      "are typed"))))))
                                  (pattern-place c v))
                         (the scope-vars nil))
                  (cons (tuple v (car (extract (car h) 1))) (rib-scope mx-fail c n (cdr as) ps)))))
          (else y
            (begin (fail-at mx-fail (string-append "`" (string-append (symbol->string n) (string-append "` is in the scope of the "
                                                (string-append "binders a part exports: not yet where a macro's "
                                                "parts are typed"))))
                            (pattern-place c n))
                   (the scope-vars nil)))))))
(define hole-scope (->* (fails clause symbol levels pvars) scope-vars)
  (lambda (mx-fail c n levels ps)
    (if (null? levels)
        nil
        (tagcase (car levels)
          (lv-rib (as) (append (rib-scope mx-fail c n as ps) (hole-scope mx-fail c n (cdr levels) ps)))
          (else y
            (begin (fail-at mx-fail (string-append "`" (string-append (symbol->string n) (string-append "` is in the scope of a "
                                                (string-append "part's ribcage: not yet where a macro's parts "
                                                "are typed"))))
                            (pattern-place c n))
                   (the scope-vars nil)))))))
(define scope-types (->* (scope-vars) syns-a)
  (lambda (vs) (if (null? vs) nil (cons (extract (car vs) 2) (scope-types (cdr vs))))))
(define scope-calls (->* (scope-vars mx-match-binds) syns-a)
  (lambda (vs bs)
    (if (null? vs) nil (append (bound-of (extract (car vs) 1) bs) (scope-calls (cdr vs) bs)))))
;; Each hole's parameter, and its call bound to it, onto `acc`: its
;; parameters, the bindings, the `plambda`'s binders.
(define hole-params (->* (fails clause pvars int int params) params)
  (lambda (mx-fail c ps a b acc)
    (if (null? ps)
        acc
        (let* ((n (extract (car ps) 1)) (info (extract (car ps) 2)) (h (extract info 4))
               (shown (symbol->string n)) (k (mx-types-kind-of (extract info 1))))
          (cond
            ((not (or (= k 0) (= k 10))) (hole-params mx-fail c (cdr ps) a b acc))
            ((null? h)
             (begin (fail-at mx-fail (string-append "`" (string-append shown (string-append "` has no type or effect, and another "
                                                 (string-append "part of this macro does: a macro whose parts "
                                                 (string-append "are typed is checked once, each `expr` and "
                                                 "`lambda` part typed")))))
                             (pattern-place c n))
                    acc))
            (else
             (let* ((scope (hole-scope mx-fail c n (reverse (extract info 2)) (extract c 4)))
                    ;; A lambda's stand-in is a lambda: its procedure, given,
                    ;; called with the lambda's arguments, of the types its
                    ;; type says; so its type is written, `(subr E (A …) R)`,
                    ;; and making it does nothing.
                    (lambda-ty (if (= k 10) (written-subr mx-fail c n (extract (car h) 1)) nil))
                    (ty (hole-part (extract (car h) 1) "type" shown a b))
                    (eff (if (= k 10)
                             (list (mk-symbol "pure" a b))
                             (hole-part (extract (car h) 2) "effect" shown a b)))
                    (hole (mk-symbol (symbol->string (fresh-symbol (string-append "%h:" (string-append shown ":"))))
                                     a b))
                    (subr (mk-list (list (mk-symbol "subr" a b) (car eff)
                                         (list-at a b 0 (scope-types scope) nil) (car ty))
                                   a b))
                    (call (list-at a b 0 (cons hole (scope-calls scope (extract acc 2))) nil))
                    (stand-in (if (null? lambda-ty)
                                  call
                                  (lambda-stand-in call (car lambda-ty) a b)))
                    (acc2 (tuple (append (extract acc 1)
                                         (the syns-a (list (mk-list (list hole subr) a b))))
                                 (append (extract acc 2) (the mx-match-binds (list (tuple n stand-in))))
                                 (append (extract acc 3) (append (cdr ty) (cdr eff))))))
               (hole-params mx-fail c (cdr ps) a b acc2))))))))
(define template-check (->* (fails symbol macro clause) syn)
  (lambda (mx-fail n m c)
    (let* ((template (extract c 2)) (a (syn-start template)) (b (syn-end template))
           (binders (bvar-binds mx-fail c (extract c 4) nil))
           (made (hole-params mx-fail c (extract c 4) a b (tuple (the syns-a nil) binders
                                                              (extract m 3))))
           (body (begin (set checking #t) (def-env! 0 (extract c 6))
                        (let ((x (transcribe mx-fail c template nil (extract made 2) 0)))
                          (begin (set checking #f) x))))
           (params (extract made 1)) (poly (extract made 3))
           (lam (mk-list (list (mk-symbol "lambda" a b) (list-at a b 0 params nil) body) a b))
           (proc (if (null? poly)
                     lam
                     (mk-list (list (mk-symbol "plambda" a b) (mk-list poly a b) lam) a b)))
           (hidden (fresh-symbol (string-append "%template:" (string-append (symbol->string n) ":")))))
      (mk-list (list (mk-symbol "define" a b) (atom hidden a b 0) proc) a b))))
(define checks-of (->* (fails symbol macro clauses) syns-a)
  (lambda (mx-fail n m cs)
    (cond ((null? cs) nil)
          ((mx-templates-untyped? (extract (car cs) 4)) (checks-of mx-fail n m (cdr cs)))
          (else (let ((c (template-check mx-fail n m (car cs))))
                  (cons c (checks-of mx-fail n m (cdr cs))))))))
;;; ------------------------------------------------------------ checking templates

;; The checks of macro `n`'s templates, once, where it is defined
;; (`Macros::template_checks`; `typed-macros.md`, "Proposed: typed holes"):
;; for each clause whose parts are typed, a hidden definition of the
;; template as a procedure over its holes, each hole a parameter called
;; where the template places it, `(subr E (U …) T)`, given the template's
;; binders in its scope, of the `bvar`s' types; each part left out a binder
;; of the procedure's `plambda`. Checked as any definition is.
(define template-checks (->* (fails symbol) syns-a)
  (lambda (mx-fail n)
    ;; A trailmark has no templates.
    (let ((m (macro-of n (get defined))))
      (if (null? m) nil (checks-of mx-fail n (car m) (extract (car m) 2))))))

;;; ============================================================ macros-imports.fx (whole)

(define instances (ref mx-parser-types-names mxp) (new nil))
(define without-name (->* (symbol mx-parser-types-names) mx-parser-types-names)
  (lambda (n ns)
    (cond ((null? ns) nil)
          ((symbol=? n (car ns)) (without-name n (cdr ns)))
          (else (cons (car ns) (without-name n (cdr ns)))))))
;; Notes a top-level `(define name (load-module "path"))`, the file
;; `import-with-syntax` of `name` reads, or `(define name ((proj (load-module
;; "path") arg …)))`, an instance of it, which `instances` notes; any other
;; definition of `name` forgets it (`Checker::note_load_def`).
(define note-load-def! (->* (syn) unit)
  (lambda (form)
    (let ((items (proper form)))
      (if (and (>= (len items) 3) (string=? (mx-files-head-name form) "define") (syn-symbol? (nth items 1)))
          (let* ((n (syn-symbol (nth items 1)))
                 (init (nth items (- (len items) 1)))
                 (path (def-path init))
                 (kept (without-load n (get loads)))
                 (ok (and (not (null? path)) (<= (len items) 4))))
            (begin
              (set loads (if ok (cons (tuple n (car path)) kept) kept))
              (set instances (if (and ok (not (null? (instance-path init))))
                                 (cons n (without-name n (get instances)))
                                 (without-name n (get instances))))))
          #u))))
;; The standard bindings' names, which a file's macro may name: what the
;; driver says, as the Rust checker has them (`standard-names!`).
(define standard-names (ref mx-parser-types-names mxp) (new nil))
(define standard-names! (->* (mx-parser-types-names) unit) (lambda (ns) (set standard-names ns)))
(define raw-files (ref raw mxp) (new nil))
(define raw-files! (->* (raw) unit) (lambda (r) (set raw-files r)))
(define raw-at (->* (int raw syns-a) syns-a)
  (lambda (base r forms)
    (cond ((null? r) forms)
          ((= base (extract (car r) 1)) (extract (car r) 2))
          (else (raw-at base (cdr r) forms)))))
(define raw-forms (->* (loaded-file) syns-a)
  (lambda (f) (raw-at (extract f 2) (get raw-files) (extract f 5))))
;; The files read, `fs`, each with its forms as read.
(define raw-loaded (->* (files) files)
  (lambda (fs)
    (if (null? fs)
        nil
        (let ((f (car fs)))
          (cons (product (1 (extract f 1)) (2 (extract f 2)) (3 (extract f 3)) (4 (extract f 4))
                         (5 (raw-forms f)) (6 (extract f 6)) (7 (extract f 7)) (8 (extract f 8)))
                (raw-loaded (cdr fs)))))))
;; The macros the last `import-file-macros` defined by their names, which
;; the parser holds to the globals (`top-made`).
(define mx-imports-imported (ref mx-parser-types-names mxp) (new nil))
(define taken-names (->* (mx-parser-types-names (listof mx-files-filter acyclic)) mx-parser-types-names)
  (lambda (ns mx-files-filter)
    (cond ((null? ns) nil)
          ((taken? (car ns) mx-files-filter) (cons (car ns) (taken-names (cdr ns) mx-files-filter)))
          (else (taken-names (cdr ns) mx-files-filter)))))
(define not-load (->* (symbol) string)
  (lambda (lib)
    (string-append "`import-with-syntax` takes a name defined as `(load-module \"path\")`, "
                   (string-append "and `" (string-append (symbol->string lib) "` is not one")))))
;; The `include` that gives the rest, of module `lib`: of the names `filter`
;; lists, those not macros.
(define include-of (->* (syn (listof mx-files-filter acyclic) mx-parser-types-names int int) syns-a)
  (lambda (lib mx-files-filter exported a b)
    (let ((head (list (mk-symbol "include" a b) lib)))
      (if (null? mx-files-filter)
          (list (mk-list head a b))
          (let ((rest (not-macros (extract (car mx-files-filter) 2) exported)))
            (if (and (extract (car mx-files-filter) 1) (null? rest))
                nil
                (list (mk-list (append head
                                       (list (mk-list (cons (mk-symbol (if (extract (car mx-files-filter) 1)
                                                                           "only"
                                                                           "except")
                                                                       a b)
                                                            rest)
                                                      a b)))
                               a b))))))))
(define mx-imports-given-names (->* (mx-imports-given) mx-parser-types-names)
  (lambda (g) (if (null? g) nil (cons (extract (car g) 1) (mx-imports-given-names (cdr g))))))
(define without-file-load (->* (symbol file-loads) file-loads)
  (lambda (n ls)
    (cond ((null? ls) nil)
          ((symbol=? n (extract (car ls) 1)) (without-file-load n (cdr ls)))
          (else (cons (car ls) (without-file-load n (cdr ls)))))))
(define file-load-noted (->* (syn file-loads) file-loads)
  (lambda (x ls)
    (let ((items (proper x)))
      (if (and (>= (len items) 3) (string=? (mx-files-head-name x) "define") (syn-symbol? (nth items 1)))
          (let ((n (syn-symbol (nth items 1))) (init (nth items (- (len items) 1))))
            (if (and (not (null? (def-path init))) (<= (len items) 4))
                (cons (tuple n init) (without-file-load n ls))
                (without-file-load n ls)))
          ls))))
;; The load `lib` names, of loads `ls`, in a list of one; and the file it
;; read.
(define load-of-file (->* (symbol file-loads) syns-a)
  (lambda (lib ls)
    (cond ((null? ls) nil)
          ((symbol=? lib (extract (car ls) 1)) (list (extract (car ls) 2)))
          (else (load-of-file lib (cdr ls))))))
;; Whether a module file's `lib`, of its loads `ls`, is an instance of a file
;; with module parameters, whose macros a module file cannot import: its
;; values' `include` would be of an item (`Checker::load_in_file`). Its
;; load, in a list of one.
(define instance-loaded (->* (symbol file-loads) syns-a)
  (lambda (lib ls)
    (let ((l (load-of-file lib ls)))
      (if (and (not (null? l)) (not (null? (instance-path (car l))))) l nil))))
(define instance-in-file (->* (symbol) string)
  (lambda (lib)
    (string-append "`" (string-append (symbol->string lib) (string-append "` is an instance of a file with module parameters: "
                   "a module file cannot import its macros, not yet")))))
(define file-of (->* (symbol file-loads files) files)
  (lambda (lib ls fs)
    (let ((load (load-of-file lib ls))) (if (null? load) nil (mx-files-read-at (syn-start (car load)) fs)))))
(define taken-given (->* (mx-imports-given (listof mx-files-filter acyclic)) mx-imports-given)
  (lambda (g mx-files-filter)
    (cond ((null? g) nil)
          ((taken? (extract (car g) 1) mx-files-filter) (cons (car g) (taken-given (cdr g) mx-files-filter)))
          (else (taken-given (cdr g) mx-files-filter)))))
(define onto-mades (->* (mx-parser-types-names bool mades) mades)
  (lambda (ns h made) (if (null? ns) made (onto-mades (cdr ns) h (cons (tuple (car ns) h) made)))))
;; The trailmark `n` is in `ms`, defined there, or 0.
(define own-mark (->* (symbol marked) int)
  (lambda (n ms)
    (cond ((null? ms) 0)
          ((symbol=? n (extract (car ms) 1)) (extract (car ms) 2))
          (else (own-mark n (cdr ms))))))
;; Each of macros `ns` of the file's by the name the importer has it by too,
;; for the file's uses that expand to it.
(define by-internal-each (->* (mx-parser-types-names (listof (tupleof symbol symbol) acyclic)) unit)
  (lambda (ns internal)
    (if (null? ns)
        #u
        (let ((i (internal-of (car ns) internal)) (k (own-mark (car ns) (get marks)))
              (m (macro-of (car ns) (get defined))))
          (begin
            (cond ((null? i) #u)
                  ((> k 0) (set marks (cons (tuple (car i) k) (get marks))))
                  ((not (null? m)) (set defined (cons (tuple (car i) (car m)) (get defined))))
                  (else #u))
            (by-internal-each (cdr ns) internal))))))
(define reverse-mades (->* (mades mades) mades)
  (lambda (xs acc) (if (null? xs) acc (reverse-mades (cdr xs) (cons (car xs) acc)))))
;; Of a file's macros, `own`, those by the name the importer has them by.
(define internal-name? (->* (symbol) bool)
  (lambda (n)
    (let ((s (symbol->string n)))
      (and (> (string-length s) 8) (string=? (substring s 0 8) "%syntax:")))))
(define internal-macros (->* (macro-env) macro-env)
  (lambda (own)
    (cond ((null? own) nil)
          ((internal-name? (extract (car own) 1)) (cons (car own) (internal-macros (cdr own))))
          (else (internal-macros (cdr own))))))
(define internal-marks (->* (marked) marked)
  (lambda (ms)
    (cond ((null? ms) nil)
          ((internal-name? (extract (car ms) 1)) (cons (car ms) (internal-marks (cdr ms))))
          (else (internal-marks (cdr ms))))))
;; Macro `n`, which the file gives, by its own name if `filter` takes it:
;; with the name the importer has it by, in a list of one.
(define give-one
  (->* (fails symbol macro-env marked (listof (tupleof symbol symbol) acyclic)
        (listof mx-files-filter acyclic) syn)
       mx-imports-given)
  (lambda (mx-fail n own own-marks internal mx-files-filter first)
    (let ((k (own-mark n own-marks)))
      (begin
        (if (taken? n mx-files-filter)
            (if (> k 0)
                ;; A trailmark of the file's: bound as its macros are.
                (if (or (macro? n) (> (own-mark n (get marks)) 0))
                    (fail-at mx-fail (string-append "`" (string-append (symbol->string n) (string-append "` is defined already: "
                                                 "a file's trailmark may not be defined again")))
                             first)
                    (set marks (cons (tuple n k) (get marks))))
                (if (macro? n)
                    (fail-at mx-fail (string-append "`" (string-append (symbol->string n) (string-append "` is a macro defined "
                                                 "already: a file's may not be defined again")))
                             first)
                    (set defined (cons (tuple n (car (macro-of n own))) (get defined)))))
            #u)
        (list (tuple n (car (internal-of n internal))))))))
(define give-each
  (->* (fails mades macro-env marked (listof (tupleof symbol symbol) acyclic)
        (listof mx-files-filter acyclic) syn mx-parser-types-names)
       mx-imports-given)
  (lambda (mx-fail made own own-marks internal mx-files-filter first seen)
    (if (null? made)
        nil
        (let* ((n (extract (car made) 1))
               (twice (if (mx-files-name-in? n seen)
                          (fail-at mx-fail (string-append "`" (string-append (symbol->string n) (string-append "` is a macro this "
                                                       "file gives twice")))
                                   first)
                          #u))
               (one (if (extract (car made) 2) nil (give-one mx-fail n own own-marks internal mx-files-filter
                                                             first))))
          (append one (give-each mx-fail (cdr made) own own-marks internal mx-files-filter first
                                 (cons n seen)))))))
(define-rec
;; The macros of the file read as `f`, which `form` imports: each defined
;; as the file means it (`define-file`); the macros it gives
;; (`Checker::file_macros_at`). `inst`: the instance of it imported, if any,
;; in a list of one (`binding-shapes.md`, step 4 (c)).
(file-macros-of (->* (fails syn loaded-file (listof mx-files-filter acyclic) files mx-parser-types-names) mx-imports-given)
  (lambda (mx-fail form f mx-files-filter fs inst)
    (let ((forms (raw-forms f)) (path (extract f 4)))
      (if (and (null? inst) (not (null? forms))
               (string=? (mx-files-head-name (car forms)) "module-parameters"))
          (begin (fail-at mx-fail (string-append "`" (string-append path (string-append "` takes module parameters: import the "
                                              (string-append "macros of an instance of it, `(define name ((proj "
                                              (string-append "(load-module \"" (string-append path "\") arg …)))`"))))))
                          form)
                 nil)
          (let* ((key (extract f 7)) (base (extract f 2))
                 ;; An error in the file, said in it, where the form starts.
                 (in-file (lambda ((m string) (x int) (y int))
                            (if (and (>= x base) (< x (+ base load-base)))
                                (let ((at (syn-start form)))
                                  (mx-fail (in-loaded-as fs base path m x) at at))
                                (mx-fail m x y))))
                 (ms (file-syntax forms))
                 (shown (names-shown (file-items forms) nil nil))
                 ;; An instance's macros by names of their own, and its uses'
                 ;; keys (`instance-prefix`), so that two instances' are apart.
                 (ikey (if (null? inst) key (string-append key (string-append "@" (symbol->string (car inst))))))
                 (prefix (if (null? inst)
                             (file-prefix key #t)
                             (instance-prefix (file-prefix key #t) (car inst))))
                 (internal (internal-names ikey (macro-names ms)))
                 (shared (if (null? inst)
                             (string->symbol (string-append "%shared:" key))
                             (car inst)))
                 (mx-nodes-scope (tuple shared (extract shown 1) (extract shown 2) internal
                               (get standard-names))))
            (define-file in-file forms mx-nodes-scope mx-files-filter fs ikey prefix (if (null? inst) 1 3)
                         (if (null? ms) form (extract (car ms) 1))))))))
;; A file's macros, of its forms `forms`, defined as the file means them
;; (`scope`): among themselves first, by their names in it, as its
;; templates name them, with the macros of the files it imports and those
;; its top-level uses make (`TODO.md` §287), each use expanded as a form of
;; its own, its key of the file's (`file-prefix`), where the file's items
;; are as its importer has them; then each by the name the importer has it
;; by, and each it gives (not hidden, nor of a hidden import) that `filter`
;; takes by its own name too (`Macros::define_file`). The macros it gives.
(define-file
  (->* (fails syns-a mx-nodes-scope (listof mx-files-filter acyclic) files string string int syn) mx-imports-given)
  (lambda (mx-fail forms mx-nodes-scope mx-files-filter fs key prefix kind first)
    (let ((program (get defined)) (program-marks (get marks)) (program-phase1 (phase1-forms-now))
          (program-env (def-env-is)) (keys (file-keys-begin! prefix)))
      (begin
        (set defined nil)
        (set marks nil)
        ;; The file's phase-1 forms its own, for its macros alone.
        (phase1-forms! nil)
        (def-env-now! (tuple kind (extract mx-nodes-scope 1) (extract mx-nodes-scope 2)))
        (let* ((st (define-forms mx-fail forms #f mx-nodes-scope fs key (tuple (extract mx-nodes-scope 4) nil nil)))
               (own (get defined)) (own-marks (get marks)))
          (begin
            (set defined program)
            (set marks program-marks)
            (phase1-forms! program-phase1)
            (def-env-now! program-env)
            (file-keys-end! keys)
            ;; Every macro of the file's by the name the importer has it by,
            ;; hidden or not, which those it gives may expand to.
            (set defined (append (internal-macros own) (get defined)))
            (set marks (append (internal-marks own-marks) (get marks)))
            (give-each mx-fail (reverse-mades (extract st 2) nil) own own-marks (extract st 1) mx-files-filter
                       first nil)))))))
(define-forms (->* (fails syns-a bool mx-nodes-scope files string file-state) file-state)
  (lambda (mx-fail xs h mx-nodes-scope fs key st)
    (if (null? xs)
        st
        (let* ((x (car xs))
               (st2 (tuple (extract st 1) (extract st 2) (file-load-noted x (extract st 3)))))
          (define-forms mx-fail (cdr xs) h mx-nodes-scope fs key (define-form mx-fail x h mx-nodes-scope fs key st2))))))
(define-form (->* (fails syn bool mx-nodes-scope files string file-state) file-state)
  (lambda (mx-fail x h mx-nodes-scope fs key st)
    (let ((internal (extract st 1)) (made (extract st 2)) (ls (extract st 3)))
      (cond
        ((and (not h) (hide-form? x)) (define-forms mx-fail (cdr (proper x)) #t mx-nodes-scope fs key st))
        ;; The file's phase-1 forms, in force for its macros after them.
        ((for-syntax? x) (begin (for-syntax! mx-fail x) st))
        ((defines-syntax? x)
         (let ((ns (define-macro mx-fail x (list (tuple (extract mx-nodes-scope 1) (extract mx-nodes-scope 2)
                                                    (extract mx-nodes-scope 3) internal
                                                    (extract mx-nodes-scope 5))))))
           (begin (by-internal-each ns internal)
                  (tuple internal (onto-mades ns h made) ls))))
        ((imports-syntax? x)
         (let* ((items (proper x))
                (mx-files-filter (if (= (len items) 3)
                            (the (listof mx-files-filter acyclic) (list (name-filter mx-fail (nth items 2))))
                            nil))
                (f (if (and (>= (len items) 2) (syn-symbol? (nth items 1)))
                       (file-of (syn-symbol (nth items 1)) ls fs)
                       nil)))
           (if (null? f)
               (begin (fail-at mx-fail (cond ((not (and (>= (len items) 2)
                                                     (syn-symbol? (nth items 1))))
                                           import-usage)
                                          ((not (null? (instance-loaded (syn-symbol (nth items 1))
                                                                        ls)))
                                           (instance-in-file (syn-symbol (nth items 1))))
                                          (else (not-load (syn-symbol (nth items 1)))))
                               x)
                      st)
               (let ((g (taken-given (file-macros-of mx-fail x (car f) mx-files-filter fs nil) mx-files-filter)))
                 (tuple (append g internal) (onto-mades (mx-imports-given-names g) h made) ls)))))
        ;; A use of a macro that makes definitions, which may make macros.
        ((let ((items (proper x)))
           (and (not (null? items)) (syn-symbol? (car items))
                (makes-top-defns? (syn-symbol (car items)))))
         (let* ((s (use-begin!))
                (ns (begin (form-begin! x) (expand-top mx-fail x) (made-syntax-taken!)))
                (back (use-end! s))
                (more (internal-names key ns)))
           (begin (by-internal-each ns more)
                  (tuple (append more internal) (onto-mades ns h made) ls))))
        (else st))))))
;; `(import-with-syntax lib)`, `lib` a name defined as `(load-module
;; "path")`: the file's macros defined, each as the file means it, and the
;; rest of what it gives as `(include lib)` gives it: that `include`'s form,
;; in a list of one; none if `(only …)` names macros alone. Of `fs`, the
;; files the driver read (`Checker::import_file_macros`).
(define import-file-macros (->* (fails syn files) syns-a)
  (lambda (mx-fail form fs)
    (let* ((items (proper form)) (a (syn-start form)) (b (syn-end form)))
      (if (not (and (or (= (len items) 2) (= (len items) 3)) (syn-symbol? (nth items 1))))
          (begin (fail-at mx-fail import-usage form) nil)
          (let* ((lib (syn-symbol (nth items 1)))
                 (mx-files-filter (if (= (len items) 3)
                             (the (listof mx-files-filter acyclic) (list (name-filter mx-fail (nth items 2))))
                             nil))
                 (path (mx-files-load-of lib (get loads))))
            (if (null? path)
                (begin (fail-at mx-fail (not-load lib) form) nil)
                (let ((f (loaded-path (car path) fs)))
                  (if (null? f)
                      (begin (fail-at mx-fail (string-append "cannot read `" (string-append (car path) "`")) form) nil)
                      (let* ((inst (if (mx-files-name-in? lib (get instances)) (list lib) nil))
                             (g (mx-imports-given-names (file-macros-of mx-fail form (car f) mx-files-filter fs inst))))
                        (begin (set mx-imports-imported (taken-names g mx-files-filter))
                               (include-of (mk-symbol (symbol->string lib) a b) mx-files-filter g
                                           a b)))))))))))
(define imported-names (->* () mx-parser-types-names) (lambda () (get mx-imports-imported)))
;; An `import-with-syntax`, `form`, in a module file, of loads `ls`, the
;; file's so far (`file-loads`): its file's macros defined, each as the
;; file means it, and the rest the `include` of the hidden global every
;; load of `lib`'s file is, which `lib` is too, by its name, a module's
;; `include` being made before its items (`Checker::file_own_expanded_syn`).
(define import-in-file (->* (fails syn file-loads files) syns-a)
  (lambda (mx-fail form ls fs)
    (let* ((items (proper form)) (a (syn-start form)) (b (syn-end form)))
      (if (not (and (or (= (len items) 2) (= (len items) 3)) (syn-symbol? (nth items 1))))
          (begin (fail-at mx-fail import-usage form) nil)
          (let* ((mx-files-filter (if (= (len items) 3)
                             (the (listof mx-files-filter acyclic) (list (name-filter mx-fail (nth items 2))))
                             nil))
                 (load (load-of-file (syn-symbol (nth items 1)) ls))
                 (f (if (null? load) nil (mx-files-read-at (syn-start (car load)) fs))))
            (cond
              ((not (null? (instance-loaded (syn-symbol (nth items 1)) ls)))
               (begin (fail-at mx-fail (instance-in-file (syn-symbol (nth items 1))) form) nil))
              ((null? f) (begin (fail-at mx-fail (not-load (syn-symbol (nth items 1))) form) nil))
              (else
               (include-of (mk-symbol (string-append "%shared:" (extract (car f) 7)) a b) mx-files-filter
                           (mx-imports-given-names (file-macros-of mx-fail form (car f) mx-files-filter fs nil))
                           a b))))))))

;;; ============================================================ macros-walk.fx (whole)

;;; ------------------------------------------------------------ a program's walk

;; Program `form`, after expansion, aligned and walked, its heads read by
;; their scopes (`program-scoped`): its root node, and what the walk found,
;; each binder and name referred to with its environment.
(define mx-walk-walked (->* (fails syn bool) (tupleof int mx-nodes-found))
  (lambda (mx-fail form top)
    (let ((r (program-scoped mx-fail (if top (s-item) (s-expr)) form #f nil)))
      (tuple (extract r 1) (extract r 2)))))
(define walked-out (ref string mxp) (new ""))
(define pfail-walk (->* (string int int) void)
  (lambda (message a b) (abort-current-continuation parse-tag (p-err message a b))))
(define atm-lits (->* (atms (listof int acyclic)) (listof int acyclic))
  (lambda (as out)
    (if (null? as)
        out
        (atm-lits (cdr as) (tagcase (car as) (atm-lit (b d) (cons b out)) (else y out))))))
(define lits-onto (->* (levels (listof int acyclic)) (listof int acyclic))
  (lambda (ls out)
    (if (null? ls)
        out
        (lits-onto (cdr ls)
                   (tagcase (car ls) (lv-rib (as) (atm-lits as out)) (else y out))))))
;; The nodes a module's items or a program's definitions bind by: what a
;; definition's form exports.
(define defined-binders (->* (fails int int (listof int acyclic)) (listof int acyclic))
  (lambda (mx-fail i n out)
    (if (= i n)
        out
        (defined-binders mx-fail (+ i 1) n
          (let ((k (mx-types-kind-of (node-core i))))
            (if (or (= k 14) (= k 11))
                (tagcase (node-kids i)
                  (k-form (x)
                    (let ((ex (node-export x)))
                      (if (null? ex)
                          out
                          (lits-onto (evaluate mx-fail (extract (car ex) 1) (extract (car ex) 2)
                                       (node-syn i))
                                     out))))
                  (else y out))
                out))))))
;; Value binder `n`'s node in `env`, innermost first: -1 if none binds it,
;; -2 if a module's names are looked past first.
(define binder-in (->* (symbol mx-env-indexed-env) int)
  (lambda (n mx-env-indexed-env)
    (let ((l (env-lookup mx-env-indexed-env n #f 0 mx-nodes-binder-name)))
      (case (extract l 1) ((0) (extract l 2)) ((1) -2) (else -1)))))
(define targets (->* (mx-nodes-found mx-parser-types-names pairs) (tupleof mx-parser-types-names pairs))
  (lambda (mx-nodes-found kept out)
    (if (null? mx-nodes-found)
        (tuple kept out)
        (let ((at (extract (car mx-nodes-found) 1)) (mx-env-indexed-env (extract (car mx-nodes-found) 2)))
          (tagcase (node-kids at)
            (k-ref (n d free)
              (if d
                  (targets (cdr mx-nodes-found) kept out)
                  (let ((b (binder-in n mx-env-indexed-env)))
                    (cond ((= b -1) (targets (cdr mx-nodes-found) kept out))
                          ((= b -2) (targets (cdr mx-nodes-found) (cons n kept) out))
                          (else (targets (cdr mx-nodes-found) kept (cons (tuple at b) out)))))))
            (else y (targets (cdr mx-nodes-found) kept out)))))))
(define mx-walk-int-in? (->* (int (listof int acyclic)) bool)
  (lambda (i is) (and (not (null? is)) (or (= i (car is)) (mx-walk-int-in? i (cdr is))))))
(define sym-in? (->* (symbol mx-parser-types-names) bool)
  (lambda (n ns) (and (not (null? ns)) (or (symbol=? n (car ns)) (sym-in? n (cdr ns))))))
;; Each value binder's node from `i` to `n`, but those of `defined` and of
;; a name of `kept`.
(define renamable
  (->* (int int (listof int acyclic) mx-parser-types-names (listof int acyclic)) (listof int acyclic))
  (lambda (i n defined kept out)
    (if (= i n)
        out
        (renamable (+ i 1) n defined kept
                   (tagcase (node-kids i)
                     (k-binder (m d)
                       (if (or d (mx-walk-int-in? i defined) (sym-in? m kept)) out (cons i out)))
                     (else y out))))))
(define map-firsts (->* ((listof (tupleof int addr) acyclic)) (listof int acyclic))
  (lambda (xs) (if (null? xs) nil (cons (extract (car xs) 1) (map-firsts (cdr xs))))))
(define addr<? (->* (addr addr) bool)
  (lambda (a b)
    (cond ((null? b) #f)
          ((null? a) #t)
          ((mx-eq? (car a) (car b)) (addr<? (cdr a) (cdr b)))
          (else (car a)))))
(define mx-walk-addr=? (->* (addr addr) bool)
  (lambda (a b)
    (cond ((null? a) (null? b))
          ((null? b) #f)
          (else (and (mx-eq? (car a) (car b)) (mx-walk-addr=? (cdr a) (cdr b)))))))
(define mx-walk-inserted (->* ((tupleof int addr) (listof (tupleof int addr) acyclic))
                      (listof (tupleof int addr) acyclic))
  (lambda (x out)
    (cond ((null? out) (list x))
          ((mx-walk-addr=? (extract x 2) (extract (car out) 2)) out)
          ((addr<? (extract x 2) (extract (car out) 2)) (cons x out))
          (else (cons (car out) (mx-walk-inserted x (cdr out)))))))
;; Nodes `xs` in the order their parts are written, a list's first part
;; before its rest, one of each path. (A program walk's paths are kept
;; innermost step first, `path+`: each in order, once.)
(define sorted-binders
  (->* ((listof int acyclic) (listof (tupleof int addr) acyclic)) (listof int acyclic))
  (lambda (xs out)
    (if (null? xs)
        (the (listof int acyclic) (map-firsts out))
        (sorted-binders (cdr xs) (mx-walk-inserted (tuple (car xs) (reverse (node-path (car xs)))) out)))))
(define mx-walk-numbered (->* ((listof int acyclic) int mx-walk-renames) mx-walk-renames)
  (lambda (bs k out)
    (if (null? bs)
        out
        (mx-walk-numbered (cdr bs) (+ k 1)
                  (cons (tuple (node-path (car bs))
                               (string-append (syn-name (node-syn (car bs))) (string-append "%" (int->string k))))
                        out)))))
(define mx-walk-name-at (->* (addr mx-walk-renames) (listof string acyclic))
  (lambda (p rs)
    (cond ((null? rs) nil)
          ((mx-walk-addr=? p (extract (car rs) 1)) (list (extract (car rs) 2)))
          (else (mx-walk-name-at p (cdr rs))))))
;; Each reference of `pairs` whose binder is renamed, with that name.
(define referred (->* (pairs mx-walk-renames mx-walk-renames) mx-walk-renames)
  (lambda (ps mx-parser-types-names out)
    (if (null? ps)
        out
        (let ((nm (mx-walk-name-at (node-path (extract (car ps) 2)) mx-parser-types-names)))
          (referred (cdr ps) mx-parser-types-names
                    (if (null? nm) out (cons (tuple (node-path (extract (car ps) 1)) (car nm))
                                         out)))))))
;; Path `p`, innermost step first, `k` cdrs further.
(define mx-walk-falses (->* (addr int) addr)
  (lambda (p k) (if (= k 0) p (mx-walk-falses (cons #f p) (- k 1)))))
(define-rec
;; `s`, at `path` (innermost step first, as a walk's), each name at a path
;; of `rs` renamed so.
(renamed-syn (->* (syn addr mx-walk-renames) syn)
  (lambda (s path rs)
    (tagcase s
      (mx-eager-reader-types-lst (items d a b x) (mx-eager-reader-types-lst (renamed-items items path rs) d a b x))
      (dotted (items tail d a b x)
        (dotted (renamed-items items path rs) (renamed-syn tail (mx-walk-falses path (len items)) rs) d a
          b x))
      ;; A string written empty: what it holds binds nothing, and is not
      ;; escaped as written (`show-syn`).
      (atom (dt a b x)
        (cond ((string? dt) (atom "" a b x))
              ((syn-symbol? s)
               (let ((nm (mx-walk-name-at path rs)))
                 (if (null? nm) s (atom (string->symbol (car nm)) a b x))))
              (else s)))
      (else y s))))
(renamed-items (->* (syns-a addr mx-walk-renames) syns-a)
  (lambda (items path rs)
    (if (null? items)
        nil
        (cons (renamed-syn (car items) (cons #t path) rs)
              (renamed-items (cdr items) (cons #f path) rs))))))
;; Program `form` with each local value binder renamed, `name%k`, `k` its
;; place among them in the order they are written, and each reference to
;; it: but a definition's name, and every binder of a name some reference
;; reaches only past what a `with` or `include` gives, or that the walk
;; cannot place.
(define alpha-renamed (->* (fails syn) syn)
  (lambda (mx-fail form)
    (let* ((w (mx-walk-walked mx-fail form #t)) (root (extract w 1)) (mx-nodes-found (extract w 2))
           (defined (defined-binders mx-fail 0 (get nnodes) nil))
           (t (targets mx-nodes-found nil nil))
           (kept (append (extract t 1) (get unplaced)))
           (binders (sorted-binders (renamable 0 (get nnodes) defined kept nil) nil))
           (mx-parser-types-names (mx-walk-numbered binders 1 nil))
           (refs (referred (extract t 2) mx-parser-types-names nil)))
      (renamed-syn form nil (append mx-parser-types-names refs)))))
;; `s` written.
(define show-renamed (->* (syn) string)
  (lambda (s) (show-syn s)))
(define walked-one (->* (syn) string)
  (lambda (form)
    (let ((r (prompt parse-tag
                     (begin (set walked-out (show-renamed (alpha-renamed pfail-walk form)))
                            (p-ok nil))
                     (lambda ((r presult)) r))))
      (begin
        (set program-walk #f)
        (tagcase r
          (p-err (why a b) (string-append "walk: " why))
          (else y (get walked-out)))))))
;; Each of `forms`, a program's after expansion, its local value binders
;; renamed as the walk finds their scopes, written (`macros.rs`'s
;; `Macros::alpha_renamed`, for `tests/scopes.rs`, which holds the two walks
;; to one another): `walk: ` and why, where one does not walk.
(define walked-forms (->* (syns-a) (listof string acyclic))
  (lambda (forms)
    (if (null? forms)
        nil
        (cons (walked-one (car forms)) (walked-forms (cdr forms))))))
(define value-binder-in (->* (symbol atms) int)
  (lambda (n as)
    (if (null? as)
        -1
        (tagcase (car as)
          (atm-lit (b d)
            (if (and (not d) (tagcase (node-kids b) (k-binder (m e) (symbol=? m n)) (else y #f)))
                b
                (value-binder-in n (cdr as))))
          (else y (value-binder-in n (cdr as)))))))

;;; ============================================================ macros-tree.fx (whole)

;; Each node's parent (-1 for none); the list node each is an element of
;; (-1 for none) and its place there; each node's place in a walk of the
;; tree before and after its children; its list's elements, once asked.
(define parents (ref (arrayof int mxp) mxp) (new (make-array 1 -1)))
(define lists (ref (arrayof int mxp) mxp) (new (make-array 1 -1)))
(define mx-tree-places (ref (arrayof int mxp) mxp) (new (make-array 1 0)))
(define pres (ref (arrayof int mxp) mxp) (new (make-array 1 0)))
(define posts (ref (arrayof int mxp) mxp) (new (make-array 1 0)))
(define elems-done (ref (arrayof bool mxp) mxp) (new (make-array 1 #f)))
(define elems-made (ref (arrayof mx-tree-ints mxp) mxp) (new (make-array 1 (the mx-tree-ints nil))))
(define tops (ref (arrayof int mxp) mxp) (new (make-array 1 -1)))
;; Node `i`'s children, in order.
(define kids-of (->* (int) (listof int acyclic))
  (lambda (i)
    (tagcase (node-kids i)
      (k-pair (x y) (list x y))
      (k-form (x) (list x))
      (mx-nodes-k-seq (x) (list x))
      (k-desc (xs) xs)
      (else y nil))))
(define parent-each (->* ((listof int acyclic) int) unit)
  (lambda (xs i)
    (if (null? xs) #u (begin (array-set! (get parents) (car xs) i) (parent-each (cdr xs) i)))))
(define parents-from (->* (int int) unit)
  (lambda (i n)
    (if (= i n) #u (begin (parent-each (kids-of i) i) (parents-from (+ i 1) n)))))
(define parent-of (->* (int) int) (lambda (i) (if (< i 0) -1 (array-ref (get parents) i))))
(define chain-head? (->* (int) bool)
  (lambda (i)
    (let ((q (parent-of i)))
      (not (and (>= q 0) (tagcase (node-kids q) (k-pair (a d) (= d i)) (else y #f)))))))
(define chain-each! (->* (int int int) unit)
  (lambda (c l k)
    (tagcase (node-kids c)
      ;; Placed only where `a`'s parent is this pair, as the parents array
      ;; has it (the last written), from which its place was found before.
      (k-pair (a d)
        (begin (if (= (parent-of a) c)
                   (begin (array-set! (get lists) a l) (array-set! (get mx-tree-places) a k))
                   #u)
               (chain-each! d l (+ k 1))))
      (else y #u))))
(define chain-placed! (->* (int) unit)
  (lambda (h)
    (let* ((q (parent-of h))
           (form (and (>= q 0) (tagcase (node-kids q) (k-form (x) (= x h)) (else y #f))))
           (list-node (if form q h))
           (from (if (and form (not (mx-walk-addr=? (node-path h) (node-path q)))) 1 0)))
      (chain-each! h list-node from))))
(define desc-placed! (->* ((listof int acyclic) int int) unit)
  (lambda (xs p k)
    (if (null? xs)
        #u
        (begin
          ;; A part twice: its first place, as a search of the parts finds it.
          (if (and (= (parent-of (car xs)) p) (< (array-ref (get lists) (car xs)) 0))
              (begin (array-set! (get lists) (car xs) p) (array-set! (get mx-tree-places) (car xs) k))
              #u)
          (desc-placed! (cdr xs) p (+ k 1))))))
;;; ------------------------------------------------------------ places in lists

;; Each list's elements given their list node and place: a chain of pairs
;; from its head (a pair no pair's cdr is), its list a form's (the form's
;; head counted as its place 0 where the chain is the whole form) or its
;; own; a description's parts, by their order.
(define places-from (->* (int int) unit)
  (lambda (i n)
    (if (= i n)
        #u
        (begin
          (tagcase (node-kids i)
            (k-pair (x y) (if (chain-head? i) (chain-placed! i) #u))
            (k-desc (xs) (desc-placed! xs i 0))
            (else y #u))
          (places-from (+ i 1) n)))))
;; Nodes `xs` onto `stack`, the first on top.
(define kids-pushed (->* ((listof int acyclic) (listof (tupleof int bool) acyclic))
                         (listof (tupleof int bool) acyclic))
  (lambda (xs stack)
    (if (null? xs) stack (cons (tuple (car xs) #f) (kids-pushed (cdr xs) stack)))))
(define mx-tree-numbered (->* ((listof (tupleof int bool) acyclic) int) int)
  (lambda (stack k)
    (if (null? stack)
        k
        (let ((i (extract (car stack) 1)))
          (if (extract (car stack) 2)
              (begin (array-set! (get posts) i k) (mx-tree-numbered (cdr stack) (+ k 1)))
              (begin (array-set! (get pres) i k)
                     (mx-tree-numbered (kids-pushed (kids-of i) (cons (tuple i #t) (cdr stack)))
                               (+ k 1))))))))
;;; ------------------------------------------------------------ under

;; Each node's place in a walk of the tree, before and after its children,
;; from each root, the walk's stack its own: a list may be long.
(define numbered-from (->* (int int int) unit)
  (lambda (i n k)
    (cond ((= i n) #u)
          ((< (parent-of i) 0) (numbered-from (+ i 1) n (mx-tree-numbered (list (tuple i #f)) k)))
          (else (numbered-from (+ i 1) n k)))))
;; The tree of nodes 0 to `n`, found.
(define tree-begin! (->* (int) unit)
  (lambda (n)
    (begin
      (set parents (the (arrayof int mxp) (make-array n -1)))
      (set lists (the (arrayof int mxp) (make-array n -1)))
      (set mx-tree-places (the (arrayof int mxp) (make-array n 0)))
      (set pres (the (arrayof int mxp) (make-array n -1)))
      (set posts (the (arrayof int mxp) (make-array n -1)))
      (set elems-done (the (arrayof bool mxp) (make-array n #f)))
      (set elems-made (the (arrayof mx-tree-ints mxp) (make-array n (the mx-tree-ints nil))))
      (set tops (the (arrayof int mxp) (make-array n -1)))
      (parents-from 0 n)
      (places-from 0 n)
      (numbered-from 0 n 0))))
;; The list node `b` is an element of, and its place there; (-1 . 0) if
;; none.
(define position (->* (int) (tupleof int int))
  (lambda (b)
    (if (or (< b 0) (< (array-ref (get lists) b) 0))
        (tuple -1 0)
        (tuple (array-ref (get lists) b) (array-ref (get mx-tree-places) b)))))
(define cars (->* (int) (listof int acyclic))
  (lambda (x) (tagcase (node-kids x) (k-pair (a d) (cons a (cars d))) (else y nil))))
(define elems-found (->* (int) (listof int acyclic))
  (lambda (at)
    (tagcase (node-kids at)
      (k-form (x)
        (if (mx-walk-addr=? (node-path x) (node-path at)) (cars x) (cons -1 (cars x))))
      (k-pair (a d) (cars at))
      (k-desc (xs) xs)
      (else y nil))))
;; The nodes of node `at`'s list's elements, by place (-1 where no node is,
;; a form's head).
(define elems (->* (int) (listof int acyclic))
  (lambda (at)
    (if (array-ref (get elems-done) at)
        (array-ref (get elems-made) at)
        (let ((e (elems-found at)))
          (begin (array-set! (get elems-made) at e) (array-set! (get elems-done) at #t) e)))))
(define mx-tree-int-at (->* ((listof int acyclic) int) int)
  (lambda (xs k) (cond ((null? xs) -1) ((= k 0) (car xs)) (else (mx-tree-int-at (cdr xs) (- k 1))))))
;; The `k`th element of node `at`'s list, or -1.
(define elem (->* (int int) int)
  (lambda (at k) (mx-tree-int-at (elems at) k)))
(define path-top (->* (int) int)
  (lambda (m)
    (let ((t (array-ref (get tops) m)))
      (if (>= t 0)
          t
          (let* ((q (parent-of m))
                 (top (if (and (>= q 0) (mx-walk-addr=? (node-path q) (node-path m))) (path-top q) m)))
            (begin (array-set! (get tops) m top) top))))))
;; Whether node `at` is node `m` or under it, by its path (each innermost
;; step first, `path+`: `m`'s is the end of `at`'s): under the outermost
;; node of `m`'s path, which every node whose path ends with it is under.
(define under? (->* (int int) bool)
  (lambda (at m)
    (let ((t (path-top m)))
      (and (<= (array-ref (get pres) t) (array-ref (get pres) at))
           (<= (array-ref (get posts) at) (array-ref (get posts) t))))))

;;; ============================================================ macros-shaper.fx (whole)

;;; ------------------------------------------------------------ shapes

;; Shapes (`shape`, `shapes`, `members`): `parser-types.fx`'s, which the
;; checker makes too, of the types it knows (`k-type-shape`).
(define one (->* (mx-parser-types-shape) mx-parser-types-shapes) (lambda (x) (list x)))
(define any-known? (->* ((listof mx-parser-types-shapes acyclic)) bool)
  (lambda (ps) (and (not (null? ps)) (or (not (null? (car ps))) (any-known? (cdr ps))))))
;; A function's shape, if anything of it is known.
(define sh-fun-of (->* ((listof mx-parser-types-shapes acyclic) mx-parser-types-shapes) mx-parser-types-shapes)
  (lambda (ps r) (if (or (not (null? r)) (any-known? ps)) (one (sh-fun ps r)) nil)))
;; Member `n` of members `ms`, in a list of one.
(define mx-shaper-member-of (->* (symbol mx-parser-types-members) (listof (tupleof symbol mx-parser-types-shapes) acyclic))
  (lambda (n ms)
    (cond ((null? ms) nil)
          ((symbol=? n (extract (car ms) 1)) (list (car ms)))
          (else (mx-shaper-member-of n (cdr ms))))))
(define names-find-hashed (->* (symbol int name-tree) (listof (tupleof symbol mx-parser-types-shapes) acyclic))
  (lambda (n h t)
    (tagcase t
      (nm-node (k m s l r)
        (cond ((< h k) (names-find-hashed n h l))
              ((and (= h k) (symbol=? n m)) (list (tuple m s)))
              (else (names-find-hashed n h r))))
      (else y nil))))
;; Member `n` of `t`, in a list of one, as `member-of` gives it.
(define names-find (->* (symbol name-tree) (listof (tupleof symbol mx-parser-types-shapes) acyclic))
  (lambda (n t) (names-find-hashed n (symbol-name-hash n) t)))
(define names-with-hashed (->* (name-tree symbol int mx-parser-types-shapes) name-tree)
  (lambda (t n h s)
    (tagcase t
      (nm-node (k m x l r)
        (cond ((< h k) (nm-node k m x (names-with-hashed l n h s) r))
              ((and (= h k) (symbol=? n m)) (nm-node k m s l r))
              (else (nm-node k m x l (names-with-hashed r n h s)))))
      (else y (nm-node h n s (nm-leaf) (nm-leaf))))))
;; `t` with `n` (its shape `s`) in its place, a later one of a name in its
;; place.
(define names-with (->* (name-tree symbol mx-parser-types-shapes) name-tree)
  (lambda (t n s) (names-with-hashed t n (symbol-name-hash n) s)))
;; Each of members `xs` onto `t`, in order.
(define names-onto (->* (mx-parser-types-members name-tree) name-tree)
  (lambda (xs t)
    (if (null? xs)
        t
        (names-onto (cdr xs) (names-with t (extract (car xs) 1) (extract (car xs) 2))))))
(define names-empty? (->* (name-tree) bool)
  (lambda (t) (tagcase t (nm-leaf () #t) (else y #f))))
;; Whether shape `s` gives `n`, a description's if `d`, else a value's.
(define gives? (->* (mx-parser-types-shapes symbol bool) bool)
  (lambda (s n d)
    (and (not (null? s))
         (tagcase (car s)
           (sh-module (vs ds) (not (null? (mx-shaper-member-of n (if d ds vs)))))
           (else y #f)))))
;; Name `n`'s own shape in `s`, a description's if `d`.
(define mx-shaper-member-shape (->* (mx-parser-types-shapes symbol bool) mx-parser-types-shapes)
  (lambda (s n d)
    (if (null? s)
        nil
        (tagcase (car s)
          (sh-module (vs ds)
            (let ((m (mx-shaper-member-of n (if d ds vs)))) (if (null? m) nil (extract (car m) 2))))
          (else y nil)))))
(define result-of (->* (mx-parser-types-shapes) mx-parser-types-shapes)
  (lambda (s) (if (null? s) nil (tagcase (car s) (sh-fun (ps r) r) (else y nil)))))
;; The `k`th of `ps`, or none.
(define shape-at (->* ((listof mx-parser-types-shapes acyclic) int) mx-parser-types-shapes)
  (lambda (ps k) (cond ((null? ps) nil) ((= k 0) (car ps)) (else (shape-at (cdr ps) (- k 1))))))
(define param-of (->* (mx-parser-types-shapes int) mx-parser-types-shapes)
  (lambda (s k)
    (if (null? s) nil (tagcase (car s) (sh-fun (ps r) (shape-at ps k)) (else y nil)))))
;; The last of `xs`, or -1.
(define last-int (->* ((listof int acyclic)) int)
  (lambda (xs) (cond ((null? xs) -1) ((null? (cdr xs)) (car xs)) (else (last-int (cdr xs))))))
(define mx-shaper-int-at (->* ((listof int acyclic) int) int)
  (lambda (xs k) (cond ((null? xs) -1) ((= k 0) (car xs)) (else (mx-shaper-int-at (cdr xs) (- k 1))))))
(define without-member (->* (mx-parser-types-members symbol) mx-parser-types-members)
  (lambda (ms n)
    (cond ((null? ms) nil)
          ((symbol=? n (extract (car ms) 1)) (without-member (cdr ms) n))
          (else (cons (car ms) (without-member (cdr ms) n))))))
;; `ms` with `n` (its shape `s`) in its place, a later one of a name in its
;; place.
(define with-member (->* (mx-parser-types-members symbol mx-parser-types-shapes) mx-parser-types-members)
  (lambda (ms n s) (append (without-member ms n) (the mx-parser-types-members (list (tuple n s))))))
(define members-onto (->* (mx-parser-types-members mx-parser-types-members) mx-parser-types-members)
  (lambda (xs ms)
          (if (null? xs) ms
              (members-onto (cdr xs) (with-member ms (extract (car xs) 1) (extract (car xs) 2))))))
;; `xs` onto `out`, the last first.
(define members-reversed (->* (mx-parser-types-members mx-parser-types-members) mx-parser-types-members)
  (lambda (xs out) (if (null? xs) out (members-reversed (cdr xs) (cons (car xs) out)))))
;; Of `rs`, gathered the last first, each name's last, in the order of the
;; last definitions; `seen`, the names already kept, onto `out`.
(define members-latest (->* (mx-parser-types-members name-tree mx-parser-types-members) mx-parser-types-members)
  (lambda (rs seen out)
    (cond ((null? rs) out)
          ((not (null? (names-find (extract (car rs) 1) seen))) (members-latest (cdr rs) seen out))
          (else (members-latest (cdr rs) (names-with seen (extract (car rs) 1) nil)
                                (cons (car rs) out))))))
;; Those of `ms` that `hidden` has not.
(define members-hidden (->* (mx-parser-types-members name-tree) mx-parser-types-members)
  (lambda (ms hidden)
    (cond ((null? ms) nil)
          ((not (null? (names-find (extract (car ms) 1) hidden))) (members-hidden (cdr ms) hidden))
          (else (cons (car ms) (members-hidden (cdr ms) hidden))))))
(define top-vals (ref name-tree mxp) (new (nm-leaf)))
(define top-descs (ref name-tree mxp) (new (nm-leaf)))
(define shaping-top (ref bool mxp) (new #f))
(define file-shapes (ref (listof file-shape acyclic) mxp) (new nil))
(define file-shape-keyed (->* (string (listof file-shape acyclic)) (listof file-shape acyclic))
  (lambda (key fs)
    (cond ((null? fs) nil)
          ((string=? key (extract (car fs) 1)) (list (car fs)))
          (else (file-shape-keyed key (cdr fs))))))
;; The files the driver read, by where each load starts (`parser-loads.fx`).
(define files-given (ref loaded-files mxp) (new nil))
(define file-given-at (->* (int loaded-files) loaded-files)
  (lambda (a fs)
    (cond ((null? fs) nil)
          ((= a (extract (car fs) 1)) (list (car fs)))
          (else (file-given-at a (cdr fs))))))
(define input-key? (->* (string) bool)
  (lambda (k) (and (> (string-length k) 6) (string=? (substring k 0 6) "input:"))))
;; The key of file `f`, loaded as an input's if `input?`, as `parse-loaded`
;; makes it: what its hidden global is named by, `%shared:key`.
(define file-key (->* (loaded-file bool) string)
  (lambda (f input?)
    (let ((raw (extract f 7)))
      (if (input-key? raw) raw (string-append (if input? "input:" "") raw)))))
;; Whether the load at `a` is an input's, as the prepass noted it.
(define inputs-at (ref (listof int acyclic) mxp) (new nil))
(define load-input-at? (->* (int) bool)
  (lambda (a) (mx-walk-int-in? a (get inputs-at))))
;; The shape of the file a load at `a` reads, in a list of one.
(define file-at (->* (int) (listof file-shape acyclic))
  (lambda (a)
    (let ((f (file-given-at a (get files-given))))
      (if (null? f) nil (file-shape-keyed (file-key (car f) (load-input-at? a))
                                          (get file-shapes))))))
;; What the load at `a` makes, or (`module?`) the module it describes: its
;; file's shape; or, its file made by a form before and not read for this
;; one, its hidden global's, `%shared:key`, as the top level has it.
(define load-shape (->* (int bool) mx-parser-types-shapes)
  (lambda (a module?)
    (let* ((f (file-at a))
           (known (cond ((null? f) nil) (module? (extract (car f) 3)) (else (extract (car f) 2)))))
      (cond ((not (null? known)) known)
            (else
             (let ((g (file-given-at a (get files-given))))
               (if (null? g)
                   nil
                   ;; The module it describes, of a value that is a function
                   ;; (a file of module parameters', an input's of modules):
                   ;; what that makes, as `file-kept!` keeps it.
                   (let* ((v (names-find (string->symbol
                                          (string-append "%shared:"
                                                         (file-key (car g) (load-input-at? a))))
                                         (get top-vals)))
                          (s (if (null? v) nil (extract (car v) 2)))
                          (fun? (and (not (null? s))
                                     (tagcase (car s) (sh-fun (ps r) #t) (else y #f)))))
                     (if (and module? fun?) (result-of s) s)))))))))
;; The program's top level and files forgotten, for another; its files
;; `fs`.
(define shapes-reset! (->* (loaded-files) unit)
  (lambda (fs)
    (begin (set top-vals (nm-leaf)) (set top-descs (nm-leaf)) (set file-shapes nil)
           (set inputs-at nil)
           (set files-given fs))))
;; The files the driver read for the forms to come, `fs`.
(define shapes-files! (->* (loaded-files) unit)
  (lambda (fs) (set files-given fs)))
;;; ------------------------------------------------------------ one walk's shapes

;; The walk's: each node's environment if the walk found it (its tree:
;; `macros-tree.fx`); each node's shape as an expression and as a type, once
;; found (0 not yet, 1 being found, 2 found); what each module, by its
;; expression's node, was asked; and whether each node is in a module an
;; `include` includes.
(define envs (ref (arrayof mx-env-indexed-env mxp) mxp) (new (make-array 1 env-empty)))
(define exp-state (ref (arrayof int mxp) mxp) (new (make-array 1 0)))
(define exp-made (ref (arrayof mx-parser-types-shapes mxp) mxp) (new (make-array 1 (the mx-parser-types-shapes nil))))
(define ty-state (ref (arrayof int mxp) mxp) (new (make-array 1 0)))
(define ty-made (ref (arrayof mx-parser-types-shapes mxp) mxp) (new (make-array 1 (the mx-parser-types-shapes nil))))
(define in-included (ref (arrayof bool mxp) mxp) (new (make-array 1 #f)))
;; By each module's node: the names asked, the last first; those of
;; values and of descriptions, by name; whether its shape was not known
;; where asked. And what each module gives, by name, once found.
(define ask-names (ref (arrayof (listof (tupleof symbol bool bool) acyclic) mxp) mxp)
  (new (make-array 1 (the (listof (tupleof symbol bool bool) acyclic) nil))))
(define ask-vals (ref (arrayof name-tree mxp) mxp) (new (make-array 1 (nm-leaf))))
(define ask-descs (ref (arrayof name-tree mxp) mxp) (new (make-array 1 (nm-leaf))))
(define ask-unknown (ref (arrayof bool mxp) mxp) (new (make-array 1 #f)))
(define given-done (ref (arrayof bool mxp) mxp) (new (make-array 1 #f)))
(define given-vals (ref (arrayof name-tree mxp) mxp) (new (make-array 1 (nm-leaf))))
(define given-descs (ref (arrayof name-tree mxp) mxp) (new (make-array 1 (nm-leaf))))
(define envs-from (->* (mx-nodes-found) unit)
  (lambda (fs)
    (if (null? fs)
        #u
        (begin (array-set! (get envs) (extract (car fs) 1) (extract (car fs) 2))
               (envs-from (cdr fs))))))
(define-rec
(mark-under (->* (int) unit)
  (lambda (x)
    (begin
      (array-set! (get in-included) x #t)
      (tagcase (node-kids x)
        (k-pair (a d) (begin (mark-under a) (mark-under d)))
        (k-form (y) (mark-under y))
        (mx-nodes-k-seq (y) (mark-under y))
        (k-desc (ys) (mark-each ys))
        (else y #u)))))
(mark-each (->* ((listof int acyclic)) unit)
  (lambda (xs) (if (null? xs) #u (begin (mark-under (car xs)) (mark-each (cdr xs)))))))
(define mx-shaper-form-node? (->* (int) bool)
  (lambda (i) (tagcase (node-kids i) (k-form (x) #t) (else y #f))))
;; The name at the head of list `s`, or `no-head`: asked of every list a
;; pass reaches, its name not made a string (`head-of`).
(define no-head symbol (string->symbol ""))
(define head-sym (->* (syn) symbol)
  (lambda (s)
    (let ((items (proper-items s)))
      (if (and (not (null? items)) (syn-symbol? (car items))) (syn-symbol (car items)) no-head))))
(define mx-shaper-head-is? (->* (syn symbol) bool) (lambda (s n) (symbol=? (head-sym s) n)))
;; Each node in a module an `include` includes, marked.
(define includes-marked (->* (int int) unit)
  (lambda (i n)
    (if (= i n)
        #u
        (begin
          (if (and (mx-shaper-form-node? i) (mx-shaper-head-is? (node-syn i) 'include))
              (let ((m (elem i 1))) (if (>= m 0) (mark-under m) #u))
              #u)
          (includes-marked (+ i 1) n)))))
;; The walk of form `form`'s state made, its root `root`, what it found
;; `found`.
(define shaper-begin! (->* (int mx-nodes-found) unit)
  (lambda (root mx-nodes-found)
    (let ((n (get nnodes)))
      (begin
        (tree-begin! n)
        (set envs (the (arrayof mx-env-indexed-env mxp) (make-array n env-empty)))
        (set exp-state (the (arrayof int mxp) (make-array n 0)))
        (set exp-made (the (arrayof mx-parser-types-shapes mxp) (make-array n (the mx-parser-types-shapes nil))))
        (set ty-state (the (arrayof int mxp) (make-array n 0)))
        (set ty-made (the (arrayof mx-parser-types-shapes mxp) (make-array n (the mx-parser-types-shapes nil))))
        (set in-included (the (arrayof bool mxp) (make-array n #f)))
        (set ask-names (the (arrayof (listof (tupleof symbol bool bool) acyclic) mxp)
                            (make-array n (the (listof (tupleof symbol bool bool) acyclic) nil))))
        (set ask-vals (the (arrayof name-tree mxp) (make-array n (nm-leaf))))
        (set ask-descs (the (arrayof name-tree mxp) (make-array n (nm-leaf))))
        (set ask-unknown (the (arrayof bool mxp) (make-array n #f)))
        (set given-done (the (arrayof bool mxp) (make-array n #f)))
        (set given-vals (the (arrayof name-tree mxp) (make-array n (nm-leaf))))
        (set given-descs (the (arrayof name-tree mxp) (make-array n (nm-leaf))))
        (envs-from mx-nodes-found)
        (includes-marked 0 n)))))
;; The name at the head of list `s`, or "".
(define mx-shaper-head-of (->* (syn) string)
  (lambda (s)
    (let ((items (proper-items s)))
      (if (and (not (null? items)) (syn-symbol? (car items))) (syn-name (car items)) ""))))
(define no-source source (tuple -1 0 0 0 '-))
(define source-in-list (->* (int int int) source)
  (lambda (b q i)
    (let* ((mx-list (node-syn q)) (n (len (proper-items mx-list))) (head (mx-shaper-head-of mx-list)))
      (cond
        ((and (string=? head "define") (= i 1) (= n 3)) (tuple 0 (elem q 2) 0 0 '-))
        ((and (or (string=? head "define") (string=? head "define*")) (= i 1) (= n 4))
         (tuple 1 (elem q 2) 0 0 '-))
        ((and (string=? head "define-type") (= i 1) (= n 3)) (tuple 1 (elem q 2) 0 0 '-))
        ((and (string=? head "import") (>= i 2))
         (tagcase (node-kids b)
           (k-binder (m d) (if d no-source (tuple 2 (elem q 1) 0 0 m)))
           (else y no-source)))
        ;; An element of a list of bindings, `(x e)`, `(x T)`, `(x T e)`:
        ;; which, by the form whose operand the list is.
        ((and (= i 0) (or (= n 2) (= n 3)))
         (let* ((r (extract (position q) 1))
                (fk (if (>= r 0) (position r) (tuple -1 0)))
                (f (extract fk 1)) (k (extract fk 2)) (second (elem q 1))
                (fh (if (>= f 0) (mx-shaper-head-of (node-syn f)) "")))
           (cond ((and (= k 1) (or (string=? fh "let") (string=? fh "let*") (string=? fh "par")))
                  (tuple 0 second 0 0 '-))
                 ((or (and (= k 1) (or (string=? fh "letrec") (string=? fh "lambda")
                                       (string=? fh "module-parameters")))
                      (and (= k 2) (string=? fh "rlambda")))
                  (tuple 1 second 0 0 '-))
                 (else no-source))))
        (else no-source)))))
(define declared-type (->* (int) int)
  (lambda (at)
    (let* ((pi (position at)) (p (extract pi 1)) (i (extract pi 2))
           (h (if (>= p 0) (mx-shaper-head-of (node-syn p)) "")))
      (cond ((< p 0) -1)
            ((and (string=? h "plambda") (= i 2)) (declared-type p))
            ((and (or (string=? h "define") (string=? h "define*")) (= i 3)) (elem p 2))
            ((and (string=? h "the") (= i 2)) (elem p 1))
            (else -1)))))
;; Whether `s` is a dependent parameter, `(c : T)`.
(define dependent? (->* (syn) bool)
  (lambda (s)
    (let ((items (proper-items s)))
      (and (= (len items) 3) (syn-symbol? (nth items 1)) (string=? (syn-name (nth items 1)) ":")))))
(define param-of-type (->* (int int) source)
  (lambda (t k)
    (let* ((s (node-syn t)) (h (mx-shaper-head-of s)) (items (proper-items s)))
      (cond
        ((syn-symbol? s) (tuple 3 0 0 k (syn-symbol s)))
        ((or (string=? h "allof") (string=? h "poly"))
         (let ((x (elem t (if (string=? h "allof") 1 (- (len items) 1)))))
           (if (< x 0) no-source (param-of-type x k))))
        ((or (string=? h "->") (string=? h "subr"))
         (let* ((conv (and (string=? h "subr") (> (len items) 1)
                           (string=? (mx-shaper-head-of (nth items 1)) "conv")))
                (ps (elem t (cond ((string=? h "->") 1) (conv 3) (else 2))))
                (p (if (< ps 0) -1 (elem ps k))))
           (cond ((< p 0) no-source)
                 ((dependent? (node-syn p)) (tuple 1 (elem p 2) 0 0 '-))
                 (else (tuple 1 p 0 0 '-)))))
        (else no-source)))))
;; The `k`th parameter of lambda `f` as the type it is declared at writes
;; it: a typed `define`'s or a `the`'s, through `plambda`s, in a `subr` or
;; `->` under `poly`s and `allof`s.
(define declared-param (->* (int int) source)
  (lambda (f k)
    (let ((t (declared-type f)))
      (if (< t 0) no-source (param-of-type t k)))))
;; The node of application `at`'s operator, or -1.
(define operator (->* (int) int)
  (lambda (at)
    (tagcase (node-kids at)
      (k-form (x)
        (if (mx-walk-addr=? (node-path x) (node-path at))
            (tagcase (node-kids x) (k-pair (f d) f) (else y -1))
            -1))
      (else y -1))))
;; Parameter `i` of lambda `f`, given as an argument of an application.
(define argument-param (->* (int int) source)
  (lambda (f i)
    (let* ((pj (position f)) (app (extract pj 1)) (j (extract pj 2))
           (op (if (>= app 0) (operator app) -1)))
      (if (and (>= op 0) (>= j 1)) (tuple 4 op (- j 1) i '-) no-source))))
(define mx-shaper-source-of (->* (fails int) source)
  (lambda (mx-fail b)
    (let* ((pq (position b)) (q (extract pq 1)) (i (extract pq 2)))
      (if (< q 0)
          no-source
          (let ((fq (position q)))
            (cond
              ;; A lambda's parameter of no type of its own: the procedure
              ;; type the lambda is declared at says it.
              ((and (= (extract fq 2) 1) (>= (extract fq 1) 0)
                    (string=? (mx-shaper-head-of (node-syn (extract fq 1))) "lambda"))
               (let ((d (declared-param (extract fq 1) i)))
                 (if (>= (extract d 1) 0) d (argument-param (extract fq 1) i))))
              ;; A description function's name, `(define-type (f …) T)`.
              ((and (= i 0) (= (extract fq 2) 1) (>= (extract fq 1) 0)
                    (string=? (mx-shaper-head-of (node-syn (extract fq 1))) "define-type"))
               (tuple 1 (elem (extract fq 1) 2) 0 0 '-))
              (else (source-in-list b q i))))))))
;; The top level's names' shapes as the checker has their types, for the
;; forms to be parsed (`check-global-shapes!`): values', descriptions'.
(define global-vals (ref name-tree mxp) (new (nm-leaf)))
(define global-descs (ref name-tree mxp) (new (nm-leaf)))
;; Name `n`'s shape, a description's if `d`, as the checker has its type
;; (`Global`): of a top-level form only.
(define global-shape (->* (symbol bool) mx-parser-types-shapes)
  (lambda (n d)
    (if (get shaping-top)
        (let ((m (names-find n (get (if d global-descs global-vals)))))
          (if (null? m) nil (extract (car m) 2)))
        nil)))
;; Whether node `m` is the module an `include` includes.
(define included? (->* (int) bool)
  (lambda (m)
    (let ((fi (position m)))
      (and (= (extract fi 2) 1) (>= (extract fi 1) 0)
           (mx-shaper-head-is? (node-syn (extract fi 1)) 'include)))))
;; Module `m` asked for `n`, and its answer.
(define ask! (->* (int symbol bool bool bool) unit)
  (lambda (m n d given unknown)
    (cond (unknown (array-set! (get ask-unknown) m #t))
          ((not (null? (names-find n (array-ref (get (if d ask-descs ask-vals)) m)))) #u)
          (else
           (begin
             (array-set! (get (if d ask-descs ask-vals)) m
                         (names-with (array-ref (get (if d ask-descs ask-vals)) m) n nil))
             (array-set! (get ask-names) m
                         (cons (tuple n d given) (array-ref (get ask-names) m))))))))
;; Members `ms` by name, the first of a name kept, as `member-of` finds it.
(define names-first (->* (mx-parser-types-members) name-tree)
  (lambda (ms) (names-onto (members-reversed ms nil) (nm-leaf))))
(define given-by (->* (int mx-parser-types-shapes bool) name-tree)
  (lambda (m s d)
    (begin
      (if (array-ref (get given-done) m)
          #u
          (let ((both (tagcase (if (null? s) (sh-module nil nil) (car s))
                        (sh-module (vs ds) (tuple (names-first vs) (names-first ds)))
                        (else y (tuple (nm-leaf) (nm-leaf))))))
            (begin (array-set! (get given-vals) m (extract both 1))
                   (array-set! (get given-descs) m (extract both 2))
                   (array-set! (get given-done) m #t))))
      (array-ref (get (if d given-descs given-vals)) m))))
;; Whether module `m`, of shape `s`, gives `n`; its shape there: by names
;; found once for each module node (`given-by`), not its members searched.
(define node-gives? (->* (int mx-parser-types-shapes symbol bool) bool)
  (lambda (m s n d) (not (null? (names-find n (given-by m s d))))))
(define node-member-shape (->* (int mx-parser-types-shapes symbol bool) mx-parser-types-shapes)
  (lambda (m s n d)
    (let ((x (names-find n (given-by m s d)))) (if (null? x) nil (extract (car x) 2)))))
;; A top-level name's shape; a file's hidden global's, its file's.
(define top-shape (->* (symbol bool) mx-parser-types-shapes)
  (lambda (n d)
    (let* ((name (symbol->string n))
           (shared (and (not d) (> (string-length name) 8)
                        (string=? (substring name 0 8) "%shared:")))
           (m (if (get shaping-top) (names-find n (get (if d top-descs top-vals))) nil)))
      (cond ((and (not (null? m)) (not (null? (extract (car m) 2)))) (extract (car m) 2))
            (shared
             (let ((f (file-shape-keyed (substring name 8 (string-length name)) (get file-shapes))))
               (if (null? f) (global-shape n d) (extract (car f) 2))))
            ;; Else as the checker found its type: a definition read from a
            ;; `.fasl`, a stage's module, which no form read here defines.
            (else (global-shape n d))))))
(define syn-start-of-list (->* (syn) int)
  (lambda (s) (tagcase s (mx-eager-reader-types-lst (items d a b x) a) (dotted (items t d a b x) a) (else y -1))))
(define-rec
;; The shape of `(modnames member …)`, its members `ms`: each value's name
;; (`x`, `(val x)`), a module's with its own shape (`(modnames m member
;; …)`), each description's (`(abs t)`, `(desc u)`, `(eff e)`).
(modnames-shape (->* (syns-a) mx-parser-types-shapes)
  (lambda (ms) (modnames-from ms nil nil)))
(modnames-from (->* (syns-a mx-parser-types-members mx-parser-types-members) mx-parser-types-shapes)
  (lambda (ms vs ds)
    (if (null? ms)
        (one (sh-module (reverse vs) (reverse ds)))
        (let* ((m (car ms)) (parts (proper-items m))
               (w (if (and (> (len parts) 1) (syn-symbol? (car parts))) (syn-name (car parts)) ""))
               (n (if (> (len parts) 1) (nth parts 1) m)))
          (cond ((syn-symbol? m)
                 (modnames-from (cdr ms) (cons (tuple (syn-symbol m) (the mx-parser-types-shapes nil)) vs) ds))
                ((not (syn-symbol? n)) nil)
                ((string=? w "val")
                 (modnames-from (cdr ms) (cons (tuple (syn-symbol n) (the mx-parser-types-shapes nil)) vs) ds))
                ((string=? w "modnames")
                 (modnames-from (cdr ms)
                                (cons (tuple (syn-symbol n) (modnames-shape (cdr (cdr parts)))) vs)
                                ds))
                ((or (string=? w "abs") (string=? w "desc") (string=? w "eff"))
                 (modnames-from (cdr ms) vs (cons (tuple (syn-symbol n) (the mx-parser-types-shapes nil)) ds)))
                (else nil)))))))
(define mx-shaper-symbols-of (->* ((listof syn acyclic)) (listof symbol acyclic))
  (lambda (xs)
    (cond ((null? xs) nil)
          ((syn-symbol? (car xs)) (cons (syn-symbol (car xs)) (mx-shaper-symbols-of (cdr xs))))
          (else (mx-shaper-symbols-of (cdr xs))))))
(define symbol-in? (->* (symbol (listof symbol acyclic)) bool)
  (lambda (n xs) (and (not (null? xs)) (or (symbol=? n (car xs)) (symbol-in? n (cdr xs))))))
(define members-kept (->* (mx-parser-types-members (listof symbol acyclic)) mx-parser-types-members)
  (lambda (ms kept)
    (cond ((null? ms) nil)
          ((symbol-in? (extract (car ms) 1) kept) (cons (car ms) (members-kept (cdr ms) kept)))
          (else (members-kept (cdr ms) kept)))))
(define all-symbols? (->* ((listof syn acyclic)) bool)
  (lambda (xs) (or (null? xs) (and (syn-symbol? (car xs)) (all-symbols? (cdr xs))))))
(define abs-onto (->* ((listof symbol acyclic) mx-parser-types-members) mx-parser-types-members)
  (lambda (ns ds)
          (if (null? ns) ds (abs-onto (cdr ns) (cons (tuple (car ns) (the mx-parser-types-shapes nil)) ds)))))
(define name-in-list? (->* (string (listof string acyclic)) bool)
  (lambda (h hs) (and (not (null? hs)) (or (string=? h (car hs)) (name-in-list? h (cdr hs))))))
;; `extend`'s: the first's names, then the second's, the second's winning.
(define extended (->* (mx-parser-types-shapes mx-parser-types-shapes) mx-parser-types-shapes)
  (lambda (a b)
    (if (or (null? a) (null? b))
        nil
        (tagcase (car a)
          (sh-module (v0 d0)
            (tagcase (car b)
              (sh-module (v1 d1) (one (sh-module (members-onto v1 v0) (members-onto d1 d0))))
              (else y nil)))
          (else y nil)))))
;; What exports `ex` bind: levels, innermost first.
(define mx-shaper-exported (->* (fails rooted syn) levels)
  (lambda (mx-fail ex s)
    (if (null? ex)
        nil
        (append (evaluate mx-fail (extract (car ex) 1) (extract (car ex) 2) s)
                (mx-shaper-exported mx-fail (cdr ex) s)))))
(define-rec
;; Whether two shapes are the same.
(shapes=? (->* (mx-parser-types-shapes mx-parser-types-shapes) bool)
  (lambda (a b)
    (cond ((null? a) (null? b))
          ((null? b) #f)
          (else
           (tagcase (car a)
             (sh-module (v0 d0)
               (tagcase (car b) (sh-module (v1 d1) (and (members=? v0 v1) (members=? d0 d1)))
                        (else y #f)))
             (sh-fun (p0 r0)
               (tagcase (car b) (sh-fun (p1 r1) (and (shape-lists=? p0 p1) (shapes=? r0 r1)))
                        (else y #f))))))))
(members=? (->* (mx-parser-types-members mx-parser-types-members) bool)
  (lambda (a b)
    (cond ((null? a) (null? b))
          ((null? b) #f)
          (else (and (symbol=? (extract (car a) 1) (extract (car b) 1))
                     (shapes=? (extract (car a) 2) (extract (car b) 2))
                     (members=? (cdr a) (cdr b)))))))
(shape-lists=? (->* ((listof mx-parser-types-shapes acyclic) (listof mx-parser-types-shapes acyclic)) bool)
  (lambda (a b)
    (cond ((null? a) (null? b))
          ((null? b) #f)
          (else (and (shapes=? (car a) (car b)) (shape-lists=? (cdr a) (cdr b))))))))
(define-rec
(binder-shape (->* (fails int) mx-parser-types-shapes)
  (lambda (mx-fail b)
    (let ((src (mx-shaper-source-of mx-fail b)))
      (case (extract src 1)
        ((0) (exp-shape mx-fail (extract src 2)))
        ((1) (mx-shaper-ty-shape mx-fail (extract src 2)))
        ((2) (mx-shaper-member-shape (exp-shape mx-fail (extract src 2)) (extract src 5) #f))
        ;; The `k`th parameter of the procedure type a name names, as the
        ;; checker has the type (`global-shape`).
        ((3) (param-of (global-shape (extract src 5) #t) (extract src 4)))
        ((4) (param-of (param-of (exp-shape mx-fail (extract src 2)) (extract src 3)) (extract src 4)))
        (else nil)))))
;;; ------------------------------------------------------------ what a name is bound by

;; What reference `at`, of `n`, a description's if `d`, is bound by: a
;; binder's node (>= 0); the module a `with` or `include` opens around it
;; that gives it (its node in the second); free (-1); or, past a module
;; whose shape is not known, not to be said (-2). Each module looked at is
;; asked, and the answer kept (`askeds`).
(resolve (->* (fails int symbol bool) (tupleof int int))
  (lambda (mx-fail at n d) (resolve-in mx-fail at n d (array-ref (get envs) at) 0)))
(resolve-in (->* (fails int symbol bool mx-env-indexed-env int) (tupleof int int))
  (lambda (mx-fail at n d mx-env-indexed-env from)
    (let* ((l (env-lookup mx-env-indexed-env n d from mx-nodes-binder-name)) (m (extract l 2)) (next (extract l 3)))
      (case (extract l 1)
        ((0) (tuple m 0))
        ((1)
         ;; A module's own expression is not in the scope of what it gives;
         ;; nor is an `include`'s of what any includes.
         (if (or (under? at m) (and (array-ref (get in-included) at) (included? m)))
             (resolve-in mx-fail at n d mx-env-indexed-env next)
             (let ((s (exp-shape mx-fail m)))
               (if (null? s)
                   (begin (ask! m n d #f #t) (tuple -2 0))
                   (let ((given (node-gives? m s n d)))
                     (begin (ask! m n d given #f)
                            (if given (tuple -3 m) (resolve-in mx-fail at n d mx-env-indexed-env next))))))))
        (else (tuple -1 0))))))
;; The shape of what reference `at`, of `n`, refers to.
(mx-shaper-reference (->* (fails int symbol bool) mx-parser-types-shapes)
  (lambda (mx-fail at n d)
    (let* ((r (resolve mx-fail at n d)) (b (extract r 1)))
      (cond ((>= b 0) (binder-shape mx-fail b))
            ((= b -3) (node-member-shape (extract r 2) (exp-shape mx-fail (extract r 2)) n d))
            ((= b -1) (top-shape n d))
            (else nil)))))
;;; ------------------------------------------------------------ expressions', types', modules'

;; The shape of expression node `at`, if it makes a module (or a function
;; that makes one) whose shape is known.
(exp-shape (->* (fails int) mx-parser-types-shapes)
  (lambda (mx-fail at)
    (case (array-ref (get exp-state) at)
      ((2) (array-ref (get exp-made) at))
      ((1) nil)
      (else
       (begin
         (array-set! (get exp-state) at 1)
         (let ((s (exp-found mx-fail at)))
           (begin (array-set! (get exp-made) at s) (array-set! (get exp-state) at 2) s)))))))
(mx-shaper-ty-shape (->* (fails int) mx-parser-types-shapes)
  (lambda (mx-fail at)
    (if (< at 0)
        nil
        (case (array-ref (get ty-state) at)
          ((2) (array-ref (get ty-made) at))
          ((1) nil)
          (else
           (begin
             (array-set! (get ty-state) at 1)
             (let ((s (ty-found mx-fail at)))
               (begin (array-set! (get ty-made) at s) (array-set! (get ty-state) at 2) s))))))))
(exp-found (->* (fails int) mx-parser-types-shapes)
  (lambda (mx-fail at)
    (tagcase (node-kids at)
      (k-ref (n d free) (mx-shaper-reference mx-fail at n d))
      (k-binder (n d) (binder-shape mx-fail at))
      (else y
        (let* ((s (node-syn at)) (items (proper-items s)) (h (mx-shaper-head-of s)) (e (elems at))
               (k (len items)) (last (if (> k 1) (last-int e) -1))
               (at-e (lambda ((i int)) (mx-shaper-int-at e i))))
          (cond
            ((= k 0) nil)
            ((string=? h "module") (module-shape mx-fail (cdr e)))
            ;; A hidden global's definition, `(%loaded "key" "path")`, too
            ;; (`parse-loaded`).
            ((or (string=? h "load-module") (string=? h "load-input") (string=? h "%loaded"))
             (load-shape (syn-start-of-list s) #f))
            ((string=? h "the") (mx-shaper-ty-shape mx-fail (at-e 1)))
            ((string=? h "extend") (extended (exp-shape mx-fail (at-e 1)) (exp-shape mx-fail last)))
            ((and (string=? h "with") (= k 3) (syn-symbol? (nth items 1))
                  (string=? (syn-name (nth items 1)) "#%toplevel") (syn-symbol? (nth items 2)))
             (top-shape (syn-symbol (nth items 2)) #f))
            ((name-in-list? h (list "with" "let" "let*" "letrec" "begin" "letregion" "letfreeze"
                                    "letrena" "letreap" "plambda" "dlambda"))
             (if (>= last 0) (exp-shape mx-fail last) nil))
            ((string=? h "proj") (exp-shape mx-fail (at-e 1)))
            ((and (string=? h "if") (= k 4))
             (let ((a (exp-shape mx-fail (at-e 2))) (b (exp-shape mx-fail (at-e 3))))
               (if (and (not (null? a)) (shapes=? a b)) a nil)))
            ((or (string=? h "lambda") (string=? h "rlambda"))
             (let ((formals (at-e (if (string=? h "lambda") 1 2))))
               (sh-fun-of (if (>= formals 0) (formals-shapes mx-fail formals) nil)
                          (if (>= last 0) (exp-shape mx-fail last) nil))))
            ;; An application: its operator's result.
            (else (let ((op (operator at)))
                    (if (< op 0) nil (result-of (exp-shape mx-fail op)))))))))))
;; The shapes of lambda formals `f`'s parameters: a typed one's type's, an
;; untyped one's as its binder has it.
(formals-shapes (->* (fails int) (listof mx-parser-types-shapes acyclic))
  (lambda (mx-fail f)
    (map-shapes mx-fail (elems f))))
(map-shapes (->* (fails (listof int acyclic)) (listof mx-parser-types-shapes acyclic))
  (lambda (mx-fail ps)
    (if (null? ps)
        nil
        (cons (let ((p (car ps)))
                (if (< p 0)
                    nil
                    (tagcase (node-kids p)
                      (k-binder (n d) (binder-shape mx-fail p))
                      (else y (mx-shaper-ty-shape mx-fail (elem p 1))))))
              (map-shapes mx-fail (cdr ps))))))
;; The shape of the module whose items are the nodes `items`: the names each
;; exports, but a `module-parameters`'; without what a `hide` holds or
;; gives, though the module define it too.
;; Each item's members gathered onto `vs`, `ds` (and what a `hide` holds,
;; onto `hvs`, `hds`), the last first, so that gathering copies nothing;
;; at the end each name once, where its last definition puts it
;; (`members-latest`), as `with-member` one at a time would have it.
(module-shape (->* (fails (listof int acyclic)) mx-parser-types-shapes)
  (lambda (mx-fail items) (module-from mx-fail items nil nil nil nil)))
(module-from (->* (fails (listof int acyclic) mx-parser-types-members mx-parser-types-members mx-parser-types-members mx-parser-types-members) mx-parser-types-shapes)
  (lambda (mx-fail items vs ds hvs hds)
    (if (null? items)
        (one (sh-module (members-hidden (members-latest vs (nm-leaf) nil)
                                        (names-onto hvs (nm-leaf)))
                        (members-hidden (members-latest ds (nm-leaf) nil)
                                        (names-onto hds (nm-leaf)))))
        (let* ((x (car items)) (h (if (< x 0) "" (mx-shaper-head-of (node-syn x)))))
          (cond
            ((< x 0) nil)
            ((string=? h "module-parameters") (module-from mx-fail (cdr items) vs ds hvs hds))
            (else
             (tagcase (node-kids x)
               (k-form (f)
                 (let ((ex (node-export f)))
                   (if (null? ex)
                       (module-from mx-fail (cdr items) vs ds hvs hds)
                       (let ((got (levels-members mx-fail (mx-shaper-exported mx-fail ex (node-syn x))
                                                  (tuple (the mx-parser-types-members nil) (the mx-parser-types-members nil) #t))))
                         (cond ((not (extract got 3)) nil)
                               ((string=? h "hide")
                                (module-from mx-fail (cdr items) vs ds
                                             (append (extract got 1) hvs)
                                             (append (extract got 2) hds)))
                               (else
                                (module-from mx-fail (cdr items) (append (extract got 1) vs)
                                             (append (extract got 2) ds) hvs hds)))))))
               (else y (module-from mx-fail (cdr items) vs ds hvs hds)))))))))
;; What levels `ls` bind, as members, onto `got`, the last first (and a
;; name more than once: `module-from` keeps each one's last); its third,
;; whether every module among them has a shape.
(levels-members
        (->* (fails levels (tupleof mx-parser-types-members mx-parser-types-members bool)) (tupleof mx-parser-types-members mx-parser-types-members bool))
  (lambda (mx-fail ls got)
    (if (or (null? ls) (not (extract got 3)))
        got
        (levels-members mx-fail (cdr ls)
          (tagcase (car ls)
            (lv-rib (as) (atms-members mx-fail as got))
            (lv-earlier (as) (atms-members mx-fail as got))
            (lv-shape (m)
              (let ((s (exp-shape mx-fail m)))
                (if (null? s)
                    (tuple (extract got 1) (extract got 2) #f)
                    (tagcase (car s)
                      (sh-module (v d)
                        (tuple (members-reversed v (extract got 1))
                               (members-reversed d (extract got 2)) #t))
                      (else y (tuple (extract got 1) (extract got 2) #f))))))
            (lv-cage (c) (tuple (extract got 1) (extract got 2) #f)))))))
(atms-members
        (->* (fails atms (tupleof mx-parser-types-members mx-parser-types-members bool)) (tupleof mx-parser-types-members mx-parser-types-members bool))
  (lambda (mx-fail as got)
    (if (null? as)
        got
        (atms-members mx-fail (cdr as)
          (tagcase (car as)
            (atm-lit (b d)
              (tagcase (node-kids b)
                (k-binder (n e)
                  (let ((s (binder-shape mx-fail b)))
                    (if d
                        (tuple (extract got 1) (cons (tuple n s) (extract got 2)) #t)
                        (tuple (cons (tuple n s) (extract got 1)) (extract got 2) #t))))
                (else y got)))
            (else y got))))))
;; The shape of type node `at`, if it describes a module (or a function
;; that makes one) whose shape is known.
(ty-found (->* (fails int) mx-parser-types-shapes)
  (lambda (mx-fail at)
    (tagcase (node-kids at)
      (k-ref (n d free) (if d (mx-shaper-reference mx-fail at n #t) nil))
      (else y
        (let* ((s (node-syn at)) (items (proper-items s)) (h (mx-shaper-head-of s)) (e (elems at))
               (k (len items))
               (at-e (lambda ((i int)) (mx-shaper-int-at e i))))
          (cond
            ;; A template's free description name, the top level's,
            ;; qualified.
            ((and (syn-symbol? s) (> (string-length (syn-name s)) 11)
                  (string=? (substring (syn-name s) 0 11) "#%toplevel:"))
             (let ((m (names-find (string->symbol (substring (syn-name s) 11
                                                             (string-length (syn-name s))))
                                  (get top-descs))))
               (if (null? m) nil (extract (car m) 2))))
            ((= k 0) nil)
            ((string=? h "moduleof") (moduleof-shape mx-fail (cdr items) (cdr e) nil nil))
            ;; A shape written (`binding-shapes.md`): the names it gives.
            ((string=? h "modnames") (modnames-shape (cdr items)))
            ((or (string=? h "load-moduleof") (string=? h "load-input-moduleof"))
             (load-shape (syn-start-of-list s) #t))
            ((string=? h "restrict")
             (let ((m (mx-shaper-ty-shape mx-fail (at-e 1))) (kept (mx-shaper-symbols-of (cdr (cdr items)))))
               (if (null? m)
                   nil
                   (tagcase (car m)
                     (sh-module (vs ds)
                       (one (sh-module (members-kept vs kept) (members-kept ds kept))))
                     (else y nil)))))
            ((string=? h "poly") (mx-shaper-ty-shape mx-fail (at-e (- k 1))))
            ;; A type a module gives: its own shape, if it is a module's.
            ((and (string=? h "select") (= k 3) (syn-symbol? (nth items 2)))
             (mx-shaper-member-shape (exp-shape mx-fail (at-e 1)) (syn-symbol (nth items 2)) #t))
            ((and (string=? h "->") (= k 3))
             (if (< (at-e 1) 0) nil
                 (sh-fun-of (param-types mx-fail (at-e 1)) (mx-shaper-ty-shape mx-fail (at-e 2)))))
            ((string=? h "subr")
             (let* ((conv (and (> k 1) (string=? (mx-shaper-head-of (nth items 1)) "conv")))
                    (j (if conv 4 3)))
               (if (< (at-e (- j 1)) 0)
                   nil
                   (sh-fun-of (param-types mx-fail (at-e (- j 1))) (mx-shaper-ty-shape mx-fail (at-e j))))))
            ;; A description function applied: what it makes, whatever it
            ;; is given.
            ((and (>= (at-e 0) 0) (tagcase (node-kids (at-e 0)) (k-ref (n d free) d) (else y #f)))
             (mx-shaper-ty-shape mx-fail (at-e 0)))
            (else nil)))))))
;; The shapes of a procedure type's parameters, node `ps` their list, each
;; a type or a dependent `(c : T)`.
(param-types (->* (fails int) (listof mx-parser-types-shapes acyclic))
  (lambda (mx-fail ps) (param-types-of mx-fail (elems ps))))
(param-types-of (->* (fails (listof int acyclic)) (listof mx-parser-types-shapes acyclic))
  (lambda (mx-fail ps)
    (if (null? ps)
        nil
        (cons (let ((p (car ps)))
                (cond ((< p 0) nil)
                      ((dependent? (node-syn p)) (mx-shaper-ty-shape mx-fail (elem p 2)))
                      (else (mx-shaper-ty-shape mx-fail p))))
              (param-types-of mx-fail (cdr ps))))))
;; A `moduleof`'s shape, from its members `ms` (their nodes `es`): each
;; `val`'s and `desc`'s by its type, an `abs`'s not known.
(moduleof-shape
  (->* (fails (listof syn acyclic) (listof int acyclic) mx-parser-types-members mx-parser-types-members) mx-parser-types-shapes)
  (lambda (mx-fail ms es vs ds)
    (if (null? ms)
        (one (sh-module (reverse vs) (reverse ds)))
        (let* ((parts (proper-items (car ms))) (x (if (null? es) -1 (car es)))
               (word (if (and (not (null? parts)) (syn-symbol? (car parts))) (syn-name (car parts))
                         ""))
               (second (if (> (len parts) 1) (nth parts 1) (car ms)))
               ;; A label written as a keyword, `(abs #:t u type)`: its binder
               ;; after it, its type one part further on.
               (at (if (and (syn-keyword? second) (not (string=? word "val")) (= (len parts) 4))
                       3 2))
               (mx-parser-types-names (cond ((syn-symbol? second) (list (syn-symbol second)))
                            ((syn-keyword? second)
                             (tagcase second
                               (atom (d a b x) (if (mx-keyword? d) (list (mx-keyword->symbol d)) nil))
                               (else y nil)))
                            ((null? (proper-items second)) nil)
                            ((string=? word "abs")
                             (if (all-symbols? (proper-items second))
                                 (mx-shaper-symbols-of (proper-items second))
                                 (mx-shaper-symbols-of (list (car (proper-items second))))))
                            (else (mx-shaper-symbols-of (list (car (proper-items second)))))))
               (t (lambda ()
                          (if (< x 0) nil
                              (let ((y (elem x at))) (if (< y 0) nil (mx-shaper-ty-shape mx-fail y)))))))
          (cond
            ((or (< (len parts) 2) (and (null? mx-parser-types-names) (not (string=? word "abs")))) nil)
            ((string=? word "val")
             (moduleof-shape mx-fail (cdr ms) (if (null? es) nil (cdr es))
                             (cons (tuple (car mx-parser-types-names) (t)) vs) ds))
            ((string=? word "abs")
             (moduleof-shape mx-fail (cdr ms) (if (null? es) nil (cdr es)) vs (abs-onto mx-parser-types-names ds)))
            ((string=? word "desc")
             (moduleof-shape mx-fail (cdr ms) (if (null? es) nil (cdr es)) vs
                             (cons (tuple (car mx-parser-types-names) (t)) ds)))
            (else nil)))))))
(define mx-shaper-binder-named (->* (symbol bool atms) int)
  (lambda (n d as)
    (if (null? as)
        -1
        (tagcase (car as)
          (atm-lit (b e)
            (if (and (mx-eq? d e) (tagcase (node-kids b) (k-binder (m f) (symbol=? m n)) (else y #f)))
                b
                (mx-shaper-binder-named n d (cdr as))))
          (else y (mx-shaper-binder-named n d (cdr as)))))))
;; What module `m` was asked, in the order asked.
(define asked-of (->* (int) asked)
  (lambda (m)
    (if (< m 0)
        (tuple nil #f)
        (tuple (reverse (array-ref (get ask-names) m)) (array-ref (get ask-unknown) m)))))
(define globals-given! (->* (mx-parser-types-members mx-parser-types-members) unit)
  (lambda (vs ds)
    (begin (set global-vals (names-onto vs (nm-leaf)))
           (set global-descs (names-onto ds (nm-leaf))))))

;;; ============================================================ macros-shapes.fx (whole)

;; Every reference that reaches a module's names asks the module.
(define asked-all (->* (fails mx-nodes-found) unit)
  (lambda (mx-fail fs)
    (if (null? fs)
        #u
        (begin
          (let ((at (extract (car fs) 1)))
            (tagcase (node-kids at)
              (k-ref (n d free)
                (if (env-any-shape? (extract (car fs) 2)) (begin (resolve mx-fail at n d) #u) #u))
              (else y #u)))
          (asked-all mx-fail (cdr fs))))))
(define opens? (->* (syn) bool)
  (lambda (s)
    (or (mx-shaper-head-is? s 'with) (mx-shaper-head-is? s 'extend) (mx-shaper-head-is? s 'include)
        (mx-shaper-head-is? s 'import-with-syntax))))
(define toplevel-or-fx? (->* (syn) bool)
  (lambda (s)
    (let ((items (proper-items s)))
      (and (> (len items) 1) (syn-symbol? (nth items 1))
           (or (string=? (syn-name (nth items 1)) "#%fx")
               (string=? (syn-name (nth items 1)) "#%toplevel"))))))
(define writing-at (->* (addr (listof writing acyclic)) (listof writing acyclic))
  (lambda (p ws)
    (cond ((null? ws) nil)
          ((mx-walk-addr=? p (extract (car ws) 1)) (list (car ws)))
          (else (writing-at p (cdr ws))))))
(define mx-shapes-given-names (->* ((listof (tupleof symbol bool bool) acyclic) bool int int) syns-a)
  (lambda (xs d a b)
    (cond ((null? xs) nil)
          ((and (mx-eq? d (extract (car xs) 2)) (extract (car xs) 3))
           (cons (mk-symbol (symbol->string (extract (car xs) 1)) a b)
                 (mx-shapes-given-names (cdr xs) d a b)))
          (else (mx-shapes-given-names (cdr xs) d a b)))))
(define syn-span (->* (syn) (tupleof int int))
  (lambda (s)
    (tagcase s
      (mx-eager-reader-types-lst (items d a b x) (tuple a b))
      (dotted (items t d a b x) (tuple a b))
      (atom (d a b x) (tuple a b))
      (else y (tuple 0 0)))))
(define asked-written (->* (asked syn) syn)
  (lambda (a s)
    (let ((ab (syn-span s)))
      (mk-list (list (mk-list (mx-shapes-given-names (extract a 1) #f (extract ab 1) (extract ab 2))
                              (extract ab 1) (extract ab 2))
                     (mk-list (mx-shapes-given-names (extract a 1) #t (extract ab 1) (extract ab 2))
                              (extract ab 1) (extract ab 2)))
               (extract ab 1) (extract ab 2)))))
(define writings (->* (int int (listof writing acyclic)) (listof writing acyclic))
  (lambda (k n out)
    (if (= k n)
        out
        (writings (+ k 1) n
          (let ((s (node-syn k)))
            (if (and (mx-shaper-form-node? k) (opens? s) (null? (writing-at (node-path k) out))
                     (not (toplevel-or-fx? s)) (>= (elem k 1) 0))
                (let ((a (asked-of (elem k 1))))
                  (cons (tuple (node-path k) (if (extract a 2) nil (list (asked-written a s))))
                        out))
                out))))))
;; `ws`, each its path outermost step first: what is left of it below a
;; node, as a walk down from the top takes its steps (`written-under`).
(define writings-from-top (->* ((listof writing acyclic)) (listof writing acyclic))
  (lambda (ws)
    (if (null? ws)
        nil
        (cons (tuple (the addr (reverse (extract (car ws) 1))) (extract (car ws) 2))
              (writings-from-top (cdr ws))))))
;; Those of `ws` whose next step is `step`, past it.
(define step-taken (->* ((listof writing acyclic) bool) (listof writing acyclic))
  (lambda (ws step)
    (cond ((null? ws) nil)
          ((let ((p (extract (car ws) 1))) (and (not (null? p)) (mx-eq? (car p) step)))
           (cons (tuple (cdr (extract (car ws) 1)) (extract (car ws) 2))
                 (step-taken (cdr ws) step)))
          (else (step-taken (cdr ws) step)))))
;; Those of `ws` past `k` cdrs: a dotted list's tail's.
(define steps-taken (->* ((listof writing acyclic) int) (listof writing acyclic))
  (lambda (ws k) (if (= k 0) ws (steps-taken (step-taken ws #f) (- k 1)))))
;; What to write into list `s`, in a list of one, if it is a form that
;; opens a module and one of `ws` is at it, no step left.
(define written-here (->* (syn (listof writing acyclic)) (listof (listof syn acyclic) acyclic))
  (lambda (s ws)
    (if (opens? s)
        (let ((w (writing-at nil ws))) (if (null? w) nil (list (extract (car w) 2))))
        nil)))
(define-rec
;; `s` with each of `ws`, under it by the steps left of each, written:
;; itself where none is. Each step of each path taken once, at its list
;; (`written-items`), so that a long list costs no more than its length.
(written-under (->* (syn (listof writing acyclic)) syn)
  (lambda (s ws)
    (if (null? ws)
        s
        (tagcase s
          (mx-eager-reader-types-lst (items d a b x)
            (let ((items2 (written-items items ws (written-here s ws) 0 a b)))
              (mx-eager-reader-types-lst items2 (syn-datums items2) a b x)))
          (dotted (items tail d a b x)
            (let ((items2 (written-items items ws (written-here s ws) 0 a b)))
              (dotted items2 (written-under tail (steps-taken ws (len items))) (syn-datums items2)
                      a b x)))
          (else y s)))))
;; List items `items`, from the `k`th, each with those of `ws` under it
;; written (its first step `#t`, after `k` cdrs taken), the record `here`
;; after the first operand.
(written-items
  (->* (syns-a (listof writing acyclic) (listof (listof syn acyclic) acyclic) int int int)
       syns-a)
  (lambda (items ws here k a b)
    (let ((record (if (null? here)
                      (the syns-a nil)
                      (list (atom (mx-symbol->keyword 'relied) a b 0)
                            (if (null? (car here)) (mk-symbol "#f" a b) (car (car here)))))))
      (cond ((null? items) (if (= k 2) record nil))
            ((and (= k 2) (not (null? here)))
             (append record (written-items items ws nil k a b)))
            (else (cons (written-under (car items) (step-taken ws #t))
                        (written-items (cdr items) (step-taken ws #f) here (+ k 1) a b))))))))
;; `s` with each of `ws` written into the form at its path, after its
;; first operand.
(define with-written (->* (syn (listof writing acyclic)) syn)
  (lambda (s ws) (written-under s (writings-from-top ws))))
(define defined-atoms (->* (fails atms) unit)
  (lambda (mx-fail as)
    (if (null? as)
        #u
        (begin
          (tagcase (car as)
            (atm-lit (b d)
              (tagcase (node-kids b)
                (k-binder (n e)
                  (let ((s (binder-shape mx-fail b)))
                    (if d (set top-descs (names-with (get top-descs) n s))
                        (set top-vals (names-with (get top-vals) n s)))))
                (else y #u)))
            (else y #u))
          (defined-atoms mx-fail (cdr as))))))
(define defined-onto (->* (fails levels) unit)
  (lambda (mx-fail ls)
    (if (null? ls)
        #u
        (begin
          (tagcase (car ls)
            (lv-rib (as) (defined-atoms mx-fail as))
            (lv-shape (m)
              (let ((s (exp-shape mx-fail m)))
                (if (null? s)
                    #u
                    (tagcase (car s)
                      (sh-module (vs ds) (begin (set top-vals (names-onto vs (get top-vals)))
                                                (set top-descs (names-onto ds (get top-descs)))))
                      (else y #u)))))
            (else y #u))
          (defined-onto mx-fail (cdr ls))))))
;; What the top level's definitions in `form` bind, kept for the forms
;; after it.
(define defined-kept (->* (fails int syn) unit)
  (lambda (mx-fail root form)
    (tagcase (node-kids root)
      (k-form (x)
        (let ((ex (node-export x)))
          (if (null? ex) #u (defined-onto mx-fail (mx-shaper-exported mx-fail ex form)))))
      (else y #u))))
;;; ------------------------------------------------------------ a form's shapes written

;; Form `form`, a program's after expansion (a top-level form if `top`; or,
;; a file's items, a `module` of them): each `with`, `extend` and `include`
;; in it with what its module was relied on for written in, `#:relied
;; ((value …) (description …))`, the names asked of it that it gives, or
;; `#:relied #f` where its shape was not known where asked
;; (`binding-shapes.md`, step 2; `Macros::shaped`); and, at the top level,
;; what its definitions bind kept for the forms after it. Its own shape, as
;; an expression's, in the second.
(define shaped (->* (fails syn bool) (tupleof syn mx-parser-types-shapes))
  (lambda (mx-fail form top)
    (let* ((w (mx-walk-walked mx-fail form top)) (root (extract w 1)) (mx-nodes-found (extract w 2)))
      (begin
        (set shaping-top top)
        (shaper-begin! root mx-nodes-found)
        (asked-all mx-fail mx-nodes-found)
        (let ((written (writings 0 (get nnodes) nil)))
          (if top
              (begin (defined-kept mx-fail root form) (tuple (with-written form written) nil))
              (tuple (with-written form written) (exp-shape mx-fail root))))))))
(define-rec
;; Each load in `s`, where it starts and whether it is an input's, onto
;; `out`.
(loads-in (->* (syn (listof (tupleof int bool) acyclic)) (listof (tupleof int bool) acyclic))
  (lambda (s out)
    (tagcase s
      (mx-eager-reader-types-lst (items d a b x)
        (let* ((h (head-sym s))
               ;; A hidden global's definition, `(%loaded "key" "path")`
               ;; (`parse-loaded`), an input's if its key says so.
               (hoisted (and (symbol=? h '%loaded) (> (len items) 1) (syn-string? (nth items 1))))
               (input (or (symbol=? h 'load-input) (symbol=? h 'load-input-moduleof)
                          (and hoisted (input-key? (syn-string (nth items 1))))))
               (load (or hoisted
                         (and (or input (symbol=? h 'load-module) (symbol=? h 'load-moduleof))
                              (> (len items) 1) (syn-string? (nth items 1)))))
               (out2 (if load
                         (begin (if input (set inputs-at (cons a (get inputs-at))) #u)
                                (append out (list (tuple a input))))
                         out)))
          (loads-in-each items out2)))
      (dotted (items t d a b x) (loads-in t (loads-in-each items out)))
      (else y out))))
(loads-in-each
        (->* (syns-a (listof (tupleof int bool) acyclic)) (listof (tupleof int bool) acyclic))
  (lambda (xs out) (if (null? xs) out (loads-in-each (cdr xs) (loads-in (car xs) out))))))
;; The file of key `key` read, in `fs`: in a list of one.
(define file-read-keyed (->* (string loaded-files) loaded-files)
  (lambda (key fs)
    (cond ((null? fs) nil)
          ((and (string=? (extract (car fs) 7) key) (> (extract (car fs) 2) 0))
           (the loaded-files (list (car fs))))
          (else (file-read-keyed key (cdr fs))))))
(define file-out (ref (tupleof syn mx-parser-types-shapes) mxp) (new (tuple (mk-symbol "-" 0 0) (the mx-parser-types-shapes nil))))
(define file-shape-replaced
  (->* (string file-shape (listof file-shape acyclic)) (listof file-shape acyclic))
  (lambda (key x fs)
    (cond ((null? fs) nil)
          ((string=? key (extract (car fs) 1)) (cons x (cdr fs)))
          (else (cons (car fs) (file-shape-replaced key x (cdr fs)))))))
(define relied-at-2? (->* (syns-a) bool)
  (lambda (xs)
    (and (> (len xs) 2)
         (tagcase (nth xs 2)
           (atom (d a b x) (and (mx-keyword? d) (symbol=? (mx-keyword->symbol d) 'relied)))
           (else y #f)))))
(define-rec
(relied-in? (->* (syn) bool)
  (lambda (s)
    (tagcase s
      (mx-eager-reader-types-lst (items d a b x) (or (relied-at-2? items) (any-relied? items)))
      (dotted (items t d a b x) (or (relied-at-2? items) (any-relied? items) (relied-in? t)))
      (else y #f))))
(any-relied? (->* (syns-a) bool)
  (lambda (xs) (and (not (null? xs)) (or (relied-in? (car xs)) (any-relied? (cdr xs)))))))
;; `xs` without its third and fourth, a record's keyword and what follows.
(define cut-record (->* (syns-a) syns-a)
  (lambda (xs)
    (cons (car xs) (cons (nth xs 1) (if (> (len xs) 3) (cdr (cdr (cdr (cdr xs)))) nil)))))
(define-rec
;; `s` without what was written into it of the modules it opens (`#:relied
;; …`), as it was expanded: what a walk reads, whatever a program wrote
;; (`macros/shapes.rs`'s `unshaped`). Itself if nothing was.
(unshaped (->* (syn) syn)
  (lambda (s) (if (relied-in? s) (unshaped-in s) s)))
(unshaped-in (->* (syn) syn)
  (lambda (s)
    (tagcase s
      (mx-eager-reader-types-lst (items d a b x)
        (let ((xs (unshaped-items (if (relied-at-2? items) (cut-record items) items))))
          (mx-eager-reader-types-lst xs (syn-datums xs) a b x)))
      (dotted (items t d a b x)
        (let ((xs (unshaped-items (if (relied-at-2? items) (cut-record items) items))))
          (dotted xs (unshaped t) (syn-datums xs) a b x)))
      (else y s))))
(unshaped-items (->* (syns-a) syns-a)
  (lambda (xs) (if (null? xs) nil (cons (unshaped (car xs)) (unshaped-items (cdr xs)))))))
;; A file read from its `.fasl`, its forms the one expression the Rust
;; checker read it as (`(the T %fasl-value)`; a file of parameters', a
;; function of them; an input's, a module of the item `input`): its shape
;; that expression's (an input's, its `input`'s); its module's, what a
;; function makes.
(define fasl-kept! (->* (string syns-a bool) unit)
  (lambda (key forms input?)
    (if (or (null? forms) (not (null? (cdr forms))))
        #u
        (let* ((form (unshaped (car forms)))
               (r (prompt parse-tag
                          (begin (set file-out (shaped pfail-walk form #f)) (p-ok nil))
                          (lambda ((r presult)) r)))
               (ok (begin (set program-walk #f) (tagcase r (p-err (why a b) #f) (else y #t))))
               (whole (if ok (extract (get file-out) 2) nil))
               (m (if input? (mx-shaper-member-shape whole 'input #f) whole))
               (fun (and (not (null? m)) (tagcase (car m) (sh-fun (ps r) #t) (else y #f)))))
          (set file-shapes
               (file-shape-replaced key (tuple key m (if fun (result-of m) m) forms)
                                    (get file-shapes)))))))
;; The shapes named by `define-syntax` (`(define-syntax s (modnames …))`,
;; `binding-shapes.md`), each its `modnames` form, by its name.
(define named-shapes (ref mx-shapes-named mxp) (new nil))
(define named-shape-in (->* (symbol mx-shapes-named) (listof syn acyclic))
  (lambda (n ns)
    (cond ((null? ns) nil)
          ((symbol=? n (extract (car ns) 1)) (list (extract (car ns) 2)))
          (else (named-shape-in n (cdr ns))))))
(define shape-named (->* (symbol) (listof syn acyclic))
  (lambda (n) (named-shape-in n (get named-shapes))))
;; Whether `xs` are `ys`, each the same object.
(define same-items? (->* (syns-a syns-a) bool)
  (lambda (xs ys)
    (cond ((null? xs) (null? ys))
          ((null? ys) #f)
          (else (and #f (same-items? (cdr xs) (cdr ys)))))))
;; The `modnames` form type `t` is: itself, or the shape its name names; in
;; a list of one, none if neither.
(define mx-shapes-shape-of (->* (syn) (listof syn acyclic))
  (lambda (t)
    (cond ((syn-symbol? t) (shape-named (syn-symbol t)))
          ((string=? (mx-shaper-head-of t) "modnames") (list t))
          (else nil))))
(define-rec
;; Form `s` with each shape a name stands for written where a value supplies
;; its types, a `define`'s or `the`'s type, and each such `define` made
;; `(define x (the (modnames …) e))`, which the checkers seal there; outside
;; quotes. A shape used as a form's head, refused. As
;; `Macros::sealed_written`. Itself where nothing in it is written.
(sealed-written (->* (fails syn) syn)
  (lambda (mx-fail s)
    (tagcase s
      (mx-eager-reader-types-lst (items d a b x)
        (let ((h (head-sym s)))
          (cond
            ((or (symbol=? h 'quote) (symbol=? h 'quasiquote) (symbol=? h '%datum)) s)
            ((and (not (symbol=? h no-head)) (not (null? (shape-named h))))
             (begin (mx-fail (string-append "`" (string-append (symbol->string h) (string-append "` is a shape, not a macro: it is "
                                         "written as a `define`'s or `the`'s type")))
                          a b)
                    s))
            (else
             (let* ((xs (sealed-each mx-fail items))
                    (n (len xs))
                    (ys (cond ((and (symbol=? h 'define) (= n 4)
                                    (not (null? (mx-shapes-shape-of (nth xs 2)))))
                               (list (car xs) (nth xs 1)
                                     (mk-list (list (mk-symbol "the" a b)
                                                    (car (mx-shapes-shape-of (nth xs 2))) (nth xs 3))
                                              a b)))
                              ((and (symbol=? h 'the) (= n 3) (not (null? (mx-shapes-shape-of (nth xs 1)))))
                               (list (car xs) (car (mx-shapes-shape-of (nth xs 1))) (nth xs 2)))
                              (else xs))))
               (if (same-items? ys items) s (mx-eager-reader-types-lst ys (syn-datums ys) a b x)))))))
      (dotted (items t d a b x)
        (let ((xs (sealed-each mx-fail items)) (t2 (sealed-written mx-fail t)))
          (if (and (same-items? xs items) #f) s (dotted xs t2 (syn-datums xs) a b x))))
      (else y s))))
(sealed-each (->* (fails syns-a) syns-a)
  (lambda (mx-fail xs)
    (if (null? xs) nil (cons (sealed-written mx-fail (car xs)) (sealed-each mx-fail (cdr xs)))))))
(define sealed-out (ref syns-a mxp) (new nil))
;; File forms `xs`, sealed (`sealed-written`); as they are if that fails.
(define sealed-items (->* (syns-a) syns-a)
  (lambda (xs)
    (let ((r (prompt parse-tag (begin (set sealed-out (sealed-each pfail-walk xs)) (p-ok nil))
                     (lambda ((r presult)) r))))
      (tagcase r (p-err (why a b) xs) (else y (get sealed-out))))))
(define file-kept! (->* (string syns-a bool) unit)
  (lambda (key forms0 input?)
   (let ((forms (sealed-items (unshaped-items forms0))))
    (let* ((params (and (not (null? forms)) (string=? (mx-shaper-head-of (car forms)) "module-parameters")))
           (wrapped (not (and input? (not params))))
           (form (cond ((null? forms) nil)
                       (wrapped
                        (let* ((ab (syn-span (car forms))) (a (extract ab 1)) (b (extract ab 2)))
                          (list (mk-list (cons (mk-symbol "module" a b) forms) a b))))
                       ((null? (cdr forms)) (list (car forms)))
                       (else nil))))
      (if (null? form)
          #u
          (let* ((r (prompt parse-tag
                            (begin (set file-out (shaped pfail-walk (car form) #f)) (p-ok nil))
                            (lambda ((r presult)) r)))
                 (ok (begin (set program-walk #f) (tagcase r (p-err (why a b) #f) (else y #t))))
                 (m (if ok (extract (get file-out) 2) nil))
                 (written (if ok (extract (get file-out) 1) (car form)))
                 (vm (cond (params (tuple (sh-fun-of nil m) m))
                           ((and input? (not (null? m))
                                 (tagcase (car m) (sh-fun (ps r) #t) (else y #f)))
                            (tuple m (result-of m)))
                           (else (tuple m m)))))
            (set file-shapes
                 (file-shape-replaced key (tuple key (extract vm 1) (extract vm 2)
                                                 (if wrapped (cdr (proper-items written))
                                                     (list written)))
                                      (get file-shapes)))))))))
(define-rec
(loads-shaped! (->* ((listof (tupleof int bool) acyclic)) unit)
  (lambda (ls)
    (if (null? ls)
        #u
        (begin
          (let ((f (file-given-at (extract (car ls) 1) (get files-given))))
            (if (null? f) #u (file-shaped! (car f) (extract (car ls) 2))))
          (loads-shaped! (cdr ls))))))
;; File `f`'s shape, loaded as an input's if `input?`, found and kept, if
;; it was not: none while it is found, so that a cycle finds none; none if
;; it was not read, or is read from its `.fasl`, or does not walk.
(file-shaped! (->* (loaded-file bool) unit)
  (lambda (f input?)
    (let ((key (file-key f input?)))
      (if (not (null? (file-shape-keyed key (get file-shapes))))
          #u
          (begin
            (set file-shapes (cons (tuple key (the mx-parser-types-shapes nil) (the mx-parser-types-shapes nil) (extract f 5))
                                   (get file-shapes)))
            (cond ((<= (extract f 2) 0)
                   ;; A file a form before made, read too where a file that
                   ;; loads it imports a file's macros (`syn.rs`'s
                   ;; `supply_loaded_in`): as that reading is.
                   (let ((g (file-read-keyed (extract f 7) (get files-given))))
                     (if (null? g)
                         #u
                         (begin
                           (loads-shaped! (loads-in-each (extract (car g) 5) nil))
                           (file-kept! key (extract (car g) 5) input?)))))
                  ((extract f 8) (fasl-kept! key (extract f 5) input?))
                  (else
                   (begin
                     (loads-shaped! (loads-in-each (extract f 5) nil))
                     (file-kept! key (extract f 5) input?))))))))))
;;; ------------------------------------------------------------ files' shapes

;; Each file a load in `s` reads, its shape found and kept, those it loads
;; first (`Checker::file_shapes_for`): as its forms are given, before
;; expansion, as the Rust checker reads them.
(define files-shaped! (->* (syn) unit)
  (lambda (s) (loads-shaped! (loads-in s nil))))
;; Top-level form `x`, after expansion, with what each module it opens was
;; relied on for written in, the files it loads shaped first; as it is if
;; it does not walk, for the checkers to say why.
(define top-shaped (->* (syn) syn)
  (lambda (x0)
    (let ((x (unshaped x0)))
     (begin
      (files-shaped! x)
      (let ((r (prompt parse-tag
                       (begin (set file-out (shaped pfail-walk x #t)) (p-ok nil))
                       (lambda ((r presult)) r))))
        (begin
          (set program-walk #f)
          (tagcase r (p-err (why a b) x) (else y (extract (get file-out) 1)))))))))
(define shaped-each (->* (syns-a) (listof string acyclic))
  (lambda (forms)
    (if (null? forms)
        nil
        (cons (show-renamed (top-shaped (car forms))) (shaped-each (cdr forms))))))
;; Each of `forms`, a program's top-level forms after expansion, with what
;; each module it opens was relied on for written in (`shaped`), written:
;; for `tests/scopes.rs`, which holds the two expanders' records to one
;; another; the files read for its loads `fs`.
(define shaped-forms (->* (syns-a loaded-files) (listof string acyclic))
  (lambda (forms fs) (begin (shapes-reset! fs) (shaped-each forms))))
;; File `f`'s forms, loaded as an input's if `input?`, with what each
;; module they open was relied on for written in: its shape found first, if
;; it was not.
(define file-forms-written (->* (loaded-file bool) syns-a)
  (lambda (f input?)
    (begin
      (file-shaped! f input?)
      (let ((s (file-shape-keyed (file-key f input?) (get file-shapes))))
        (if (null? s) (extract f 5) (extract (car s) 4))))))
;; Files `fs` (`parser-loads.fx`'s), each with its forms as its shape wrote
;; them, where it was shaped: what the parser reads of a load.
(define forms-written-in (->* (loaded-files) loaded-files)
  (lambda (fs)
    (if (null? fs)
        nil
        (cons (let* ((f (car fs))
                     (s (file-shape-keyed (file-key f (load-input-at? (extract f 1)))
                                          (get file-shapes))))
                (if (null? s)
                    f
                    (product (1 (extract f 1)) (2 (extract f 2)) (3 (extract f 3)) (4 (extract f 4))
                             (5 (extract (car s) 4)) (6 (extract f 6)) (7 (extract f 7))
                             (8 (extract f 8)))))
              (forms-written-in (cdr fs))))))
(define shapes-by-state (ref (listof shapes-state acyclic) mxp) (new nil))
(define state-shapes (->* (int (listof shapes-state acyclic)) (listof shapes-state acyclic))
  (lambda (s xs)
    (cond ((null? xs) nil)
          ((= s (extract (car xs) 1)) (list (car xs)))
          (else (state-shapes s (cdr xs))))))
;; The top level's shapes as reading state `s` left them; the files read
;; for the forms to come, `fs`, none shaped yet: their forms are this
;; reading's, at its places, as the parser reads each file again.
(define shapes-at! (->* (int loaded-files) unit)
  (lambda (s fs)
    (let ((at (state-shapes s (get shapes-by-state))))
      (begin
        (set files-given fs)
        (set file-shapes nil)
        (set inputs-at nil)
        (if (null? at)
            (begin (set top-vals (nm-leaf)) (set top-descs (nm-leaf)) (set named-shapes nil))
            (begin (set top-vals (extract (car at) 2)) (set top-descs (extract (car at) 3))
                   (set named-shapes (extract (car at) 4))))))))
(define shapes-after! (->* (int) unit)
  (lambda (s)
    (if (and (names-empty? (get top-vals)) (names-empty? (get top-descs))
             (null? (get named-shapes)))
        #u
        (set shapes-by-state
             (cons (tuple s (get top-vals) (get top-descs) (get named-shapes))
                   (get shapes-by-state))))))
;; Whether `x` is `(define-syntax s (modnames …))`.
(define shape-definition? (->* (syn) bool)
  (lambda (x)
    (let ((items (proper-items x)))
      (and (= (len items) 3) (string=? (mx-shaper-head-of x) "define-syntax") (syn-symbol? (nth items 1))
           (string=? (mx-shaper-head-of (nth items 2)) "modnames")))))
(define named-without (->* (symbol mx-shapes-named) mx-shapes-named)
  (lambda (n ns)
    (cond ((null? ns) nil)
          ((symbol=? n (extract (car ns) 1)) (named-without n (cdr ns)))
          (else (cons (car ns) (named-without n (cdr ns)))))))
;; `(define-syntax s (modnames …))`, `x`: the shape kept, by its name.
(define shape-defined! (->* (syn) unit)
  (lambda (x)
    (let ((items (proper-items x)))
      (set named-shapes (cons (tuple (syn-symbol (nth items 1)) (nth items 2))
                              (named-without (syn-symbol (nth items 1)) (get named-shapes)))))))
(define shapes-count (->* ((listof file-shape acyclic) int) int)
  (lambda (fs n) (if (null? fs) n (shapes-count (cdr fs) (+ n 1)))))
;; How many files have been shaped (or are being).
(define files-shaped (->* () int)
  (lambda () (shapes-count (get file-shapes) 0)))
;; Hidden global `n`, `%shared:key`, defined as its file's module: its
;; shape the file's, for the forms after (as the Rust checker finds it from
;; its type), whose own parses forget the files'.
(define shared-shaped! (->* (symbol) unit)
  (lambda (n)
    (let* ((name (symbol->string n))
           (f (if (> (string-length name) 8)
                  (file-shape-keyed (substring name 8 (string-length name)) (get file-shapes))
                  nil)))
      (if (null? f) #u (set top-vals (names-with (get top-vals) n (extract (car f) 2)))))))
;;; ------------------------------------------------------------ aliases resolved

;; Hygiene, one mechanism (`binding-shapes.md`, step 3; `macros/hygiene.rs`):
;; top-level form `x`, after expansion, each alias an expansion left free,
;; `%~K.N:x`, made the name it stands for where its macro is defined
;; (`def-env-of`): a top-level macro's the top level's, written so (`(with
;; #%toplevel x)`, `#%toplevel:x`); a file's macro's the file's value,
;; `(with %shared:key x)`, else the standard one, `(with #%fx x)`; a module
;; file's own macro's the plain name, a binder at the use that would capture
;; it renamed, `%x~N`, and each reference to that binder; an alias the
;; expansion bound, here or at the top level before, a name of its own.
;; Itself if it has no alias. No alias reaches the checker. A captor's tag
;; is `tag`'s, the expander's (`macros-fresh.fx`'s `captor-tag`), whose
;; instance this one is not.
(define-type tagger (subr parses () string))
(define mx-shapes-digits-end (->* (string int) int)
  (lambda (s i)
    (if (and (< i (string-length s)) (tag-char? (string-ref s i))) (mx-shapes-digits-end s (+ i 1)) i)))
(define mx-shapes-alias-base-from (->* (string bool) string)
  (lambda (n any)
    (let ((c (if (and (> (string-length n) 2) (string=? (substring n 0 2) "%~"))
                 (mx-shapes-digits-end n 2)
                 -1)))
      (if (and (> c 2) (< c (string-length n)) (char=? (string-ref n c) #\:))
          (mx-shapes-alias-base-from (substring n (+ c 1) (string-length n)) #t)
          (if any n "")))))
;; `x`'s name as where its alias's macro was defined, if it is an alias
;; (`%~K.N:x`): every `%~K.N:` taken off; else "" (`macros-procedural.fx`'s).
(define mx-shapes-alias-base (->* (string) string)
  (lambda (n) (mx-shapes-alias-base-from n #f)))
(define-rec
;; Whether `s` holds an alias, outside quotes.
(alias-in? (->* (syn) bool)
  (lambda (s)
    (tagcase s
      (atom (d a b x) (and (syn-symbol? s) (not (string=? (mx-shapes-alias-base (syn-name s)) ""))))
      (mx-eager-reader-types-lst (items d a b x)
        (let ((h (head-sym s)))
          (and (not (or (symbol=? h 'quote) (symbol=? h 'quasiquote) (symbol=? h '%datum)))
               (aliases-in? items))))
      (dotted (items t d a b x) (or (aliases-in? items) (alias-in? t)))
      (else y #f))))
(aliases-in? (->* (syns-a) bool)
  (lambda (xs) (and (not (null? xs)) (or (alias-in? (car xs)) (aliases-in? (cdr xs)))))))
;; Whether alias `base`, written as `free` says, names the instance a file's
;; macro is defined in, `h` (kind 3): one of its values, `(with lib name)`,
;; or `lib` as a type's module, `(select lib T)` (`act-instance-type`).
(define instance-named? (->* (string int def-env) bool)
  (lambda (base free h)
    (and (= (extract h 1) 3)
         (or (and (= free 0) (mx-syntax-name-in? (string->symbol base) (extract h 3)))
             (and (= free 1) (string=? base (symbol->string (extract h 2))))))))
(define cdrs-of (->* (addr int) int)
  (lambda (p k) (if (or (null? p) (car p)) k (cdrs-of (cdr p) (+ k 1)))))
(define list-tail-of (->* (addr int) addr)
  (lambda (p k) (if (= k 0) p (list-tail-of (cdr p) (- k 1)))))
;; Whether path `p`, in `s`, outermost step first, is inside a `(module …)`
;; of it, not its head (`hygiene.rs`'s `within_module`).
(define within-module? (->* (syn addr) bool)
  (lambda (s p)
    (let ((items (tagcase s (mx-eager-reader-types-lst (xs d a b y) xs) (else y nil))) (k (cdrs-of p 0)))
      (cond ((null? items) #f)
            ((and (not (null? p)) (syn-symbol? (car items))
                  (string=? (syn-name (car items)) "module"))
             #t)
            ((or (null? (list-tail-of p k)) (>= k (len items))) #f)
            (else (within-module? (nth items k) (list-tail-of p (+ k 1))))))))
(define mx-shapes-int-in? (->* (int mx-shapes-ints) bool)
  (lambda (k ks) (and (not (null? ks)) (or (= k (car ks)) (mx-shapes-int-in? k (cdr ks))))))
;; Binder `c` (`captor-of`'s) onto `cs`, if it captures `base`, which a
;; module file's own macro introduces at `s`, or the instance an imported
;; macro names, if `inst`, in form `s`: not if it is none, or one of `top`;
;; refused where a module's names come first, which may give it, or inside a
;; `module` of form `x`, where it may be an item, which its name labels.
(define captor-noted (->* (fails int mx-shapes-ints string bool syn syn mx-shapes-ints) mx-shapes-ints)
  (lambda (mx-fail c top base inst s x cs)
    (cond ((= c -1) cs)
          ((and (>= c 0) (not (mx-shapes-int-in? c top)) (within-module? x (reverse (node-path c))))
           (begin (fail-at mx-fail (string-append "`" (string-append base (string-append "`, which an expansion here names as its "
                                               (string-append "macro has it, is bound inside a `module` around "
                                               (string-append "the use, where renaming it may change what the "
                                               "module gives: not yet")))))
                           s)
                  cs))
          ((= c -2)
           (begin (fail-at mx-fail (string-append "`" (string-append base
                                               (string-append (if inst
                                                   "`, the instance an imported macro names, "
                                                   "`, which a macro of this file names, ")
                                               (string-append "where a `with` or `include` around its use may "
                                               "give it: not yet"))))
                           s)
                  cs))
          ((or (mx-shapes-int-in? c top) (mx-shapes-int-in? c cs)) cs)
          (else (cons c cs)))))
;; The binder `n` (a description's if `d`) refers to in `env`, innermost
;; first: -1 if none; -2 if a module's names come first, which may have it.
(define captor-of (->* (symbol bool mx-env-indexed-env) int)
  (lambda (n d mx-env-indexed-env)
    (let ((l (env-lookup mx-env-indexed-env n d 0 mx-nodes-binder-name)))
      (case (extract l 1) ((0) (extract l 2)) ((1) -2) (else -1)))))
;; Alias `base`, of a description's if `d`, written as `free` says (0 a
;; value, 1 qualified, 2 as written, 3 a `with`'s module) where its macro
;; is defined, `h`, at `s`'s place (`macros/hygiene.rs`'s `top_made`).
(define alias-written (->* (fails string bool int def-env syn) syn)
  (lambda (mx-fail base d free h s)
    (let* ((ab (syn-span s)) (a (extract ab 1)) (b (extract ab 2))
           (with-of (lambda ((m string))
                      (mk-list (list (mk-symbol "with" a b) (mk-symbol m a b) (mk-symbol base a b))
                               a b))))
      (case (extract h 1)
        ((0)
          (cond ((or (= free 0) (and (= free 3) (not d))) (with-of "#%toplevel"))
                ((= free 2) (mk-symbol base a b))
                (else (mk-symbol (string-append "#%toplevel:" base) a b))))
        ((2) (mk-symbol base a b))
        (else
          (cond ((and (= free 0) (mx-syntax-name-in? (string->symbol base) (extract h 3)))
                 (with-of (symbol->string (extract h 2))))
                ((instance-named? base free h) (mk-symbol base a b))
                ((= free 0) (with-of "#%fx"))
                ((= free 2) (mk-symbol base a b))
                ;; A file's macro writes its descriptions through its module
                ;; whole (`act-module-type`).
                (else
                 (begin (fail-at mx-fail (string-append "`" (string-append base (string-append "`: a file's macro's description "
                                                     "named apart from its module, not yet")))
                                 s)
                        s))))))))
;; Where the expansion of tag `k` had its macro defined, of `hs`: the top
;; level if none is said.
(define def-env-of (->* (string mx-nodes-def-envs) def-env)
  (lambda (k hs)
    (cond ((null? hs) (def-env-top))
          ((string=? k (extract (car hs) 1)) (extract (car hs) 2))
          (else (def-env-of k (cdr hs))))))
;; The tag of the expansion that made alias `x`, its outermost `%~T:`'s
;; `T`; else "".
(define alias-made (->* (string) string)
  (lambda (x)
    (let ((c (if (and (> (string-length x) 2) (string=? (substring x 0 2) "%~"))
                 (mx-shapes-digits-end x 2)
                 -1)))
      (if (and (> c 2) (< c (string-length x)) (char=? (string-ref x c) #\:))
          (substring x 2 c)
          ""))))
;; The same, looked for from place `from`, past each module's names.
(define bound-from? (->* (symbol bool mx-env-indexed-env int) bool)
  (lambda (n d mx-env-indexed-env from)
    (let ((l (env-lookup mx-env-indexed-env n d from mx-nodes-binder-name)))
      (case (extract l 1) ((0) #t) ((1) (bound-from? n d mx-env-indexed-env (extract l 3))) (else #f)))))
;; Whether a binder in `env` binds `n`, a description's if `d`.
(define mx-shapes-bound-in? (->* (symbol bool mx-env-indexed-env) bool)
  (lambda (n d mx-env-indexed-env) (bound-from? n d mx-env-indexed-env 0)))
;; Each alias an expansion defined at the top level, a description's if
;; marked: a name of its own wherever later forms name it.
(define aliases-defined (ref (listof (tupleof symbol bool) acyclic) mxp) (new nil))
(define alias-defined-in? (->* (symbol bool (listof (tupleof symbol bool) acyclic)) bool)
  (lambda (n d xs)
    (and (not (null? xs))
         (or (and (symbol=? n (extract (car xs) 1)) (mx-eq? d (extract (car xs) 2)))
             (alias-defined-in? n d (cdr xs))))))
(define alias-defined? (->* (symbol bool) bool)
  (lambda (n d) (alias-defined-in? n d (get aliases-defined))))
(define aliases-written (->* (fails mx-nodes-found mx-nodes-def-envs mx-shapes-ints syn mx-shapes-written-of) mx-shapes-written-of)
  (lambda (mx-fail fs hs top x out)
    (if (null? fs)
        out
        (aliases-written
          mx-fail (cdr fs) hs top x
          (let ((at (extract (car fs) 1)) (mx-env-indexed-env (extract (car fs) 2)))
            (tagcase (node-kids at)
              (k-ref (n d free)
                (let* ((base (mx-shapes-alias-base (symbol->string n)))
                       (h (def-env-of (alias-made (symbol->string n)) hs))
                       (within (= (extract h 1) 2))
                       (inst (instance-named? base free h)))
                  (if (or (string=? base "") (mx-shapes-bound-in? n d mx-env-indexed-env)
                          (and (not within) (alias-defined? n d)))
                      out
                      (let ((mx-shapes-parts (cons (tuple (node-path at)
                                                (alias-written mx-fail base d free h (node-syn at)))
                                         (extract out 1))))
                        (cond
                          (within
                           (tuple mx-shapes-parts (captor-noted mx-fail (captor-of (string->symbol base) d mx-env-indexed-env)
                                                      top base #f (node-syn at) x (extract out 2))))
                          (inst
                           (tuple mx-shapes-parts (captor-noted mx-fail (captor-of (extract h 2) #f mx-env-indexed-env) top
                                                      (symbol->string (extract h 2)) #t x x
                                                      (extract out 2))))
                          (else (tuple mx-shapes-parts (extract out 2))))))))
              (else y out)))))))
(define atms-binders (->* (atms) mx-shapes-ints)
  (lambda (as)
    (if (null? as)
        nil
        (append (tagcase (car as) (atm-lit (b d) (list b)) (else y nil))
                (atms-binders (cdr as))))))
(define levels-binders (->* (levels) mx-shapes-ints)
  (lambda (ls)
    (if (null? ls)
        nil
        (append (tagcase (car ls) (lv-rib (as) (atms-binders as)) (else y nil))
                (levels-binders (cdr ls))))))
;; The binders a form's definitions bind (its root `root`'s export).
(define root-binders (->* (fails int syn) mx-shapes-ints)
  (lambda (mx-fail root x)
    (tagcase (node-kids root)
      (k-form (f)
        (let ((ex (node-export f)))
          (if (null? ex) nil (levels-binders (mx-shaper-exported mx-fail ex x)))))
      (else y nil))))
(define captors-named (->* (mx-shapes-ints captors tagger) captors)
  (lambda (cs out tag)
    (if (null? cs)
        out
        (tagcase (node-kids (car cs))
          (k-binder (n e)
            (captors-named (cdr cs)
                           (cons (tuple (car cs)
                                        (string->symbol
                                         (string-append "%" (string-append (symbol->string n) (string-append "~" (tag))))))
                                 out)
                           tag))
          (else y (captors-named (cdr cs) out tag))))))
(define renamed-at (->* (symbol syn) syn)
  (lambda (n s)
    (let ((ab (syn-span s))) (mk-symbol (symbol->string n) (extract ab 1) (extract ab 2)))))
;; Each captor of `fresh` written as its fresh name, onto `out`.
(define binders-renamed (->* (captors mx-shapes-parts) mx-shapes-parts)
  (lambda (fresh out)
    (if (null? fresh)
        out
        (binders-renamed (cdr fresh)
                         (cons (tuple (node-path (extract (car fresh) 1))
                                      (renamed-at (extract (car fresh) 2)
                                                  (node-syn (extract (car fresh) 1))))
                               out)))))
(define captor-name (->* (int captors) (listof symbol acyclic))
  (lambda (b fresh)
    (cond ((null? fresh) nil)
          ((= b (extract (car fresh) 1)) (list (extract (car fresh) 2)))
          (else (captor-name b (cdr fresh))))))
;; Each reference `fs` reaches to a captor of `fresh`, written as its fresh
;; name, onto `out`.
(define captors-renamed (->* (mx-nodes-found captors mx-shapes-parts) mx-shapes-parts)
  (lambda (fs fresh out)
    (if (or (null? fs) (null? fresh))
        out
        (captors-renamed
          (cdr fs) fresh
          (let ((at (extract (car fs) 1)))
            (tagcase (node-kids at)
              (k-ref (n d free)
                (let ((f (if (string=? (mx-shapes-alias-base (symbol->string n)) "")
                             (captor-name (captor-of n d (extract (car fs) 2)) fresh)
                             nil)))
                  (if (null? f)
                      out
                      (cons (tuple (node-path at) (renamed-at (car f) (node-syn at))) out))))
              (else y out)))))))
(define atms-aliases-noted (->* (atms) unit)
  (lambda (as)
    (if (null? as)
        #u
        (begin
          (tagcase (car as)
            (atm-lit (b d)
              (tagcase (node-kids b)
                (k-binder (n e)
                  (if (string=? (mx-shapes-alias-base (symbol->string n)) "")
                      #u
                      (set aliases-defined (cons (tuple n d) (get aliases-defined)))))
                (else y #u)))
            (else y #u))
          (atms-aliases-noted (cdr as))))))
(define aliases-noted (->* (levels) unit)
  (lambda (ls)
    (if (null? ls)
        #u
        (begin
          (tagcase (car ls) (lv-rib (as) (atms-aliases-noted as)) (else y #u))
          (aliases-noted (cdr ls))))))
;; The aliases form `x`'s definitions (its root `root`'s export) bind, noted.
(define aliases-defined-in (->* (fails int syn) unit)
  (lambda (mx-fail root x)
    (tagcase (node-kids root)
      (k-form (f)
        (let ((ex (node-export f)))
          (if (null? ex) #u (aliases-noted (mx-shaper-exported mx-fail ex x)))))
      (else y #u))))
;; `ws`, each its path outermost step first (`writings-from-top`).
(define parts-from-top (->* (mx-shapes-parts) mx-shapes-parts)
  (lambda (ws)
    (if (null? ws)
        nil
        (cons (tuple (the addr (reverse (extract (car ws) 1))) (extract (car ws) 2))
              (parts-from-top (cdr ws))))))
;; Those of `ws` whose next step is `step`, past it; past `k` cdrs.
(define part-step (->* (mx-shapes-parts bool) mx-shapes-parts)
  (lambda (ws step)
    (cond ((null? ws) nil)
          ((let ((p (extract (car ws) 1))) (and (not (null? p)) (mx-eq? (car p) step)))
           (cons (tuple (cdr (extract (car ws) 1)) (extract (car ws) 2)) (part-step (cdr ws) step)))
          (else (part-step (cdr ws) step)))))
(define parts-past (->* (mx-shapes-parts int) mx-shapes-parts)
  (lambda (ws k) (if (= k 0) ws (parts-past (part-step ws #f) (- k 1)))))
(define mx-shapes-part-at (->* (addr mx-shapes-parts) (listof syn acyclic))
  (lambda (p ws)
    (cond ((null? ws) nil)
          ((mx-walk-addr=? p (extract (car ws) 1)) (list (extract (car ws) 2)))
          (else (mx-shapes-part-at p (cdr ws))))))
(define-rec
;; `s` with each of `ws`, under it by the steps left of each, put in place:
;; itself where none is; each step taken once (`written-under`).
(parts-under (->* (syn mx-shapes-parts) syn)
  (lambda (s ws)
    (let ((w (mx-shapes-part-at nil ws)))
      (cond ((not (null? w)) (car w))
            ((null? ws) s)
            (else
             (tagcase s
               (mx-eager-reader-types-lst (items d a b x)
                 (let ((xs (parts-items items ws))) (mx-eager-reader-types-lst xs (syn-datums xs) a b x)))
               (dotted (items t d a b x)
                 (let ((xs (parts-items items ws)))
                   (dotted xs (parts-under t (parts-past ws (len items))) (syn-datums xs) a b x)))
               (else y s)))))))
(parts-items (->* (syns-a mx-shapes-parts) syns-a)
  (lambda (xs ws)
    (if (null? xs)
        nil
        (cons (parts-under (car xs) (part-step ws #t)) (parts-items (cdr xs) (part-step ws #f)))))))
;; `s`, at `path` (innermost step first, as a walk's), each part of `ws` at
;; its path written so.
(define with-parts (->* (syn addr (listof (tupleof addr syn) acyclic)) syn)
  (lambda (s path ws) (parts-under s (parts-from-top ws))))
(define aliases-resolved (->* (fails syn mx-nodes-def-envs tagger) syn)
  (lambda (mx-fail x hs tag)
    (if (not (alias-in? x))
        x
        (let* ((w (mx-walk-walked mx-fail x #t)) (root (extract w 1)) (mx-nodes-found (extract w 2))
               (made (aliases-written mx-fail mx-nodes-found hs (root-binders mx-fail root x) x
                                      (tuple (the mx-shapes-parts nil) (the mx-shapes-ints nil))))
               (fresh (captors-named (extract made 2) nil tag))
               (written (append (captors-renamed mx-nodes-found fresh (binders-renamed fresh nil))
                                (extract made 1))))
          (begin (aliases-defined-in mx-fail root x)
                 (with-parts x nil written))))))

"macros-driver.kfx: the engine's driver of Fixpt's expander (C431), written
 by hand, read into the expander's module last (scripts/macros-port.py),
 as Fixpt's `parser-top.fx` drives it (`parse-tops`, `top-made`,
 `expanded-tops`, `written-top`): each top-level form, in order, made
 Fixpt's syntax; a `define-syntax` defined, its templates' checks (a
 typed macro's) given the checker as hidden definitions; any other form
 expanded, each form it makes shaped, sealed and its aliases resolved;
 and made the reader's datums again, for kayak's checker and compiler,
 which see only expansions, as Fixpt's checkers do.

 Places: a file's positions as Fixpt's are, each file `load-base` apart,
 its index the order it is met in (`drive-files`); a part a template made,
 its key's place in the template (`source-of`). A list's line, kayak's
 checker's place, kept by its position (`drive-lines`). Not yet: a file's
 macros (`import-with-syntax`), refused, saying so. A procedural macro's
 transformer, and `begin-for-syntax`, run by the host (C446), asked by a
 call (C449, `drive-asker!`)."

;;; ------------------------------------------------------------ places

(define drive-files (ref (listof string acyclic) mxp) (new nil))
"Each list's position and line, as met, the last first; then, sorted,
 the positions and the lines."
(define drive-met (ref (listof (tupleof int int) acyclic) mxp) (new nil))
;; Arrays of the standard environment, not the prelude's flat arrays, which
;; a file loaded does not see (C443).
(define drive-at (ref (arrayof i64 mxp) mxp) (new (make-array 1 (the i64 0))))
(define drive-line (ref (arrayof i64 mxp) mxp) (new (make-array 1 (the i64 0))))
(define drive-count (ref i64 mxp) (new 0))

(define drive-length (->* ((listof string acyclic)) int)
  (lambda (fs) (if (null? fs) 0 (+ 1 (drive-length (cdr fs))))))
(define drive-nth (->* ((listof string acyclic) int) string)
  (lambda (fs i) (if (= i 0) (car fs) (drive-nth (cdr fs) (- i 1)))))
(define drive-index-in (->* (string (listof string acyclic) int) int)
  (lambda (f fs i)
    (cond ((null? fs) -1) ((string=? f (car fs)) i) (else (drive-index-in f (cdr fs) (+ i 1))))))
"File `f`'s index, given the first time it is met."
(define drive-file-index (->* (string) int)
  (lambda (f)
    (let ((i (drive-index-in f (get drive-files) 0)))
      (if (>= i 0)
          i
          (begin (set drive-files (append (get drive-files) (list f))) (- (drive-length (get drive-files)) 1))))))
(define drive-place (->* (string i64) int)
  (lambda (f pos) (+ (* (drive-file-index f) load-base) (i64->int pos))))

(define drive-merge (->* ((listof (tupleof int int) acyclic) (listof (tupleof int int) acyclic)) (listof (tupleof int int) acyclic))
  (lambda (xs ys)
    (cond ((null? xs) ys)
          ((null? ys) xs)
          ((<= (extract (car xs) 1) (extract (car ys) 1)) (cons (car xs) (drive-merge (cdr xs) ys)))
          (else (cons (car ys) (drive-merge xs (cdr ys)))))))
(define drive-split (->* ((listof (tupleof int int) acyclic) (listof (tupleof int int) acyclic) (listof (tupleof int int) acyclic))
                         (tupleof (listof (tupleof int int) acyclic) (listof (tupleof int int) acyclic)))
  (lambda (xs a b) (if (null? xs) (tuple a b) (drive-split (cdr xs) b (cons (car xs) a)))))
(define drive-sorted (->* ((listof (tupleof int int) acyclic)) (listof (tupleof int int) acyclic))
  (lambda (xs)
    (if (or (null? xs) (null? (cdr xs)))
        xs
        (let ((halves (drive-split xs nil nil)))
          (drive-merge (drive-sorted (extract halves 1)) (drive-sorted (extract halves 2)))))))
(define drive-count-of (->* ((listof (tupleof int int) acyclic) i64) i64)
  (lambda (xs n) (if (null? xs) n (drive-count-of (cdr xs) (i64+ n 1)))))
(define drive-fill! (->* ((listof (tupleof int int) acyclic) i64) unit)
  (lambda (xs i)
    (if (null? xs)
        #u
        (begin (array-set! (get drive-at) (i64->int i) (int->i64 (extract (car xs) 1)))
               (array-set! (get drive-line) (i64->int i) (int->i64 (extract (car xs) 2)))
               (drive-fill! (cdr xs) (i64+ i 1))))))
"The lines met, made a table to search."
(define drive-lines! (->* () unit)
  (lambda ()
    (let* ((xs (drive-sorted (get drive-met))) (n (i64+ 1 (drive-count-of xs 0))))
      (begin (set drive-at (make-array (i64->int n) (the i64 0)))
             (set drive-line (make-array (i64->int n) (the i64 1)))
             (set drive-count (i64- n 1))
             (drive-fill! xs 0)))))
"The line of the last list met at or before place `a`, by halving."
(define drive-search (->* (i64 i64 i64) i64)
  (lambda (a lo hi)
    (if (i64>= lo hi)
        (if (i64= lo 0) 1 (array-ref (get drive-line) (i64->int (i64- lo 1))))
        (let ((mid (i64-quotient (i64+ lo hi) 2)))
          (if (i64<= (array-ref (get drive-at) (i64->int mid)) a)
              (drive-search a (i64+ mid 1) hi)
              (drive-search a lo mid))))))
"Place `a`, a text's or a key a template's part has, as the file, the
 line and the position in it."
(define drive-placed (->* (int) (tupleof string i64 i64))
  (lambda (a0)
    (let* ((a (if (>= a0 source-base) (extract (mx-procedural-source-of a0) 1) a0))
           (i (quotient a load-base))
           (fs (get drive-files)))
      (if (or (>= a source-base) (>= i (drive-length fs)))
          (tuple "" 0 0)
          (tuple (drive-nth fs i) (drive-search (int->i64 a) 0 (get drive-count)) (int->i64 (remainder a load-base)))))))

;;; ------------------------------------------------------------ located datums and syntax

"The forms given and made, as located datums (C443), which the engine's
 reading makes of its own and reads back (`macros-bridge.kfx`): a node an
 atom, `(0 file line pos d)`, or a list, `(1 file line pos node ...)`; a
 dotted list's `.` and a vector's `#vector` among its nodes, as kayak's
 reader reads them. Standard data, so that this file, an instance made in
 a place of its own, sees nothing of the program that loads it."
(define loc-nth (->* (datum int) datum)
  (lambda (d i) (if (pair? d) (if (= i 0) (car d) (loc-nth (cdr d) (- i 1))) (the datum nil))))
(define loc-after (->* (datum int) datum)
  (lambda (d i) (if (and (pair? d) (> i 0)) (loc-after (cdr d) (- i 1)) d)))
(define loc-int (->* (datum) int) (lambda (d) (if (int? d) d 0)))
(define loc-string (->* (datum) string) (lambda (d) (if (string? d) d "")))
(define loc-nodes (->* (datum) (listof datum acyclic))
  (lambda (d) (if (pair? d) (cons (car d) (loc-nodes (cdr d))) nil)))
"Whether node `n` is the atom `name`, a symbol."
(define loc-is? (->* (datum string) bool)
  (lambda (n name)
    (and (= (loc-int (loc-nth n 0)) 0)
         (let ((d (loc-nth n 4))) (and (symbol? d) (string=? (symbol->string d) name))))))
"Whether a list's nodes are dotted: `.` its next to last."
(define drive-dotted? (->* ((listof datum acyclic)) bool)
  (lambda (xs)
    (cond ((or (null? xs) (null? (cdr xs)) (null? (cdr (cdr xs)))) #f)
          ((null? (cdr (cdr (cdr xs)))) (and (loc-is? (car (cdr xs)) ".") (not (loc-is? (car xs) "."))))
          (else (drive-dotted? (cdr xs))))))
(define drive-before-dot (->* ((listof datum acyclic)) (listof datum acyclic))
  (lambda (xs) (if (loc-is? (car xs) ".") nil (cons (car xs) (drive-before-dot (cdr xs))))))
(define drive-last (->* ((listof datum acyclic)) datum)
  (lambda (xs) (if (null? (cdr xs)) (car xs) (drive-last (cdr xs)))))
(define-rec
  (drive-syn (->* (datum) syn)
    (lambda (e)
      "A located datum as Fixpt's syntax, its list's line kept."
      (let ((a (drive-place (loc-string (loc-nth e 1)) (int->i64 (loc-int (loc-nth e 3))))))
        (if (= (loc-int (loc-nth e 0)) 1)
            (let ((xs (loc-nodes (loc-after e 4))))
              (begin
                (set drive-met (cons (tuple a (loc-int (loc-nth e 2))) (get drive-met)))
                (cond ((and (not (null? xs)) (loc-is? (car xs) "#vector"))
                       (let ((items (drive-syns (cdr xs)))) (vec items (mx-datum-list (syn-datums items)) a a 0)))
                      ((drive-dotted? xs)
                       (let ((items (drive-syns (drive-before-dot xs))) (tail (drive-syn (drive-last xs))))
                         (dotted items tail (mx-datum-dotted (syn-datums items) (syn->datum tail)) a a 0)))
                      (else (let ((items (drive-syns xs))) (mk-list items a a))))))
            (atom (loc-nth e 4) a a 0)))))
  (drive-syns (->* ((listof datum acyclic)) syns-a)
    (lambda (xs) (if (null? xs) nil (cons (drive-syn (car xs)) (drive-syns (cdr xs)))))))

"Node of kind `k` at `f`, `l`, `p`: an atom of `d`, or a list of nodes `d`."
(define loc-dotted (->* ((listof datum acyclic) datum) datum)
  (lambda (xs tail) (if (null? xs) tail (the datum (cons (car xs) (loc-dotted (cdr xs) tail))))))
(define loc-node (->* (int string i64 i64 datum) datum)
  (lambda (k f l p d)
    (loc-dotted (list (the datum k) (the datum f) (the datum (i64->int l)) (the datum (i64->int p)))
               (if (= k 0) (loc-dotted (list d) (the datum nil)) d))))
"What the expander noted of the `with` at `a`..`b` (`with-uses`), for the
 checker (Fixpt's `with_as`; C440): each macro its body uses by a name
 (0), each name a template's `(with m name)` projects (1), and each path
 `m::name` (2, its path for the name; C474, Fixpt's `TODO.md` §296), as
 `(name KIND)`, in the order met."
(define drive-uses-in (->* (mx-exports-uses int int string i64 i64) (listof datum acyclic))
  (lambda (us a b f l p)
    (cond ((null? us) nil)
          ((and (= (extract (car us) 1) a) (= (extract (car us) 2) b))
           (cons (loc-node 1 f l p (mx-datum-list (list (loc-node 0 f l p (the datum (extract (car us) 3)))
                                                        (loc-node 0 f l p (the datum (extract (car us) 6))))))
                 (drive-uses-in (cdr us) a b f l p)))
          (else (drive-uses-in (cdr us) a b f l p)))))
(define-rec
  (drive-loc (->* (syn) datum)
    (lambda (s)
      "Syntax as a located datum, where its place says."
      (let* ((w (drive-placed (syn-start s))) (f (extract w 1)) (l (extract w 2)) (p (extract w 3)))
        (tagcase s
          (atom (d a b x) (loc-node 0 f l p d))
          (mx-eager-reader-types-lst (items d a b x)
            (let* ((xs (drive-locs items))
                   ;; A `with`'s uses noted, written after its module (C440).
                   (us (if (and (>= (len items) 3) (syn-symbol? (car items)) (string=? (syn-name (car items)) "with"))
                           (drive-uses-in (with-uses-given) a b f l p)
                           nil)))
              (loc-node 1 f l p (mx-datum-list (if (null? us) xs (cons (car xs) (cons (car (cdr xs)) (cons (loc-node 0 f l p (the datum (string->symbol "#:uses"))) (cons (loc-node 1 f l p (mx-datum-list us)) (cdr (cdr xs)))))))))))
          (dotted (items tail d a b x)
            (loc-node 1 f l p (mx-datum-list (append (drive-locs items) (list (loc-node 0 f l p (the datum (string->symbol "."))) (drive-loc tail))))))
          (vec (items d a b x) (loc-node 1 f l p (mx-datum-list (cons (loc-node 0 f l p (the datum (string->symbol "#vector"))) (drive-locs items)))))))))
  (drive-locs (->* (syns-a) (listof datum acyclic))
    (lambda (xs) (if (null? xs) nil (cons (drive-loc (car xs)) (drive-locs (cdr xs)))))))

;;; ------------------------------------------------------------ forms

(define drive-bare-names (->* (syns-a) mx-parser-types-names)
  (lambda (xs)
    (cond ((null? xs) nil)
          ((syn-symbol? (car xs)) (cons (syn-symbol (car xs)) (drive-bare-names (cdr xs))))
          (else (drive-bare-names (cdr xs))))))
(define drive-heads (->* (syns-a) mx-parser-types-names)
  (lambda (vs)
    (cond ((null? vs) nil)
          ((and (not (null? (proper-items (car vs)))) (syn-symbol? (car (proper-items (car vs)))))
           (cons (syn-symbol (car (proper-items (car vs)))) (drive-heads (cdr vs))))
          (else (drive-heads (cdr vs))))))
(define drive-rec-names (->* (syns-a) mx-parser-types-names)
  (lambda (bs)
    (cond ((null? bs) nil)
          ((and (not (null? (proper-items (car bs)))) (syn-symbol? (car (proper-items (car bs)))))
           (cons (syn-symbol (car (proper-items (car bs)))) (drive-rec-names (cdr bs))))
          (else (drive-rec-names (cdr bs))))))
"The names a form the checker reads defines as values: as Fixpt's parser's
 `top-names` finds them, of the forms an expansion makes."
(define drive-head (->* (syn) string)
  (lambda (s) (let ((xs (proper-items s))) (if (and (not (null? xs)) (syn-symbol? (car xs))) (syn-name (car xs)) ""))))
(define drive-names (->* (syn) mx-parser-types-names)
  (lambda (s)
    (let ((xs (proper-items s)) (h (drive-head s)))
      (cond ((and (or (string=? h "define") (string=? h "define*")) (>= (len xs) 3) (syn-symbol? (nth xs 1)))
             (list (syn-symbol (nth xs 1))))
            ((string=? h "define-rec") (drive-rec-names (cdr xs)))
            ;; `(import m n … (type t …) (effect e …))`: its values, each a
            ;; definition (the parser's `expand-import`).
            ((and (string=? h "import") (>= (len xs) 2)) (drive-bare-names (cdr (cdr xs))))
            ;; A datatype's constructors (`expand-datatype`'s definitions).
            ((and (string=? h "define-datatype") (>= (len xs) 3)) (drive-heads (cdr (cdr xs))))
            ;; A generative type's conversions, `#:seal mk #:unseal get`.
            ((and (string=? h "define-generative") (> (len xs) 3)) (drive-bare-names (cdr (cdr (cdr xs)))))
            ((and (or (string=? h "define-syscall") (string=? h "define-optional-syscall")) (>= (len xs) 2) (syn-symbol? (nth xs 1)))
             (list (syn-symbol (nth xs 1))))
            (else nil)))))
"Each name a form defines: a macro's, refused (`guard-defs`); else a
 global from now on."
(define drive-guard-names (->* (mx-parser-types-names syn) unit)
  (lambda (ns s)
    (cond ((null? ns) #u)
          ((macro? (car ns))
           (pfail-at (string-append "`" (string-append (symbol->string (car ns)) "` is a macro's name, which is not defined again"))
                     (syn-start s) (syn-end s)))
          (else (begin (globals! (list (car ns))) (drive-guard-names (cdr ns) s))))))
(define drive-guard (->* (syns-a) unit)
  (lambda (xs)
    (if (null? xs)
        #u
        (begin (drive-guard-names (drive-names (car xs)) (car xs)) (drive-guard (cdr xs))))))
"Macros `ns`, of a name defined as a global already: refused at `s`."
(define drive-taken (->* (mx-parser-types-names syn) unit)
  (lambda (ns s)
    (cond ((null? ns) #u)
          ((global-now? (car ns))
           (pfail-at (string-append "`" (string-append (symbol->string (car ns)) "` is defined already: a macro may not take its name"))
                     (syn-start s) (syn-end s)))
          (else (drive-taken (cdr ns) s)))))
"A form an expansion made, as the parser is given it (`written-top`)."
(define drive-written (->* (syn) syn)
  (lambda (x)
    (let ((h (drive-head x)))
      (if (or (string=? h "include") (string=? h "import") (string=? h "define-datatype"))
          x
          (top-shaped (aliases-resolved pfail-at (sealed-written pfail-at x) (def-envs-given) captor-tag))))))
(define drive-written-each (->* (syns-a) syns-a)
  (lambda (xs) (if (null? xs) nil (cons (drive-written (car xs)) (drive-written-each (cdr xs))))))
"Each check of macros `ns`' templates, expanded (`parse-checks`)."
(define drive-checked (->* (syns-a) syns-a)
  (lambda (cs)
    (if (null? cs)
        nil
        (cons (aliases-resolved pfail-at (expand pfail-at (car cs)) (def-envs-given) captor-tag) (drive-checked (cdr cs))))))
(define drive-checks (->* (mx-parser-types-names) syns-a)
  (lambda (ns)
    (if (null? ns)
        nil
        (append (drive-checked (template-checks pfail-at (car ns))) (drive-checks (cdr ns))))))
(define drive-not-yet (->* (syn string) syns-a)
  (lambda (s what)
    (begin (pfail-at (string-append what ": not yet in kayak's expander") (syn-start s) (syn-end s)) nil)))
"Top-level form `s` begun (Fixpt's `form-begun!`): its key, which its fresh
 names and aliases are of (`form-begin!`); and what the expander keeps
 within one form as at any form's start, whatever a failure before left."
(define drive-begun! (->* (syn) unit)
  (lambda (s)
    (begin (form-begin! s) (made-syntax-taken!) (set program-walk #f) (set heads-on #f)
           (set open nil) (set checking #f))))
"What top-level form `s` makes, the forms the checker reads (`top-made`)."
(define-rec
  (drive-form (->* (syn) syns-a)
    (lambda (s)
     (begin
      (drive-begun! s)
      (cond ;; A shape named (`binding-shapes.md`): held to the globals, as a
            ;; macro is, and defined; it makes nothing (C440).
            ((shape-definition? s) (begin (drive-taken (syntax-names s) s) (shape-defined! s) nil))
            ((defines-syntax? s)
             (begin (drive-taken (syntax-names s) s)
                    (drive-checks (define-macro pfail-at s nil))))
            ;; Phase 1's forms, in force for the transformers defined after
            ;; (C446): checked and run in their namespace, as the host
            ;; answers `%forms`; nothing of the program's.
            ((for-syntax? s) (begin (for-syntax! pfail-at s) nil))
            ((imports-syntax? s) (drive-not-yet s "a file's macros, `import-with-syntax`"))
            ;; A `hide`: its forms', as if written there, a program exporting
            ;; nothing to hide them from (`tops-made`).
            ((string=? (drive-head s) "hide") (drive-forms (cdr (proper-items s))))
            (else
             (let* ((xs (expand-top pfail-at s)) (ms (made-syntax-taken!)) (ws (drive-written-each xs)))
               (begin (drive-taken ms s) (drive-guard ws) (append ws (drive-checks ms)))))))))
  (drive-forms (->* (syns-a) syns-a)
    (lambda (xs) (if (null? xs) nil (let ((made (drive-form (car xs)))) (append made (drive-forms (cdr xs))))))))

(define drive-out (ref syns-a mxp) (new nil))
(define drive-put (ref mx-parser-types-names mxp) (new nil))
(define drive-macros-before (ref int mxp) (new 0))
(define drive-globals-before (ref int mxp) (new 0))
"A refusal where Fixpt's parse is refused, as kayak's checker says one."
(define drive-refused (->* (string int) void)
  (lambda (m a)
    (let ((w (drive-placed a)))
      (error (string-append "kfx: " (string-append (extract w 1) (string-append ":" (string-append (int->string (i64->int (extract w 2)))
               (string-append ": " m)))))))))
(define drive-each (->* (syns-a) (listof datum acyclic))
  (lambda (xs)
    (if (null? xs)
        nil
        (let ((r (prompt parse-tag (begin (set drive-out (drive-form (car xs))) (p-ok nil)) (lambda (r) r))))
          (tagcase r
            (p-err (m a b) (begin (drive-refused m a) nil))
            (p-ok (ts) (let ((made (drive-locs (get drive-out)))) (append made (drive-each (cdr xs))))))))))
"A program's forms, each expanded as Fixpt's parser expands it."
(define drive-program* (->* ((listof datum acyclic)) (listof datum acyclic))
  (lambda (forms)
    (begin
      (set drive-files nil)
      (set drive-met nil)
      (macros-at! 0)
      (shapes-at! 0 nil)
      (let ((xs (drive-syns forms)))
        (begin (drive-lines!) (drive-each xs))))))

;;; ------------------------------------------------------------ a session's entries

"A REPL's session (C439), as Fixpt's reads its entries (`parse-program`):
 each entry from the reading state the entry before it left, its state
 after kept by a number of its own (`macros-after!`, `shapes-after!`),
 which the session gives the entry after it. The places met so far are
 the session's, each entry's file its own, so that a macro an entry
 defines is placed where it was written when a later one uses it; a
 draft's are let go of after it."
(define drive-states (ref int mxp) (new 0))
(define drive-rev-onto (->* ((listof datum acyclic) (listof datum acyclic)) (listof datum acyclic))
  (lambda (xs out) (if (null? xs) out (drive-rev-onto (cdr xs) (cons (car xs) out)))))
"Each form's expansion, or the first refusal, said as the checker says its
 own: the nodes and the empty string, or none and why."
(define drive-each-said (->* (syns-a (listof datum acyclic)) (tupleof (listof datum acyclic) string))
  (lambda (xs out)
    (if (null? xs)
        (tuple (reverse out) "")
        (let ((r (prompt parse-tag (begin (set drive-out (drive-form (car xs))) (p-ok nil)) (lambda (r) r))))
          (tagcase r
            (p-err (m a b)
              (let ((w (drive-placed a)))
                (tuple nil (string-append "kfx: " (string-append (extract w 1) (string-append ":" (string-append (int->string (i64->int (extract w 2))) (string-append ": " m))))))))
            (p-ok (ts) (drive-each-said (cdr xs) (drive-rev-onto (drive-locs (get drive-out)) out))))))))
"Forms `forms` expanded from reading state `s`."
(define drive-from (->* ((listof datum acyclic) int) (tupleof (listof datum acyclic) string))
  (lambda (forms s)
    (begin
      (macros-at! s)
      (shapes-at! s nil)
      (let ((xs (drive-syns forms)))
        (begin (drive-lines!) (drive-each-said xs nil))))))
"A REPL's entry, expanded from the state `s` the entries before it left:
 its forms, and the state it leaves, or, refused, why, and `s` again."
(define drive-entry* (->* ((listof datum acyclic) int) (tupleof (listof datum acyclic) string int))
  (lambda (forms s)
    (let ((r (drive-from forms s)))
      (if (string=? (extract r 2) "")
          (let ((k (begin (set drive-states (+ (get drive-states) 1)) (get drive-states))))
            (begin (macros-after! k) (shapes-after! k) (tuple (extract r 1) "" k)))
          (tuple nil (extract r 2) s)))))
"A draft's, `,type`'s or a hint's forms, expanded from state `s`, which
 they leave as it was."
(define drive-draft* (->* ((listof datum acyclic) int) (tupleof (listof datum acyclic) string))
  (lambda (forms s)
    (let* ((met (get drive-met)) (r (drive-from forms s)))
      (begin (set drive-met met) r))))
"Whether state `s` has a macro: an entry read in it is expanded, as
 every one is once a macro is defined, though it names none."
(define drive-macros?* (->* (int) bool)
  (lambda (s)
    (let ((at (state-at s (get by-state))))
      (and (not (null? at)) (not (null? (extract (car at) 2)))))))

;;; ------------------------------------------------------------ a form alone

"A form expanded alone (C445), as kc expands each, a query of its own
 (`docs/research/kc.md`, step 15; kc/expand.kfx): in an instance of its
 own, given none of the forms before it but the definitions of the macros
 it names, and of those their definitions name (kc finds them, by name),
 each defined in turn; the shapes of the globals they name, and which are
 globals; and how many forms of each hash were begun before each
 (`form-begin!`, its fresh names' key). So what it makes depends on what
 it names, not on the forms that name nothing of it.

 The symbols of `s`, onto `out`, each once."
(define-rec
  (drive-symbols (->* (syn mx-parser-types-names) mx-parser-types-names)
    (lambda (s out)
      (tagcase s
        (atom (d a b x) (if (and (symbol? d) (not (mx-files-name-in? d out))) (cons d out) out))
        (mx-eager-reader-types-lst (items d a b x) (drive-symbols-each items out))
        (dotted (items tail d a b x) (drive-symbols tail (drive-symbols-each items out)))
        (vec (items d a b x) (drive-symbols-each items out)))))
  (drive-symbols-each (->* (syns-a mx-parser-types-names) mx-parser-types-names)
    (lambda (xs out) (if (null? xs) out (drive-symbols-each (cdr xs) (drive-symbols (car xs) out))))))
(define drive-names-count (->* (mx-parser-types-names int) int)
  (lambda (ns k) (if (null? ns) k (drive-names-count (cdr ns) (+ k 1)))))
"A form and the count of the hashes begun before it."
(define-type drive-counted (listof (tupleof syn seen-keys) acyclic))
(define-rec
  (drive-chosen-symbols (->* (drive-counted mx-parser-types-names) mx-parser-types-names)
    (lambda (ds ns) (if (null? ds) ns (drive-chosen-symbols (cdr ds) (drive-symbols (extract (car ds) 1) ns))))))
(define-rec
  (drive-define-each (->* (drive-counted) unit)
    (lambda (ds)
      (if (null? ds)
          #u
          (begin (seen! (extract (car ds) 2)) (drive-form (extract (car ds) 1)) (drive-define-each (cdr ds)))))))
"The top level's shapes (`top-vals`, `top-descs`) as data (C445), so that
 a form alone is given those of the globals it names, which the forms that
 define them found, and gives those of the globals it defines: an entry
 `(name kind shapes)`, `kind` 0 a value's, 1 a description's; a shape
 `(m members members)` or `(f (shapes ...) shapes)`, a member `(name
 shapes)`, `shapes` a list of none or one."
(define drive-sym (->* (string) datum) (lambda (n) (the datum (string->symbol n))))
(define loc-sym (->* (datum) symbol) (lambda (d) (if (symbol? d) d (string->symbol "?"))))
(define-rec
  (drive-shape-datum (->* (mx-parser-types-shape) datum)
    (lambda (x)
      (tagcase x
        (sh-module (vs ds) (mx-datum-list (list (drive-sym "m") (drive-members-datum vs) (drive-members-datum ds))))
        (sh-fun (ps r) (mx-datum-list (list (drive-sym "f") (mx-datum-list (drive-shapes-each ps)) (drive-shapes-datum r)))))))
  (drive-shapes-datum (->* (mx-parser-types-shapes) datum)
    (lambda (ss) (mx-datum-list (drive-shape-items ss))))
  (drive-shape-items (->* (mx-parser-types-shapes) (listof datum acyclic))
    (lambda (ss) (if (null? ss) nil (cons (drive-shape-datum (car ss)) (drive-shape-items (cdr ss))))))
  (drive-shapes-each (->* ((listof mx-parser-types-shapes acyclic)) (listof datum acyclic))
    (lambda (xs) (if (null? xs) nil (cons (drive-shapes-datum (car xs)) (drive-shapes-each (cdr xs))))))
  (drive-members-datum (->* (mx-parser-types-members) datum)
    (lambda (ms) (mx-datum-list (drive-member-items ms))))
  (drive-member-items (->* (mx-parser-types-members) (listof datum acyclic))
    (lambda (ms)
      (if (null? ms)
          nil
          (cons (mx-datum-list (list (the datum (extract (car ms) 1)) (drive-shapes-datum (extract (car ms) 2))))
                (drive-member-items (cdr ms)))))))
(define loc-is-sym? (->* (datum string) bool)
  (lambda (d n) (and (symbol? d) (string=? (symbol->string d) n))))
(define-rec
  (drive-datum-shape (->* (datum) mx-parser-types-shape)
    (lambda (d)
      (if (loc-is-sym? (loc-nth d 0) "m")
          (sh-module (drive-datum-members (loc-nth d 1)) (drive-datum-members (loc-nth d 2)))
          (sh-fun (drive-datum-shapes-each (loc-nth d 1)) (drive-datum-shapes (loc-nth d 2))))))
  (drive-datum-shapes (->* (datum) mx-parser-types-shapes)
    (lambda (d) (if (pair? d) (let ((x (car d))) (cons (drive-datum-shape x) (drive-datum-shapes (cdr d)))) nil)))
  (drive-datum-shapes-each (->* (datum) (listof mx-parser-types-shapes acyclic))
    (lambda (d) (if (pair? d) (let ((x (car d))) (cons (drive-datum-shapes x) (drive-datum-shapes-each (cdr d)))) nil)))
  (drive-datum-members (->* (datum) mx-parser-types-members)
    (lambda (d)
      (if (pair? d)
          (let ((m (car d))) (cons (tuple (loc-sym (loc-nth m 0)) (drive-datum-shapes (loc-nth m 1))) (drive-datum-members (cdr d))))
          nil))))
"Of entries `es`, the oldest first, those of names `ns` put in the state:
 a shape in the top level's, a later one of a name in an earlier's place;
 a global (kind 3) one from now on, so that a macro may not take its name
 (`drive-taken`). The names whose shapes were put. (A macro a form made
 but a `define-syntax`, kind 2, is no form's to be given: kc refuses a
 form that names one, not yet.)"
(define-rec
  (drive-inject! (->* (mx-parser-types-names (listof datum acyclic) mx-parser-types-names) mx-parser-types-names)
    (lambda (ns es put)
      (if (null? es)
          put
          (let* ((e (car es)) (n (loc-sym (loc-nth e 0))) (k (loc-int (loc-nth e 1))) (sh (drive-datum-shapes (loc-nth e 2))))
            (cond ((not (mx-files-name-in? n ns)) (drive-inject! ns (cdr es) put))
                  ((= k 3) (begin (globals! (list n)) (drive-inject! ns (cdr es) put)))
                  ((null? sh) (drive-inject! ns (cdr es) put))
                  ((= k 1) (begin (set top-descs (names-with (get top-descs) n sh)) (drive-inject! ns (cdr es) (cons n put))))
                  (else (begin (set top-vals (names-with (get top-vals) n sh)) (drive-inject! ns (cdr es) (cons n put))))))))))
"How many macros `m` defines; the names of its first `k`, the newest
 first; the first `k` of names `ns`."
(define-rec
  (drive-env-count (->* (macro-env int) int)
    (lambda (m k) (if (null? m) k (drive-env-count (cdr m) (+ k 1))))))
(define-rec
  (drive-env-first (->* (macro-env int) mx-parser-types-names)
    (lambda (m k) (if (or (null? m) (<= k 0)) nil (cons (extract (car m) 1) (drive-env-first (cdr m) (- k 1)))))))
(define-rec
  (drive-names-first (->* (mx-parser-types-names int) mx-parser-types-names)
    (lambda (ns k) (if (or (null? ns) (<= k 0)) nil (cons (car ns) (drive-names-first (cdr ns) (- k 1)))))))
"Names `ns` as entries of kind `k`, onto `out`."
(define-rec
  (drive-name-entries (->* (mx-parser-types-names int (listof datum acyclic)) (listof datum acyclic))
    (lambda (ns k out)
      (if (null? ns) out
          (drive-name-entries (cdr ns) k (cons (mx-datum-list (list (the datum (car ns)) (the datum k) (the datum nil))) out))))))
"The entries of tree `t`, of kind `k`, a shape known, but those of names
 `put`, onto `out`."
(define-rec
  (drive-tree-entries (->* (name-tree int mx-parser-types-names (listof datum acyclic)) (listof datum acyclic))
    (lambda (t k put out)
      (tagcase t
        (nm-node (h m sh l r)
          (drive-tree-entries l k put
            (drive-tree-entries r k put
              (if (or (null? sh) (mx-files-name-in? m put))
                  out
                  (cons (mx-datum-list (list (the datum m) (the datum k) (drive-shapes-datum sh))) out)))))
        (else y out)))))
"Form `s`, begun after counts `ks`, expanded in a state of definitions
 `ds` alone, given the shapes of the globals it names of entries `es`:
 what it makes, or its refusal, `(file line message)`; and the entries of
 the globals it defines."
(define drive-alone-said (->* (drive-counted syn seen-keys (listof datum acyclic)) (tupleof (listof datum acyclic) datum (listof datum acyclic)))
  (lambda (ds s ks es)
    (begin
      (macros-at! 0)
      (shapes-at! 0 nil)
      (let* ((ns (drive-chosen-symbols ds (drive-symbols s nil)))
             (r (prompt parse-tag
                        (begin (drive-define-each ds)
                               (set drive-put (drive-inject! ns es nil))
                               (set drive-macros-before (drive-env-count (get defined) 0))
                               (set drive-globals-before (drive-names-count (get mx-top-globals) 0))
                               (seen! ks)
                               (set drive-out (drive-form s))
                               (p-ok nil))
                        (lambda (r) r))))
        (tagcase r
          (p-err (m a b)
            (let ((w (drive-placed a)))
              (tuple nil (mx-datum-list (list (the datum (extract w 1)) (the datum (i64->int (extract w 2))) (the datum m))) nil)))
          (p-ok (ts)
            ;; What it defines, for the forms after it: its globals' shapes;
            ;; its globals; the macros it made but by a `define-syntax`.
            (let ((made (if (defines-syntax? s) nil (drive-env-first (get defined) (- (drive-env-count (get defined) 0) (get drive-macros-before)))))
                  (globals (drive-names-first (get mx-top-globals) (- (drive-names-count (get mx-top-globals) 0) (get drive-globals-before)))))
              (tuple (drive-locs (get drive-out)) (the datum nil)
                     (drive-name-entries made 2 (drive-name-entries globals 3
                       (drive-tree-entries (get top-vals) 0 (get drive-put) (drive-tree-entries (get top-descs) 1 (get drive-put) nil))))))))))))
"What kc reads of each of a file's forms (C445), as written: `(hashes
 names macros)`, the hashes of the forms it begins, in order (its own, and
 a `hide`'s forms'); the names it defines, written there, values' and
 descriptions'; the macros, a `define-syntax`'s."
(define drive-desc-name (->* (syn) mx-parser-types-names)
  (lambda (s)
    (let ((xs (proper-items s)) (h (drive-head s)))
      (if (and (or (string=? h "define-type") (string=? h "define-datatype") (string=? h "define-effect") (string=? h "define-generative"))
               (>= (len xs) 2))
          (let ((n (nth xs 1)))
            (cond ((syn-symbol? n) (list (syn-symbol n)))
                  ((and (not (null? (proper-items n))) (syn-symbol? (car (proper-items n)))) (list (syn-symbol (car (proper-items n)))))
                  (else nil)))
          nil))))
(define-rec
  (drive-begun-hashes (->* (syn (listof datum acyclic)) (listof datum acyclic))
    (lambda (s out)
      (let ((after (cons (the datum (syn-hash s)) out)))
        (if (string=? (drive-head s) "hide") (drive-begun-each (cdr (proper-items s)) after) after))))
  (drive-begun-each (->* (syns-a (listof datum acyclic)) (listof datum acyclic))
    (lambda (xs out) (if (null? xs) out (drive-begun-each (cdr xs) (drive-begun-hashes (car xs) out))))))
(define-rec
  (map-names-data (->* (mx-parser-types-names) (listof datum acyclic))
    (lambda (ns) (if (null? ns) nil (cons (the datum (car ns)) (map-names-data (cdr ns)))))))
(define drive-names-data (->* (mx-parser-types-names) datum)
  (lambda (ns) (mx-datum-list (map-names-data ns))))
(define-rec
  (drive-facts (->* (syns-a) (listof datum acyclic))
    (lambda (xs)
      (if (null? xs)
          nil
          (let ((s (car xs)))
            (cons (mx-datum-list (list (mx-datum-list (reverse (drive-begun-hashes s nil)))
                                       (drive-names-data (append (drive-names s) (drive-desc-name s)))
                                       (drive-names-data (syntax-names s))))
                  (drive-facts (cdr xs))))))))
"Counts `(hash count)` ..., as data, as the state keeps them."
(define-rec
  (drive-seen-of (->* (datum) seen-keys)
    (lambda (d)
      (if (pair? d)
          (let ((x (car d))) (cons (tuple (loc-int (loc-nth x 0)) (loc-int (loc-nth x 1))) (drive-seen-of (cdr d))))
          nil))))
(define-rec
  (drive-given-defs (->* ((listof datum acyclic)) drive-counted)
    (lambda (ds)
      (if (null? ds)
          nil
          (let ((d (car ds))) (cons (tuple (drive-syn (loc-nth d 0)) (drive-seen-of (loc-nth d 1))) (drive-given-defs (cdr ds))))))))
"A form given alone, `(form counts)`, with definitions `defs`, each so, in
 the order they are defined, and entries `es`, the oldest first, as kc's
 `expansion` query gives them: what `drive-alone-said` says of it."
(define drive-given* (->* ((listof datum acyclic) (listof datum acyclic) datum) (tupleof (listof datum acyclic) datum (listof datum acyclic)))
  (lambda (defs es target)
    (begin
      (set drive-files nil)
      (set drive-met nil)
      (let* ((ds (drive-given-defs defs)) (s (drive-syn (loc-nth target 0))) (ks (drive-seen-of (loc-nth target 1))))
        (begin (drive-lines!) (drive-alone-said ds s ks es))))))

;;; ------------------------------------------------------------ what an instance gives

"What the file gives (C443), each of a written type, so its loader's
 interface (C442): the forms, each a located datum, and what is made of
 them, a list of them, as a datum."
(define drive-program (subr (maxeff (read @globals) spin (read mxp) (write mxp) (alloc mxp) (goto mxp) (read @globals) (io process)) (datum) datum)
  (lambda (forms) (mx-datum-list (drive-program* (loc-nodes forms)))))
(define drive-entry (subr (maxeff (read @globals) spin (read mxp) (write mxp) (alloc mxp) (goto mxp) (read @globals) (io process)) (datum int) (tupleof datum string int))
  (lambda (forms s)
    (let ((r (drive-entry* (loc-nodes forms) s)))
      (tuple (mx-datum-list (extract r 1)) (extract r 2) (extract r 3)))))
(define drive-draft (subr (maxeff (read @globals) spin (read mxp) (write mxp) (alloc mxp) (goto mxp) (read @globals) (io process)) (datum int) (tupleof datum string))
  (lambda (forms s)
    (let ((r (drive-draft* (loc-nodes forms) s)))
      (tuple (mx-datum-list (extract r 1)) (extract r 2)))))
(define drive-macros? (subr (maxeff (read @globals) spin (read mxp) (write mxp) (alloc mxp) (goto mxp)) (int) bool)
  (lambda (s) (drive-macros?* s)))
"A procedural macro's transformer's asker (C449): what answers each
 request the instance's expansions make, of its host's effect, which
 scripts/macros-port.py writes in (`ASKS`);
 until one is given, none, each request said not yet."
(define drive-asker! (subr (maxeff (read @globals) spin (read mxp) (write mxp) (alloc mxp) (goto mxp) (read @globals) (io process)) ((subr (maxeff (read @globals) (io process) spin) (string) string)) unit)
  (lambda (f) (set mx-phase1-asker f)))
(define drive-written-facts (subr (maxeff (read @globals) spin (read mxp) (write mxp) (alloc mxp) (goto mxp) (read @globals) (io process)) (datum) datum)
  (lambda (forms)
    (begin (set drive-files nil) (set drive-met nil) (mx-datum-list (drive-facts (drive-syns (loc-nodes forms)))))))
(define drive-given (subr (maxeff (read @globals) spin (read mxp) (write mxp) (alloc mxp) (goto mxp) (read @globals) (io process)) (datum datum datum) (tupleof datum datum datum))
  (lambda (defs es target)
    (let ((r (drive-given* (loc-nodes defs) (loc-nodes es) target)))
      (tuple (mx-datum-list (extract r 1)) (extract r 2) (mx-datum-list (extract r 3))))))
