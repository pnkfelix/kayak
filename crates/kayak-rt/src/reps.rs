//! Types as reps (`docs/research/type-reps.md`, C300): the standard
//! procedures `rep-of-nodes` and `rep-nodes`, FX-26's (Fixpt's
//! `standard.rs`), by `kayak-reps`, Fixpt's crate taken whole, as Fixpt's
//! runtime has them (`prim.rs`'s `%fx26-rep-of-nodes`,
//! `%fx26-rep-nodes`). A rep crosses as its binary form, a bytevector.

use crate::world::{bytes_of, to_vec};
use crate::{
    Bytes, BytesAt, Int, List, Show, Str, Symbol, fail, heap_alloc, heap_cons, i64_to_int,
    int_to_i64, leak_string, nil, string_to_symbol, symbol_to_string,
};
use kayak_reps::{Arg, Builder, Const};

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub struct Product_11_12<A0, A1>(pub A0, pub A1);

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub struct Product_11_12_13<A0, A1, A2>(pub A0, pub A1, pub A2);

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub struct Product_11_12_13_14_15<A0, A1, A2, A3, A4>(pub A0, pub A1, pub A2, pub A3, pub A4);

#[allow(non_camel_case_types)]
pub enum Sum_4node_5bound_4hole_5const_3sym_3int_4flag<A0, A1, A2, A3, A4, A5, A6> {
    Node(A0),
    Bound(A1),
    Hole(A2),
    Const(A3),
    Sym(A4),
    Int(A5),
    Flag(A6),
}

impl<A0, A1> Show for Product_11_12<A0, A1> {
    fn show(&self, out: &mut String) {
        out.push_str("#<product of 2>");
    }
}

impl<A0, A1, A2> Show for Product_11_12_13<A0, A1, A2> {
    fn show(&self, out: &mut String) {
        out.push_str("#<product of 3>");
    }
}

impl<A0, A1, A2, A3, A4> Show for Product_11_12_13_14_15<A0, A1, A2, A3, A4> {
    fn show(&self, out: &mut String) {
        out.push_str("#<product of 5>");
    }
}

impl<A0, A1, A2, A3, A4, A5, A6> Show
    for Sum_4node_5bound_4hole_5const_3sym_3int_4flag<A0, A1, A2, A3, A4, A5, A6>
{
    fn show(&self, out: &mut String) {
        out.push_str(match self {
            Self::Node(..) => "#<sum node>",
            Self::Bound(..) => "#<sum bound>",
            Self::Hole(..) => "#<sum hole>",
            Self::Const(..) => "#<sum const>",
            Self::Sym(..) => "#<sum sym>",
            Self::Int(..) => "#<sum int>",
            Self::Flag(..) => "#<sum flag>",
        })
    }
}

/// An operand, as the standard procedures give and take it.
pub type RepOperand = &'static Sum_4node_5bound_4hole_5const_3sym_3int_4flag<
    Int,
    Product_11_12<Int, Int>,
    Int,
    Product_11_12_13<Str, Int, Str>,
    Symbol,
    Int,
    bool,
>;

/// A node: its slot, its head, whether it is a set, its binders' names, its
/// operands.
pub type RepNode = Product_11_12_13_14_15<Int, Symbol, bool, List<Symbol>, List<RepOperand>>;

fn items<T: Copy>(mut l: crate::ListAt<'_, T>) -> Vec<T> {
    let mut out = Vec::new();
    while let Some(c) = l {
        out.push(c.car);
        l = c.cdr;
    }
    out
}

fn listed<T: 'static>(xs: Vec<T>) -> List<T> {
    xs.into_iter().rev().fold(nil(), |acc, x| heap_cons(x, acc))
}

fn small(n: Int) -> Result<i64, String> {
    Ok(int_to_i64(n))
}

