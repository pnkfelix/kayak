//! `f64`: IEEE doubles, as FX-26's are, its operations Rust's own
//! (Fixpt's `fixpt-runtime` `prim.rs` and `num.rs` are the reference:
//! `f64-round` rounds ties to even; `f64->int` is exact, and an error for a
//! value not integral; written as Fixpt writes one).

use crate::{Int, Str, fail, i64_to_int, int_lit, int_to_string, leak_string};

#[inline]
pub fn f64_plus(a: f64, b: f64) -> f64 {
    a + b
}

#[inline]
pub fn f64_minus(a: f64, b: f64) -> f64 {
    a - b
}

#[inline]
pub fn f64_star(a: f64, b: f64) -> f64 {
    a * b
}

#[inline]
pub fn f64_slash(a: f64, b: f64) -> f64 {
    a / b
}

#[inline]
pub fn f64_min(a: f64, b: f64) -> f64 {
    a.min(b)
}

#[inline]
pub fn f64_max(a: f64, b: f64) -> f64 {
    a.max(b)
}

#[inline]
pub fn f64_lt(a: f64, b: f64) -> bool {
    a < b
}

#[inline]
pub fn f64_lt_eq(a: f64, b: f64) -> bool {
    a <= b
}

#[inline]
pub fn f64_gt(a: f64, b: f64) -> bool {
    a > b
}

#[inline]
pub fn f64_gt_eq(a: f64, b: f64) -> bool {
    a >= b
}

#[inline]
pub fn f64_eq(a: f64, b: f64) -> bool {
    a == b
}

#[inline]
pub fn f64_abs(x: f64) -> f64 {
    x.abs()
}

#[inline]
pub fn f64_neg(x: f64) -> f64 {
    -x
}

#[inline]
pub fn f64_sqrt(x: f64) -> f64 {
    x.sqrt()
}

#[inline]
pub fn f64_floor(x: f64) -> f64 {
    x.floor()
}

#[inline]
pub fn f64_ceiling(x: f64) -> f64 {
    x.ceil()
}

#[inline]
pub fn f64_truncate(x: f64) -> f64 {
    x.trunc()
}

#[inline]
pub fn f64_round(x: f64) -> f64 {
    x.round_ties_even()
}

#[inline]
pub fn f64_nan_p(x: f64) -> bool {
    x.is_nan()
}

#[inline]
pub fn f64_infinite_p(x: f64) -> bool {
    x.is_infinite()
}

#[inline]
pub fn f64_finite_p(x: f64) -> bool {
    x.is_finite()
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
pub fn f64_to_string(x: f64) -> Str {
    let s = write_f64(x);
    if x.is_finite() && !s.contains(['.', 'e', 'E']) {
        leak_string(format!("{s}.0"))
    } else {
        leak_string(s)
    }
}

/// Correctly rounded, through the decimal digits, as Rust reads them.
#[inline]
pub fn int_to_f64(n: Int) -> f64 {
    match n {
        Int::Small(v) => v as f64,
        Int::Big(_) => int_to_string(n).as_str().parse().unwrap_or(f64::NAN),
    }
}

/// Exact: an integral double, as an integer, a bignum past an `i64`.
pub fn f64_to_int(x: f64) -> Int {
    if !(x.is_finite() && x.fract() == 0.0) {
        fail("f64->int: not an integer");
    }
    if x.abs() < 9.2e18 {
        return i64_to_int(x as i64);
    }
    // Past an i64: the mantissa, times a power of two.
    let bits = x.abs().to_bits();
    let exponent = ((bits >> 52) & 0x7ff) as i64 - 1075;
    let mantissa = (bits & ((1 << 52) - 1)) | (1 << 52);
    let mut n = i64_to_int(mantissa as i64);
    for _ in 0..exponent {
        n = crate::_star(n, int_lit(2));
    }
    if x < 0.0 {
        crate::_minus(int_lit(0), n)
    } else {
        n
    }
}

// `f32`: IEEE singles, rounded at each operation, as the machine does;
// as `f64`'s, but `int->f32` rounds once, from the integer, not through a
// double.

macro_rules! f32_binary {
    ($($name:ident, $op:tt);*) => {
        $(#[inline]
        pub fn $name(a: f32, b: f32) -> f32 {
            a $op b
        })*
    };
}

f32_binary!(f32_plus, +; f32_minus, -; f32_star, *; f32_slash, /);

macro_rules! f32_compare {
    ($($name:ident, $op:tt);*) => {
        $(#[inline]
        pub fn $name(a: f32, b: f32) -> bool {
            a $op b
        })*
    };
}

f32_compare!(f32_lt, <; f32_lt_eq, <=; f32_gt, >; f32_gt_eq, >=; f32_eq, ==);

#[inline]
pub fn f32_min(a: f32, b: f32) -> f32 {
    a.min(b)
}

#[inline]
pub fn f32_max(a: f32, b: f32) -> f32 {
    a.max(b)
}

#[inline]
pub fn f32_abs(x: f32) -> f32 {
    x.abs()
}

#[inline]
pub fn f32_neg(x: f32) -> f32 {
    -x
}

#[inline]
pub fn f32_sqrt(x: f32) -> f32 {
    x.sqrt()
}

#[inline]
pub fn f32_floor(x: f32) -> f32 {
    x.floor()
}

#[inline]
pub fn f32_ceiling(x: f32) -> f32 {
    x.ceil()
}

#[inline]
pub fn f32_truncate(x: f32) -> f32 {
    x.trunc()
}

#[inline]
pub fn f32_round(x: f32) -> f32 {
    x.round_ties_even()
}

#[inline]
pub fn f32_nan_p(x: f32) -> bool {
    x.is_nan()
}

#[inline]
pub fn f32_infinite_p(x: f32) -> bool {
    x.is_infinite()
}

#[inline]
pub fn f32_finite_p(x: f32) -> bool {
    x.is_finite()
}

/// A single as Fixpt writes a value: as a double is, its digits the
/// fewest that read back as the single.
pub fn write_f32(x: f32) -> String {
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

pub fn f32_to_string(x: f32) -> Str {
    let s = write_f32(x);
    if x.is_finite() && !s.contains(['.', 'e', 'E']) {
        leak_string(format!("{s}.0"))
    } else {
        leak_string(s)
    }
}

/// Correctly rounded: an `i64` by Rust's cast, which rounds to nearest; a
/// bignum through its decimal digits, as Rust reads them.
#[inline]
pub fn int_to_f32(n: Int) -> f32 {
    match n {
        Int::Small(v) => v as f32,
        Int::Big(_) => int_to_string(n).as_str().parse().unwrap_or(f32::NAN),
    }
}

/// Exact, through a double, which holds every single.
pub fn f32_to_int(x: f32) -> Int {
    if !(x.is_finite() && x.fract() == 0.0) {
        fail("f32->int: not an integer");
    }
    f64_to_int(x as f64)
}

#[inline]
pub fn f32_to_f64(x: f32) -> f64 {
    x as f64
}

#[inline]
pub fn f64_to_f32(x: f64) -> f32 {
    x as f32
}

#[inline]
pub fn f32_to_bits(x: f32) -> u32 {
    x.to_bits()
}

#[inline]
pub fn bits_to_f32(b: u32) -> f32 {
    f32::from_bits(b)
}

#[inline]
pub fn f64_to_bits(x: f64) -> u64 {
    x.to_bits()
}

#[inline]
pub fn bits_to_f64(b: u64) -> f64 {
    f64::from_bits(b)
}
