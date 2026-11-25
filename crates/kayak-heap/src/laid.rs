//! An object as the collector reads it (C540; kayak-rt's `obj::laid`,
//! whose fields are its `HeapWord`s): `object_at`'s walk, its fields a
//! slice of plain words of the caller's provenance.

use crate::object_at;

/// An object at its first word: its identity (its main header's address,
/// or a pair's first word), the words that may hold values, and its end.
pub struct Laid {
    pub pair: bool,
    /// What a collector forwards.
    pub id: usize,
    pub fields: &'static [u64],
    /// Where its suffix starts, where an object's word points (0 for a
    /// pair).
    pub suffix: usize,
    pub end: usize,
}

/// The object whose first word is `at`.
///
/// # Safety
/// `at` is the first word of an object of the heap, kept while the
/// result is read.
#[inline]
pub unsafe fn at(at: *const u64) -> Laid {
    // SAFETY: the caller's: its words are the object's, as its header says.
    let o = object_at(at.addr(), |a| unsafe { *at.byte_add(a - at.addr()) });
    // SAFETY: the object's words from its main header's next (a pair's two
    // from its first), `traced` of them, its own.
    let fields = unsafe {
        let first = if o.pair {
            at
        } else {
            at.byte_add(o.main - at.addr()).add(1)
        };
        std::slice::from_raw_parts(first, o.traced)
    };
    Laid {
        pair: o.pair,
        id: o.main,
        fields,
        suffix: o.suffix,
        end: o.end,
    }
}
