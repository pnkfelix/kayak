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
#[inline]
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
    crate::stats::note::<Big>(std::mem::size_of::<Big>() + 8 * magnitude.len());
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
    // A power of two (the engine's limbs are taken by 2^32): a shift and
    // the bits below it.
    if b[..b.len() - 1].iter().all(|x| *x == 0) && b[b.len() - 1].is_power_of_two() {
        let k = (b.len() - 1) * 32 + b[b.len() - 1].trailing_zeros() as usize;
        let mut q = shifted_right(a, k);
        trim(&mut q);
        let mut r: Vec<u32> = a.iter().take(k.div_ceil(32)).copied().collect();
        if !k.is_multiple_of(32) && r.len() == k.div_ceil(32) {
            let last = r.len() - 1;
            r[last] &= (1u32 << (k % 32)) - 1;
        }
        trim(&mut r);
        return (q, r);
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
            sub_in_place(&mut r, b);
            q[i / 32] |= 1 << (i % 32);
        }
    }
    trim(&mut q);
    trim(&mut r);
    (q, r)
}

/// `r - b`, into `r`, for `r >= b`, with no zero left at the top.
fn sub_in_place(r: &mut Vec<u32>, b: &[u32]) {
    let mut borrow = 0i64;
    for (i, x) in r.iter_mut().enumerate() {
        let d = *x as i64 - b.get(i).copied().unwrap_or(0) as i64 - borrow;
        (*x, borrow) = if d < 0 {
            ((d + (1 << 32)) as u32, 1)
        } else {
            (d as u32, 0)
        };
    }
    trim(r);
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

#[inline]
pub fn _plus(a: Int, b: Int) -> Int {
    if let (Int::Small(x), Int::Small(y)) = (a, b)
        && let Some(s) = x.checked_add(y)
    {
        return Int::Small(s);
    }
    add(wide(a), wide(b))
}

#[inline]
pub fn _minus(a: Int, b: Int) -> Int {
    if let (Int::Small(x), Int::Small(y)) = (a, b)
        && let Some(s) = x.checked_sub(y)
    {
        return Int::Small(s);
    }
    add(wide(a), negate(wide(b)))
}

#[inline]
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

#[inline]
pub fn quotient(a: Int, b: Int) -> Int {
    divide(a, b, "quotient").0
}

#[inline]
pub fn remainder(a: Int, b: Int) -> Int {
    divide(a, b, "remainder").1
}

/// The remainder of the divisor's sign.
#[inline]
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

#[inline]
pub fn _eq(a: Int, b: Int) -> bool {
    compare(a, b) == Ordering::Equal
}

#[inline]
pub fn _lt(a: Int, b: Int) -> bool {
    compare(a, b) == Ordering::Less
}

#[inline]
pub fn _gt(a: Int, b: Int) -> bool {
    compare(a, b) == Ordering::Greater
}

#[inline]
pub fn _lt_eq(a: Int, b: Int) -> bool {
    compare(a, b) != Ordering::Greater
}

#[inline]
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

#[inline]
pub fn i64_to_int(n: i64) -> Int {
    Int::Small(n)
}

#[inline]
pub fn zero_p(n: Int) -> bool {
    matches!(n, Int::Small(0))
}

#[inline]
pub fn max(a: Int, b: Int) -> Int {
    if _lt(a, b) { b } else { a }
}

#[inline]
pub fn min(a: Int, b: Int) -> Int {
    if _lt(b, a) { b } else { a }
}

/// An integer literal past an `i64`, from its decimal digits, `-` first if
/// it is negative.
#[inline]
pub fn int_text(s: &str) -> Int {
    let (negative, digits) = match s.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, s),
    };
    let ten = int_lit(10);
    let n = digits.bytes().fold(int_lit(0), |n, d| {
        _plus(_star(n, ten), int_lit(i64::from(d - b'0')))
    });
    if negative { _minus(int_lit(0), n) } else { n }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn show(n: Int) -> String {
        int_to_string(n).as_str().to_owned()
    }

    /// The bit at a time division, as it was before the fast paths.
    fn divided_slowly(a: &[u32], b: &[u32]) -> (Vec<u32>, Vec<u32>) {
        let mut q = vec![0u32; a.len()];
        let mut r: Vec<u32> = Vec::new();
        for i in (0..a.len() * 32).rev() {
            r.insert(0, 0);
            r = shifted_right(&r, 31);
            trim(&mut r);
            if (a[i / 32] >> (i % 32)) & 1 == 1 {
                r = add_magnitudes(&r, &[1]);
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

    #[test]
    fn division_by_powers_of_two_and_by_many_limbs_as_by_bits() {
        let a: Vec<u32> = (1..40u32).map(|i| i.wrapping_mul(2654435761)).collect();
        let mut divisors: Vec<Vec<u32>> =
            vec![vec![0, 1], vec![0, 0, 8], vec![1 << 31], vec![0, 1 << 31]];
        divisors.push(vec![7, 9]);
        divisors.push(vec![0xffff_ffff, 0xffff_ffff, 3]);
        divisors.push(a[..20].to_vec());
        for b in &divisors {
            assert_eq!(divide_magnitudes(&a, b), divided_slowly(&a, b), "by {b:?}");
        }
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

// Bitwise operations, as FX-26's: on two's complement, of any size.

/// `w`, two's complement, in `len` 32-bit words, the low first.
fn twos(w: &Wide, len: usize) -> Vec<u32> {
    let mut m = w.magnitude.clone();
    m.resize(len, 0);
    if w.negative {
        let mut carry = 1u64;
        for x in m.iter_mut() {
            let v = (!*x) as u64 + carry;
            *x = v as u32;
            carry = v >> 32;
        }
    }
    m
}

/// The integer whose two's complement `m` is, its top bit its sign.
fn from_twos(mut m: Vec<u32>) -> Int {
    let negative = m.last().is_some_and(|x| x >> 31 == 1);
    if negative {
        let mut carry = 1u64;
        for x in m.iter_mut() {
            let v = (!*x) as u64 + carry;
            *x = v as u32;
            carry = v >> 32;
        }
    }
    int(Wide {
        negative,
        magnitude: m,
    })
}

fn bitwise(a: Int, b: Int, f: impl Fn(u32, u32) -> u32) -> Int {
    if let (Int::Small(x), Int::Small(y)) = (a, b) {
        return Int::Small(
            f(x as u32, y as u32) as i64 & 0xffff_ffff
                | ((f((x >> 32) as u32, (y >> 32) as u32) as i64) << 32),
        );
    }
    let (x, y) = (wide(a), wide(b));
    let len = x.magnitude.len().max(y.magnitude.len()) + 1;
    let (tx, ty) = (twos(&x, len), twos(&y, len));
    from_twos(tx.iter().zip(&ty).map(|(p, q)| f(*p, *q)).collect())
}

pub fn bitwise_and(a: Int, b: Int) -> Int {
    bitwise(a, b, |p, q| p & q)
}

pub fn bitwise_ior(a: Int, b: Int) -> Int {
    bitwise(a, b, |p, q| p | q)
}

pub fn bitwise_xor(a: Int, b: Int) -> Int {
    bitwise(a, b, |p, q| p ^ q)
}

pub fn bitwise_not(a: Int) -> Int {
    _minus(int_lit(-1), a)
}

fn shifted_left(m: &[u32], k: usize) -> Vec<u32> {
    let (words, bits) = (k / 32, k % 32);
    let mut out = vec![0u32; words];
    let mut carry = 0u32;
    for &x in m {
        out.push(if bits == 0 { x } else { (x << bits) | carry });
        carry = if bits == 0 { 0 } else { x >> (32 - bits) };
    }
    out.push(carry);
    out
}

fn shifted_right(m: &[u32], k: usize) -> Vec<u32> {
    let (words, bits) = (k / 32, k % 32);
    if words >= m.len() {
        return Vec::new();
    }
    let rest = &m[words..];
    (0..rest.len())
        .map(|i| {
            let lo = rest[i] >> bits;
            let hi = if bits == 0 {
                0
            } else {
                rest.get(i + 1).map_or(0, |x| x << (32 - bits))
            };
            lo | hi
        })
        .collect()
}

/// Left by a positive count, right (the floor) by a negative one; a count
/// past 2^24 is too large unless `a` is 0, and a count past an `i64`,
/// right, leaves the sign alone.
pub fn arithmetic_shift(a: Int, k: Int) -> Int {
    let zero = sign(a) == 0;
    let k = match k {
        Int::Small(k) => k,
        Int::Big(b) if b.negative => return if sign(a) < 0 { int_lit(-1) } else { int_lit(0) },
        Int::Big(_) if zero => return int_lit(0),
        Int::Big(_) => crate::fail("arithmetic-shift: the count is too large"),
    };
    if zero {
        return int_lit(0);
    }
    if k > 1 << 24 {
        crate::fail("arithmetic-shift: the count is too large")
    }
    let w = wide(a);
    if k >= 0 {
        return int(Wide {
            negative: w.negative,
            magnitude: shifted_left(&w.magnitude, k as usize),
        });
    }
    let n = k.unsigned_abs().min(1 << 32) as usize;
    if !w.negative {
        return int(Wide {
            negative: false,
            magnitude: shifted_right(&w.magnitude, n),
        });
    }
    // The floor of a negative: -((|a| - 1) >> n) - 1.
    let less = wide(_minus(
        int(Wide {
            negative: false,
            magnitude: w.magnitude,
        }),
        int_lit(1),
    ));
    _minus(
        int_lit(-1),
        int(Wide {
            negative: false,
            magnitude: shifted_right(&less.magnitude, n),
        }),
    )
}

#[cfg(test)]
mod bitwise_tests {
    use super::*;

    fn n(s: &str) -> Int {
        int_text(s)
    }

    fn text(i: Int) -> String {
        int_to_string(i).as_str().to_string()
    }

    #[test]
    fn as_scheme_does() {
        assert_eq!(text(bitwise_and(int_lit(12), int_lit(10))), "8");
        assert_eq!(text(bitwise_ior(int_lit(-12), int_lit(10))), "-2");
        assert_eq!(text(bitwise_xor(int_lit(-1), int_lit(5))), "-6");
        assert_eq!(text(bitwise_not(int_lit(0))), "-1");
        assert_eq!(
            text(bitwise_and(
                n("-36893488147419103232"),
                n("36893488147419103231")
            )),
            "0"
        );
        assert_eq!(
            text(bitwise_ior(n("36893488147419103232"), int_lit(1))),
            "36893488147419103233"
        );
        assert_eq!(
            text(arithmetic_shift(int_lit(1), int_lit(70))),
            "1180591620717411303424"
        );
        assert_eq!(text(arithmetic_shift(int_lit(-5), int_lit(-1))), "-3");
        assert_eq!(
            text(arithmetic_shift(n("-1180591620717411303424"), int_lit(-70))),
            "-1"
        );
        assert_eq!(
            text(arithmetic_shift(n("-1180591620717411303425"), int_lit(-70))),
            "-2"
        );
        assert_eq!(text(arithmetic_shift(int_lit(7), int_lit(-3))), "0");
        assert_eq!(
            text(arithmetic_shift(i64_to_int(i64::MIN), int_lit(1))),
            "-18446744073709551616"
        );
    }
}
