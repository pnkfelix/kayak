//! What a task may do, as far as the scheduler needs to know: which regions
//! it reads, writes, and transfers control through.
//!
//! FX-26's effects are lowered into these (`coopt_fx26::descriptor`; R7
//! §2.2): regions by number, allocation dropped, control as control, and an
//! atom on a region not known until run time as `read_any` or `write_any`.
//! It keeps the one property the scheduler relies on (`PLAN.md` §1, I3):
//! two effects either may interleave between their tasks' suspension
//! points, or they interfere and may not.
// coopt-source: crates/coopt-sched/src/effect.rs@9283bba

use std::collections::BTreeSet;

/// A region, by number. FX-26's regions will be numbered into these.
pub type Region = u32;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Effect {
    reads: BTreeSet<Region>,
    writes: BTreeSet<Region>,
    control: BTreeSet<Region>,
    /// Reads, or writes, regions not known until run time: every region.
    read_all: bool,
    write_all: bool,
    /// Unknown: assumed to touch every region. What a task gets if it does
    /// not say, so that saying nothing is safe.
    any: bool,
}

impl Effect {
    /// Touches nothing a program can observe. The collector's slices, the
    /// compiler's queries and the runners' deliveries say this: they change
    /// only state no program reads.
    pub fn pure() -> Effect {
        Effect::default()
    }

    /// Unknown, so interfering with everything but `pure`.
    pub fn any() -> Effect {
        Effect {
            any: true,
            ..Effect::default()
        }
    }

    pub fn read(mut self, r: Region) -> Effect {
        self.reads.insert(r);
        self
    }

    pub fn write(mut self, r: Region) -> Effect {
        self.writes.insert(r);
        self
    }

    /// Reads a region not known until run time: any.
    pub fn read_any(mut self) -> Effect {
        self.read_all = true;
        self
    }

    /// Writes, or transfers control through, a region not known until run
    /// time: any.
    pub fn write_any(mut self) -> Effect {
        self.write_all = true;
        self
    }

    /// Both effects at once.
    pub fn union(mut self, other: &Effect) -> Effect {
        self.reads.extend(&other.reads);
        self.writes.extend(&other.writes);
        self.control.extend(&other.control);
        self.read_all |= other.read_all;
        self.write_all |= other.write_all;
        self.any |= other.any;
        self
    }

    /// Whether everything this may do, `other` may do too: what narrowing
    /// a task's effect requires.
    pub fn within(&self, other: &Effect) -> bool {
        if other.any {
            return true;
        }
        if self.any {
            return false;
        }
        let writes_ok = other.write_all
            || (!self.write_all
                && self.writes.is_subset(&other.writes)
                && self.control.is_subset(&other.control));
        let reads_ok = other.read_all
            || other.write_all
            || (!self.read_all
                && self
                    .reads
                    .iter()
                    .all(|r| other.reads.contains(r) || other.writes.contains(r)));
        writes_ok && reads_ok
    }

    /// `goto` or `comefrom` on `r`: control leaves or re-enters through it.
    pub fn control(mut self, r: Region) -> Effect {
        self.control.insert(r);
        self
    }

    pub fn is_pure(&self) -> bool {
        !self.any
            && !self.read_all
            && !self.write_all
            && self.reads.is_empty()
            && self.writes.is_empty()
            && self.control.is_empty()
    }

    /// Whether it writes, or transfers control through, anything.
    fn writes_some(&self) -> bool {
        self.write_all || !self.writes.is_empty() || !self.control.is_empty()
    }

    fn touches(&self, r: &Region) -> bool {
        self.read_all
            || self.write_all
            || self.reads.contains(r)
            || self.writes.contains(r)
            || self.control.contains(r)
    }

    /// Whether a task with this effect and one with `other` could see each
    /// other's actions if they interleaved. Reads never interfere with
    /// reads; a write, or control, interferes with anything on its region.
    /// The relation is symmetric.
    pub fn interferes(&self, other: &Effect) -> bool {
        if self.is_pure() || other.is_pure() {
            return false;
        }
        if self.any || other.any {
            return true;
        }
        // `a` writes what `b` touches: a region both name, or, for a write
        // of any region, anything `b` does; for a read of any, any write.
        let one_way = |a: &Effect, b: &Effect| {
            (a.write_all && !b.is_pure())
                || (b.read_all && a.writes_some())
                || a.writes.iter().chain(&a.control).any(|r| b.touches(r))
        };
        one_way(self, other) || one_way(other, self)
    }
}

#[cfg(test)]
mod tests {
    use super::Effect;

    #[test]
    fn reads_and_writes_of_any_region() {
        let (r1, w1) = (Effect::pure().read(1), Effect::pure().write(1));
        let (ra, wa) = (Effect::pure().read_any(), Effect::pure().write_any());
        // A read of any region meets a write of any one, not a read.
        assert!(ra.interferes(&w1) && w1.interferes(&ra));
        assert!(!ra.interferes(&r1));
        assert!(!ra.interferes(&ra));
        // A write of any region meets everything but pure.
        assert!(wa.interferes(&r1) && wa.interferes(&ra) && wa.interferes(&wa));
        assert!(!wa.interferes(&Effect::pure()));
        // Control counts as a write.
        assert!(ra.interferes(&Effect::pure().control(3)));
        // Union.
        let u = r1.clone().union(&Effect::pure().write(2));
        assert!(u.interferes(&Effect::pure().read(2)));
        assert!(!u.interferes(&Effect::pure().read(1)));
    }
}

#[cfg(test)]
mod within {
    use super::Effect;

    #[test]
    fn narrowing_only_shrinks() {
        let rw = Effect::pure().read(1).write(2);
        assert!(Effect::pure().read(1).within(&rw));
        assert!(Effect::pure().read(2).within(&rw), "a write covers a read");
        assert!(!Effect::pure().write(1).within(&rw));
        assert!(rw.within(&Effect::any()));
        assert!(!Effect::any().within(&rw));
        assert!(Effect::pure().write(9).within(&Effect::pure().write_any()));
        assert!(!Effect::pure().read_any().within(&rw));
    }
}
