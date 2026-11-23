//! A thread's collection of its pages (C511; `TODO.md` §54, a collector's
//! fourth step): Bartlett's mostly-copying collector, over objects of
//! Fixpt's layout, read by their headers alone (`laid`).
//!
//! - **The roots** (`roots::roots_from`) are ambiguous, each a word that
//!   may be a pointer: the registers and stack, the thread's owned blocks
//!   and thread locals, and every kept object, whole. An allocation one
//!   reaches is pinned, its page with it: nothing on a pinned page moves.
//!   So is the allocation before a word that is an allocation's first, as
//!   an object's word with no suffix points one past it.
//! - **A constant's** every allocation it reaches is pinned, not only the
//!   first: any thread may read it, as it runs.
//! - **The heap's words are precise**: an object's fields, each a Fixpt
//!   word, found by its header. Each is followed to its allocation; every
//!   allocation reached on a page of this thread's that no root pins is
//!   copied to a new page, a forwarding word left in its first (Fixpt's
//!   tag 7, as `kcrt-core`'s collector and Fixpt's leave one), and every
//!   reached object's fields are then made to point where their targets
//!   went, each word's offset into its allocation kept. An object of
//!   another thread's (a page of its, or a large object it made) is never
//!   written: its words are read as ambiguous, pinning what they reach.
//! - **The pages copied from** are given back (`space::end_collection`);
//!   the string cursor, keyed by address, forgotten. A large object is
//!   never moved, nor yet freed.
//!
//! The mark is first and whole, the copy after it: a kept object reached
//! late may pin what is reached early, and nothing is moved before every
//! pin is known.

use super::space::{self, View, Whose};
use super::{HeapWord, laid};
use std::collections::HashSet;
use std::hash::{BuildHasherDefault, Hasher};

/// Fixpt's forwarding word's tag.
const TAG_FORWARD: u64 = 0b111;
const TAG_MASK: u64 = 0b111;
const TAG_PAIR: u64 = 0b001;
const TAG_OBJECT: u64 = 0b100;
const TAG_HEADER: u64 = 0b110;

/// Addresses, hashed by a multiply: they are spread already.
#[derive(Default)]
struct Addresses(u64);

impl Hasher for Addresses {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 = (self.0.rotate_left(8) ^ u64::from(b)).wrapping_mul(0x9e37_79b9_7f4a_7c15);
        }
    }
    fn write_usize(&mut self, n: usize) {
        self.0 = (self.0 ^ n as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
    }
}

type Set<T> = HashSet<T, BuildHasherDefault<Addresses>>;

/// A collection's count, for a test or a report.
static COLLECTIONS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn collections() -> u64 {
    COLLECTIONS.load(std::sync::atomic::Ordering::Relaxed)
}

/// What the thread's last collection did.
#[derive(Clone, Copy, Default, Debug)]
pub(crate) struct Last {
    pub reached: usize,
    pub moved: usize,
    pub pinned_pages: usize,
    pub freed_pages: usize,
}

thread_local!(static LAST: std::cell::Cell<Last> = const { std::cell::Cell::new(Last { reached: 0, moved: 0, pinned_pages: 0, freed_pages: 0 }) });

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn last() -> Last {
    LAST.get()
}

/// A collection of this thread's pages, now. Its frame a small one, a
/// safepoint's, as `roots::reached`'s: the registers spilled there, the
/// stack read from there.
#[inline(never)]
pub(crate) fn collect() {
    let mut saved = [0usize; 12];
    super::roots::spill(&mut saved);
    let saved = std::hint::black_box(saved);
    collect_from(&saved, std::ptr::from_ref(&saved).addr());
    std::hint::black_box(&saved);
}

