// radix-2^52 field arithmetic (ported from bitcoin-core/libsecp256k1 field_5x52)
// plus point operations for high-speed sequential generation.

use crate::ec;
use sha2::Digest;

pub type Fe51 = [u64; 5];

const M52: u64 = 0xFFFFFFFFFFFFF;
const M48: u64 = 0x0FFFFFFFFFFFF;
const CK: u64 = 0x1000003D1;
const R16: u64 = 0x1000003D10;
const C0: u64 = 0xFFFFEFFFFFC2F;

pub const ONE: Fe51 = [1, 0, 0, 0, 0];

#[derive(Clone, Copy)]
pub struct Jacobian51 {
    pub x: Fe51,
    pub y: Fe51,
    pub z: Fe51,
}

pub const INF: Jacobian51 = Jacobian51 {
    x: [0; 5],
    y: [0; 5],
    z: [0; 5],
};

#[inline(always)]
pub fn fe_mul(a: &Fe51, b: &Fe51) -> Fe51 {
    let a0 = a[0];
    let a1 = a[1];
    let a2 = a[2];
    let a3 = a[3];
    let a4 = a[4];
    let b0 = b[0];
    let b1 = b[1];
    let b2 = b[2];
    let b3 = b[3];
    let b4 = b[4];
    let mut c: u128;
    let mut d: u128;
    let mut t3: u64;
    let mut t4: u64;
    let mut tx: u64;
    let mut u0: u64;
    let mut r = [0u64; 5];

    d = (a0 as u128) * (b3 as u128);
    d = d.wrapping_add((a1 as u128) * (b2 as u128));
    d = d.wrapping_add((a2 as u128) * (b1 as u128));
    d = d.wrapping_add((a3 as u128) * (b0 as u128));
    c = (a4 as u128) * (b4 as u128);
    d = d.wrapping_add((R16 as u128) * ((c as u64) as u128));
    c >>= 64;
    t3 = (d as u64) & M52;
    d >>= 52;
    d = d.wrapping_add((a0 as u128) * (b4 as u128));
    d = d.wrapping_add((a1 as u128) * (b3 as u128));
    d = d.wrapping_add((a2 as u128) * (b2 as u128));
    d = d.wrapping_add((a3 as u128) * (b1 as u128));
    d = d.wrapping_add((a4 as u128) * (b0 as u128));
    d = d.wrapping_add(((R16 << 12) as u128) * ((c as u64) as u128));
    t4 = (d as u64) & M52;
    d >>= 52;
    tx = t4 >> 48;
    t4 &= M52 >> 4;
    c = (a0 as u128) * (b0 as u128);
    d = d.wrapping_add((a1 as u128) * (b4 as u128));
    d = d.wrapping_add((a2 as u128) * (b3 as u128));
    d = d.wrapping_add((a3 as u128) * (b2 as u128));
    d = d.wrapping_add((a4 as u128) * (b1 as u128));
    u0 = (d as u64) & M52;
    d >>= 52;
    u0 = (u0 << 4) | tx;
    c = c.wrapping_add((u0 as u128) * (CK as u128));
    r[0] = (c as u64) & M52;
    c >>= 52;
    c = c.wrapping_add((a0 as u128) * (b1 as u128));
    c = c.wrapping_add((a1 as u128) * (b0 as u128));
    d = d.wrapping_add((a2 as u128) * (b4 as u128));
    d = d.wrapping_add((a3 as u128) * (b3 as u128));
    d = d.wrapping_add((a4 as u128) * (b2 as u128));
    c = c.wrapping_add(((d as u64 & M52) as u128) * (R16 as u128));
    d >>= 52;
    r[1] = (c as u64) & M52;
    c >>= 52;
    c = c.wrapping_add((a0 as u128) * (b2 as u128));
    c = c.wrapping_add((a1 as u128) * (b1 as u128));
    c = c.wrapping_add((a2 as u128) * (b0 as u128));
    d = d.wrapping_add((a3 as u128) * (b4 as u128));
    d = d.wrapping_add((a4 as u128) * (b3 as u128));
    c = c.wrapping_add((R16 as u128) * ((d as u64) as u128));
    d >>= 64;
    r[2] = (c as u64) & M52;
    c >>= 52;
    c = c.wrapping_add(((R16 << 12) as u128) * ((d as u64) as u128));
    c = c.wrapping_add(t3 as u128);
    r[3] = (c as u64) & M52;
    c >>= 52;
    r[4] = (c as u64) + t4;
    r
}

