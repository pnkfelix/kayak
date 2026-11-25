;; expander-types.fx: the types of Fixpt's macro expander that name no
;; region of its, written by scripts/macros-port.py (C429, C443): edit
;; that, not this. A file of no parameters, loaded once for every
;; instance of expander.fx: its constructors are no globals, as in Fixpt,
;; where each is another file's item.
;;; ============================================================ eager-reader-types.fx (the definitions the expander reaches)

;; What is read, with where each piece starts and ends. A vector's elements
;; keep theirs; any other datum that is not a list is an `atom`. Each piece
;; also carries the plain datum it reads as, made once, when it was read,
;; from the datums the marks already hold. And last, the expansion that
;; made it, 0 if none did, as the Rust front end's `Span` has it: a macro's
;; template's part keeps its place in the template, and says which use of
;; the macro made it (`macros.fx`), for errors said at each use.
(define-datatype syn
  (atom datum int int int)
  (mx-eager-reader-types-lst (listof syn acyclic) datum int int int)
  (dotted (listof syn acyclic) syn datum int int int)
  (vec (listof syn acyclic) datum int int int))

;;; ============================================================ parser-types.fx (the definitions the expander reaches)

(define-type syns-a (listof syn acyclic))
(define-type mx-parser-types-names (listof symbol acyclic))
;; What the expander relied on of a module a form opens, as it wrote it
;; (`#:relied`, `macros-shapes.fx`; `ast::Relied`): none written; not
;; known (`#f`); or the names references it found reached the module by
;; and found it gives, values' and descriptions'.
(define-datatype relied (rl-none) (rl-unknown) (rl-names mx-parser-types-names mx-parser-types-names))
;; What a module gives, as expansion needs it (`macros-shaper.fx`) (`macros/shapes.rs`'s
;; `Shape`): its values' names, each with its own shape where it is known
;; and a module's (or a function's that makes one), and its descriptions'
;; (types, effects, functions of descriptions), each with its own where it
;; is a module type's; or a function's, its parameters' shapes and its
;; result's; or a cell's, `(ref T r)`, what it holds (never none), or a
;; product's, `(productof (f T) …)`, its fields' of known shapes (TODO §304,
;; Heist's should-work note, entry 17). A shape where it may not be known
;; is a list of none or one.
(define-datatype mx-parser-types-shape (sh-module (listof (tupleof symbol (listof mx-parser-types-shape acyclic)) acyclic) (listof (tupleof symbol (listof mx-parser-types-shape acyclic)) acyclic)) (sh-fun (listof (listof mx-parser-types-shape acyclic) acyclic) (listof mx-parser-types-shape acyclic))
  (sh-cell (listof mx-parser-types-shape acyclic)) (sh-product (listof (tupleof symbol (listof mx-parser-types-shape acyclic)) acyclic)))
(define-type mx-parser-types-shapes (listof mx-parser-types-shape acyclic))
(define-type mx-parser-types-members (listof (tupleof symbol mx-parser-types-shapes) acyclic))
;;; ------------------------------------------------------------------ trees
;;; Each node ends with where it starts and ends, and which of the
;;; expressions there it is: 0 for the first, in the order of a walk of the
;;; program's trees that takes each node before what is in it, then 1, 2,
;;; … for those the parser made there besides (`number-tops`). The three
;;; are the expression's identity, by which what is found of it is kept
;;; (`docs/research/loads-and-places.md`). A list of none or one stands
;;; for something optional: a parameter's type, a tagcase's `else`.

;; What the checker found an operation takes, which only the types say:
;; an `extract`'s field, by position, or a blobref's, by its offset; a
;; blobref's scalar's or flat sum's layout, by its code; a flat array's
;; length and element size; the bytes a new blobref takes. Kept in the
;; operation's node once a compiler has put it there (`c-resolve-extracts`);
;; none, `nil`, until then.
(define-datatype k-datum
  (datum-field int) (datum-blobref-field int) (datum-layout int) (datum-array int int)
  (datum-fits int))
(define-type k-datums (listof k-datum acyclic))
(define-datatype exp
  (e-var symbol int int int int)
  (e-int int int int int int)
  (e-bool bool int int int int)
  (e-str string int int int int)
  (e-char char int int int int)
  (e-float f64 int int int int)
  (e-sym symbol int int int int)
  ;; A quoted list or vector, `(%datum x)`, as the quote rewrites make it:
  ;; one constant, made once and interned (TODO §229).
  (e-quote syn int int int int)
  (e-unit int int int int)
  (e-lambda (listof (productof (1 symbol) (2 syns-a)) acyclic) exp int int int int)
  (e-app exp (listof exp acyclic) int int int int)
  (e-plambda syn exp int int int int)
  (e-proj exp syns-a int int int int)
  (e-if exp exp exp int int int int)
  (e-letrec (listof (productof (1 symbol) (2 syn) (3 exp)) acyclic) exp int int int int)
  (e-let (listof (productof (1 symbol) (2 exp)) acyclic) exp int int int int)
  (e-begin (listof exp acyclic) int int int int)
  (e-prompt exp exp exp int int int int)
  ;; `(letregion name body …)`, `(letrena name body …)` or `(letreap name
  ;; body …)`, or `(letfreeze name body …)`: what it makes besides the
  ;; region (0 nothing, 1 an arena, 2 a reap, 3 nothing, its region's data
  ;; frozen as it ends: `docs/research/places-and-regions.md`),
  ;; the region variable's name, the place a `letfreeze` freezes into
  ;; (`heap` unless given), and the body.
  (e-letregion int symbol symbol exp int int int int)
  ;; `(rlambda region (param …) body …)`: the region, and the `lambda`.
  (e-rlambda exp exp int int int int)
  (e-the syn exp int int int int)
  ;; `(convention C expression)`: the procedure converted to `C`.
  (e-convention syn exp int int int int)
  ;; A bloblet form, by name, with its field index, or -1, and its datum.
  (e-bloblet symbol int (listof exp acyclic) k-datums int int int int)
  (e-product (listof (productof (1 symbol) (2 exp)) acyclic) int int int int)
  ;; `(extract e label)`, and its datum.
  (e-extract exp symbol k-datums int int int int)
  (e-sum symbol exp int int int int)
  ;; Arms: the tag, whether it takes a product apart, the names, the body.
  ;; Then, as `e-extract`'s, its datum (a flat sum's layout).
  (e-tagcase exp (listof (productof (1 symbol) (2 bool) (3 mx-parser-types-names) (4 exp)) acyclic)
             (listof (productof (1 symbol) (2 exp)) acyclic) k-datums int int int int)
  ;; `(module item …)` (`docs/research/first-class-modules.md`): each item
  ;; what it is (0 `define-generative`, 1 `define-type`, 2 `define`, 3
  ;; `define-rec`), the names it defines, their types (a `define`'s, none or
  ;; one), and its expressions: the values, or an abstract type's two
  ;; conversions, each `(lambda (x) x)` spanning the item.
  (e-module (listof (productof (1 int) (2 mx-parser-types-names) (3 syns-a) (4 (listof exp acyclic))) acyclic)
            int int int int)
  ;; `(with module body …)`: the body, with the module's values in scope;
  ;; what the expander relied on of the module (`relied`).
  (e-with symbol exp int int int int relied))