#[inline(never)]
fn collect_from(saved: &[usize; 12], from: usize) {
    space::begin_collection();
    let roots = super::roots::roots_from(saved, from);
    let view = View::new();
    let mine = space::my_pages();
    let mut m = Mark {
        view: &view,
        order: Vec::new(),
        others: Set::default(),
        pinned: Set::default(),
        work: Vec::new(),
    };
    for &w in &roots.constants {
        m.ambiguous(w);
    }
    m.drain(true);
    let mut pins = [m.pinned.len(); 5];
    for (i, &w) in roots.words.iter().enumerate() {
        m.ambiguous(w);
        for (k, &e) in roots.ends.iter().enumerate() {
            if i + 1 == e {
                pins[k + 1] = m.pinned.len();
            }
        }
    }
    pins[4] = m.pinned.len();
    m.drain(false);
    if verbose() {
        eprintln!(
            "kayak: pinned pages: constants {}, stack {}, owned {}, kept {}, locals {}; words: stack {}, owned {}, kept {}",
            pins[0],
            pins[1] - pins[0],
            pins[2] - pins[1],
            pins[3] - pins[2],
            pins[4] - pins[3],
            roots.ends[0],
            roots.ends[1] - roots.ends[0],
            roots.ends[2] - roots.ends[1],
        );
    }
    let Mark {
        order,
        others,
        pinned,
        ..
    } = m;

    // From-space: this thread's pages that nothing pins.
    let from_pages: Vec<_> = mine
        .iter()
        .copied()
        .filter(|&(c, p)| !pinned.contains(&space::page_key(c, p)))
        .collect();
    for &(c, p) in &from_pages {
        c.set_from(p, true);
    }
    let in_from = |view: &View, a: usize| view.page_of(a).is_some_and(|(c, _)| c.is_from(a));

    // Each reached allocation of from-space copied, a forwarding word
    // left in its first.
    let mut moved = 0;
    for &a in &order {
        if !in_from(&view, a) {
            continue;
        }
        let mut end = a;
        each_object(&view, a, |o| end = o.end);
        let n = view.extent_of(a, end);
        let to = space::words(n);
        // SAFETY: `n` words of the allocation, read whole into `n` new
        // words of to-space, which no other object overlaps; then its
        // first word, its forwarding word.
        unsafe {
            let at = space::word_mut(&view, a);
            std::ptr::copy_nonoverlapping(at, to, n);
            at.write(to.addr() as u64 | TAG_FORWARD);
        }
        moved += 1;
    }

    // Each reached object this thread may write, where it is now, its
    // fields made to point where their targets went. The space seen again,
    // with to-space's chunks.
    let view = View::new();
    let forwarded = |s: usize| -> Option<usize> {
        if !in_from(&view, s) {
            return None;
        }
        // SAFETY: an allocation's first word in from-space, its words kept
        // till the collection's end.
        let w = unsafe { std::ptr::with_exposed_provenance::<u64>(s).read() };
        (w & TAG_MASK == TAG_FORWARD).then_some((w & !TAG_MASK) as usize)
    };
    for &a in order.iter().chain(others.iter()) {
        let at = forwarded(a).unwrap_or(a);
        each_object(&view, at, |o| {
            for f in o.fields {
                let w = f.raw() as u64;
                let s = match w & TAG_MASK {
                    TAG_PAIR => (w & !TAG_MASK) as usize,
                    TAG_OBJECT => {
                        match view.object_of(((w & !TAG_MASK) as usize).wrapping_sub(8)) {
                            Some(s) => s,
                            None => continue,
                        }
                    }
                    _ => continue,
                };
                let Some(to) = forwarded(s) else { continue };
                let new = w.wrapping_add(to as u64).wrapping_sub(s as u64);
                // SAFETY: a field of a reached object this thread may write.
                unsafe {
                    space::word_mut(&view, std::ptr::from_ref(f).addr()).write(new);
                }
            }
        });
    }
    for &(c, p) in &mine {
        c.clear_marks(p);
    }
    for &(c, p) in &from_pages {
        c.set_from(p, false);
    }
    crate::forget_cursor();
    LAST.set(Last {
        reached: order.len() + others.len(),
        moved,
        pinned_pages: pinned.len(),
        freed_pages: from_pages.len(),
    });
    if verbose() {
        let l = LAST.get();
        eprintln!(
            "kayak: collected: {} reached, {} moved, {} pages pinned, {} freed",
            l.reached, l.moved, l.pinned_pages, l.freed_pages
        );
    }
    space::end_collection(&from_pages);
    COLLECTIONS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

/// `KAYAK_GC_VERBOSE=1`: each collection said.
fn verbose() -> bool {
    static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *V.get_or_init(|| std::env::var("KAYAK_GC_VERBOSE").is_ok_and(|v| v != "0"))
}

/// Each object of the allocation at `start`, as `laid::allocation` finds
/// them, with no `Vec`: one, or a datatype's value's two (C500).
#[inline]
fn each_object(view: &View, start: usize, mut f: impl FnMut(&laid::Laid)) {
    // Where the allocation may end at the most: its page's end, or its
    // large object's.
    let (chunk, limit) = match view.page_of(start) {
        Some((c, _)) => (
            Some(c),
            (start & !(space::PAGE_BYTES - 1)) + space::PAGE_BYTES,
        ),
        None => match view.large_owner(start) {
            Some((p, w, _)) => (None, p + 8 * w),
            None => return,
        },
    };
    let mut at = start;
    loop {
        // SAFETY: an allocation's first word, or the next object of the
        // same allocation, which a header begins.
        let o = unsafe { laid::at(std::ptr::with_exposed_provenance::<u64>(at)) };
        f(&o);
        if o.pair || o.end >= limit || chunk.is_some_and(|c| c.is_start(o.end)) {
            return;
        }
        // SAFETY: a word of the same allocation, past its last object.
        if unsafe { std::ptr::with_exposed_provenance::<u64>(o.end).read() } & TAG_MASK
            != TAG_HEADER
        {
            return;
        }
        at = o.end;
    }
}

/// The mark: this thread's allocations reached, in the order found (each
/// marked on its page), and another's that may be followed (an ended
/// thread's, a large object); the pages pinned.
struct Mark<'v> {
    view: &'v View,
    order: Vec<usize>,
    others: Set<usize>,
    pinned: Set<(usize, usize)>,
    work: Vec<usize>,
}