#[inline(always)]
pub fn fe_sqr(a: &Fe51) -> Fe51 {
    let mut a0 = a[0];
    let mut a1 = a[1];
    let mut a2 = a[2];
    let mut a3 = a[3];
    let mut a4 = a[4];
    let mut c: u128;
    let mut d: u128;
    let mut t3: u64;
    let mut t4: u64;
    let mut tx: u64;
    let mut u0: u64;
    let mut r = [0u64; 5];

    d = ((a0 as u128) * 2) * (a3 as u128);
    d = d.wrapping_add(((a1 as u128) * 2) * (a2 as u128));
    c = (a4 as u128) * (a4 as u128);
    d = d.wrapping_add((R16 as u128) * ((c as u64) as u128));
    c >>= 64;
    t3 = (d as u64) & M52;
    d >>= 52;
    a4 *= 2;
    d = d.wrapping_add((a0 as u128) * (a4 as u128));
    d = d.wrapping_add(((a1 as u128) * 2) * (a3 as u128));
    d = d.wrapping_add((a2 as u128) * (a2 as u128));
    d = d.wrapping_add(((R16 << 12) as u128) * ((c as u64) as u128));
    t4 = (d as u64) & M52;
    d >>= 52;
    tx = t4 >> 48;
    t4 &= M52 >> 4;
    c = (a0 as u128) * (a0 as u128);
    d = d.wrapping_add((a1 as u128) * (a4 as u128));
    d = d.wrapping_add(((a2 as u128) * 2) * (a3 as u128));
    u0 = (d as u64) & M52;
    d >>= 52;
    u0 = (u0 << 4) | tx;
    c = c.wrapping_add((u0 as u128) * (CK as u128));
    r[0] = (c as u64) & M52;
    c >>= 52;
    a0 *= 2;
    c = c.wrapping_add((a0 as u128) * (a1 as u128));
    d = d.wrapping_add((a2 as u128) * (a4 as u128));
    d = d.wrapping_add((a3 as u128) * (a3 as u128));
    c = c.wrapping_add(((d as u64 & M52) as u128) * (R16 as u128));
    d >>= 52;
    r[1] = (c as u64) & M52;
    c >>= 52;
    c = c.wrapping_add((a0 as u128) * (a2 as u128));
    c = c.wrapping_add((a1 as u128) * (a1 as u128));
    d = d.wrapping_add((a3 as u128) * (a4 as u128));
    c = c.wrapping_add((R16 as u128) * ((d as u64) as u128));
    d >>= 64;
    r[2] = (c as u64) & M52;
    c >>= 52;
    c = c.wrapping_add(((R16 << 12) as u128) * ((d as u64) as u128));
    c = c.wrapping_add(t3 as u128);
    r[3] = (c as u64) & M52;
    c >>= 52;
    r[4] = (c as u64) + t4;
    r
}

#[inline(always)]
pub fn fe_add(a: &Fe51, b: &Fe51) -> Fe51 {
    [
        a[0] + b[0],
        a[1] + b[1],
        a[2] + b[2],
        a[3] + b[3],
        a[4] + b[4],
    ]
}

#[inline(always)]
fn fe_negate(a: &Fe51) -> Fe51 {
    [
        C0 * 18 - a[0],
        M52 * 18 - a[1],
        M52 * 18 - a[2],
        M52 * 18 - a[3],
        M48 * 18 - a[4],
    ]
}

pub fn fe_normalize_weak(r: &mut Fe51) {
    let mut t0 = r[0];
    let mut t1 = r[1];
    let mut t2 = r[2];
    let mut t3 = r[3];
    let mut t4 = r[4];
    let x = t4 >> 48;
    t4 &= M48;
    t0 += x * CK;
    t1 += t0 >> 52;
    t0 &= M52;
    t2 += t1 >> 52;
    t1 &= M52;
    t3 += t2 >> 52;
    t2 &= M52;
    t4 += t3 >> 52;
    t3 &= M52;
    r[0] = t0;
    r[1] = t1;
    r[2] = t2;
    r[3] = t3;
    r[4] = t4;
}

pub fn fe_normalize(r: &mut Fe51) {
    let mut t0 = r[0];
    let mut t1 = r[1];
    let mut t2 = r[2];
    let mut t3 = r[3];
    let mut t4 = r[4];
    let mut x = t4 >> 48;
    t4 &= M48;
    t0 += x * CK;
    t1 += t0 >> 52;
    t0 &= M52;
    t2 += t1 >> 52;
    t1 &= M52;
    let mut m = t1;
    t3 += t2 >> 52;
    t2 &= M52;
    m &= t2;
    t4 += t3 >> 52;
    t3 &= M52;
    m &= t3;
    x = (t4 >> 48) | ((t4 == M48) && (m == M52) && (t0 >= C0)) as u64;
    t0 += x * CK;
    t1 += t0 >> 52;
    t0 &= M52;
    t2 += t1 >> 52;
    t1 &= M52;
    t3 += t2 >> 52;
    t2 &= M52;
    t4 += t3 >> 52;
    t3 &= M52;
    t4 &= M48;
    r[0] = t0;
    r[1] = t1;
    r[2] = t2;
    r[3] = t3;
    r[4] = t4;
}

