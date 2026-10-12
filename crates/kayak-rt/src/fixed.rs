//! FX-26's fixed-width integers beside `i64`: `i8`, `u8`, `i32`, `u32`, `u64`, whose
//! arithmetic wraps, as Rust's `wrapping_` operations do. Each operation
//! is named as KFX names it, mangled: `u32+` is `u32_plus`.

use crate::{Int, Str, fail, int_lit, int_text, int_to_i64, str_lit};

macro_rules! fixed {
    ($t:ident, $plus:ident, $minus:ident, $star:ident, $quot:ident, $rem:ident, $and:ident, $or:ident,
     $xor:ident, $not:ident, $shl:ident, $shr:ident, $eq:ident, $lt:ident, $gt:ident, $le:ident, $ge:ident,
     $from_int:ident, $to_int:ident, $to_string:ident) => {
        #[inline]
        pub fn $plus(a: $t, b: $t) -> $t {
            a.wrapping_add(b)
        }
        #[inline]
        pub fn $minus(a: $t, b: $t) -> $t {
            a.wrapping_sub(b)
        }
        #[inline]
        pub fn $star(a: $t, b: $t) -> $t {
            a.wrapping_mul(b)
        }
        #[inline]
        pub fn $quot(a: $t, b: $t) -> $t {
            if b == 0 {
                fail(concat!(stringify!($t), "-quotient: division by zero"))
            }
            a.wrapping_div(b)
        }
        #[inline]
        pub fn $rem(a: $t, b: $t) -> $t {
            if b == 0 {
                fail(concat!(stringify!($t), "-remainder: division by zero"))
            }
            a.wrapping_rem(b)
        }
        #[inline]
        pub fn $and(a: $t, b: $t) -> $t {
            a & b
        }
        #[inline]
        pub fn $or(a: $t, b: $t) -> $t {
            a | b
        }
        #[inline]
        pub fn $xor(a: $t, b: $t) -> $t {
            a ^ b
        }
        #[inline]
        pub fn $not(a: $t) -> $t {
            !a
        }
        /// The count taken modulo the width, as FX-26's.
        #[inline]
        pub fn $shl(a: $t, n: Int) -> $t {
            a.wrapping_shl(int_to_i64(n).rem_euclid(<$t>::BITS as i64) as u32)
        }
        /// Arithmetic if signed, else logical; the count modulo the width.
        #[inline]
        pub fn $shr(a: $t, n: Int) -> $t {
            a.wrapping_shr(int_to_i64(n).rem_euclid(<$t>::BITS as i64) as u32)
        }
        #[inline]
        pub fn $eq(a: $t, b: $t) -> bool {
            a == b
        }
        #[inline]
        pub fn $lt(a: $t, b: $t) -> bool {
            a < b
        }
        #[inline]
        pub fn $gt(a: $t, b: $t) -> bool {
            a > b
        }
        #[inline]
        pub fn $le(a: $t, b: $t) -> bool {
            a <= b
        }
        #[inline]
        pub fn $ge(a: $t, b: $t) -> bool {
            a >= b
        }
        /// Wraps, as FX-26's: the integer's low bits.
        #[inline]
        pub fn $from_int(n: Int) -> $t {
            int_to_i64(n) as $t
        }
        #[inline]
        pub fn $to_int(n: $t) -> Int {
            match i64::try_from(n) {
                Ok(v) => int_lit(v),
                Err(_) => int_text(&n.to_string()),
            }
        }
        #[inline]
        pub fn $to_string(n: $t) -> Str {
            str_lit(Box::leak(n.to_string().into_boxed_str()))
        }
    };
}

fixed!(
    u8,
    u8_plus,
    u8_minus,
    u8_star,
    u8_quotient,
    u8_remainder,
    u8_and,
    u8_or,
    u8_xor,
    u8_not,
    u8_shl,
    u8_shr,
    u8_eq,
    u8_lt,
    u8_gt,
    u8_lt_eq,
    u8_gt_eq,
    int_to_u8,
    u8_to_int,
    u8_to_string
);
fixed!(
    i8,
    i8_plus,
    i8_minus,
    i8_star,
    i8_quotient,
    i8_remainder,
    i8_and,
    i8_or,
    i8_xor,
    i8_not,
    i8_shl,
    i8_shr,
    i8_eq,
    i8_lt,
    i8_gt,
    i8_lt_eq,
    i8_gt_eq,
    int_to_i8,
    i8_to_int,
    i8_to_string
);
fixed!(
    i32,
    i32_plus,
    i32_minus,
    i32_star,
    i32_quotient,
    i32_remainder,
    i32_and,
    i32_or,
    i32_xor,
    i32_not,
    i32_shl,
    i32_shr,
    i32_eq,
    i32_lt,
    i32_gt,
    i32_lt_eq,
    i32_gt_eq,
    int_to_i32,
    i32_to_int,
    i32_to_string
);
fixed!(
    u32,
    u32_plus,
    u32_minus,
    u32_star,
    u32_quotient,
    u32_remainder,
    u32_and,
    u32_or,
    u32_xor,
    u32_not,
    u32_shl,
    u32_shr,
    u32_eq,
    u32_lt,
    u32_gt,
    u32_lt_eq,
    u32_gt_eq,
    int_to_u32,
    u32_to_int,
    u32_to_string
);
fixed!(
    u64,
    u64_plus,
    u64_minus,
    u64_star,
    u64_quotient,
    u64_remainder,
    u64_and,
    u64_or,
    u64_xor,
    u64_not,
    u64_shl,
    u64_shr,
    u64_eq,
    u64_lt,
    u64_gt,
    u64_lt_eq,
    u64_gt_eq,
    int_to_u64,
    u64_to_int,
    u64_to_string
);

/// `i64-shl` and `i64-shr`, as the others'.
#[inline]
pub fn i64_shl(a: i64, n: Int) -> i64 {
    a.wrapping_shl(int_to_i64(n).rem_euclid(64) as u32)
}
#[inline]
pub fn i64_shr(a: i64, n: Int) -> i64 {
    a.wrapping_shr(int_to_i64(n).rem_euclid(64) as u32)
}