impl Mark<'_> {
    /// An ambiguous word: the allocation it is a word of, and the one
    /// before when it is an allocation's first, reached and pinned.
    fn ambiguous(&mut self, w: usize) {
        let a = self.view.object_of(w);
        if let Some(a) = a {
            self.visit(a, true);
        }
        if a.is_none_or(|a| a == w & !7)
            && let Some(b) = self.view.object_of((w & !7).wrapping_sub(8))
        {
            self.visit(b, true);
        }
    }

    #[inline]
    fn visit(&mut self, a: usize, pin: bool) {
        // A live thread's object is not read: it may be being made. What
        // it holds of this thread's, it holds only by what is shared, and
        // nothing is while `space::Concurrent` lives but the constants,
        // pinned whole.
        if let Some((c, p)) = self.view.page_of(a) {
            match self.view.whose_page(c, p) {
                Whose::Live => {}
                Whose::Mine => {
                    if pin {
                        self.pinned.insert(space::page_key(c, p));
                    }
                    if c.mark(a) {
                        self.order.push(a);
                        self.work.push(a);
                    }
                }
                Whose::Dead => {
                    if self.others.insert(a) {
                        self.work.push(a);
                    }
                }
            }
        } else if let Some((_, _, whose)) = self.view.large_owner(a)
            && whose != Whose::Live
            && self.others.insert(a)
        {
            self.work.push(a);
        }
    }

    /// Each allocation reached, its words followed, precisely (this
    /// thread's, or an ended thread's). Under a constant (`all`), each
    /// allocation reached is pinned.
    fn drain(&mut self, all: bool) {
        while let Some(a) = self.work.pop() {
            if all {
                self.visit(a, true);
            }
            let view = self.view;
            let mut found = Vec::new();
            each_object(view, a, |o| {
                for &f in o.fields {
                    if let Some(t) = target(view, f) {
                        found.push(t);
                    }
                }
            });
            for t in found {
                self.visit(t, all);
            }
        }
    }
}

