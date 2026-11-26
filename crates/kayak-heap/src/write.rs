//! Values' text as Fixpt writes it, once (C543; the plan's step 4): a
//! character, a string, a double. kcrt-core's printer (`written`) and
//! kayak-rt's (`Show`) both write through these; kayak-rt's had lost a
//! string's `\r` and a control character's hex form.

use std::fmt::Write as _;

// Ported from Fixpt (scripts/fixpt-drift.py): `put_char`, `put_string`.
// fixpt-source: crates/fixpt-runtime/src/print.rs@c5b5991

/// A character as Fixpt writes it (`print.rs`'s `put_char`).
#[inline]
pub fn put_char(out: &mut String, c: char) {
    out.push_str("#\\");
    match c {
        ' ' => out.push_str("space"),
        '\n' => out.push_str("newline"),
        '\t' => out.push_str("tab"),
        '\r' => out.push_str("return"),
        '\0' => out.push_str("null"),
        '\u{7}' => out.push_str("alarm"),
        '\u{8}' => out.push_str("backspace"),
        '\u{7f}' => out.push_str("delete"),
        '\u{1b}' => out.push_str("escape"),
        c if (c as u32) < 0x20 => {
            let _ = write!(out, "x{:x}", c as u32);
        }
        c => out.push(c),
    }
}

/// A string as Fixpt writes one (`print.rs`'s `put_string`).
#[inline]
pub fn put_string(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\x{:x};", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// A double as Fixpt writes a value: an integral one below 1e21 with one
/// decimal, any other as Rust writes it.
#[inline]
pub fn write_f64(x: f64) -> String {
    if x.is_nan() {
        "+nan.0".into()
    } else if x.is_infinite() {
        if x > 0.0 { "+inf.0" } else { "-inf.0" }.into()
    } else if x == x.trunc() && x.abs() < 1e21 {
        format!("{x:.1}")
    } else {
        format!("{x}")
    }
}

/// `f64->string`: as a value is written, and with `.0` if it would have no
/// point, so that it reads back as a double (Fixpt's `format_flonum`).
#[inline]
pub fn f64_text(x: f64) -> String {
    let s = write_f64(x);
    if x.is_finite() && !s.contains(['.', 'e', 'E']) {
        format!("{s}.0")
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_fixpt_writes_them() {
        let mut s = String::new();
        put_char(&mut s, '\u{1f}');
        put_char(&mut s, ' ');
        put_string(&mut s, "a\r\u{2}\"");
        assert_eq!(s, "#\\x1f#\\space\"a\\r\\x2;\\\"\"");
        assert_eq!(f64_text(1e21), "1000000000000000000000.0");
        assert_eq!(f64_text(2.0), "2.0");
        assert_eq!(f64_text(0.1), "0.1");
        assert_eq!(write_f64(f64::NEG_INFINITY), "-inf.0");
    }
}

#[cfg(test)]
mod flonums {
    use super::f64_text as flonum_text;

    /// `f64->string` (C470) as Fixpt's `format_flonum` writes each (each
    /// string the pinned Fixpt's `f64->string`): kc's programs make no
    /// `f64` yet, the expander's data may hold one.
    #[test]
    fn written_as_fixpt_writes_them() {
        for (x, s) in [
            (1.0, "1.0"),
            (-0.0, "-0.0"),
            (0.1, "0.1"),
            (2.5, "2.5"),
            (1e20, "100000000000000000000.0"),
            (1e21, "1000000000000000000000.0"),
            (
                1.5e300,
                "1500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000.0",
            ),
            (1e-7, "0.0000001"),
            (f64::NAN, "+nan.0"),
            (f64::INFINITY, "+inf.0"),
            (f64::NEG_INFINITY, "-inf.0"),
        ] {
            assert_eq!(flonum_text(x), s, "{x:?}");
        }
    }
}
