// 160-bit scan key: private-key indices in [1, 2^160), the full P2WPKH (Bech32) address space.
// Represented as 4 little-endian u64 limbs (256-bit container; values < 2^160).
pub type Key = [u64; 4];

pub const LIMBS: usize = 4;
// 2^160 = bit 160 -> limb 2, bit offset 32 (bits 128-191 live in limb 2).
pub const MAX_KEY: Key = [0, 0, 0x1_0000_0000, 0];
pub const ONE: Key = [1, 0, 0, 0];
pub const ZERO: Key = [0, 0, 0, 0];

#[inline]
pub fn from_u64(n: u64) -> Key {
    [n, 0, 0, 0]
}

pub fn parse(s: &str) -> Option<Key> {
    let b = s.trim().as_bytes();
    if b.is_empty() {
        return None;
    }
    let mut k = ZERO;
    for &c in b {
        if !c.is_ascii_digit() {
            return None;
        }
        let d = (c - b'0') as u64;
        k = mul_add_10(&k)?;
        k = add_limb(&k, d)?;
        if cmp(&k, &MAX_KEY) != Ordering::Less {
            return None;
        }
    }
    if is_zero(&k) {
        return None;
    }
    Some(k)
}

#[inline]
fn add_limb(k: &Key, d: u64) -> Option<Key> {
    let mut out = *k;
    let mut carry = d;
    for i in 0..LIMBS {
        let (sum, c1) = out[i].overflowing_add(carry);
        out[i] = sum;
        carry = u64::from(c1);
        if carry == 0 {
            break;
        }
    }
    if carry != 0 {
        None
    } else {
        Some(out)
    }
}

#[inline]
fn mul_add_10(k: &Key) -> Option<Key> {
    let mut out = ZERO;
    let mut carry = 0u64;
    for i in 0..LIMBS {
        let wide = (k[i] as u128) * 10 + (carry as u128);
        out[i] = wide as u64;
        carry = (wide >> 64) as u64;
    }
    if carry != 0 {
        None
    } else {
        Some(out)
    }
}

pub fn to_string(k: &Key) -> String {
    if is_zero(k) {
        return "0".to_string();
    }
    let mut chunks = Vec::new();
    let mut cur = *k;
    let d19 = 10_000_000_000_000_000_000u64;
    while !is_zero(&cur) {
        let (q, r) = divmod(&cur, d19);
        chunks.push(r);
        cur = q;
    }
    let mut s = chunks.pop().unwrap().to_string();
    while let Some(c) = chunks.pop() {
        s.push_str(&format!("{:019}", c));
    }
    s
}

fn divmod(k: &Key, m: u64) -> (Key, u64) {
    let mut out = ZERO;
    let mut rem = 0u64;
    for i in (0..LIMBS).rev() {
        let cur = ((rem as u128) << 64) | (k[i] as u128);
        out[i] = (cur / m as u128) as u64;
        rem = (cur % m as u128) as u64;
    }
    (out, rem)
}

#[inline]
pub fn cmp(a: &Key, b: &Key) -> Ordering {
    for i in (0..LIMBS).rev() {
        if a[i] != b[i] {
            return if a[i] < b[i] { Ordering::Less } else { Ordering::Greater };
        }
    }
    Ordering::Equal
}

#[inline]
pub fn is_zero(k: &Key) -> bool {
    k[0] == 0 && k[1] == 0 && k[2] == 0 && k[3] == 0
}

#[inline]
pub fn lt(a: &Key, b: &Key) -> bool {
    cmp(a, b) == Ordering::Less
}

// a + n  (assumes no overflow past 2^160; caller checks)
pub fn add(a: &Key, n: u64) -> Key {
    let mut out = *a;
    let mut carry = n;
    for i in 0..LIMBS {
        let (sum, c1) = out[i].overflowing_add(carry);
        out[i] = sum;
        carry = u64::from(c1);
        if carry == 0 {
            break;
        }
    }
    out
}

// k / m -> (quotient, remainder), m < 2^64
pub fn div_small(k: &Key, m: u64) -> (Key, u64) {
    divmod(k, m)
}

// valid only when the value fits in the low limb (higher limbs zero)
pub fn to_small(k: &Key) -> u64 {
    k[0]
}

// a + n, None on overflow to/past 2^160 (keys are < 2^160)
pub fn add_checked(a: &Key, n: u64) -> Option<Key> {
    let out = add(a, n);
    if cmp(&out, &MAX_KEY) != Ordering::Less {
        None
    } else {
        Some(out)
    }
}

// a - b (a >= b)
pub fn sub(a: &Key, b: &Key) -> Key {
    let mut out = [0u64; LIMBS];
    let mut borrow = 0u64;
    for i in 0..LIMBS {
        let (d1, b1) = a[i].overflowing_sub(b[i]);
        let (d2, b2) = d1.overflowing_sub(borrow);
        out[i] = d2;
        borrow = u64::from(b1 | b2);
    }
    out
}

// Rough float approximation for rate display only.
pub fn to_f64(k: &Key) -> f64 {
    let mut v = 0f64;
    for i in (0..LIMBS).rev() {
        v = v * 18446744073709551616.0 + k[i] as f64;
    }
    v
}

use std::cmp::Ordering;
