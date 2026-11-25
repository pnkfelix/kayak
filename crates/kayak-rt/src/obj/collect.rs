//! kayak-rt's collector, the shared heap's since C540
//! (`kayak_heap::collect`, which was this file): kayak-rt gives it its
//! roots, as it finds them (`obj::roots`: its stacks, scanned
//! conservatively; its kept blocks; its constants; its threads' lists),
//! and is told what it reached of others' and what it did not of its own.
//! The collector's tests, of kayak-rt's types, are kayak-rt's still.

pub(crate) use kayak_heap::collect::*;

/// kayak-rt as the collector's runtime.
pub(crate) struct Rt;

impl Host for Rt {
    fn spill(&self, saved: &mut [usize; 12]) {
        super::roots::spill(saved);
    }
    fn roots(&self, saved: &[usize; 12], from: usize) -> kayak_heap::roots::Gathered {
        super::roots::roots_from(saved, from)
    }
    fn reported(&self, report: Vec<usize>, from: usize, minor: bool) {
        super::roots::reported(report, from, minor);
    }
    fn retire(&self, reached: &dyn Fn(usize) -> bool) {
        super::roots::retire_kept(reached);
    }
    fn kept_extent(&self) -> (usize, usize) {
        super::roots::kept_extent()
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

    /// Garbage made between collections: less under stress, where each
    /// allocation may collect already.
    fn garbage() -> i64 {
        if crate::obj::space::stressed() {
            100
        } else {
            3000
        }
    }

    fn of_word(w: crate::obj::HeapWord) -> List<Int> {
        crate::obj::ListAt(w, std::marker::PhantomData)
    }

    /// `n` cells of this thread's before `tail`: 0, 1, …, n - 1.
    fn before(n: i64, tail: List<Int>) -> List<Int> {
        (0..n)
            .rev()
            .fold(tail, |acc, i| heap_cons(i64_to_int(i), acc))
    }

    #[test]
    fn a_par_branch_collects_beside_its_maker() {
        // The branch puts cells of its own before its maker's list, which
        // its closure holds, collecting as it goes; the maker collects all
        // the while. After the join, the branch's pages are the maker's.
        let xs = list(2000);
        let w = xs.0;
        let (mine, theirs) = crate::process::par2(
            || {
                for _ in 0..30 {
                    let _ = std::hint::black_box(list(garbage()));
                    super::collect();
                }
                sum(xs)
            },
            move || {
                let mut ys = of_word(w);
                for _ in 0..30 {
                    ys = before(100, ys);
                    let _ = std::hint::black_box(list(garbage()));
                    super::collect();
                }
                ys.0
            },
        );
        assert_eq!(mine, 1000 * 1999 * 2000 / 2);
        for _ in 0..3 {
            let _ = std::hint::black_box(list(garbage()));
            super::collect();
        }
        assert_eq!(
            sum(of_word(theirs)),
            30 * 99 * 100 / 2 + 1000 * 1999 * 2000 / 2
        );
    }

    #[test]
    fn a_minor_collection_reads_the_young_alone() {
        // An old list (a major collection made it so) is neither read nor
        // moved by a minor one: a major one right after reaches its 300
        // cells more than the minor did, and both keep both lists.
        let old = list(300);
        super::collect();
        let old_at = old.0.raw();
        let young = list(300);
        super::collect_minor();
        let minor = super::last().reached;
        assert_eq!(
            old.0.raw(),
            old_at,
            "an old object moved by a minor collection"
        );
        super::collect();
        let major = super::last().reached;
        assert!(
            minor + 300 <= major,
            "a minor collection reached {minor}, a major {major}"
        );
        assert_eq!(sum(old), 1000 * 299 * 300 / 2);
        assert_eq!(sum(young), 1000 * 299 * 300 / 2);
    }

    #[test]
    fn a_young_list_an_old_box_holds_kept_by_a_minor_collection() {
        // A box made old, then given a young list (the barrier notes its
        // slot): a minor collection keeps the list through the slot alone
        // and makes the slot point where the list went. Without the
        // barrier (`Slot::set`'s `remember`), the list's pages are freed.
        let r = heap_new(nil::<Int>());
        super::collect();
        crate::set(r, list(500));
        for _ in 0..3 {
            let _ = std::hint::black_box(list(garbage()));
            super::collect_minor();
        }
        assert_eq!(sum(crate::get(r)), 1000 * 499 * 500 / 2);
    }

    /// A box holding a kept cell (kayak-rt's own memory, whose stores no
    /// barrier sees), made old, the cell then reached by the box alone.
    #[inline(never)]
    fn an_old_box_of_a_kept_cell() -> crate::Ref<&'static std::cell::Cell<List<Int>>> {
        let r = heap_new(crate::leak_collected(std::cell::Cell::new(nil::<Int>())));
        super::collect();
        r
    }