pub fn fe_normalizes_to_zero(a: &Fe51) -> bool {
    let mut t0 = a[0];
    let mut t1 = a[1];
    let mut t2 = a[2];
    let mut t3 = a[3];
    let mut t4 = a[4];
    let mut z0: u64;
    let mut z1: u64;
    let x = t4 >> 48;
    t4 &= M48;
    t0 += x * CK;
    t1 += t0 >> 52;
    t0 &= M52;
    z0 = t0;
    z1 = t0 ^ (CK - 1);
    t2 += t1 >> 52;
    t1 &= M52;
    z0 |= t1;
    z1 &= t1;
    t3 += t2 >> 52;
    t2 &= M52;
    z0 |= t2;
    z1 &= t2;
    t4 += t3 >> 52;
    t3 &= M52;
    z0 |= t3;
    z1 &= t3;
    z0 |= t4;
    z1 &= t4 ^ 0xF000000000000;
    (z0 == 0) | (z1 == M52)
}

#[inline(always)]
pub fn fe_sub(a: &Fe51, b: &Fe51) -> Fe51 {
    let n = fe_negate(b);
    let mut s = [
        a[0] + n[0],
        a[1] + n[1],
        a[2] + n[2],
        a[3] + n[3],
        a[4] + n[4],
    ];
    fe_normalize_weak(&mut s);
    s
}

pub fn fe_is_zero(a: &Fe51) -> bool {
    (a[0] | a[1] | a[2] | a[3] | a[4]) == 0
}

pub fn fe_from_b32(a: &[u8; 32]) -> Fe51 {
    let n0 = a[31] as u64
        | (a[30] as u64) << 8
        | (a[29] as u64) << 16
        | (a[28] as u64) << 24
        | (a[27] as u64) << 32
        | (a[26] as u64) << 40
        | ((a[25] & 0xF) as u64) << 48;
    let n1 = ((a[25] >> 4) & 0xF) as u64
        | (a[24] as u64) << 4
        | (a[23] as u64) << 12
        | (a[22] as u64) << 20
        | (a[21] as u64) << 28
        | (a[20] as u64) << 36
        | (a[19] as u64) << 44;
    let n2 = a[18] as u64
        | (a[17] as u64) << 8
        | (a[16] as u64) << 16
        | (a[15] as u64) << 24
        | (a[14] as u64) << 32
        | (a[13] as u64) << 40
        | ((a[12] & 0xF) as u64) << 48;
    let n3 = ((a[12] >> 4) & 0xF) as u64
        | (a[11] as u64) << 4
        | (a[10] as u64) << 12
        | (a[9] as u64) << 20
        | (a[8] as u64) << 28
        | (a[7] as u64) << 36
        | (a[6] as u64) << 44;
    let n4 = a[5] as u64
        | (a[4] as u64) << 8
        | (a[3] as u64) << 16
        | (a[2] as u64) << 24
        | (a[1] as u64) << 32
        | (a[0] as u64) << 40;
    [n0, n1, n2, n3, n4]
}

pub fn fe_to_b32(a: &Fe51) -> [u8; 32] {
    let mut r = [0u8; 32];
    let w0 = (((a[4] as u128) << 16) | ((a[3] as u128) >> 36)) as u64;
    let w1 = (((a[3] as u128) << 28) | ((a[2] as u128) >> 24)) as u64;
    let w2 = (((a[2] as u128) << 40) | ((a[1] as u128) >> 12)) as u64;
    let w3 = (((a[1] as u128) << 52) | (a[0] as u128)) as u64;
    r[0..8].copy_from_slice(&w0.to_be_bytes());
    r[8..16].copy_from_slice(&w1.to_be_bytes());
    r[16..24].copy_from_slice(&w2.to_be_bytes());
    r[24..32].copy_from_slice(&w3.to_be_bytes());
    r
}

pub fn fe_inv(a: &Fe51) -> Fe51 {
    let mut r = ONE;
    let mut base = *a;
    let p2: [u64; 4] = [0xFFFFFFFEFFFFFC2D, u64::MAX, u64::MAX, u64::MAX];
    for word in p2 {
        for i in 0..64 {
            if (word >> i) & 1 == 1 {
                r = fe_mul(&r, &base);
            }
            base = fe_sqr(&base);
        }
    }
    r
}

pub fn batch_invert(zs: &mut [Fe51]) {
    let n = zs.len();
    if n == 0 {
        return;
    }
    if n == 1 {
        zs[0] = fe_inv(&zs[0]);
        return;
    }
    let mut prefix = Vec::with_capacity(n);
    let mut acc = zs[0];
    prefix.push(acc);
    for i in 1..n {
        acc = fe_mul(&acc, &zs[i]);
        prefix.push(acc);
    }
    let mut inv = fe_inv(&acc);
    for i in (0..n).rev() {
        let zi = zs[i];
        let pprev = if i == 0 { ONE } else { prefix[i - 1] };
        zs[i] = fe_mul(&inv, &pprev);
        inv = fe_mul(&inv, &zi);
    }
}

