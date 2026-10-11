//! `int`: FX-26's integer, which has no bound (`CLAUDES_CHOICES.md`, C15).
//!
//! A value that fits an `i64` is one; one that does not is a sign and a
//! magnitude, base 2^32, least significant first, with no zero at the top,
//! leaked as everything is. Every result that fits an `i64` is made one, so
//! that each value has one representation and the common case stays cheap.
//!
//! The operations mean what FX-26's mean: `quotient` truncates toward zero,
//! `remainder` has the dividend's sign and `modulo` the divisor's; division
//! by zero is an error; `int->i64` keeps the low 64 bits, two's complement.

use crate::{Str, leak_string};
use std::cmp::Ordering;

#[derive(Clone, Copy)]
pub enum Int {
    Small(i64),
    Big(&'static Big),
}

pub struct Big {
    negative: bool,
    magnitude: Vec<u32>,
}

/// A literal: KFX's literals are those of an `i64`.
pub fn int_lit(n: i64) -> Int {
    Int::Small(n)
}

/// A signed magnitude, the form arithmetic past an `i64` is done in.
struct Wide {
    negative: bool,
    magnitude: Vec<u32>,
}

fn wide(n: Int) -> Wide {
    match n {
        Int::Small(v) => {
            let m = v.unsigned_abs();
            let mut magnitude = vec![m as u32, (m >> 32) as u32];
            trim(&mut magnitude);
            Wide {
                negative: v < 0,
                magnitude,
            }
        }
        Int::Big(b) => Wide {
            negative: b.negative,
            magnitude: b.magnitude.clone(),
        },
    }
}

fn trim(m: &mut Vec<u32>) {
    while m.last() == Some(&0) {
        m.pop();
    }
}

/// A signed magnitude as an `Int`: small if it fits an `i64`.
fn int(w: Wide) -> Int {
    let mut magnitude = w.magnitude;
    trim(&mut magnitude);
    if magnitude.len() <= 2 {
        let m = magnitude.first().copied().unwrap_or(0) as u64
            | (magnitude.get(1).copied().unwrap_or(0) as u64) << 32;
        if !w.negative && m <= i64::MAX as u64 {
            return Int::Small(m as i64);
        }
        if w.negative && m <= 1u64 << 63 {
            return Int::Small((m as i64).wrapping_neg());
        }
    }
    let negative = w.negative && !magnitude.is_empty();
    Int::Big(Box::leak(Box::new(Big {
        negative,
        magnitude,
    })))
}

fn compare_magnitudes(a: &[u32], b: &[u32]) -> Ordering {
    a.len()
        .cmp(&b.len())
        .then_with(|| a.iter().rev().cmp(b.iter().rev()))
}

fn add_magnitudes(a: &[u32], b: &[u32]) -> Vec<u32> {
    let mut out = Vec::with_capacity(a.len().max(b.len()) + 1);
    let mut carry = 0u64;
    for i in 0..a.len().max(b.len()) {
        let s =
            a.get(i).copied().unwrap_or(0) as u64 + b.get(i).copied().unwrap_or(0) as u64 + carry;
        out.push(s as u32);
        carry = s >> 32;
    }
    if carry != 0 {
        out.push(carry as u32);
    }
    out
}

/// `a - b`, for `a` at least `b`.
fn sub_magnitudes(a: &[u32], b: &[u32]) -> Vec<u32> {
    let mut out = Vec::with_capacity(a.len());
    let mut borrow = 0i64;
    for (i, &x) in a.iter().enumerate() {
        let mut d = x as i64 - b.get(i).copied().unwrap_or(0) as i64 - borrow;
        borrow = 0;
        if d < 0 {
            d += 1 << 32;
            borrow = 1;
        }
        out.push(d as u32);
    }
    trim(&mut out);
    out
}

fn mul_magnitudes(a: &[u32], b: &[u32]) -> Vec<u32> {
    let mut out = vec![0u32; a.len() + b.len()];
    for (i, &x) in a.iter().enumerate() {
        let mut carry = 0u64;
        for (j, &y) in b.iter().enumerate() {
            let t = out[i + j] as u64 + x as u64 * y as u64 + carry;
            out[i + j] = t as u32;
            carry = t >> 32;
        }
        let mut k = i + b.len();
        while carry != 0 {
            let t = out[k] as u64 + carry;
            out[k] = t as u32;
            carry = t >> 32;
            k += 1;
        }
    }
    trim(&mut out);
    out
}

/// Quotient and remainder of magnitudes, `b` not zero: long division, a
/// bit at a time, which is slow for big divisors and simple to trust.
fn divide_magnitudes(a: &[u32], b: &[u32]) -> (Vec<u32>, Vec<u32>) {
    if b.len() == 1 {
        let d = b[0] as u64;
        let mut q = vec![0u32; a.len()];
        let mut r = 0u64;
        for i in (0..a.len()).rev() {
            let cur = (r << 32) | a[i] as u64;
            q[i] = (cur / d) as u32;
            r = cur % d;
        }
        trim(&mut q);
        let mut rem = vec![r as u32];
        trim(&mut rem);
        return (q, rem);
    }
    let mut q = vec![0u32; a.len()];
    let mut r: Vec<u32> = Vec::new();
    for i in (0..a.len() * 32).rev() {
        // r = r * 2 + bit i of a
        let bit = (a[i / 32] >> (i % 32)) & 1;
        let mut carry = bit;
        for limb in r.iter_mut() {
            let next = *limb >> 31;
            *limb = (*limb << 1) | carry;
            carry = next;
        }
        if carry != 0 {
            r.push(carry);
        }
        if compare_magnitudes(&r, b) != Ordering::Less {
            r = sub_magnitudes(&r, b);
            q[i / 32] |= 1 << (i % 32);
        }
    }
    trim(&mut q);
    trim(&mut r);
    (q, r)
}

fn add(a: Wide, b: Wide) -> Int {
    if a.negative == b.negative {
        return int(Wide {
            negative: a.negative,
            magnitude: add_magnitudes(&a.magnitude, &b.magnitude),
        });
    }
    match compare_magnitudes(&a.magnitude, &b.magnitude) {
        Ordering::Less => int(Wide {
            negative: b.negative,
            magnitude: sub_magnitudes(&b.magnitude, &a.magnitude),
        }),
        _ => int(Wide {
            negative: a.negative,
            magnitude: sub_magnitudes(&a.magnitude, &b.magnitude),
        }),
    }
}

fn negate(w: Wide) -> Wide {
    Wide {
        negative: !w.negative,
        magnitude: w.magnitude,
    }
}

pub fn _plus(a: Int, b: Int) -> Int {
    if let (Int::Small(x), Int::Small(y)) = (a, b)
        && let Some(s) = x.checked_add(y)
    {
        return Int::Small(s);
    }
    add(wide(a), wide(b))
}

pub fn _minus(a: Int, b: Int) -> Int {
    if let (Int::Small(x), Int::Small(y)) = (a, b)
        && let Some(s) = x.checked_sub(y)
    {
        return Int::Small(s);
    }
    add(wide(a), negate(wide(b)))
}

pub fn _star(a: Int, b: Int) -> Int {
    if let (Int::Small(x), Int::Small(y)) = (a, b)
        && let Some(s) = x.checked_mul(y)
    {
        return Int::Small(s);
    }
    let (a, b) = (wide(a), wide(b));
    int(Wide {
        negative: a.negative != b.negative,
        magnitude: mul_magnitudes(&a.magnitude, &b.magnitude),
    })
}

/// Quotient, truncated toward zero, and remainder, of the dividend's sign.
fn divide(a: Int, b: Int, name: &str) -> (Int, Int) {
    if let (Int::Small(x), Int::Small(y)) = (a, b) {
        if y == 0 {
            crate::error(leak_string(format!("{name}: division by zero")));
        }
        if let (Some(q), Some(r)) = (x.checked_div(y), x.checked_rem(y)) {
            return (Int::Small(q), Int::Small(r));
        }
    }
    let (a, b) = (wide(a), wide(b));
    if b.magnitude.is_empty() {
        crate::error(leak_string(format!("{name}: division by zero")));
    }
    let (q, r) = divide_magnitudes(&a.magnitude, &b.magnitude);
    (
        int(Wide {
            negative: a.negative != b.negative,
            magnitude: q,
        }),
        int(Wide {
            negative: a.negative,
            magnitude: r,
        }),
    )
}

pub fn quotient(a: Int, b: Int) -> Int {
    divide(a, b, "quotient").0
}

pub fn remainder(a: Int, b: Int) -> Int {
    divide(a, b, "remainder").1
}

/// The remainder of the divisor's sign.
pub fn modulo(a: Int, b: Int) -> Int {
    let r = remainder(a, b);
    if sign(r) != 0 && sign(r) != sign(b) {
        _plus(r, b)
    } else {
        r
    }
}

fn sign(n: Int) -> i32 {
    match n {
        Int::Small(v) => v.signum() as i32,
        Int::Big(b) => {
            if b.negative {
                -1
            } else {
                1
            }
        }
    }
}

fn compare(a: Int, b: Int) -> Ordering {
    if let (Int::Small(x), Int::Small(y)) = (a, b) {
        return x.cmp(&y);
    }
    let (a, b) = (wide(a), wide(b));
    match (a.negative, b.negative) {
        (false, true) => Ordering::Greater,
        (true, false) => Ordering::Less,
        (false, false) => compare_magnitudes(&a.magnitude, &b.magnitude),
        (true, true) => compare_magnitudes(&b.magnitude, &a.magnitude),
    }
}

pub fn _eq(a: Int, b: Int) -> bool {
    compare(a, b) == Ordering::Equal
}

pub fn _lt(a: Int, b: Int) -> bool {
    compare(a, b) == Ordering::Less
}

pub fn _gt(a: Int, b: Int) -> bool {
    compare(a, b) == Ordering::Greater
}

pub fn _lt_eq(a: Int, b: Int) -> bool {
    compare(a, b) != Ordering::Greater
}

pub fn _gt_eq(a: Int, b: Int) -> bool {
    compare(a, b) != Ordering::Less
}

pub fn int_to_string(n: Int) -> Str {
    match n {
        Int::Small(v) => leak_string(v.to_string()),
        Int::Big(b) => {
            // Nine decimal digits at a time, least significant first.
            let mut m = b.magnitude.clone();
            let mut chunks = Vec::new();
            while !m.is_empty() {
                let (q, r) = divide_magnitudes(&m, &[1_000_000_000]);
                chunks.push(r.first().copied().unwrap_or(0));
                m = q;
            }
            let mut s = String::new();
            if b.negative {
                s.push('-');
            }
            s.push_str(&chunks.last().copied().unwrap_or(0).to_string());
            for c in chunks.iter().rev().skip(1) {
                s.push_str(&format!("{c:09}"));
            }
            leak_string(s)
        }
    }
}

/// The low 64 bits, two's complement, as FX-26's `int->i64` wraps.
pub fn int_to_i64(n: Int) -> i64 {
    match n {
        Int::Small(v) => v,
        Int::Big(b) => {
            let low = b.magnitude.first().copied().unwrap_or(0) as u64
                | (b.magnitude.get(1).copied().unwrap_or(0) as u64) << 32;
            if b.negative {
                low.wrapping_neg() as i64
            } else {
                low as i64
            }
        }
    }
}

pub fn i64_to_int(n: i64) -> Int {
    Int::Small(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn show(n: Int) -> String {
        int_to_string(n).as_str().to_owned()
    }

    fn power(b: i64, e: u32) -> Int {
        (0..e).fold(int_lit(1), |acc, _| _star(acc, int_lit(b)))
    }

    #[test]
    fn grows_past_an_i64_and_back() {
        let big = _plus(int_lit(i64::MAX), int_lit(1));
        assert_eq!(show(big), "9223372036854775808");
        assert!(matches!(_minus(big, int_lit(1)), Int::Small(i64::MAX)));
        assert_eq!(
            show(_minus(int_lit(i64::MIN), int_lit(1))),
            "-9223372036854775809"
        );
        assert_eq!(show(power(2, 100)), "1267650600228229401496703205376");
        assert_eq!(show(power(-3, 41)), "-36472996377170786403");
    }

    #[test]
    fn divides_as_fx26_does() {
        let n = power(10, 30);
        let d = _plus(power(2, 70), int_lit(12345));
        let (q, r) = (quotient(n, d), remainder(n, d));
        assert!(_eq(_plus(_star(q, d), r), n));
        assert!(_lt(r, d));
        assert_eq!(show(quotient(int_lit(-7), int_lit(2))), "-3");
        assert_eq!(show(remainder(int_lit(-7), int_lit(2))), "-1");
        assert_eq!(show(modulo(int_lit(-7), int_lit(2))), "1");
        assert_eq!(show(modulo(int_lit(7), int_lit(-2))), "-1");
        let neg = _star(n, int_lit(-1));
        assert_eq!(
            show(remainder(neg, int_lit(7))),
            show(_star(remainder(n, int_lit(7)), int_lit(-1)))
        );
    }

    #[test]
    fn wraps_to_an_i64() {
        assert_eq!(int_to_i64(_plus(power(2, 64), int_lit(5))), 5);
        assert_eq!(int_to_i64(power(2, 63)), i64::MIN);
        assert_eq!(int_to_i64(_star(power(2, 64), int_lit(-1))), 0);
    }
}
