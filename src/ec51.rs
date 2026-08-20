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
    let mut acc = ONE;
    let mut zeros = 0usize;
    for i in 0..n {
        prefix.push(acc);
        if zs[i][0] | zs[i][1] | zs[i][2] | zs[i][3] | zs[i][4] == 0 {
            zeros += 1;
        } else {
            acc = fe_mul(&acc, &zs[i]);
        }
    }
    if zeros > 1 {
        for z in zs.iter_mut() {
            *z = [0; 5];
        }
        return;
    }
    let mut inv = fe_inv(&acc);
    for i in (0..n).rev() {
        if zs[i][0] | zs[i][1] | zs[i][2] | zs[i][3] | zs[i][4] == 0 {
            zs[i] = [0; 5];
        } else {
            let zi = zs[i];
            let pprev = prefix[i];
            zs[i] = fe_mul(&inv, &pprev);
            inv = fe_mul(&inv, &zi);
        }
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

pub fn compress_ripemd(h: &mut [u32; 5], block: &[u8; 64]) {
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

// ---- optimized single-block hash160 path (33-byte compressed -> 20-byte hash160) ----

const K32X4: [[u32; 4]; 16] = [
    [0xe9b5dba5, 0xb5c0fbcf, 0x71374491, 0x428a2f98],
    [0xab1c5ed5, 0x923f82a4, 0x59f111f1, 0x3956c25b],
    [0x550c7dc3, 0x243185be, 0x12835b01, 0xd807aa98],
    [0xc19bf174, 0x9bdc06a7, 0x80deb1fe, 0x72be5d74],
    [0x240ca1cc, 0x0fc19dc6, 0xefbe4786, 0xe49b69c1],
    [0x76f988da, 0x5cb0a9dc, 0x4a7484aa, 0x2de92c6f],
    [0xbf597fc7, 0xb00327c8, 0xa831c66d, 0x983e5152],
    [0x14292967, 0x06ca6351, 0xd5a79147, 0xc6e00bf3],
    [0x53380d13, 0x4d2c6dfc, 0x2e1b2138, 0x27b70a85],
    [0x92722c85, 0x81c2c92e, 0x766a0abb, 0x650a7354],
    [0xc76c51a3, 0xc24b8b70, 0xa81a664b, 0xa2bfe8a1],
    [0x106aa070, 0xf40e3585, 0xd6990624, 0xd192e819],
    [0x34b0bcb5, 0x2748774c, 0x1e376c08, 0x19a4c116],
    [0x682e6ff3, 0x5b9cca4f, 0x4ed8aa4a, 0x391c0cb3],
    [0x8cc70208, 0x84c87814, 0x78a5636f, 0x748f82ee],
    [0xc67178f2, 0xbef9a3f7, 0xa4506ceb, 0x90befffa],
];

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn sha_schedule(v0: core::arch::x86_64::__m128i, v1: core::arch::x86_64::__m128i, v2: core::arch::x86_64::__m128i, v3: core::arch::x86_64::__m128i) -> core::arch::x86_64::__m128i {
    use core::arch::x86_64::*;
    let t1 = _mm_sha256msg1_epu32(v0, v1);
    let t2 = _mm_alignr_epi8(v3, v2, 4);
    let t3 = _mm_add_epi32(t1, t2);
    _mm_sha256msg2_epu32(t3, v3)
}

#[cfg(target_arch = "x86_64")]
macro_rules! sha_rounds4 {
    ($abef:ident, $cdgh:ident, $rest:expr, $i:expr) => {{
        let k = K32X4[$i];
        let kv = core::arch::x86_64::_mm_set_epi32(k[0] as i32, k[1] as i32, k[2] as i32, k[3] as i32);
        let t1 = core::arch::x86_64::_mm_add_epi32($rest, kv);
        $cdgh = core::arch::x86_64::_mm_sha256rnds2_epu32($cdgh, $abef, t1);
        let t2 = core::arch::x86_64::_mm_shuffle_epi32(t1, 0x0E);
        $abef = core::arch::x86_64::_mm_sha256rnds2_epu32($abef, $cdgh, t2);
    }};
}

#[cfg(target_arch = "x86_64")]
macro_rules! sha_schedule_rounds4 {
    ($abef:ident, $cdgh:ident, $w0:expr, $w1:expr, $w2:expr, $w3:expr, $w4:expr, $i:expr) => {{
        $w4 = sha_schedule($w0, $w1, $w2, $w3);
        sha_rounds4!($abef, $cdgh, $w4, $i);
    }};
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sha,sse2,ssse3,sse4.1")]
unsafe fn sha256_block_ni(state: &mut [u32; 8], block: &[u8; 64]) {
    use core::arch::x86_64::*;
    let mask: __m128i = _mm_set_epi64x(
        0x0C0D_0E0F_0809_0A0Bu64 as i64,
        0x0405_0607_0001_0203u64 as i64,
    );
    let state_ptr = state.as_ptr() as *const __m128i;
    let dcba = _mm_loadu_si128(state_ptr.add(0));
    let efgh = _mm_loadu_si128(state_ptr.add(1));
    let cdab = _mm_shuffle_epi32(dcba, 0xB1);
    let efgh = _mm_shuffle_epi32(efgh, 0x1B);
    let mut abef = _mm_alignr_epi8(cdab, efgh, 8);
    let mut cdgh = _mm_blend_epi16(efgh, cdab, 0xF0);
    let abef_save = abef;
    let cdgh_save = cdgh;

    let data_ptr = block.as_ptr() as *const __m128i;
    let mut w0 = _mm_shuffle_epi8(_mm_loadu_si128(data_ptr.add(0)), mask);
    let mut w1 = _mm_shuffle_epi8(_mm_loadu_si128(data_ptr.add(1)), mask);
    let mut w2 = _mm_shuffle_epi8(_mm_loadu_si128(data_ptr.add(2)), mask);
    let mut w3 = _mm_shuffle_epi8(_mm_loadu_si128(data_ptr.add(3)), mask);
    let mut w4;

    sha_rounds4!(abef, cdgh, w0, 0);
    sha_rounds4!(abef, cdgh, w1, 1);
    sha_rounds4!(abef, cdgh, w2, 2);
    sha_rounds4!(abef, cdgh, w3, 3);
    sha_schedule_rounds4!(abef, cdgh, w0, w1, w2, w3, w4, 4);
    sha_schedule_rounds4!(abef, cdgh, w1, w2, w3, w4, w0, 5);
    sha_schedule_rounds4!(abef, cdgh, w2, w3, w4, w0, w1, 6);
    sha_schedule_rounds4!(abef, cdgh, w3, w4, w0, w1, w2, 7);
    sha_schedule_rounds4!(abef, cdgh, w4, w0, w1, w2, w3, 8);
    sha_schedule_rounds4!(abef, cdgh, w0, w1, w2, w3, w4, 9);
    sha_schedule_rounds4!(abef, cdgh, w1, w2, w3, w4, w0, 10);
    sha_schedule_rounds4!(abef, cdgh, w2, w3, w4, w0, w1, 11);
    sha_schedule_rounds4!(abef, cdgh, w3, w4, w0, w1, w2, 12);
    sha_schedule_rounds4!(abef, cdgh, w4, w0, w1, w2, w3, 13);
    sha_schedule_rounds4!(abef, cdgh, w0, w1, w2, w3, w4, 14);
    sha_schedule_rounds4!(abef, cdgh, w1, w2, w3, w4, w0, 15);

    abef = _mm_add_epi32(abef, abef_save);
    cdgh = _mm_add_epi32(cdgh, cdgh_save);

    let feba = _mm_shuffle_epi32(abef, 0x1B);
    let dchg = _mm_shuffle_epi32(cdgh, 0xB1);
    let dcba = _mm_blend_epi16(feba, dchg, 0xF0);
    let hgef = _mm_alignr_epi8(dchg, feba, 8);
    let state_ptr_mut = state.as_mut_ptr() as *mut __m128i;
    _mm_storeu_si128(state_ptr_mut.add(0), dcba);
    _mm_storeu_si128(state_ptr_mut.add(1), hgef);
}

const SHA256_IV: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

// SHA-256 of a 33-byte message (one block: 33 + 0x80 + zeros + len 264).
pub fn sha256_of_33(inp: &[u8; 33]) -> [u8; 32] {
    let mut block = [0u8; 64];
    block[..33].copy_from_slice(inp);
    block[33] = 0x80;
    block[56..64].copy_from_slice(&264u64.to_be_bytes());
    let mut state = SHA256_IV;
    #[cfg(target_arch = "x86_64")]
    if std::arch::is_x86_feature_detected!("sha") {
        unsafe {
            sha256_block_ni(&mut state, &block);
        }
        let mut out = [0u8; 32];
        for i in 0..8 {
            out[i * 4..i * 4 + 4].copy_from_slice(&state[i].to_be_bytes());
        }
        return out;
    }
    let d = sha2::Sha256::digest(&block);
    let mut out = [0u8; 32];
    out.copy_from_slice(&d);
    out
}

// RIPEMD-160 of a 32-byte message (one block: 32 + 0x80 + zeros + len 256).
pub fn ripemd160_of_32(b32: &[u8]) -> [u8; 20] {
    let mut block = [0u8; 64];
    block[..32].copy_from_slice(b32);
    block[32] = 0x80;
    block[56..64].copy_from_slice(&256u64.to_le_bytes());
    let mut h = [0x67452301u32, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0];
    compress_ripemd(&mut h, &block);
    let mut out = [0u8; 20];
    for i in 0..5 {
        out[i * 4..i * 4 + 4].copy_from_slice(&h[i].to_le_bytes());
    }
    out
}

// Fast hash160 of a 33-byte compressed pubkey: single-block sha256 (SHA-NI) + single-block ripemd.
pub fn hash160_fast33(comp: &[u8; 33]) -> [u8; 20] {
    let h1 = sha256_of_33(comp);
    ripemd160_of_32(&h1)
}

// Convert a Jacobian point to affine (x, y) using one inversion.
pub fn to_affine(p: &Jacobian51) -> (Fe51, Fe51) {
    let zi = fe_inv(&p.z);
    let zinv2 = fe_mul(&zi, &zi);
    let x = fe_mul(&p.x, &zinv2);
    let zinv3 = fe_mul(&zinv2, &zi);
    let y = fe_mul(&p.y, &zinv3);
    (x, y)
}// ---- AVX2 8-lane RIPEMD-160 (processes 8 independent 32-byte messages) ----

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn ripemd_compress_8x(
    xv: &[core::arch::x86_64::__m256i; 16],
) -> [core::arch::x86_64::__m256i; 5] {
    use core::arch::x86_64::*;
    let all = _mm256_set1_epi32(u32::MAX as i32);
    macro_rules! f0v {
        ($b:expr, $c:expr, $d:expr) => {
            _mm256_xor_si256(_mm256_xor_si256($b, $c), $d)
        };
    }
    macro_rules! f1v {
        ($b:expr, $c:expr, $d:expr) => {
            _mm256_or_si256(_mm256_and_si256($b, $c), _mm256_andnot_si256($b, $d))
        };
    }
    macro_rules! f2v {
        ($b:expr, $c:expr, $d:expr) => {
            _mm256_xor_si256(
                _mm256_or_si256($b, _mm256_andnot_si256($c, all)),
                $d,
            )
        };
    }
    macro_rules! f3v {
        ($b:expr, $c:expr, $d:expr) => {
            _mm256_or_si256(_mm256_and_si256($b, $d), _mm256_andnot_si256($d, $c))
        };
    }
    macro_rules! f4v {
        ($b:expr, $c:expr, $d:expr) => {
            _mm256_xor_si256(
                $b,
                _mm256_or_si256($c, _mm256_andnot_si256($d, all)),
            )
        };
    }
    macro_rules! rolv {
        ($x:expr, $s:expr) => {
            _mm256_or_si256(_mm256_slli_epi32($x, $s), _mm256_srli_epi32($x, 32 - $s))
        };
    }
    macro_rules! rl_s {
        ($a:ident, $b:ident, $c:ident, $d:ident, $e:ident, $f:ident, $k:expr, $s:expr, $xi:expr) => {{
            let t = _mm256_add_epi32(
                _mm256_add_epi32(
                    _mm256_add_epi32($a, $f!($b, $c, $d)),
                    xv[$xi],
                ),
                _mm256_set1_epi32($k as u32 as i32),
            );
            $a = _mm256_add_epi32(rolv!(t, $s), $e);
            $c = rolv!($c, 10);
        }};
    }

    let iv0 = _mm256_set1_epi32(0x67452301 as u32 as i32);
    let iv1 = _mm256_set1_epi32(0xEFCDAB89 as u32 as i32);
    let iv2 = _mm256_set1_epi32(0x98BADCFE as u32 as i32);
    let iv3 = _mm256_set1_epi32(0x10325476 as u32 as i32);
    let iv4 = _mm256_set1_epi32(0xC3D2E1F0 as u32 as i32);

    let (mut a, mut b, mut c, mut d, mut e) = (iv0, iv1, iv2, iv3, iv4);
    let (mut a2, mut b2, mut c2, mut d2, mut e2) = (iv0, iv1, iv2, iv3, iv4);

    rl_s!(a, b, c, d, e, f0v, 0x00000000, 11, 0);
    rl_s!(e, a, b, c, d, f0v, 0x00000000, 14, 1);
    rl_s!(d, e, a, b, c, f0v, 0x00000000, 15, 2);
    rl_s!(c, d, e, a, b, f0v, 0x00000000, 12, 3);
    rl_s!(b, c, d, e, a, f0v, 0x00000000, 5, 4);
    rl_s!(a, b, c, d, e, f0v, 0x00000000, 8, 5);
    rl_s!(e, a, b, c, d, f0v, 0x00000000, 7, 6);
    rl_s!(d, e, a, b, c, f0v, 0x00000000, 9, 7);
    rl_s!(c, d, e, a, b, f0v, 0x00000000, 11, 8);
    rl_s!(b, c, d, e, a, f0v, 0x00000000, 13, 9);
    rl_s!(a, b, c, d, e, f0v, 0x00000000, 14, 10);
    rl_s!(e, a, b, c, d, f0v, 0x00000000, 15, 11);
    rl_s!(d, e, a, b, c, f0v, 0x00000000, 6, 12);
    rl_s!(c, d, e, a, b, f0v, 0x00000000, 7, 13);
    rl_s!(b, c, d, e, a, f0v, 0x00000000, 9, 14);
    rl_s!(a, b, c, d, e, f0v, 0x00000000, 8, 15);

    rl_s!(a2, b2, c2, d2, e2, f4v, 0x50A28BE6, 8, 5);
    rl_s!(e2, a2, b2, c2, d2, f4v, 0x50A28BE6, 9, 14);
    rl_s!(d2, e2, a2, b2, c2, f4v, 0x50A28BE6, 9, 7);
    rl_s!(c2, d2, e2, a2, b2, f4v, 0x50A28BE6, 11, 0);
    rl_s!(b2, c2, d2, e2, a2, f4v, 0x50A28BE6, 13, 9);
    rl_s!(a2, b2, c2, d2, e2, f4v, 0x50A28BE6, 15, 2);
    rl_s!(e2, a2, b2, c2, d2, f4v, 0x50A28BE6, 15, 11);
    rl_s!(d2, e2, a2, b2, c2, f4v, 0x50A28BE6, 5, 4);
    rl_s!(c2, d2, e2, a2, b2, f4v, 0x50A28BE6, 7, 13);
    rl_s!(b2, c2, d2, e2, a2, f4v, 0x50A28BE6, 7, 6);
    rl_s!(a2, b2, c2, d2, e2, f4v, 0x50A28BE6, 8, 15);
    rl_s!(e2, a2, b2, c2, d2, f4v, 0x50A28BE6, 11, 8);
    rl_s!(d2, e2, a2, b2, c2, f4v, 0x50A28BE6, 14, 1);
    rl_s!(c2, d2, e2, a2, b2, f4v, 0x50A28BE6, 14, 10);
    rl_s!(b2, c2, d2, e2, a2, f4v, 0x50A28BE6, 12, 3);
    rl_s!(a2, b2, c2, d2, e2, f4v, 0x50A28BE6, 6, 12);

    rl_s!(e, a, b, c, d, f1v, 0x5A827999, 7, 7);
    rl_s!(d, e, a, b, c, f1v, 0x5A827999, 6, 4);
    rl_s!(c, d, e, a, b, f1v, 0x5A827999, 8, 13);
    rl_s!(b, c, d, e, a, f1v, 0x5A827999, 13, 1);
    rl_s!(a, b, c, d, e, f1v, 0x5A827999, 11, 10);
    rl_s!(e, a, b, c, d, f1v, 0x5A827999, 9, 6);
    rl_s!(d, e, a, b, c, f1v, 0x5A827999, 7, 15);
    rl_s!(c, d, e, a, b, f1v, 0x5A827999, 15, 3);
    rl_s!(b, c, d, e, a, f1v, 0x5A827999, 7, 12);
    rl_s!(a, b, c, d, e, f1v, 0x5A827999, 12, 0);
    rl_s!(e, a, b, c, d, f1v, 0x5A827999, 15, 9);
    rl_s!(d, e, a, b, c, f1v, 0x5A827999, 9, 5);
    rl_s!(c, d, e, a, b, f1v, 0x5A827999, 11, 2);
    rl_s!(b, c, d, e, a, f1v, 0x5A827999, 7, 14);
    rl_s!(a, b, c, d, e, f1v, 0x5A827999, 13, 11);
    rl_s!(e, a, b, c, d, f1v, 0x5A827999, 12, 8);

    rl_s!(e2, a2, b2, c2, d2, f3v, 0x5C4DD124, 9, 6);
    rl_s!(d2, e2, a2, b2, c2, f3v, 0x5C4DD124, 13, 11);
    rl_s!(c2, d2, e2, a2, b2, f3v, 0x5C4DD124, 15, 3);
    rl_s!(b2, c2, d2, e2, a2, f3v, 0x5C4DD124, 7, 7);
    rl_s!(a2, b2, c2, d2, e2, f3v, 0x5C4DD124, 12, 0);
    rl_s!(e2, a2, b2, c2, d2, f3v, 0x5C4DD124, 8, 13);
    rl_s!(d2, e2, a2, b2, c2, f3v, 0x5C4DD124, 9, 5);
    rl_s!(c2, d2, e2, a2, b2, f3v, 0x5C4DD124, 11, 10);
    rl_s!(b2, c2, d2, e2, a2, f3v, 0x5C4DD124, 7, 14);
    rl_s!(a2, b2, c2, d2, e2, f3v, 0x5C4DD124, 7, 15);
    rl_s!(e2, a2, b2, c2, d2, f3v, 0x5C4DD124, 12, 8);
    rl_s!(d2, e2, a2, b2, c2, f3v, 0x5C4DD124, 7, 12);
    rl_s!(c2, d2, e2, a2, b2, f3v, 0x5C4DD124, 6, 4);
    rl_s!(b2, c2, d2, e2, a2, f3v, 0x5C4DD124, 15, 9);
    rl_s!(a2, b2, c2, d2, e2, f3v, 0x5C4DD124, 13, 1);
    rl_s!(e2, a2, b2, c2, d2, f3v, 0x5C4DD124, 11, 2);

    rl_s!(d, e, a, b, c, f2v, 0x6ED9EBA1, 11, 3);
    rl_s!(c, d, e, a, b, f2v, 0x6ED9EBA1, 13, 10);
    rl_s!(b, c, d, e, a, f2v, 0x6ED9EBA1, 6, 14);
    rl_s!(a, b, c, d, e, f2v, 0x6ED9EBA1, 7, 4);
    rl_s!(e, a, b, c, d, f2v, 0x6ED9EBA1, 14, 9);
    rl_s!(d, e, a, b, c, f2v, 0x6ED9EBA1, 9, 15);
    rl_s!(c, d, e, a, b, f2v, 0x6ED9EBA1, 13, 8);
    rl_s!(b, c, d, e, a, f2v, 0x6ED9EBA1, 15, 1);
    rl_s!(a, b, c, d, e, f2v, 0x6ED9EBA1, 14, 2);
    rl_s!(e, a, b, c, d, f2v, 0x6ED9EBA1, 8, 7);
    rl_s!(d, e, a, b, c, f2v, 0x6ED9EBA1, 13, 0);
    rl_s!(c, d, e, a, b, f2v, 0x6ED9EBA1, 6, 6);
    rl_s!(b, c, d, e, a, f2v, 0x6ED9EBA1, 5, 13);
    rl_s!(a, b, c, d, e, f2v, 0x6ED9EBA1, 12, 11);
    rl_s!(e, a, b, c, d, f2v, 0x6ED9EBA1, 7, 5);
    rl_s!(d, e, a, b, c, f2v, 0x6ED9EBA1, 5, 12);

    rl_s!(d2, e2, a2, b2, c2, f2v, 0x6D703EF3, 9, 15);
    rl_s!(c2, d2, e2, a2, b2, f2v, 0x6D703EF3, 7, 5);
    rl_s!(b2, c2, d2, e2, a2, f2v, 0x6D703EF3, 15, 1);
    rl_s!(a2, b2, c2, d2, e2, f2v, 0x6D703EF3, 11, 3);
    rl_s!(e2, a2, b2, c2, d2, f2v, 0x6D703EF3, 8, 7);
    rl_s!(d2, e2, a2, b2, c2, f2v, 0x6D703EF3, 6, 14);
    rl_s!(c2, d2, e2, a2, b2, f2v, 0x6D703EF3, 6, 6);
    rl_s!(b2, c2, d2, e2, a2, f2v, 0x6D703EF3, 14, 9);
    rl_s!(a2, b2, c2, d2, e2, f2v, 0x6D703EF3, 12, 11);
    rl_s!(e2, a2, b2, c2, d2, f2v, 0x6D703EF3, 13, 8);
    rl_s!(d2, e2, a2, b2, c2, f2v, 0x6D703EF3, 5, 12);
    rl_s!(c2, d2, e2, a2, b2, f2v, 0x6D703EF3, 14, 2);
    rl_s!(b2, c2, d2, e2, a2, f2v, 0x6D703EF3, 13, 10);
    rl_s!(a2, b2, c2, d2, e2, f2v, 0x6D703EF3, 13, 0);
    rl_s!(e2, a2, b2, c2, d2, f2v, 0x6D703EF3, 7, 4);
    rl_s!(d2, e2, a2, b2, c2, f2v, 0x6D703EF3, 5, 13);

    rl_s!(c, d, e, a, b, f3v, 0x8F1BBCDC, 11, 1);
    rl_s!(b, c, d, e, a, f3v, 0x8F1BBCDC, 12, 9);
    rl_s!(a, b, c, d, e, f3v, 0x8F1BBCDC, 14, 11);
    rl_s!(e, a, b, c, d, f3v, 0x8F1BBCDC, 15, 10);
    rl_s!(d, e, a, b, c, f3v, 0x8F1BBCDC, 14, 0);
    rl_s!(c, d, e, a, b, f3v, 0x8F1BBCDC, 15, 8);
    rl_s!(b, c, d, e, a, f3v, 0x8F1BBCDC, 9, 12);
    rl_s!(a, b, c, d, e, f3v, 0x8F1BBCDC, 8, 4);
    rl_s!(e, a, b, c, d, f3v, 0x8F1BBCDC, 9, 13);
    rl_s!(d, e, a, b, c, f3v, 0x8F1BBCDC, 14, 3);
    rl_s!(c, d, e, a, b, f3v, 0x8F1BBCDC, 5, 7);
    rl_s!(b, c, d, e, a, f3v, 0x8F1BBCDC, 6, 15);
    rl_s!(a, b, c, d, e, f3v, 0x8F1BBCDC, 8, 14);
    rl_s!(e, a, b, c, d, f3v, 0x8F1BBCDC, 6, 5);
    rl_s!(d, e, a, b, c, f3v, 0x8F1BBCDC, 5, 6);
    rl_s!(c, d, e, a, b, f3v, 0x8F1BBCDC, 12, 2);

    rl_s!(c2, d2, e2, a2, b2, f1v, 0x7A6D76E9, 15, 8);
    rl_s!(b2, c2, d2, e2, a2, f1v, 0x7A6D76E9, 5, 6);
    rl_s!(a2, b2, c2, d2, e2, f1v, 0x7A6D76E9, 8, 4);
    rl_s!(e2, a2, b2, c2, d2, f1v, 0x7A6D76E9, 11, 1);
    rl_s!(d2, e2, a2, b2, c2, f1v, 0x7A6D76E9, 14, 3);
    rl_s!(c2, d2, e2, a2, b2, f1v, 0x7A6D76E9, 14, 11);
    rl_s!(b2, c2, d2, e2, a2, f1v, 0x7A6D76E9, 6, 15);
    rl_s!(a2, b2, c2, d2, e2, f1v, 0x7A6D76E9, 14, 0);
    rl_s!(e2, a2, b2, c2, d2, f1v, 0x7A6D76E9, 6, 5);
    rl_s!(d2, e2, a2, b2, c2, f1v, 0x7A6D76E9, 9, 12);
    rl_s!(c2, d2, e2, a2, b2, f1v, 0x7A6D76E9, 12, 2);
    rl_s!(b2, c2, d2, e2, a2, f1v, 0x7A6D76E9, 9, 13);
    rl_s!(a2, b2, c2, d2, e2, f1v, 0x7A6D76E9, 12, 9);
    rl_s!(e2, a2, b2, c2, d2, f1v, 0x7A6D76E9, 5, 7);
    rl_s!(d2, e2, a2, b2, c2, f1v, 0x7A6D76E9, 15, 10);
    rl_s!(c2, d2, e2, a2, b2, f1v, 0x7A6D76E9, 8, 14);

    rl_s!(b, c, d, e, a, f4v, 0xA953FD4E, 9, 4);
    rl_s!(a, b, c, d, e, f4v, 0xA953FD4E, 15, 0);
    rl_s!(e, a, b, c, d, f4v, 0xA953FD4E, 5, 5);
    rl_s!(d, e, a, b, c, f4v, 0xA953FD4E, 11, 9);
    rl_s!(c, d, e, a, b, f4v, 0xA953FD4E, 6, 7);
    rl_s!(b, c, d, e, a, f4v, 0xA953FD4E, 8, 12);
    rl_s!(a, b, c, d, e, f4v, 0xA953FD4E, 13, 2);
    rl_s!(e, a, b, c, d, f4v, 0xA953FD4E, 12, 10);
    rl_s!(d, e, a, b, c, f4v, 0xA953FD4E, 5, 14);
    rl_s!(c, d, e, a, b, f4v, 0xA953FD4E, 12, 1);
    rl_s!(b, c, d, e, a, f4v, 0xA953FD4E, 13, 3);
    rl_s!(a, b, c, d, e, f4v, 0xA953FD4E, 14, 8);
    rl_s!(e, a, b, c, d, f4v, 0xA953FD4E, 11, 11);
    rl_s!(d, e, a, b, c, f4v, 0xA953FD4E, 8, 6);
    rl_s!(c, d, e, a, b, f4v, 0xA953FD4E, 5, 15);
    rl_s!(b, c, d, e, a, f4v, 0xA953FD4E, 6, 13);

    rl_s!(b2, c2, d2, e2, a2, f0v, 0x00000000, 8, 12);
    rl_s!(a2, b2, c2, d2, e2, f0v, 0x00000000, 5, 15);
    rl_s!(e2, a2, b2, c2, d2, f0v, 0x00000000, 12, 10);
    rl_s!(d2, e2, a2, b2, c2, f0v, 0x00000000, 9, 4);
    rl_s!(c2, d2, e2, a2, b2, f0v, 0x00000000, 12, 1);
    rl_s!(b2, c2, d2, e2, a2, f0v, 0x00000000, 5, 5);
    rl_s!(a2, b2, c2, d2, e2, f0v, 0x00000000, 14, 8);
    rl_s!(e2, a2, b2, c2, d2, f0v, 0x00000000, 6, 7);
    rl_s!(d2, e2, a2, b2, c2, f0v, 0x00000000, 8, 6);
    rl_s!(c2, d2, e2, a2, b2, f0v, 0x00000000, 13, 2);
    rl_s!(b2, c2, d2, e2, a2, f0v, 0x00000000, 6, 13);
    rl_s!(a2, b2, c2, d2, e2, f0v, 0x00000000, 5, 14);
    rl_s!(e2, a2, b2, c2, d2, f0v, 0x00000000, 15, 0);
    rl_s!(d2, e2, a2, b2, c2, f0v, 0x00000000, 13, 3);
    rl_s!(c2, d2, e2, a2, b2, f0v, 0x00000000, 11, 9);
    rl_s!(b2, c2, d2, e2, a2, f0v, 0x00000000, 11, 11);

    let t = _mm256_add_epi32(_mm256_add_epi32(iv1, c), d2);
    let h0 = t;
    let h1 = _mm256_add_epi32(_mm256_add_epi32(iv2, d), e2);
    let h2 = _mm256_add_epi32(_mm256_add_epi32(iv3, e), a2);
    let h3 = _mm256_add_epi32(_mm256_add_epi32(iv4, a), b2);
    let h4 = _mm256_add_epi32(_mm256_add_epi32(iv0, b), c2);
    [h0, h1, h2, h3, h4]
}

// Build the 16 SoA message-word vectors for 8 single-block (32-byte) messages.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn ripemd_xv_8x(shas: &[[u8; 32]; 8]) -> [core::arch::x86_64::__m256i; 16] {
    use core::arch::x86_64::*;
    let l0 = _mm256_loadu_si256(shas[0].as_ptr() as *const __m256i);
    let l1 = _mm256_loadu_si256(shas[1].as_ptr() as *const __m256i);
    let l2 = _mm256_loadu_si256(shas[2].as_ptr() as *const __m256i);
    let l3 = _mm256_loadu_si256(shas[3].as_ptr() as *const __m256i);
    let l4 = _mm256_loadu_si256(shas[4].as_ptr() as *const __m256i);
    let l5 = _mm256_loadu_si256(shas[5].as_ptr() as *const __m256i);
    let l6 = _mm256_loadu_si256(shas[6].as_ptr() as *const __m256i);
    let l7 = _mm256_loadu_si256(shas[7].as_ptr() as *const __m256i);

    let t0 = _mm256_unpacklo_epi32(l0, l1);
    let t1 = _mm256_unpackhi_epi32(l0, l1);
    let t2 = _mm256_unpacklo_epi32(l2, l3);
    let t3 = _mm256_unpackhi_epi32(l2, l3);
    let t4 = _mm256_unpacklo_epi32(l4, l5);
    let t5 = _mm256_unpackhi_epi32(l4, l5);
    let t6 = _mm256_unpacklo_epi32(l6, l7);
    let t7 = _mm256_unpackhi_epi32(l6, l7);

    let u0 = _mm256_unpacklo_epi64(t0, t2);
    let u1 = _mm256_unpackhi_epi64(t0, t2);
    let u2 = _mm256_unpacklo_epi64(t1, t3);
    let u3 = _mm256_unpackhi_epi64(t1, t3);
    let u4 = _mm256_unpacklo_epi64(t4, t6);
    let u5 = _mm256_unpackhi_epi64(t4, t6);
    let u6 = _mm256_unpacklo_epi64(t5, t7);
    let u7 = _mm256_unpackhi_epi64(t5, t7);

    let mut xv: [__m256i; 16] = [_mm256_setzero_si256(); 16];
    xv[0] = _mm256_permute2x128_si256(u0, u4, 0x20);
    xv[1] = _mm256_permute2x128_si256(u1, u5, 0x20);
    xv[2] = _mm256_permute2x128_si256(u2, u6, 0x20);
    xv[3] = _mm256_permute2x128_si256(u3, u7, 0x20);
    xv[4] = _mm256_permute2x128_si256(u0, u4, 0x31);
    xv[5] = _mm256_permute2x128_si256(u1, u5, 0x31);
    xv[6] = _mm256_permute2x128_si256(u2, u6, 0x31);
    xv[7] = _mm256_permute2x128_si256(u3, u7, 0x31);
    xv[8] = _mm256_set1_epi32(0x80);
    xv[14] = _mm256_set1_epi32(256);
    xv
}

// RIPEMD-160 of 8 independent 32-byte messages (single block each, from IV).
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn ripemd160_of_32_8x_avx2(shas: &[[u8; 32]; 8]) -> [[u8; 20]; 8] {
    use core::arch::x86_64::*;
    let xv = ripemd_xv_8x(shas);
    let hv = ripemd_compress_8x(&xv);
    let mut tmp = [[0u8; 32]; 5];
    for k in 0..5 {
        _mm256_storeu_si256(tmp[k].as_mut_ptr() as *mut __m256i, hv[k]);
    }
    let mut out = [[0u8; 20]; 8];
    for j in 0..8 {
        for k in 0..5 {
            out[j][k * 4..k * 4 + 4].copy_from_slice(&tmp[k][j * 4..j * 4 + 4]);
        }
    }
    out
}

// hash160 of 8 compressed pubkeys: scalar SHA-256 (SHA-NI) + AVX2 RIPEMD, fallback to scalar.
pub fn hash160_fast33_8x(comps8: &[[u8; 33]; 8]) -> [[u8; 20]; 8] {
    #[cfg(target_arch = "x86_64")]
    if std::arch::is_x86_feature_detected!("avx2") {
        let mut shas = [[0u8; 32]; 8];
        for i in 0..8 {
            shas[i] = sha256_of_33(&comps8[i]);
        }
        return unsafe { ripemd160_of_32_8x_avx2(&shas) };
    }
    let mut out = [[0u8; 20]; 8];
    for i in 0..8 {
        out[i] = hash160_fast33(&comps8[i]);
    }
    out
}// Debug: expose transposed xv vectors as scalars.
pub fn xv_debug(shas: &[[u8; 32]; 8]) -> [[u32; 16]; 8] {
    #[cfg(target_arch = "x86_64")]
    if std::arch::is_x86_feature_detected!("avx2") {
        use core::arch::x86_64::*;
        unsafe {
            let xv = ripemd_xv_8x(shas);
            let mut out = [[0u32; 16]; 8];
            for k in 0..16 {
                let mut tmp = [0u8; 32];
                _mm256_storeu_si256(tmp.as_mut_ptr() as *mut __m256i, xv[k]);
                for j in 0..8 {
                    out[j][k] = u32::from_le_bytes([tmp[j * 4], tmp[j * 4 + 1], tmp[j * 4 + 2], tmp[j * 4 + 3]]);
                }
            }
            return out;
        }
    }
    let mut out = [[0u32; 16]; 8];
    for j in 0..8 {
        for k in 0..8 {
            out[j][k] = u32::from_le_bytes([shas[j][k * 4], shas[j][k * 4 + 1], shas[j][k * 4 + 2], shas[j][k * 4 + 3]]);
        }
        out[j][8] = 0x80;
        out[j][15] = 256;
    }
    out
}// RIPEMD-160 of 8 independent 32-byte messages (AVX2 path, fallback scalar).
pub fn ripemd160_of_32_8x(shas: &[[u8; 32]; 8]) -> [[u8; 20]; 8] {
    #[cfg(target_arch = "x86_64")]
    if std::arch::is_x86_feature_detected!("avx2") {
        return unsafe { ripemd160_of_32_8x_avx2(shas) };
    }
    let mut out = [[0u8; 20]; 8];
    for i in 0..8 {
        out[i] = ripemd160_of_32(&shas[i]);
    }
    out
}// Debug: run the AVX2 compress on the xv of 8 messages, return h scalars.
pub fn compress_8x_debug(shas: &[[u8; 32]; 8]) -> [[u32; 5]; 8] {
    #[cfg(target_arch = "x86_64")]
    if std::arch::is_x86_feature_detected!("avx2") {
        use core::arch::x86_64::*;
        unsafe {
            let xv = ripemd_xv_8x(shas);
            let hv = ripemd_compress_8x(&xv);
            let mut out = [[0u32; 5]; 8];
            for k in 0..5 {
                let mut tmp = [0u8; 32];
                _mm256_storeu_si256(tmp.as_mut_ptr() as *mut __m256i, hv[k]);
                for j in 0..8 {
                    out[j][k] = u32::from_le_bytes([tmp[j * 4], tmp[j * 4 + 1], tmp[j * 4 + 2], tmp[j * 4 + 3]]);
                }
            }
            return out;
        }
    }
    [[0u32; 5]; 8]
}