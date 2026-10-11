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
