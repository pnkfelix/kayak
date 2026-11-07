//! kayak's allocator (C370), used directly: blocks of every class and
//! beyond, aligned as asked, used again once freed, freed on another thread
//! than made them, and kept apart while live.
#![allow(unsafe_code)]

use kayak_sys::Alloc;
use std::alloc::{GlobalAlloc, Layout};

#[test]
fn blocks_of_each_size() {
    let a = Alloc;
    let mut live = Vec::new();
    for size in [1, 8, 15, 16, 17, 100, 1000, 1024, 1025, 4096, 1 << 20] {
        for align in [1, 8, 16, 64] {
            let l = Layout::from_size_align(size, align).unwrap();
            // SAFETY: a layout of non-zero size.
            let p = unsafe { a.alloc(l) };
            assert!(
                !p.is_null() && (p as usize).is_multiple_of(align),
                "{size} {align}"
            );
            // SAFETY: `size` bytes of the block just made.
            unsafe { std::ptr::write_bytes(p, (size % 251) as u8, size) };
            live.push((p, l));
        }
    }
    for &(p, l) in &live {
        // SAFETY: each block still holds what was written.
        let first = unsafe { *p };
        assert_eq!(first, (l.size() % 251) as u8, "a block overlapped another");
    }
    for (p, l) in live {
        // SAFETY: made by `a` with `l`.
        unsafe { a.dealloc(p, l) };
    }
}

#[test]
fn freed_blocks_used_again() {
    let a = Alloc;
    let l = Layout::from_size_align(48, 8).unwrap();
    // SAFETY: as above.
    let p = unsafe { a.alloc(l) };
    unsafe { a.dealloc(p, l) };
    let q = unsafe { a.alloc(l) };
    assert_eq!(p, q, "the block freed last is the next given");
    unsafe { a.dealloc(q, l) };
}

#[test]
fn freed_on_another_thread() {
    let l = Layout::from_size_align(64, 16).unwrap();
    // SAFETY: as above.
    let made: Vec<usize> = (0..1000)
        .map(|_| unsafe { Alloc.alloc(l) } as usize)
        .collect();
    std::thread::spawn(move || {
        for p in made {
            unsafe { Alloc.dealloc(p as *mut u8, l) };
        }
    })
    .join()
    .unwrap();
}