;; A top-level form. A definition's type is a list of none or one; a
;; `define*`'s, of it and the `define*`.
(define-datatype mx-parser-types-top
  (mx-parser-types-t-define symbol syns-a exp int int)
  ;; `(define-rec (name type lambda) …)`: a top-level `letrec`.
  (mx-parser-types-t-define-rec (listof (productof (1 symbol) (2 syn) (3 exp)) acyclic) int int)
  (t-define-type syn syn int int)
  (t-define-effect syn syn int int)
  ;; `(define-convention name clause …)`: its name and the whole form, read
  ;; by the checker alone (`TODO.md` §97).
  (t-define-convention syn syn int int)
  ;; `(define-syscall name "host" type)` (required) or
  ;; `(define-optional-syscall …)`: whether required, and the whole form,
  ;; read by the checker alone, which expands it (`docs/research/syscalls.md`).
  (t-define-syscall bool syn int int)
  ;; `(define-generative head rep …)`: read by the checker alone; its two
  ;; conversions, named `names` (`seal`, `unseal`: `generative-conversions`),
  ;; follow it as definitions.
  (t-define-generative syn syn mx-parser-types-names int int)
  (mx-parser-types-t-exp exp))
(define-datatype presult (p-ok (listof mx-parser-types-top acyclic)) (p-err string int int))
;; What the driver read for each `load-module` or `load-input`, by where
;; the form starts: the file's base (0 if it could not be read or read),
;; why not (`cannot read …`, or where in it reading failed), its path, its
;; forms, its text, the path from the program's directory, which names it
;; among the loads that are one (`TODO.md` §68), and whether the forms are
;; not the file's but the one expression its `.fasl` is read as
;; (`docs/research/fasl.md`).
(define-type loaded-file
  (productof (1 int) (2 int) (3 string) (4 string) (5 syns-a) (6 string) (7 string) (8 bool)))
(define-type loaded-files (listof loaded-file acyclic))

;;; ============================================================ macros-types.fx (whole)

;;; ------------------------------------------------------------ the types

