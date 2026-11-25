//! A collection's roots, as each runtime finds them (C537; the plan's
//! step 3): the one thing about a collection the two runtimes must do
//! differently, so a trait, where their objects' format is one (the
//! plan's principle). kayak-rt's are conservative: the Rust stacks'
//! words, each perhaps a pointer, pin what they reach, and its kept
//! blocks' slots are exact. kc's are exact: every word of kc's stack is a
//! value of Fixpt's format (or a return address, outside the heap), and
//! so is every global's cell.

/// Where a collection's roots are.
pub trait Roots {
    /// Each exact root: the address of a slot holding a value, which the
    /// collection reads and rewrites where what it reaches moved.
    fn each_exact(&mut self, f: &mut dyn FnMut(usize));

    /// Each ambiguous root: a word that may be a pointer, whose object, if
    /// it is one, is kept and not moved. None, by default.
    fn each_ambiguous(&mut self, f: &mut dyn FnMut(u64)) {
        let _ = f;
    }
}

/// The slots of runs of words, each `[lo, hi)`, all exact: a native
/// stack's frames (from its pointer to its base), a run of globals' cells.
pub struct Runs<'a>(pub &'a [(usize, usize)]);

impl Roots for Runs<'_> {
    fn each_exact(&mut self, f: &mut dyn FnMut(usize)) {
        for &(lo, hi) in self.0 {
            let mut a = lo;
            while a < hi {
                f(a);
                a += 8;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_each_slot_once() {
        let mut seen = Vec::new();
        Runs(&[(64, 88), (200, 208), (300, 300)]).each_exact(&mut |a| seen.push(a));
        assert_eq!(seen, vec![64, 72, 80, 200]);
        let mut none = 0;
        Runs(&[(0, 16)]).each_ambiguous(&mut |_| none += 1);
        assert_eq!(none, 0);
    }
}