    #[inline(never)]
    fn given_a_young_list(r: crate::Ref<&'static std::cell::Cell<List<Int>>>) {
        crate::get(r).set(list(500));
    }

    #[test]
    fn a_young_list_a_kept_cell_an_old_box_holds_kept_by_a_minor_collection() {
        // A minor collection follows no old object, so a kept object only
        // an old one reaches it takes as a root (C523): else the young
        // list the cell holds, which no barrier noted, freed.
        let r = an_old_box_of_a_kept_cell();
        given_a_young_list(r);
        for _ in 0..3 {
            let _ = std::hint::black_box(list(garbage()));
            super::collect_minor();
        }
        assert_eq!(sum(crate::get(r).get()), 1000 * 499 * 500 / 2);
    }

    fn text_of(n: usize) -> String {
        (0..n).map(|i| if i % 3 == 0 { 'é' } else { 'x' }).collect()
    }

    /// The strings of each length to 40 in a list, which a box alone holds,
    /// made after them, so that no word of the stack reaches the strings'
    /// pages.
    #[inline(never)]
    fn strings_boxed() -> crate::Ref<List<crate::Str>> {
        let xs: List<crate::Str> = (0..40).rev().fold(nil(), |acc, n| {
            heap_cons(crate::leak_string(text_of(n)), acc)
        });
        for _ in 0..4 {
            let _ = std::hint::black_box(list(garbage()));
        }
        heap_new(xs)
    }

    /// Strings of each length, moved by collections, as they were made
    /// (C533: a string's second word, its count, any bits at all, not read
    /// as a trailer).
    #[test]
    fn strings_of_each_length_moved() {
        let r = strings_boxed();
        for _ in 0..3 {
            let _ = std::hint::black_box(list(garbage()));
            super::collect();
        }
        let mut at = crate::get(r);
        for n in 0..40 {
            let c = at.pair().expect("40 strings");
            assert_eq!(c.car().text(), text_of(n));
            at = c.cdr;
        }
    }

    #[test]
    fn large_objects_not_reached_freed() {
        // A large string held by the stack, kept as large garbage is made
        // and freed around it.
        let big = "ab".repeat(3 * crate::obj::space::PAGE_BYTES);
        let held = std::hint::black_box(crate::str_of_parts(&[&big]));
        for i in 0..40 {
            let _ = std::hint::black_box(crate::str_of_parts(&[&big, &i.to_string()]));
            if i % 8 == 0 {
                super::collect();
            }
        }
        super::collect();
        // What is left: the string held, and perhaps one more a stale word
        // pins.
        assert!(
            // Its object's bytes: four a code point (C533).
            crate::obj::space::my_large_bytes() <= 2 * (8 * crate::string_words(big.len()) + 64),
            "{} bytes of large objects",
            crate::obj::space::my_large_bytes()
        );
        assert_eq!(std::hint::black_box(held).text(), big);
    }