/// The rep the checker's nodes make, made canonical; its binary form
/// (Fixpt's `rep_of_nodes`).
fn rep_of(nodes: List<RepNode>, holes: List<Symbol>, root: Int) -> Result<kayak_reps::Rep, String> {
    type S<'a> = Sum_4node_5bound_4hole_5const_3sym_3int_4flag<
        Int,
        Product_11_12<Int, Int>,
        Int,
        Product_11_12_13<Str, Int, Str>,
        Symbol,
        Int,
        bool,
    >;
    let nodes = items(nodes);
    let slots = nodes
        .iter()
        .map(|n| small(n.0))
        .collect::<Result<Vec<i64>, _>>()?;
    let mut b = Builder::new();
    for _ in 0..=slots.iter().copied().max().unwrap_or(0) {
        b.reserve();
    }
    let mut hole_args = Vec::new();
    for h in items(holes) {
        hole_args.push(b.hole(symbol_to_string(h).as_str()));
    }
    for (node, slot) in nodes.iter().zip(&slots) {
        let head = symbol_to_string(node.1).as_str().to_string();
        let set = node.2;
        let binders: Vec<String> = items(node.3)
            .into_iter()
            .map(|s| symbol_to_string(s).as_str().to_string())
            .collect();
        let mut args = Vec::new();
        for arg in items(node.4) {
            args.push(match arg {
                S::Node(i) => Arg::Node(small(*i)? as u32),
                S::Bound(p) => Arg::Bound {
                    binder: small(p.0)? as u32,
                    pos: small(p.1)? as u32,
                },
                S::Hole(i) => *hole_args
                    .get(small(*i)? as usize)
                    .ok_or("a hole not named")?,
                S::Const(p) => {
                    let generative = p.0.as_str().to_string();
                    let which = small(p.1)? as u32;
                    let kind = p.2.as_str().to_string();
                    b.konst(Const {
                        key: kayak_reps::generative_key(&generative, which),
                        name: generative,
                        kind,
                    })
                }
                S::Sym(s) => Arg::Sym(b.name(symbol_to_string(*s).as_str())),
                S::Int(k) => Arg::Int(small(*k)?),
                S::Flag(f) => Arg::Flag(*f),
            });
        }
        let refs: Vec<&str> = binders.iter().map(String::as_str).collect();
        if set {
            b.fill_set(*slot as u32, &head, args);
        } else {
            b.fill(*slot as u32, &head, &refs, args);
        }
    }
    b.finish(small(root)? as u32).map(|f| f.rep)
}

/// `rep-of-nodes`: a type as a rep, from the nodes a checker made of it,
/// the holes' names and the root's slot; its binary form.
pub fn rep_of_nodes(nodes: List<RepNode>, holes: List<Symbol>, root: Int) -> Bytes {
    match rep_of(nodes, holes, root) {
        Ok(rep) => bytes_of(kayak_reps::binary::encode(&rep)),
        Err(why) => fail(&format!("rep-of-nodes: {why}")),
    }
}

fn sym(s: &str) -> Symbol {
    string_to_symbol(leak_string(s.to_string()))
}

/// `rep-nodes`: a rep's nodes, from its binary form, each constant the
/// place of the generative type of `gens` of its key (-1 if none); and its
/// holes' names (Fixpt's `rep_nodes`).
pub fn rep_nodes(
    bytes: BytesAt<'_>,
    gens: List<Str>,
) -> Product_11_12<List<RepNode>, List<Symbol>> {
    type S = Sum_4node_5bound_4hole_5const_3sym_3int_4flag<
        Int,
        Product_11_12<Int, Int>,
        Int,
        Product_11_12_13<Str, Int, Str>,
        Symbol,
        Int,
        bool,
    >;
    let rep = match kayak_reps::binary::decode(&to_vec(bytes), kayak_reps::heap::validate_all()) {
        Ok(r) => r,
        Err(why) => fail(&format!("rep-nodes: {why}")),
    };
    let keys: Vec<u64> = items(gens)
        .iter()
        .enumerate()
        .map(|(w, g)| kayak_reps::generative_key(g.as_str(), w as u32))
        .collect();
    let int = |k: i64| i64_to_int(k);
    let nodes: Vec<RepNode> = rep
        .nodes
        .iter()
        .enumerate()
        .map(|(k, node)| {
            let args: Vec<RepOperand> = node
                .args
                .iter()
                .map(|a| {
                    let s: S = match *a {
                        Arg::Node(i) => S::Node(int(i as i64)),
                        Arg::Bound { binder, pos } => {
                            S::Bound(Product_11_12(int(binder as i64), int(pos as i64)))
                        }
                        Arg::Hole(i) => S::Hole(int(i as i64)),
                        Arg::Const(i) => {
                            let c = &rep.consts[i as usize];
                            let which = keys
                                .iter()
                                .position(|k| *k == c.key)
                                .map_or(-1, |w| w as i64);
                            S::Const(Product_11_12_13(
                                leak_string(c.name.clone()),
                                int(which),
                                leak_string(c.kind.clone()),
                            ))
                        }
                        Arg::Sym(i) => S::Sym(sym(rep.name(i))),
                        Arg::Int(v) => S::Int(int(v)),
                        Arg::Flag(b) => S::Flag(b),
                    };
                    heap_alloc(s)
                })
                .collect();
            let binders: Vec<Symbol> = node.binders.iter().map(|&b| sym(rep.name(b))).collect();
            Product_11_12_13_14_15(
                int(k as i64),
                sym(rep.name(node.head)),
                node.set,
                listed(binders),
                listed(args),
            )
        })
        .collect();
    let holes: Vec<Symbol> = rep.holes.iter().map(|h| sym(h)).collect();
    Product_11_12(listed(nodes), listed(holes))
}
