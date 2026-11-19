//! `Trace` (C479) of the Rust kfx writes: a datatype's generated impl
//! reaches what its value holds of the heap, each value once.

use kfx_corpus::data::{Tree, leaf, node};

/// How many values of `T`'s name were reached.
fn reached(t: &kayak_rt::Tracer<'_>, name: &str) -> u64 {
    let (rows, _) = t.census();
    rows.iter()
        .filter(|r| r.0.ends_with(name))
        .map(|r| r.1)
        .sum()
}

#[test]
fn a_tree_reached_each_node_once() {
    let l = leaf();
    // A node of two leaves, the same leaf twice: two values, not three.
    let shared = node(l, 1, l);
    let root = node(shared, 2, node(shared, 3, leaf()));
    let t = kayak_rt::traced(root);
    // root, its right child, `shared`: three nodes; `l` and one more leaf.
    assert_eq!(reached(&t, "data::Tree"), 5);
}

#[test]
fn a_deep_tree_in_a_loop() {
    let mut t = leaf();
    for k in 0..200_000 {
        t = node(t, k, leaf());
    }
    let tr = kayak_rt::traced(t);
    // The root handle's value is reached too: 200,000 nodes, 200,001 leaves.
    assert_eq!(reached(&tr, "data::Tree"), 400_001);
    let _: &Tree = &t;
}
