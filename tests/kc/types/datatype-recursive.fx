;; => 10
(define-datatype tree (leaf) (node tree int tree))
(define sum-tree (subr (maxeff spin (read (globals sum-tree))) (tree) int) (lambda (t) (tagcase t (leaf () 0) (node (l v r) (+ v (+ (sum-tree l) (sum-tree r)))))))
(sum-tree (node (node (leaf) 1 (leaf)) 2 (node (leaf) 3 (node (leaf) 4 (leaf)))))