/// The allocation a field's word points to, if it is a pair's or an
/// object's word of one: a pair's, at an allocation's first word that is
/// no header; an object's, at the suffix of an object of the allocation it
/// is in, its header found by its trailer and checked. Checked so, since a
/// dead object on a pinned page, reached by a stale word, may hold words
/// into pages taken again since.
#[inline]
fn target(view: &View, f: HeapWord) -> Option<usize> {
    let w = f.raw() as u64;
    let read = |a: usize| -> u64 {
        // SAFETY: a word of an allocation of the space, read.
        unsafe { std::ptr::with_exposed_provenance::<u64>(a).read() }
    };
    match w & TAG_MASK {
        TAG_PAIR => {
            let t = (w & !TAG_MASK) as usize;
            (view.object_of(t) == Some(t) && read(t) & TAG_MASK != TAG_HEADER).then_some(t)
        }
        TAG_OBJECT => {
            let at = (w & !TAG_MASK) as usize;
            let s = view.object_of(at.wrapping_sub(8))?;
            let before = read(at - 8);
            let main = if before & TAG_MASK == TAG_HEADER {
                at - 8
            } else {
                let back = ((before >> 3) & ((1 << 59) - 1)) as usize;
                (at - 8).checked_sub(8 * back)?
            };
            if main < s || read(main) & TAG_MASK != TAG_HEADER {
                return None;
            }
            // An extension word before a main header (Fixpt's large
            // field count) is where its object starts.
            let first = if main > s
                && (read(main - 8) >> 3) & 0xff == 255
                && read(main - 8) & TAG_MASK == TAG_HEADER
            {
                main - 8
            } else {
                main
            };
            // SAFETY: an object's first word, its header checked.
            let o = unsafe { laid::at(std::ptr::with_exposed_provenance::<u64>(first)) };
            (!o.pair && o.suffix == at).then_some(s)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::{Int, List, heap_cons, heap_new, i64_to_int, int_to_i64, nil};

    fn list(n: i64) -> List<Int> {
        (0..n)
            .rev()
            .fold(nil(), |acc, i| heap_cons(i64_to_int(i * 1000), acc))
    }

    fn sum(xs: List<Int>) -> i64 {
        let mut s = 0;
        let mut at = xs;
        while let Some(c) = at.pair() {
            s += int_to_i64(c.car());
            at = c.cdr;
        }
        s
    }

    #[test]
    fn what_the_stack_holds_kept_the_rest_freed() {
        // Garbage first, then what is kept: a list held by the stack, its
        // cells' cdrs precise words of the heap.
        let _ = std::hint::black_box(list(5000));
        let xs = std::hint::black_box(list(3000));
        let before = super::collections();
        super::collect();
        assert!(super::collections() > before);
        let _ = std::hint::black_box(list(5000));
        super::collect();
        let l = super::last();
        assert!(
            crate::obj::space::stressed() || (l.moved > 0 && l.freed_pages > 0),
            "{l:?}"
        );
        assert_eq!(sum(std::hint::black_box(xs)), 1000 * 2999 * 3000 / 2);
    }

    #[test]
    fn a_list_only_a_box_holds_moved_and_followed() {
        // The box is held by the stack (pinned); its list, by its field
        // alone, a precise word: copied, and the field made to follow.
        let ys = list(4000);
        let r = heap_new(ys);
        let r = std::hint::black_box(r);
        super::collect();
        let l = super::last();
        // Under stress, moved already.
        assert!(
            crate::obj::space::stressed() || (l.moved >= 1000 && l.freed_pages >= 1),
            "{l:?}"
        );
        assert_eq!(sum(r.get()), 1000 * 3999 * 4000 / 2);
    }
}
