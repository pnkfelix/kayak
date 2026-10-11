//! Values written as FX-26 writes them, `fixpt eval`'s way: so that a
//! program's top-level expressions, run as Rust, can be compared with
//! Fixpt's answers (`scripts/run-fixpt-programs.py`).

use crate::{Cons, Int, Str, int_to_string, leak_string};
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

impl Show for f64 {
    fn show(&self, out: &mut String) {
        // As Fixpt shows a double: with a point, at 1e21 and above too.
        out.push_str(crate::f64_to_string(*self).as_str());
    }
}

impl Show for Int {
    fn show(&self, out: &mut String) {
        out.push_str(int_to_string(*self).as_str());
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
        out.push_str("#\\");
        match *self {
            ' ' => out.push_str("space"),
            '\n' => out.push_str("newline"),
            '\t' => out.push_str("tab"),
            '\r' => out.push_str("return"),
            '\0' => out.push_str("null"),
            '\u{7}' => out.push_str("alarm"),
            '\u{8}' => out.push_str("backspace"),
            '\u{1b}' => out.push_str("escape"),
            '\u{7f}' => out.push_str("delete"),
            c => out.push(c),
        }
    }
}

impl Show for Str {
    fn show(&self, out: &mut String) {
        out.push('"');
        for c in self.as_str().chars() {
            match c {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\t' => out.push_str("\\t"),
                c => out.push(c),
            }
        }
        out.push('"');
    }
}

/// A list, `(1 2 3)`; `nil`, `()`.
impl<T: Show> Show for Option<&Cons<'_, T>> {
    fn show(&self, out: &mut String) {
        out.push('(');
        let mut xs = *self;
        let mut first = true;
        while let Some(p) = xs {
            if !first {
                out.push(' ');
            }
            first = false;
            p.car.show(out);
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

/// A reference, as Fixpt writes one: `#<box 5>`.
impl<T: Show + Copy> Show for Cell<T> {
    fn show(&self, out: &mut String) {
        out.push_str("#<box ");
        self.get().show(out);
        out.push('>');
    }
}

/// A procedure: Fixpt writes how its machine made it, which a comparison
/// takes as any procedure.
macro_rules! show_procedure {
    ($($a:ident),*) => {
        impl<$($a,)* R> Show for dyn Fn($($a),*) -> R {
            fn show(&self, out: &mut String) {
                out.push_str("#<procedure>");
            }
        }
    };
}
show_procedure!();
show_procedure!(A);
show_procedure!(A, B);
show_procedure!(A, B, C);
show_procedure!(A, B, C, D);

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
