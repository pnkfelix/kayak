//! Values written as FX-26 writes them, `fixpt eval`'s way: so that a
//! program's top-level expressions, run as Rust, can be compared with
//! Fixpt's answers (`scripts/run-fixpt-programs.py`).

use crate::{Int, Str, int_to_string, leak_string};
use std::cell::Cell;

pub trait Show {
    fn show(&self, out: &mut String);
}

impl Show for i64 {
    fn show(&self, out: &mut String) {
        out.push_str(&self.to_string());
    }
}

impl Show for i32 {
    fn show(&self, out: &mut String) {
        out.push_str(&self.to_string());
    }
}

impl Show for u8 {
    fn show(&self, out: &mut String) {
        out.push_str(&self.to_string());
    }
}

impl Show for i8 {
    fn show(&self, out: &mut String) {
        out.push_str(&self.to_string());
    }
}

impl Show for u32 {
    fn show(&self, out: &mut String) {
        out.push_str(&self.to_string());
    }
}

impl Show for u64 {
    fn show(&self, out: &mut String) {
        out.push_str(&self.to_string());
    }
}

impl Show for f32 {
    fn show(&self, out: &mut String) {
        out.push_str(&crate::f32_to_string(*self).text());
    }
}

impl Show for f64 {
    fn show(&self, out: &mut String) {
        // As Fixpt shows a double: with a point, at 1e21 and above too.
        out.push_str(&crate::f64_to_string(*self).text());
    }
}

impl Show for Int {
    fn show(&self, out: &mut String) {
        out.push_str(&int_to_string(*self).text());
    }
}

impl Show for bool {
    fn show(&self, out: &mut String) {
        out.push_str(if *self { "#t" } else { "#f" });
    }
}

impl Show for () {
    fn show(&self, out: &mut String) {
        out.push_str("#u");
    }
}

impl Show for char {
    fn show(&self, out: &mut String) {
        // Fixpt's `put_char` (C543: kcrt's too).
        kayak_heap::write::put_char(out, *self);
    }
}

impl Show for Str {
    fn show(&self, out: &mut String) {
        // Fixpt's `put_string` (C543), which writes `\r` and a control
        // character's hex form too.
        kayak_heap::write::put_string(out, &self.text());
    }
}

/// UTF-8 text, as Fixpt writes it: `#utf8"…"`, its text a string's.
impl Show for crate::Utf8 {
    fn show(&self, out: &mut String) {
        out.push_str("#utf8");
        crate::str_of_parts(&[self.as_str()]).show(out);
    }
}

/// A list, `(1 2 3)`; `nil`, `()`.
impl<T: Show + crate::Trace> Show for crate::ListAt<'_, T> {
    fn show(&self, out: &mut String) {
        out.push('(');
        let mut xs = *self;
        let mut first = true;
        while let Some(p) = xs.pair() {
            if !first {
                out.push(' ');
            }
            first = false;
            p.car().show(out);
            xs = p.cdr;
        }
        out.push(')');
    }
}

impl<T: Show + ?Sized> Show for &T {
    fn show(&self, out: &mut String) {
        (**self).show(out);
    }
}

/// A handle's value, as a reference's (C478).
impl<T: Show + ?Sized> Show for crate::Handle<'_, T> {
    fn show(&self, out: &mut String) {
        (**self).show(out);
    }
}

/// A datatype's value (C500): its value taken apart, as its enum's
/// `Show` writes it (kfx's).
impl<D: Show + crate::data::Datatype> Show for crate::data::DataAt<'_, D> {
    fn show(&self, out: &mut String) {
        self.get().show(out);
    }
}

/// A reference, as Fixpt writes one: `#<box 5>`.
impl<T: Show + crate::Trace> Show for crate::RefAt<'_, T> {
    fn show(&self, out: &mut String) {
        out.push_str("#<box ");
        self.get().show(out);
        out.push('>');
    }
}

/// A cell of a typed suffix (a flat's, C32), as a reference is written.
impl<T: Show + Copy> Show for Cell<T> {
    fn show(&self, out: &mut String) {
        out.push_str("#<box ");
        self.get().show(out);
        out.push('>');
    }
}

/// A top-level expression's value, on a line of its own.
pub fn show_line<T: Show>(out: &mut String, v: T) {
    v.show(out);
    out.push('\n');
}

pub fn shown(out: String) -> Str {
    leak_string(out)
}

/// No `Void` is ever shown, there being none.
impl Show for crate::Void {
    fn show(&self, _: &mut String) {
        match *self {}
    }
}

/// A procedure shown at the top level, whatever made it: Fixpt writes how
/// its machine made one, which a comparison takes as any procedure.
pub struct Procedure;

impl Show for Procedure {
    fn show(&self, out: &mut String) {
        out.push_str("#<procedure>");
    }
}

/// A flat array, as Fixpt writes its own: its elements, and their bytes.
impl<T: Copy> Show for [Cell<T>] {
    fn show(&self, out: &mut String) {
        let n = self.len();
        out.push_str(&format!(
            "#<flat-array {n} fields {} bytes>",
            n * std::mem::size_of::<T>()
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::Show;

    /// A string's `\r` and control characters, and a control character,
    /// as Fixpt writes them (C543: kayak-rt had written them raw).
    #[test]
    fn written_as_fixpt_writes_them() {
        let mut out = String::new();
        crate::leak_string("a\r\u{1}\"b".to_string()).show(&mut out);
        '\u{1f}'.show(&mut out);
        assert_eq!(out, "\"a\\r\\x1;\\\"b\"#\\x1f");
    }
}