fn point_double(p: &Jacobian51) -> Jacobian51 {
    if fe_is_zero(&p.z) {
        return INF;
    }
    let x = p.x;
    let y = p.y;
    let z = p.z;

    let a = fe_sqr(&x);
    let b = fe_sqr(&y);
    let c = fe_sqr(&b);
    let t = fe_sub(&fe_sqr(&fe_add(&x, &b)), &a);
    let t = fe_sub(&t, &c);
    let d = fe_add(&t, &t);
    let t = fe_add(&a, &a);
    let e = fe_add(&t, &a);
    let f = fe_sqr(&e);

    let t = fe_add(&d, &d);
    let x3 = fe_sub(&f, &t);

    let t = fe_sub(&d, &x3);
    let t = fe_mul(&e, &t);
    let mut eight_c = c;
    eight_c = fe_add(&eight_c, &c);
    eight_c = fe_add(&eight_c, &eight_c);
    eight_c = fe_add(&eight_c, &eight_c);
    let y3 = fe_sub(&t, &eight_c);

    let t = fe_mul(&y, &z);
    let z3 = fe_add(&t, &t);

    Jacobian51 { x: x3, y: y3, z: z3 }
}

pub fn point_add(p: &Jacobian51, gx: &Fe51, gy: &Fe51) -> Jacobian51 {
    if fe_is_zero(&p.z) {
        return Jacobian51 {
            x: *gx,
            y: *gy,
            z: ONE,
        };
    }
    let z1z1 = fe_sqr(&p.z);
    let u2 = fe_mul(gx, &z1z1);
    let s2 = fe_mul(gy, &fe_mul(&p.z, &z1z1));

    let h = fe_sub(&u2, &p.x);
    let r = fe_sub(&s2, &p.y);

    if fe_normalizes_to_zero(&h) {
        if fe_normalizes_to_zero(&r) {
            return point_double(p);
        }
        return INF;
    }

    let hh = fe_sqr(&h);
    let hhh = fe_mul(&h, &hh);
    let v = fe_mul(&p.x, &hh);

    let t = fe_sub(&fe_sqr(&r), &hhh);
    let t = fe_sub(&t, &fe_add(&v, &v));
    let x3 = t;

    let t = fe_sub(&v, &x3);
    let t = fe_mul(&r, &t);
    let y3 = fe_sub(&t, &fe_mul(&p.y, &hhh));

    let z3 = fe_mul(&p.z, &h);

    Jacobian51 { x: x3, y: y3, z: z3 }
}

pub fn scalar_mult(s: &[u64; 4], gx: &Fe51, gy: &Fe51) -> Jacobian51 {
    let mut acc = INF;
    for i in (0..256).rev() {
        acc = point_double(&acc);
        let bit = (s[i / 64] >> (i % 64)) & 1;
        if bit == 1 {
            acc = point_add(&acc, gx, gy);
        }
    }
    acc
}

pub fn to_compressed_inv(p: &Jacobian51, zi: &Fe51) -> [u8; 33] {
    let zinv2 = fe_mul(zi, zi);
    let mut x = fe_mul(&p.x, &zinv2);
    let zinv3 = fe_mul(&zinv2, zi);
    let y = fe_mul(&p.y, &zinv3);
    let prefix = if y[0] & 1 == 1 { 3 } else { 2 };
    fe_normalize(&mut x);
    let xb = fe_to_b32(&x);
    let mut out = [0u8; 33];
    out[0] = prefix;
    out[1..33].copy_from_slice(&xb);
    out
}

pub fn hash160_of_n51(n: u64) -> [u8; 20] {
    let s = [n, 0, 0, 0];
    let gx = fe_from_b32_limbs(&ec::GX);
    let gy = fe_from_b32_limbs(&ec::GY);
    let p = scalar_mult(&s, &gx, &gy);
    let zi = fe_inv(&p.z);
    let comp = to_compressed_inv(&p, &zi);
    ec::hash160_from_compressed(&comp)
}

pub fn fe_from_b32_limbs(limbs: &[u64; 4]) -> Fe51 {
    let mut bytes = [0u8; 32];
    for i in 0..4 {
        bytes[i * 8..i * 8 + 8].copy_from_slice(&limbs[3 - i].to_be_bytes());
    }
    fe_from_b32(&bytes)
}

#[inline(always)]
fn f0(x: u32, y: u32, z: u32) -> u32 {
    x ^ y ^ z
}
#[inline(always)]
fn f1(x: u32, y: u32, z: u32) -> u32 {
    (x & y) | (!x & z)
}
#[inline(always)]
fn f2(x: u32, y: u32, z: u32) -> u32 {
    (x | !y) ^ z
}
#[inline(always)]
fn f3(x: u32, y: u32, z: u32) -> u32 {
    (x & z) | (y & !z)
}
#[inline(always)]
fn f4(x: u32, y: u32, z: u32) -> u32 {
    x ^ (y | !z)
}