    #[test]
    fn an_ended_threads_pages_adopted() {
        // A thread makes garbage and a list, which it leaves in an I-cell,
        // and ends: its pages are given to the next thread that collects,
        // which frees those holding nothing and keeps the list.
        let c = crate::process::make_icell::<List<Int>>();
        let (pages, number) = std::thread::spawn(move || {
            for _ in 0..20 {
                let _ = std::hint::black_box(list(garbage()));
            }
            crate::process::icell_put_x(c, list(500));
            (
                crate::obj::space::my_page_words(),
                crate::obj::space::my_number(),
            )
        })
        .join()
        .unwrap();
        assert!(!pages.is_empty());
        for _ in 0..3 {
            let _ = std::hint::black_box(list(garbage()));
            super::collect();
        }
        for p in pages {
            assert_ne!(crate::obj::space::page_owner(p), number);
        }
        assert_eq!(sum(crate::process::icell_get(c)), 1000 * 499 * 500 / 2);
    }

    #[inline(never)]
    fn marks_made(n: i64) -> i64 {
        let key = crate::marks::make_continuation_mark_key::<i64>();
        (0..n)
            .map(|i| crate::marks::with_mark(key, i, || crate::marks::first_mark(key, -1)))
            .sum()
    }

    /// `n` I-cells, each in a box, the boxes in a list: each I-cell held
    /// by a plain word of the heap alone (C508).
    #[inline(never)]
    fn icells_in_boxes(n: i64) -> List<crate::Ref<crate::ICell<i64>>> {
        (0..n).fold(nil(), |acc, i| {
            let c = crate::process::make_icell::<i64>();
            crate::process::icell_put_x(c, i);
            heap_cons(heap_new(c), acc)
        })
    }

    fn icells_sum(mut xs: List<crate::Ref<crate::ICell<i64>>>) -> i64 {
        let mut s = 0;
        while let Some(c) = xs.pair() {
            s += crate::process::icell_get(c.car().get());
            xs = c.cdr;
        }
        s
    }

    #[test]
    fn kept_objects_no_word_reaches_freed() {
        // Marks' values, each kept while its mark is pushed, then reached
        // by no word: freed. I-cells the heap alone holds: kept, as their
        // boxes move.
        let bs = std::hint::black_box(icells_in_boxes(50));
        let before = crate::obj::roots::my_kept();
        assert_eq!(marks_made(300), 299 * 300 / 2);
        for _ in 0..3 {
            let _ = std::hint::black_box(list(garbage()));
            super::collect();
        }
        let after = crate::obj::roots::my_kept();
        assert!(after < before + 10, "{after} kept, {before} before");
        assert!(after + 5 >= before, "{after} kept, {before} before");
        assert_eq!(icells_sum(std::hint::black_box(bs)), 49 * 50 / 2);
    }

    #[inline(never)]
    fn sent(p: crate::Addr<(List<Int>, crate::ICell<List<Int>>)>) -> crate::ICell<List<Int>> {
        let c = crate::process::make_icell();
        crate::process::send(p, (list(300), c));
        c
    }

    #[test]
    fn a_message_wrapped_and_sent_back_kept() {
        // A process puts cells of its own before each list it is sent, and
        // sends it back: the list then this thread's only below another's
        // cells, each side collecting as it goes.
        const ROUNDS: usize = 40;
        let p = crate::process::spawn(|b: crate::Inbox<(List<Int>, crate::ICell<List<Int>>)>| {
            for _ in 0..ROUNDS {
                let (w, c) = crate::process::receive(b);
                let ys = before(100, w);
                let _ = std::hint::black_box(list(garbage()));
                super::collect();
                crate::process::icell_put_x(c, ys);
            }
        });
        for _ in 0..ROUNDS {
            let c = sent(p);
            let _ = std::hint::black_box(list(garbage()));
            super::collect();
            let w = crate::process::icell_get(c);
            let _ = std::hint::black_box(list(garbage()));
            super::collect();
            assert_eq!(sum(w), 99 * 100 / 2 + 1000 * 299 * 300 / 2);
        }
    }
}