;; An address's steps from its root, first first: `#t` the car.
(define-type addr (listof bool acyclic))
;; What a part at an address binds (Herman's `δ`): 0 nothing, 1 a value's
;; variable, 2 a rib, 3 a ribcage, 4 a description's variable, 5 both, a
;; place (`letrena`'s), which is also a value of type `(place r)`.
(define-type entries (listof (tupleof addr int) acyclic))
;; One of a ribcage's: a rib, or the ribcage exported at an address; or, a
;; program's (`scope-forms`), what the module at an address gives (a file's
;; module parameters, each included in its items).
(define-datatype ring (ring-rib entries) (ring-at addr) (ring-shape addr))
(define-type rings (listof ring acyclic))
;; What an environment is made of: a pattern variable bound (`bvar`,
;; `dvar`), the rib or ribcage a pattern variable exports, a template's own
;; binder (its node).
;; Each but a ribcage bound in a namespace: a description's if `#t`, else a
;; value's (Herman's δ, one per namespace). A pattern variable's, with how
;; many lists by ellipsis gathered it: 0 its own repetition's, 1 every
;; repetition's, as a list's export gathers its elements' (`macros.rs`'s
;; `Atom::Var`; `TODO.md` §223).
(define-datatype atm (atm-var symbol bool int) (atm-rib-of symbol) (atm-lit int bool))
(define-type atms (listof atm acyclic))
;; What an annotation binds (Herman's `β`): a rib, or a ribcage; or a rib of
;; what it binds, as found, the binders a template's description writes
;; (`macros.rs`'s `Bind::Direct`), or what the repetitions before a repeated
;; part bound (`Bind::Earlier`).
(define-datatype mx-types-bind (bind-rib entries) (bind-cage rings) (bind-direct atms) (bind-earlier atms)
  ;; What the module at an address gives, `(shape A)`: what `with` and
  ;; `include` bind, in a program's walk (`scope-forms`).
  (bind-shape addr))
;; What a typed hole says (`macros.rs`'s `Hole`): its type and its effect,
;; each as written, in a list of one, or none if it is left out.
(define-type hole (tupleof syns-a syns-a))
;; A syntax type (`macros.rs`'s `STy`). A `mu`, `(s-mu body round part)`,
;; a value compared by its structure, its variable `(s-var n)` by how many
;; `mu`s are between them (de Bruijn's numbering), 0 the innermost; its
;; round and part, of a signature being inferred (`build`), else -1.
;; `s-typed`, an `expr` or `bvar` with its type or effect written.
(define-datatype sty
  (s-expr) (s-bvar) (s-id) (s-data) (s-type) (s-label) (s-nil)
  (s-literal symbol) (s-pair sty sty) (s-import sty mx-types-bind) (s-export sty mx-types-bind) (s-mark symbol int)
  (s-mu sty int int) (s-var int) (s-many sty) (s-union (listof sty acyclic)) (s-typed sty hole)
  ;; An expression that is a lambda, where a form wants one (`letrec`).
  (s-lambda)
  ;; A definition, where a template makes one (the kernel's `begin` of
  ;; them); no part of a signature, for now.
  (s-defn)
  ;; A description's binder; a binder's kind, which names nothing.
  (s-dvar) (s-kind)
  ;; A module's item, as a program's walk takes one (`scope-forms`): a
  ;; definition, a string (a comment), or an expression run as the module
  ;; is made.
  (s-item)
  ;; A macro's transformer a template writes (`macros.rs`'s `STy::Rules`).
  (s-rules))
(define-type stys (listof sty acyclic))
;; The trailmarks defined (`macros.rs`'s `Mark`), the last first: each name's
;; number, 0 where a macro of the name hides one; else the standard ones,
;; `else` 1 and `=>` 2, which a file sees too.
(define-type marked (listof (tupleof symbol int) acyclic))
;;; ------------------------------------------------------------ reading

;; Syntax type `s`, read: `named`'s names (the kernel's abbreviations), and
;; the `mu` variables in scope, innermost first, each `#f` that of a list by
;; ellipsis, which no name refers to.
(define-type mx-types-named (listof (tupleof symbol sty) acyclic))
(define-type mu-scope (listof (tupleof symbol bool) acyclic))
;; Whether syntax of type `a` is all of type `b`, binding structure alike:
;; coinductive, as FX-26's recursive types are, each pair a `mu` is of
;; assumed (`assumed`) while it is compared, by structure: the types a `mu`
;; unfolds to are finitely many, so the pairs are (`macros.rs`'s `sub`).
(define-type assumed (listof (tupleof sty sty) acyclic))

;;; ============================================================ macros-env-types.fx (whole)

;; One level of an environment: a rib, or a pattern variable's ribcage; or,
;; in a repeated element of a ribcage's list, what the repetitions before
;; it bound, which its own binders' names cannot tell from its own
;; (`macros-scopes.fx`).
;; Or, in a program's walk, what the module at a node gives (`bind-shape`),
;; whose names the walk does not know: a name looked for past it may be one
;; of them (`macros.rs`'s `Level::Shape`).
(define-datatype mx-env-types-level (lv-rib atms) (lv-cage symbol) (lv-earlier atms) (lv-shape int))
(define-type mx-env-types-levels (listof mx-env-types-level acyclic))
;; What a name is found as, looked for from a place in an environment
;; (`env-lookup`): `(0 b next)`, binder `b`, of a rib or of what earlier
;; repetitions bound; `(1 m next)`, the module at node `m`, met first, whose
;; names may include it, looked past from `next`; `(-1 -1 next)`, nothing.
;; A ribcage binds no name a program's walk looks for: passed.
(define-type looked (tupleof int int int))

;;; ============================================================ macros-env-indexed.fx (whole)

;; Binders by name and namespace: a tree by each name's hash
;; (`symbol-name-hash`, FNV-1a, so as deep as a tree of random keys), one of
;; the same hash as a node's but another name or namespace to its right.
(define-datatype index (ix-leaf) (ix-node int symbol bool int index index))
(define-datatype group (gr-names index) (gr-shape int))
(define-type mx-env-indexed-groups (listof group acyclic))
;; Abstract outside this file by what is imported of it: `macros-nodes.fx`
;; takes the type and the operations, never `env-of` (a datatype, since a
;; type `define-generative` makes, of a module a file makes, cannot be named
;; in that file's own type: `TODO.md` §235).
(define-datatype mx-env-indexed-env (env-of (tupleof mx-env-types-levels mx-env-indexed-groups)))

;;; ============================================================ macros-nodes.fx (whole)

;;; ------------------------------------------------------------ macros

;; What becomes of one of a template's parts where it is transcribed
;; (`macros.rs`'s `Action`): a name made fresh, one for the binder at a
;; path and each reference to it; a free name its alias, `%~K.N:n`, which
;; the resolution after expansion makes the name where the macro is defined
;; (`macros-shapes.fx`'s `aliases-resolved`); a file's macro's description read whole through
;; the file's module or the standard environment, by `select`, or an instance's, its
;; name an alias (`InstanceType`, step 4 (c)); a use of
;; another of the file's macros, by the name the importer has it by.
(define-datatype action
  (act-rename addr int) (act-alias) (act-module-type symbol) (act-instance-type symbol)
  ;; A description a file's frame holds: `(select (with lib frame) %ft:T)`,
  ;; `lib` an alias if an instance's (`macros.rs`'s `FrameType`).
  (act-frame-type symbol symbol bool)
  (act-standard-type)
  (act-macro-head symbol)
  ;; A trailmark of a file's a template writes: the name the importer has it by.
  (act-mark symbol))
(define-type actions (listof (tupleof addr action) acyclic))
;; Where a macro is defined, what its expansions' aliases are read in
;; (`macros/hygiene.rs`'s `DefEnv`), by its kind: the top level (0); a file
;; as its importer has it (1), its hidden global `%shared:key` and the
;; values its module shows (its other names the standard environment's); or
;; a module file's items, as its own macros see them (2); or a file as an
;; importer of an instance of it has it (3), the instance's name in place of
;; `%shared:key`, which a binder at a use is kept from capturing. Then each
;; of the file's macros' names and the name the importer has it by; and its
;; frame (`TODO.md` §300), its name and the values it holds, which a name it
;; does not show reaches before the standard environment's (`'-` and none,
;; if it has none). A type of its structure, the same in each instance of
;; the expander's files (`macros.fx`'s `def-envs`).
(define-type def-env
  (tupleof int symbol mx-parser-types-names (listof (tupleof symbol symbol) acyclic) symbol mx-parser-types-names
           mx-parser-types-names))
(define-type mx-nodes-def-envs (listof (tupleof string def-env) acyclic))
;; A clause: its pattern (of the operands), its template, and its actions,
;; by their paths in the template, the last made first; its pattern's
;; variables, in the order the pattern has them; each typed hole's type as
;; a use reads it, `(select #%toplevel T)`, which the use's expression is
;; held to; where its macro is defined; and its `syntax-rules`'
;; trailmarks, each the binding it is (`macros.rs`'s `Clause`).
(define-type mx-nodes-contracts (listof (tupleof symbol syn) acyclic))
;; A node of syntax aligned with a syntax type: the syntax; its path from
;; the root; the type at it, its annotations taken off; its imports; its
;; export (none or one); whether its part of the signature is `(many …)`;
;; what is in it; its part's typed hole, if the signature types it (none or
;; one).
(define-type holes (listof hole acyclic))
;;; ------------------------------------------------------------ pattern variables

;; A pattern variable: its type, its environment as the signature declares
;; it, whether it may be placed any number of times, its typed hole, if
;; the signature types it, and how many ellipses its pattern has it under.
(define-type pinfo (tupleof sty mx-env-types-levels bool holes int))
(define-type pvars (listof (tupleof symbol pinfo) acyclic))
;; The last, its template's checked uses, by where each starts
;; (`checked-uses`).
(define-type clause (tupleof syn syn actions pvars mx-nodes-contracts def-env marked (listof int acyclic)))
(define-type clauses (listof clause acyclic))
;; A macro defined: its signature, its clauses, the binders of its
;; signature's `(poly (binder …) σ)`, if it has one, and what it makes (0 an
;; expression, 1 definitions, `(macro σ defn)`, 2 a description, `(macro σ desc)`).
(define-type macro (tupleof sty clauses syns-a int))
(define-type macro-env (listof (tupleof symbol macro) acyclic))
;; What a file's macros are defined in (`macros.rs`'s `FileScope`): the
;; hidden global every load of the file is; the values and the types its
;; module shows; each of its macros' names, and the name the importer has
;; it by; the standard names; and its frame, its name and the values it
;; holds (`TODO.md` §300). None for a program's own.
(define-type mx-nodes-scope
  (tupleof symbol mx-parser-types-names mx-parser-types-names (listof (tupleof symbol symbol) acyclic) mx-parser-types-names symbol mx-parser-types-names mx-parser-types-names))
(define-type scopes (listof mx-nodes-scope acyclic))
;; Strings, `xs` sorted onto `out`; and joined, `, ` between.
(define-type mx-nodes-strings (listof string acyclic))
;;; ------------------------------------------------------------ nodes

;; What is in a node: a pair's two; nothing; a pattern variable (in a
;; pattern, or placed in a template); a template's binder, of a
;; description (`#t`) or a value; a name in a template referred to, in a
;; namespace, and what it is if the template does not bind it (0 the top
;; level's value, 1 the top level's qualified, 2 as written, 3 refused, the
;; module a template's `(with m name)` takes from: `macros.rs`'s `Free`); a
;; form in a template, its operands (or, an application's,
;; itself); a description's parts, a list a template writes; a list given
;; by ellipsis, `(p ...)`, its element, repeated; anything else.
(define-datatype mx-nodes-kids
  (k-pair int int) (k-nil) (k-pvar symbol) (k-binder symbol bool) (k-ref symbol bool int)
  (k-form int) (k-desc (listof int acyclic)) (mx-nodes-k-seq int) (k-other))
;; Each of a node's imports, outermost first, or its export, with the root
;; its addresses are from.
(define-type rooted (listof (tupleof mx-types-bind int) acyclic))
(define-type mx-nodes-node (tupleof syn addr sty rooted rooted bool mx-nodes-kids holes))
;; Each pattern variable, template binder and name referred to, by node,
;; with its environment, whether it is in a `many` part, and how many
;; ellipses it is under.
(define-type mx-nodes-found (listof (tupleof int mx-env-indexed-env bool int) acyclic))

;;; ============================================================ macros-align.fx (whole)

;; `(dletrec ((name description) …) body)`'s parts: each name in scope in
;; each description and the body.
(define-type dparts (listof (tupleof atm int syn) acyclic))

;;; ============================================================ macros-scopes.fx (whole)

;;; ------------------------------------------------------------ heads read by scope

;; Template `template`, of type `made`, aligned and walked, each pending list
;; read by its scope, to a fixed point (`typed-macros.md`, "Decided: a
;; template's heads read by their scope, to a fixed point"; `clause`'s): its
;; root, what the walk found, and each pending list's reading (`heads-read`).
;; Each macro reading kept (`heads`); one an application may become a macro
;; in a later round, never the other way: refused.
(define-type head-reading (tupleof int (listof int acyclic) (listof int acyclic)))
(define-type head-readings (listof head-reading acyclic))
(define-type syntax-binder (tupleof bool symbol int int))
;;; ------------------------------------------------------------ a program read by its scopes

;; Program form `s`, of syntax type `core`, aligned and walked, each list a
;; macro's name heads read by its scope (`macros.rs`'s `Aligner::program`,
;; `TODO.md` §295): an application where the program wrote its head (a
;; template's is its macro, by hygiene), it makes no description, and a
;; binder around it binds the name, or `bound` has it (the macros' names
;; bound around `s`); else a use, whole if `whole` (where a form is
;; expanded), else by its signature. Read to a fixed point: each list read
;; otherwise than it is now, with none such around it, read anew, as what
;; binds a head is read by the lists around it alone; so each round
;; settles one for good, and it ends. Its root, what the walk found, and
;; each such list's node, whether it is read an application, and its
;; environment.
(define-type mx-scopes-heads-read (listof (tupleof int bool mx-env-indexed-env) acyclic))
;; The uses of macros in `s`, of type `core`, `bound` the macros' names
;; bound around it, as the walk finds them, each whole (`program-scoped`;
;; `Macros::uses_found`): each by its place, the indices of the items it is
;; in, list by list from `s` (`index-path`), in the order they are written,
;; with the macros' names bound around it that the program wrote in it.
;; And each path, `a::b::c`, where a value or a description is named
;; (`TODO.md` §296); none where a name is bound.
(define-type ixs (listof int acyclic))
;; What is found at a place: a use of a macro, with the macros' names bound
;; around it; or a path, naming a description if `#t`, else a value
;; (`macros.rs`'s `Part`).
(define-datatype mx-scopes-part (pt-use mx-parser-types-names) (pt-path bool))
(define-type mx-scopes-uses (listof (tupleof ixs mx-scopes-part) acyclic))

;;; ============================================================ macros-match.fx (whole)

;;; ------------------------------------------------------------ matching

;; Pattern variables' syntax, by name.
(define-type mx-match-binds (listof (tupleof symbol syn) acyclic))

;;; ============================================================ macros-exports.fx (whole)

;;; ------------------------------------------------------------ exports

;; What an export binds of what a use gives, as compared (`macros.rs`'s
;; `exports`): its pattern variables' atoms and its ribcages, each once;
;; the template's own binders, made fresh, none of it.
(define-type mx-exports-exported (tupleof atms mx-parser-types-names))
;; The names a template binds in its uses of macros that make definitions,
;; nodes `i` to `n`, each with the macro: what their signatures do not say
;; they define, it does not see (`macros.rs`'s `binders_under`).
(define-type nested (listof (tupleof symbol symbol) acyclic))
;;; ------------------------------------------------------------ withs

;; The `with` forms around a part, each by its place.
(define-type mx-exports-places (listof (tupleof int int) acyclic))
;; By each `with` form's place, the macros used in its body by names the
;; program wrote, each with the use's place (`Macros::with_uses`): the
;; checker refuses the `with` if its module gives a value of the name (0
;; last). And, 1 last, a template's `(with m name)`, with the name's place
;; (`Macros::projections`): refused if the module gives no value of it,
;; which would otherwise be what it is where the macro is used; 2, a path's
;; `with`, its path to the name for the name, the same (`TODO.md` §296).
(define-type mx-exports-uses (listof (tupleof int int symbol int int int) acyclic))
;;; ------------------------------------------------------------ binders' depths

;; Each template binder the walk found, with how many ellipses it is under.
(define-type depths (listof (tupleof int int) acyclic))

;;; ============================================================ macros-facts.fx (whole)

;; What a labelled part gives a `bind` or an `export` (`contract.rs`'s
;; `Gives`): 0 nothing, 1 a value's binder, 2 a description's, 3 a place's,
;; 4 a rib, 5 a ribcage.
;; A label: its name; 0 here, or 1 a repeated element's; its path (here),
;; or the list's (repeated); its path in the element; what it gives.
(define-type mx-facts-steps (listof bool acyclic))
(define-type mx-facts-label (tupleof symbol int mx-facts-steps mx-facts-steps int))
(define-type mx-facts-labels (listof mx-facts-label acyclic))
;;; ------------------------------------------------------------ facts

;; Facts `fs` of a region whose labels are `ls`, on its syntax type `sigma`
;; (`contract.rs`'s `facts`): each `bind` an import on the part it names,
;; `export`s and `export*`s the region's export (none at a signature's top); `pre`, what
;; repeated elements export for what a use defines, first.
(define-type adds (listof (tupleof mx-facts-steps syn) acyclic))
;; Facts `fs` parted: those of the node, and each repetition's, `(bind v in
;; e) ...` (by the list's path, `e`'s in the element, the binding; the last
;; first), as `contract.rs`'s `facts` parts them.
(define-type eaches (listof (tupleof mx-facts-steps mx-facts-steps syn) acyclic))

;;; ============================================================ macros-files.fx (whole)

;; Each top-level `(define name (load-module "path"))` so far, the last
;; first: the files `import-with-syntax` of `name` reads.
(define-type load-defs (listof (tupleof symbol string) acyclic))
;; `withs`: the places of the `with` forms whose bodies `s` is in.
(define-type mx-files-places (listof (tupleof int int) acyclic))
(define-type files (listof loaded-file acyclic))
(define-type mades (listof (tupleof symbol bool) acyclic))
;; `(only name …)` or `(except name …)`: whether `only`, and the names, in a
;; list of one; none if there is none.
(define-type mx-files-filter (tupleof bool syns-a))
;; A file's macros, each with whether it is hidden, in order; and its
;; `begin-for-syntax`es among them, in force for the macros after each.
(define-type file-macro (tupleof syn bool))
;; A macro as its `define-syntax` writes it: its name, its signature if
;; written (none or one), its `syntax-rules`' literals and clauses, and the
;; form (or `define-syntax-rec`'s member) it is in.
(define-type mx-files-written (tupleof symbol syns-a mx-parser-types-names syns-a syn))
(define-type writtens (listof mx-files-written acyclic))
;; A signature as written: the binders of its `(poly (binder …) σ)`, the
;; syntax type, and what it makes (0 an expression, 1 definitions, 2 a
;; description).
(define-type signed (tupleof syns-a sty int))
(define-type signed-opt (listof signed acyclic))

;;; ============================================================ macros-infer.fx (whole)

;;; ------------------------------------------------------------ the shape

;; A part of the shape of a macro's operands (`infer.rs`'s `Shape`): a
;; list by ellipsis, its element; a pattern variable's, of more than one,
;; the clauses'.
(define-datatype mx-infer-shape (sh-nil) (sh-pair int int) (sh-seq int) (sh-lit symbol) (sh-leaf)
  (sh-union (listof int acyclic)))
(define-type mx-infer-ints (listof int acyclic))
;; A part: its shape, its address from its region's root, and that root.
(define-type mx-infer-part (tupleof mx-infer-shape addr int))
;; A pattern's part, of a clause, under so many ellipses in its pattern.
(define-type mx-infer-item (tupleof int syn int))
(define-type mx-infer-items (listof mx-infer-item acyclic))
(define-datatype class (cl-seq syn) (cl-wild) (cl-lit symbol) (cl-var symbol) (cl-nil)
  (cl-pair syn syn))
(define-type classes (listof class acyclic))
;; Each pattern variable's part, by its clause, and how many ellipses its
;; pattern has it under, in the order found.
(define-type vars (listof (tupleof int symbol int int) acyclic))
;; What each clause's macro's literals are.
(define-type lits-of (listof mx-parser-types-names acyclic))
(define-type spans (listof (tupleof int int) acyclic))
;;; ------------------------------------------------------------ kinds

;; What a pattern variable's part is of: a syntax type, or a part of the
;; signature being inferred (a list by ellipsis, a `union`).
(define-datatype kd (kd-ty sty) (kd-ref int))
(define-type pkinds (listof (tupleof int symbol kd) acyclic))
;; What the shape is built from: each clause's macro's literals, the
;; pattern variables' syntax types last found, and the lists by ellipsis
;; that are not to take in patterns written one by one.
(define-type mx-infer-cx (tupleof lits-of pkinds spans))
;; Each shape the patterns have here, by the indices of `cs` of it, in the
;; order first written.
(define-type mx-infer-groups (listof (tupleof class mx-infer-ints) acyclic))
;; If `its` may be one list by ellipsis, in a list of one: its elements'
;; patterns, the pattern variables of the whole list, and whether it took
;; in patterns written one by one (`infer.rs`'s `absorbable`).
(define-type took (tupleof mx-infer-items vars bool bool))
(define-type kds (listof kd acyclic))
(define-type kinds (listof (tupleof int kd) acyclic))
;;; ------------------------------------------------------------ what it makes

;; What each macro makes (0 an expression, 1 definitions, 2 a description),
;; by its templates (`infer.rs`'s `makes_all`): its name, its literals, its
;; clauses; each clause's macro, pattern and template.
(define-type unsigned (tupleof symbol mx-parser-types-names syns-a syn))
(define-type unsigneds (listof unsigned acyclic))
(define-type rule (tupleof int syn syn))
(define-type rules (listof rule acyclic))
(define-type makes-known (listof mx-infer-ints acyclic))
;;; ------------------------------------------------------------ the signature so far

;; What an iteration infers, by part (`infer.rs`'s `Sol`): each leaf's
;; syntax type, each pattern variable's, the parts `many`, the imports and
;; exports, each macro's export, and the lists by ellipsis not to take in
;; patterns written one by one.
(define-type binds-at (listof (tupleof int mx-types-bind) acyclic))
(define-type sol (tupleof kinds pkinds mx-infer-ints binds-at binds-at binds-at spans))
;;; ------------------------------------------------------------ placements

;; A pattern variable placed by a template: which, as what, in what scope,
;; whether in a `many` part, under how many ellipses.
(define-type placement (tupleof symbol sty mx-env-types-levels bool int))
(define-type placements (listof placement acyclic))
;; Clause `cid`'s template's placements, and, if it makes definitions,
;; what of what a use gives it defines (in a list of one); none if its
;; template is refused with the signature so far.
(define-type mx-infer-placed (tupleof placements (listof mx-exports-exported acyclic)))

;;; ============================================================ macros-shown.fx (whole)

;; A region of a node: its number; the region it is in (-1 none) and the list's path there; the
;; pattern there, if known; what an element exports, if it does.
(define-type zone (tupleof int int addr syns-a syns-a))
(define-type spot (tupleof int addr))
(define-type spots (listof spot acyclic))
;; What a fact names: 0 a part, 1 `(v ...)`, every repetition's; its region and path.
(define-type mx-shown-label (tupleof int int addr))
(define-type mx-shown-labels (listof mx-shown-label acyclic))
(define-type mx-shown-imports (listof (tupleof int addr syn) acyclic))
(define-type givens (listof (tupleof int addr symbol) acyclic))
;; A node being shown, its state saved while another's is: regions, the newest first; imports,
;; the newest first; the parts wanted labelled; those named; those written `place`; the names
;; used; the last `pN`.
(define-type nstate (tupleof (listof zone acyclic) mx-shown-imports spots givens spots
                             (listof string acyclic) int))
;; A part's imports, innermost first, one fact, its levels outermost first
;; (`macros-facts.fx`'s `bind`); a repetition's, a fact each.
(define-type mx-shown-binds (listof (tupleof int addr (listof mx-shown-labels acyclic)) acyclic))

;;; ============================================================ macros-solve.fx (whole)

;;; ------------------------------------------------------------ scopes

;; One level of an environment, its atoms by part; one of a ribcage's, by
;; part; what a region's root must export (`infer.rs`'s `Lv`, `RingD`,
;; `Demand`).
(define-type pents (listof (tupleof int int) acyclic))
;; A level's entries: each binder's part, what it binds, and how many lists
;; by ellipsis gathered it (`atm-var`'s).
(define-type gents (listof (tupleof int int int) acyclic))
(define-datatype lv (iv-rib gents) (iv-cage int))
(define-type lvs (listof lv acyclic))
(define-datatype rd (rd-at int) (rd-rib pents))
(define-type rds (listof rd acyclic))
(define-datatype demand (dm-rib pents) (dm-cage rds))
;; Where each of levels `ls` (innermost first) of part `p` is said: on `p`
;; while each level's binders are in or under its region; then on the part
;; whose region that is, and so on out. Each site with its region and
;; levels, in a list of one.
(define-type site (tupleof int int lvs))
(define-type sites (listof site acyclic))
;; Entries `es` in `r`, and those through each part in `r`, by part, in
;; the order first found; none if one is not under `r`, or not through as
;; many lists as gathered it (a list gathers its elements' repetitions).
(define-type thru (listof (tupleof int gents) acyclic))
;; One site's rings, of each clause's (`infer.rs`'s `merge_rings`).
(define-type site-rings (listof (tupleof int rds) acyclic))
;;; ------------------------------------------------------------ solving

;; What the templates' placements, `ps` (by clause, none where a template
;; is refused), say of each part, from `prev`; `made`, each clause's
;; template's definitions; `makes`, each macro's; `of`, each clause's macro.
(define-type by-clause (listof (listof mx-infer-placed acyclic) acyclic))
(define-type named-kds (listof (tupleof symbol kds) acyclic))
;; The scopes: each site's rings, of each clause's, in the order said.
(define-type site-list (listof (tupleof int site-rings) acyclic))
;; Each region's export, the first said (an empty one only where none is);
;; each other member of a `union` one that exports, an empty one.
(define-type got-demands (listof (tupleof int demand) acyclic))
(define-type mx-solve-strings (listof string acyclic))

;;; ============================================================ macros-fresh.fx (whole)

(define-type tags (listof (tupleof int string) acyclic))
(define-type seen-keys (listof (tupleof int int) acyclic))
;; A module file's forms begun, their keys of `prefix`: their own; what is
;; given back puts the program's back (`fresh-scope-end!`).
(define-type fresh-saved (tupleof string int int seen-keys string int tags))

;;; ============================================================ macros-contracts.fx (whole)

(define-type mx-contracts-renames (listof (tupleof symbol symbol) acyclic))
;; Parts' places; binders' names by part; what is renamed in each part.
(define-type mx-contracts-places (listof (tupleof int int) acyclic))
(define-type mx-contracts-numbered (listof (tupleof int symbol) acyclic))
(define-type renamings (listof (tupleof int int mx-contracts-renames sty) acyclic))
;; Each pattern variable the walk found: its part, whether a binder, and the
;; binders (as pattern variables) in its scope, innermost first.
(define-type part-var (tupleof int bool (listof symbol acyclic) sty))
(define-type placed-renames (listof (tupleof int int symbol) acyclic))
;; Output `s`'s nodes, aligned as a program's form is (nothing expanded) and
;; walked, each as α-equivalence takes it: its kind, the nodes under it by
;; number, a binder whatever its name, a reference by its binder or, free,
;; by its name, any other part as it is (`Macros::alpha_nodes`).
(define-type anode (tupleof int (listof int acyclic) (listof symbol acyclic) (listof syn acyclic)))
(define-type anodes (listof anode acyclic))
;;; ------------------------------------------------------------ definitions

;; Names, each with whether a description's.
(define-type named-ns (listof (tupleof symbol bool) acyclic))

;;; ============================================================ macros-procedural.fx (whole)

;;; ------------------------------------------------------------ defined, used

;; Each procedural macro, by its name: its style, and its transformer as
;; text; the last defined first.
;; And whether its signature is written, its contract
;; (`procedural::Procedural::contract`).
(define-type procedural (tupleof symbol int string def-env bool (listof string acyclic)))
;;; ------------------------------------------------------------ read

;; What the runtime gave, `text`, read: a datum's syntax, every part at
;; `a`..`b` of expansion `x`, the use's place; none if it is not one.
(define-type mx-procedural-read-one (listof (tupleof syn int) acyclic))
;; Both, as a session's state after a form keeps them (`by-state`): a state
;; read again has its own transformers, not a later form's of their names.
(define-type phase1-state (tupleof (listof string acyclic) (listof procedural acyclic)))
;; A module file's own macros' scope begun: none of these macros, nor any
;; procedural, defined, and macros defined where the file's items are; what
;; was, given back, for `file-scope-end!` (`Macros::in_file_scope`).
(define-type scope-saved
  (tupleof macro-env (listof procedural acyclic) def-env marked (listof string acyclic)))
;;; ------------------------------------------------------------ expansions that do not end

;; The uses of macros the expansion is inside, the innermost first: each
;; its macro, its form, and how many parts the form has, -1 if it was not
;; counted; and how many there are (`macros.rs`'s `Withs::chain`).
(define-type link (tupleof symbol syn int))

;;; ============================================================ macros.fx (whole)

(define-type mx-top-said (tupleof signed-opt (listof string acyclic)))
(define-type by-states
  (listof (tupleof int macro-env load-defs mx-parser-types-names seen-keys marked phase1-state) acyclic))
;;; ------------------------------------------------------------ expanding

;; Each expansion made, the last first: its number, its macro's name, where
;; its use is, and whether its clause was checked once (`Expansion::typed`);
;; `syn-expansion` says which made a part.
(define-type expansions-made (listof (tupleof int symbol int int bool) acyclic))
;; Each binder's fresh name, by its path and the repetitions it is in; how
;; many times each repeated binder was repeated, by its path and the
;; repetitions around that; and the repetition of each ellipsis
;; transcription is under, outermost first (`macros.rs`'s `Made`).
(define-type freshes (listof (tupleof addr (listof int acyclic) symbol) acyclic))
(define-type mx-top-counts (listof (tupleof addr (listof int acyclic) int) acyclic))

;;; ============================================================ macros-templates.fx (whole)

(define-type params (tupleof syns-a mx-match-binds syns-a))
;; The binders in the scope of hole `n`, outermost first (`levels`), each a
;; typed `bvar`: its name and type.
(define-type scope-vars (listof (tupleof symbol syn) acyclic))

;;; ============================================================ macros-imports.fx (whole)

;; The forms the files whose own macros' uses were expanded were read as, by
;; base (`parser-top.fx`'s `loaded-own-expanded!`): what an importer reads
;; of each, as the Rust checker reads its text again (`file_macros_at`).
(define-type raw (listof (tupleof int syns-a) acyclic))
;; Each macro a file gives, with the name the importer has it by.
(define-type mx-imports-given (listof (tupleof symbol symbol) acyclic))
;; A module file's `(define name (load-module "path"))`s so far, the last
;; first, each with its load, which the driver read it by where it starts.
(define-type file-loads (listof (tupleof symbol syn) acyclic))
;; What a file's forms defined so far: the name the importer has each of its
;; macros by, the macros it gives, each with whether it is hidden, the last
;; first, and its load definitions.
(define-type file-state (tupleof (listof (tupleof symbol symbol) acyclic) mades file-loads))

;;; ============================================================ macros-walk.fx (whole)

;; Of `found`, each value reference's binder, innermost first: the names
;; reached only past a module's names, kept (not renamed); and each
;; reference with its binder's node.
(define-type pairs (listof (tupleof int int) acyclic))
;; Each of binders `bs` with its new name, `name%k` from `k`, by its path.
(define-type mx-walk-renames (listof (tupleof addr string) acyclic))

;;; ============================================================ macros-tree.fx (whole)

(define-type mx-tree-ints (listof int acyclic))

;;; ============================================================ macros-shaper.fx (whole)

;; Members by name, for as many as the top level has (`top-vals`): a tree
;; by each name's hash (`symbol-name-hash`, FNV-1a, so as deep as a
;; tree of random keys), a name of the same hash as a node's but another
;; to its right; persistent, so that each form's reading state keeps the
;; top level's as they were (`shapes-after!`) for nothing. Added and found
;; in the tree's depth, where a list of them was copied whole for each name
;; added (`with-member`) and walked for each found (`member-of`).
(define-datatype name-tree (nm-leaf) (nm-node int symbol mx-parser-types-shapes name-tree name-tree))
(define-type file-shape (tupleof string mx-parser-types-shapes mx-parser-types-shapes syns-a))
;; What a module was asked: each name, a description's or a value's, and
;; whether it gives it; and whether its shape was not known where asked.
(define-type asked (tupleof (listof (tupleof symbol bool bool) acyclic) bool))
;;; ------------------------------------------------------------ where a value comes from

;; Where binder `b`'s value comes from (`macros/shapes.rs`'s `Source`): the
;; expression bound (0), a type declared (1), a member of the module at a
;; node (2, its name), the `k`th parameter of the procedure type a name
;; names (3), parameter `i` of the function given as argument `k` to the
;; operator at a node (4); none (-1).
(define-type source (tupleof int int int int symbol))

;;; ============================================================ macros-shapes.fx (whole)

;; What to write into each form that opens a module, by its path: what its
;; module was asked, or none (`#f`) where its shape was not known.
(define-type writing (tupleof addr (listof syn acyclic)))
;; The top level's shapes by the reading state each form left, as the
;; expander keeps its macros (`macros-at!`, `macros-after!`); and the shapes
;; named (`named-shapes`).
(define-type mx-shapes-named (listof (tupleof symbol syn) acyclic))
(define-type shapes-state (tupleof int name-tree name-tree mx-shapes-named))
(define-type mx-shapes-ints (listof int acyclic))
;; Each free alias `found` reaches, by its node's path, what it is written
;; as, onto `out`'s; and onto its binders, each binder at a use that would
;; capture a name a module file's own macro introduces (none of `top`, the
;; form's own definitions, which are the file's items).
(define-type mx-shapes-parts (listof (tupleof addr syn) acyclic))
(define-type mx-shapes-written-of (tupleof mx-shapes-parts mx-shapes-ints))
;; Each captor, `cs`, with its fresh name, `%x~K.N`: spelled apart from a
;; template's fresh names, which another count makes (`captor-tag`).
(define-type captors (listof (tupleof int symbol) acyclic))