macro_rules! rl {
    ($a:ident,$b:ident,$c:ident,$d:ident,$e:ident,$f:ident,$k:expr,$s:expr,$x:expr) => {
        $a = $a
            .wrapping_add($f($b, $c, $d))
            .wrapping_add($x)
            .wrapping_add($k)
            .rotate_left($s)
            .wrapping_add($e);
        $c = $c.rotate_left(10);
    };
}

fn compress_ripemd(h: &mut [u32; 5], block: &[u8; 64]) {
    let mut x = [0u32; 16];
    for i in 0..16 {
        x[i] = u32::from_le_bytes([
            block[i * 4],
            block[i * 4 + 1],
            block[i * 4 + 2],
            block[i * 4 + 3],
        ]);
    }
    let (mut a, mut b, mut c, mut d, mut e) = (h[0], h[1], h[2], h[3], h[4]);
    let (mut a2, mut b2, mut c2, mut d2, mut e2) = (h[0], h[1], h[2], h[3], h[4]);

    // left round 0 (f0, K=0)
    rl!(a, b, c, d, e, f0, 0x00000000, 11, x[0]);
        rl!(e, a, b, c, d, f0, 0x00000000, 14, x[1]);
        rl!(d, e, a, b, c, f0, 0x00000000, 15, x[2]);
        rl!(c, d, e, a, b, f0, 0x00000000, 12, x[3]);
        rl!(b, c, d, e, a, f0, 0x00000000, 5, x[4]);
        rl!(a, b, c, d, e, f0, 0x00000000, 8, x[5]);
        rl!(e, a, b, c, d, f0, 0x00000000, 7, x[6]);
        rl!(d, e, a, b, c, f0, 0x00000000, 9, x[7]);
        rl!(c, d, e, a, b, f0, 0x00000000, 11, x[8]);
        rl!(b, c, d, e, a, f0, 0x00000000, 13, x[9]);
        rl!(a, b, c, d, e, f0, 0x00000000, 14, x[10]);
        rl!(e, a, b, c, d, f0, 0x00000000, 15, x[11]);
        rl!(d, e, a, b, c, f0, 0x00000000, 6, x[12]);
        rl!(c, d, e, a, b, f0, 0x00000000, 7, x[13]);
        rl!(b, c, d, e, a, f0, 0x00000000, 9, x[14]);
        rl!(a, b, c, d, e, f0, 0x00000000, 8, x[15]);

        // right round 0 (f4, K'=0x50A28BE6), r' = 5,14,7,0,9,2,11,4,13,6,15,8,1,10,3,12
        rl!(a2, b2, c2, d2, e2, f4, 0x50A28BE6, 8, x[5]);
        rl!(e2, a2, b2, c2, d2, f4, 0x50A28BE6, 9, x[14]);
        rl!(d2, e2, a2, b2, c2, f4, 0x50A28BE6, 9, x[7]);
        rl!(c2, d2, e2, a2, b2, f4, 0x50A28BE6, 11, x[0]);
        rl!(b2, c2, d2, e2, a2, f4, 0x50A28BE6, 13, x[9]);
        rl!(a2, b2, c2, d2, e2, f4, 0x50A28BE6, 15, x[2]);
        rl!(e2, a2, b2, c2, d2, f4, 0x50A28BE6, 15, x[11]);
        rl!(d2, e2, a2, b2, c2, f4, 0x50A28BE6, 5, x[4]);
        rl!(c2, d2, e2, a2, b2, f4, 0x50A28BE6, 7, x[13]);
        rl!(b2, c2, d2, e2, a2, f4, 0x50A28BE6, 7, x[6]);
        rl!(a2, b2, c2, d2, e2, f4, 0x50A28BE6, 8, x[15]);
        rl!(e2, a2, b2, c2, d2, f4, 0x50A28BE6, 11, x[8]);
        rl!(d2, e2, a2, b2, c2, f4, 0x50A28BE6, 14, x[1]);
        rl!(c2, d2, e2, a2, b2, f4, 0x50A28BE6, 14, x[10]);
        rl!(b2, c2, d2, e2, a2, f4, 0x50A28BE6, 12, x[3]);
        rl!(a2, b2, c2, d2, e2, f4, 0x50A28BE6, 6, x[12]);

        // left round 1 (f1, K=0x5A827999), r = 7,4,13,1,10,6,15,3,12,0,9,5,2,14,11,8
        rl!(e, a, b, c, d, f1, 0x5A827999, 7, x[7]);
        rl!(d, e, a, b, c, f1, 0x5A827999, 6, x[4]);
        rl!(c, d, e, a, b, f1, 0x5A827999, 8, x[13]);
        rl!(b, c, d, e, a, f1, 0x5A827999, 13, x[1]);
        rl!(a, b, c, d, e, f1, 0x5A827999, 11, x[10]);
        rl!(e, a, b, c, d, f1, 0x5A827999, 9, x[6]);
        rl!(d, e, a, b, c, f1, 0x5A827999, 7, x[15]);
        rl!(c, d, e, a, b, f1, 0x5A827999, 15, x[3]);
        rl!(b, c, d, e, a, f1, 0x5A827999, 7, x[12]);
        rl!(a, b, c, d, e, f1, 0x5A827999, 12, x[0]);
        rl!(e, a, b, c, d, f1, 0x5A827999, 15, x[9]);
        rl!(d, e, a, b, c, f1, 0x5A827999, 9, x[5]);
        rl!(c, d, e, a, b, f1, 0x5A827999, 11, x[2]);
        rl!(b, c, d, e, a, f1, 0x5A827999, 7, x[14]);
        rl!(a, b, c, d, e, f1, 0x5A827999, 13, x[11]);
        rl!(e, a, b, c, d, f1, 0x5A827999, 12, x[8]);

        // right round 1 (f3, K'=0x5C4DD124), r' = 6,11,3,7,0,13,5,10,14,15,8,12,4,9,1,2
        rl!(e2, a2, b2, c2, d2, f3, 0x5C4DD124, 9, x[6]);
        rl!(d2, e2, a2, b2, c2, f3, 0x5C4DD124, 13, x[11]);
        rl!(c2, d2, e2, a2, b2, f3, 0x5C4DD124, 15, x[3]);
        rl!(b2, c2, d2, e2, a2, f3, 0x5C4DD124, 7, x[7]);
        rl!(a2, b2, c2, d2, e2, f3, 0x5C4DD124, 12, x[0]);
        rl!(e2, a2, b2, c2, d2, f3, 0x5C4DD124, 8, x[13]);
        rl!(d2, e2, a2, b2, c2, f3, 0x5C4DD124, 9, x[5]);
        rl!(c2, d2, e2, a2, b2, f3, 0x5C4DD124, 11, x[10]);
        rl!(b2, c2, d2, e2, a2, f3, 0x5C4DD124, 7, x[14]);
        rl!(a2, b2, c2, d2, e2, f3, 0x5C4DD124, 7, x[15]);
        rl!(e2, a2, b2, c2, d2, f3, 0x5C4DD124, 12, x[8]);
        rl!(d2, e2, a2, b2, c2, f3, 0x5C4DD124, 7, x[12]);
        rl!(c2, d2, e2, a2, b2, f3, 0x5C4DD124, 6, x[4]);
        rl!(b2, c2, d2, e2, a2, f3, 0x5C4DD124, 15, x[9]);
        rl!(a2, b2, c2, d2, e2, f3, 0x5C4DD124, 13, x[1]);
        rl!(e2, a2, b2, c2, d2, f3, 0x5C4DD124, 11, x[2]);

        // left round 2 (f2, K=0x6ED9EBA1), r = 3,10,14,4,9,15,8,1,2,7,0,6,13,11,5,12
        rl!(d, e, a, b, c, f2, 0x6ED9EBA1, 11, x[3]);
        rl!(c, d, e, a, b, f2, 0x6ED9EBA1, 13, x[10]);
        rl!(b, c, d, e, a, f2, 0x6ED9EBA1, 6, x[14]);
        rl!(a, b, c, d, e, f2, 0x6ED9EBA1, 7, x[4]);
        rl!(e, a, b, c, d, f2, 0x6ED9EBA1, 14, x[9]);
        rl!(d, e, a, b, c, f2, 0x6ED9EBA1, 9, x[15]);
        rl!(c, d, e, a, b, f2, 0x6ED9EBA1, 13, x[8]);
        rl!(b, c, d, e, a, f2, 0x6ED9EBA1, 15, x[1]);
        rl!(a, b, c, d, e, f2, 0x6ED9EBA1, 14, x[2]);
        rl!(e, a, b, c, d, f2, 0x6ED9EBA1, 8, x[7]);
        rl!(d, e, a, b, c, f2, 0x6ED9EBA1, 13, x[0]);
        rl!(c, d, e, a, b, f2, 0x6ED9EBA1, 6, x[6]);
        rl!(b, c, d, e, a, f2, 0x6ED9EBA1, 5, x[13]);
        rl!(a, b, c, d, e, f2, 0x6ED9EBA1, 12, x[11]);
        rl!(e, a, b, c, d, f2, 0x6ED9EBA1, 7, x[5]);
        rl!(d, e, a, b, c, f2, 0x6ED9EBA1, 5, x[12]);

        // right round 2 (f2, K'=0x6D703EF3), r' = 15,5,1,3,7,14,6,9,11,8,12,2,10,0,4,13
        rl!(d2, e2, a2, b2, c2, f2, 0x6D703EF3, 9, x[15]);
        rl!(c2, d2, e2, a2, b2, f2, 0x6D703EF3, 7, x[5]);
        rl!(b2, c2, d2, e2, a2, f2, 0x6D703EF3, 15, x[1]);
        rl!(a2, b2, c2, d2, e2, f2, 0x6D703EF3, 11, x[3]);
        rl!(e2, a2, b2, c2, d2, f2, 0x6D703EF3, 8, x[7]);
        rl!(d2, e2, a2, b2, c2, f2, 0x6D703EF3, 6, x[14]);
        rl!(c2, d2, e2, a2, b2, f2, 0x6D703EF3, 6, x[6]);
        rl!(b2, c2, d2, e2, a2, f2, 0x6D703EF3, 14, x[9]);
        rl!(a2, b2, c2, d2, e2, f2, 0x6D703EF3, 12, x[11]);
        rl!(e2, a2, b2, c2, d2, f2, 0x6D703EF3, 13, x[8]);
        rl!(d2, e2, a2, b2, c2, f2, 0x6D703EF3, 5, x[12]);
        rl!(c2, d2, e2, a2, b2, f2, 0x6D703EF3, 14, x[2]);
        rl!(b2, c2, d2, e2, a2, f2, 0x6D703EF3, 13, x[10]);
        rl!(a2, b2, c2, d2, e2, f2, 0x6D703EF3, 13, x[0]);
        rl!(e2, a2, b2, c2, d2, f2, 0x6D703EF3, 7, x[4]);
        rl!(d2, e2, a2, b2, c2, f2, 0x6D703EF3, 5, x[13]);

        // left round 3 (f3, K=0x8F1BBCDC), r = 1,9,11,10,0,8,12,4,13,3,7,15,14,5,6,2
        rl!(c, d, e, a, b, f3, 0x8F1BBCDC, 11, x[1]);
        rl!(b, c, d, e, a, f3, 0x8F1BBCDC, 12, x[9]);
        rl!(a, b, c, d, e, f3, 0x8F1BBCDC, 14, x[11]);
        rl!(e, a, b, c, d, f3, 0x8F1BBCDC, 15, x[10]);
        rl!(d, e, a, b, c, f3, 0x8F1BBCDC, 14, x[0]);
        rl!(c, d, e, a, b, f3, 0x8F1BBCDC, 15, x[8]);
        rl!(b, c, d, e, a, f3, 0x8F1BBCDC, 9, x[12]);
        rl!(a, b, c, d, e, f3, 0x8F1BBCDC, 8, x[4]);
        rl!(e, a, b, c, d, f3, 0x8F1BBCDC, 9, x[13]);
        rl!(d, e, a, b, c, f3, 0x8F1BBCDC, 14, x[3]);
        rl!(c, d, e, a, b, f3, 0x8F1BBCDC, 5, x[7]);
        rl!(b, c, d, e, a, f3, 0x8F1BBCDC, 6, x[15]);
        rl!(a, b, c, d, e, f3, 0x8F1BBCDC, 8, x[14]);
        rl!(e, a, b, c, d, f3, 0x8F1BBCDC, 6, x[5]);
        rl!(d, e, a, b, c, f3, 0x8F1BBCDC, 5, x[6]);
        rl!(c, d, e, a, b, f3, 0x8F1BBCDC, 12, x[2]);

        // right round 3 (f1, K'=0x7A6D76E9), r' = 8,6,4,1,3,11,15,0,5,12,2,13,9,7,10,14
        rl!(c2, d2, e2, a2, b2, f1, 0x7A6D76E9, 15, x[8]);
        rl!(b2, c2, d2, e2, a2, f1, 0x7A6D76E9, 5, x[6]);
        rl!(a2, b2, c2, d2, e2, f1, 0x7A6D76E9, 8, x[4]);
        rl!(e2, a2, b2, c2, d2, f1, 0x7A6D76E9, 11, x[1]);
        rl!(d2, e2, a2, b2, c2, f1, 0x7A6D76E9, 14, x[3]);
        rl!(c2, d2, e2, a2, b2, f1, 0x7A6D76E9, 14, x[11]);
        rl!(b2, c2, d2, e2, a2, f1, 0x7A6D76E9, 6, x[15]);
        rl!(a2, b2, c2, d2, e2, f1, 0x7A6D76E9, 14, x[0]);
        rl!(e2, a2, b2, c2, d2, f1, 0x7A6D76E9, 6, x[5]);
        rl!(d2, e2, a2, b2, c2, f1, 0x7A6D76E9, 9, x[12]);
        rl!(c2, d2, e2, a2, b2, f1, 0x7A6D76E9, 12, x[2]);
        rl!(b2, c2, d2, e2, a2, f1, 0x7A6D76E9, 9, x[13]);
        rl!(a2, b2, c2, d2, e2, f1, 0x7A6D76E9, 12, x[9]);
        rl!(e2, a2, b2, c2, d2, f1, 0x7A6D76E9, 5, x[7]);
        rl!(d2, e2, a2, b2, c2, f1, 0x7A6D76E9, 15, x[10]);
        rl!(c2, d2, e2, a2, b2, f1, 0x7A6D76E9, 8, x[14]);

        // left round 4 (f4, K=0xA953FD4E), r = 4,0,5,9,7,12,2,10,14,1,3,8,11,6,15,13
        rl!(b, c, d, e, a, f4, 0xA953FD4E, 9, x[4]);
        rl!(a, b, c, d, e, f4, 0xA953FD4E, 15, x[0]);
        rl!(e, a, b, c, d, f4, 0xA953FD4E, 5, x[5]);
        rl!(d, e, a, b, c, f4, 0xA953FD4E, 11, x[9]);
        rl!(c, d, e, a, b, f4, 0xA953FD4E, 6, x[7]);
        rl!(b, c, d, e, a, f4, 0xA953FD4E, 8, x[12]);
        rl!(a, b, c, d, e, f4, 0xA953FD4E, 13, x[2]);
        rl!(e, a, b, c, d, f4, 0xA953FD4E, 12, x[10]);
        rl!(d, e, a, b, c, f4, 0xA953FD4E, 5, x[14]);
        rl!(c, d, e, a, b, f4, 0xA953FD4E, 12, x[1]);
        rl!(b, c, d, e, a, f4, 0xA953FD4E, 13, x[3]);
        rl!(a, b, c, d, e, f4, 0xA953FD4E, 14, x[8]);
        rl!(e, a, b, c, d, f4, 0xA953FD4E, 11, x[11]);
        rl!(d, e, a, b, c, f4, 0xA953FD4E, 8, x[6]);
        rl!(c, d, e, a, b, f4, 0xA953FD4E, 5, x[15]);
        rl!(b, c, d, e, a, f4, 0xA953FD4E, 6, x[13]);

        // right round 4 (f0, K'=0), r' = 12,15,10,4,1,5,8,7,6,2,13,14,0,3,9,11
        rl!(b2, c2, d2, e2, a2, f0, 0x00000000, 8, x[12]);
        rl!(a2, b2, c2, d2, e2, f0, 0x00000000, 5, x[15]);
        rl!(e2, a2, b2, c2, d2, f0, 0x00000000, 12, x[10]);
        rl!(d2, e2, a2, b2, c2, f0, 0x00000000, 9, x[4]);
        rl!(c2, d2, e2, a2, b2, f0, 0x00000000, 12, x[1]);
        rl!(b2, c2, d2, e2, a2, f0, 0x00000000, 5, x[5]);
        rl!(a2, b2, c2, d2, e2, f0, 0x00000000, 14, x[8]);
        rl!(e2, a2, b2, c2, d2, f0, 0x00000000, 6, x[7]);
        rl!(d2, e2, a2, b2, c2, f0, 0x00000000, 8, x[6]);
        rl!(c2, d2, e2, a2, b2, f0, 0x00000000, 13, x[2]);
        rl!(b2, c2, d2, e2, a2, f0, 0x00000000, 6, x[13]);
        rl!(a2, b2, c2, d2, e2, f0, 0x00000000, 5, x[14]);
        rl!(e2, a2, b2, c2, d2, f0, 0x00000000, 15, x[0]);
        rl!(d2, e2, a2, b2, c2, f0, 0x00000000, 13, x[3]);
        rl!(c2, d2, e2, a2, b2, f0, 0x00000000, 11, x[9]);
        rl!(b2, c2, d2, e2, a2, f0, 0x00000000, 11, x[11]);

    let t = h[1].wrapping_add(c).wrapping_add(d2);
    h[1] = h[2].wrapping_add(d).wrapping_add(e2);
    h[2] = h[3].wrapping_add(e).wrapping_add(a2);
    h[3] = h[4].wrapping_add(a).wrapping_add(b2);
    h[4] = h[0].wrapping_add(b).wrapping_add(c2);
    h[0] = t;
}

pub fn ripemd160_opt(data: &[u8]) -> [u8; 20] {
    let mut h: [u32; 5] = [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0];
    let bits = (data.len() as u64) * 8;
    let mut block = [0u8; 64];
    let mut pos = 0usize;
    loop {
        block.fill(0);
        let n = data.len() - pos;
        if n >= 64 {
            block.copy_from_slice(&data[pos..pos + 64]);
            pos += 64;
            compress_ripemd(&mut h, &block);
            continue;
        }
        block[..n].copy_from_slice(&data[pos..]);
        block[n] = 0x80;
        if n >= 56 {
            compress_ripemd(&mut h, &block);
            block.fill(0);
        }
        block[56..64].copy_from_slice(&bits.to_le_bytes());
        compress_ripemd(&mut h, &block);
        break;
    }
    let mut out = [0u8; 20];
    for i in 0..5 {
        out[i * 4..i * 4 + 4].copy_from_slice(&h[i].to_le_bytes());
    }
    out
}

pub fn hash160_fast(compressed: &[u8]) -> [u8; 20] {
    let h1 = sha2::Sha256::digest(compressed);
    ripemd160_opt(&h1)
}