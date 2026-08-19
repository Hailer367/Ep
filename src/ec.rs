use sha2::{Digest, Sha256};

pub const BECH32_CHARSET: &[u8] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";

pub const P: [u64; 4] = [
    0xFFFFFFFEFFFFFC2F,
    0xFFFFFFFFFFFFFFFF,
    0xFFFFFFFFFFFFFFFF,
    0xFFFFFFFFFFFFFFFF,
];

pub const GX: [u64; 4] = [
    0x59F2815B16F81798,
    0x029BFCDB2DCE28D9,
    0x55A06295CE870B07,
    0x79BE667EF9DCBBAC,
];

pub const GY: [u64; 4] = [
    0x9C47D08FFB10D4B8,
    0xFD17B448A6855419,
    0x5DA4FBFC0E1108A8,
    0x483ADA7726A3C465,
];

pub const CK: u64 = 0x00000001000003D1;

pub type Fe = [u64; 4];

pub const ONE: Fe = [1, 0, 0, 0];

pub const INFINITY: Jacobian = Jacobian {
    x: [0; 4],
    y: [0; 4],
    z: [0; 4],
};

#[derive(Clone, Copy)]
pub struct Jacobian {
    pub x: Fe,
    pub y: Fe,
    pub z: Fe,
}

pub fn scalar_from_u64(n: u64) -> Fe {
    [n, 0, 0, 0]
}

fn add256(a: &[u64; 4], b: &[u64; 4]) -> [u64; 4] {
    let mut r = [0u64; 4];
    let mut carry = 0u64;
    for i in 0..4 {
        let s = (a[i] as u128) + (b[i] as u128) + (carry as u128);
        r[i] = s as u64;
        carry = (s >> 64) as u64;
    }
    r
}

fn sub256(a: &[u64; 4], b: &[u64; 4]) -> ([u64; 4], bool) {
    let mut r = [0u64; 4];
    let mut borrow = 0u64;
    for i in 0..4 {
        let s = (a[i] as u128)
            .wrapping_sub(b[i] as u128)
            .wrapping_sub(borrow as u128);
        r[i] = s as u64;
        borrow = ((s >> 64) != 0) as u64;
    }
    (r, borrow != 0)
}

fn add_at(r: &mut [u64; 4], off: usize, v: u64) {
    let mut carry = v as u128;
    for k in off..4 {
        let t = (r[k] as u128) + carry;
        r[k] = t as u64;
        carry = t >> 64;
        if carry == 0 {
            return;
        }
    }
    if carry != 0 {
        let fold = carry * (CK as u128);
        add_at(r, off, fold as u64);
        let hi = (fold >> 64) as u64;
        if hi != 0 {
            add_at(r, off + 1, hi);
        }
    }
}

fn add_mod(a: &[u64; 4], b: &[u64; 4]) -> [u64; 4] {
    let mut r = [0u64; 4];
    let mut carry = 0u64;
    for i in 0..4 {
        let s = (a[i] as u128) + (b[i] as u128) + (carry as u128);
        r[i] = s as u64;
        carry = (s >> 64) as u64;
    }
    if carry != 0 {
        add_at(&mut r, 0, CK);
    }
    if ge_p(&r) {
        r = sub256(&r, &P).0;
    }
    r
}

fn sub_mod(a: &[u64; 4], b: &[u64; 4]) -> [u64; 4] {
    let (t, borrow) = sub256(a, b);
    if borrow {
        add256(&t, &P)
    } else {
        t
    }
}

fn ge_p(a: &[u64; 4]) -> bool {
    for i in (0..4).rev() {
        if a[i] != P[i] {
            return a[i] > P[i];
        }
    }
    true
}

fn mul_raw(a: &[u64; 4], b: &[u64; 4]) -> [u64; 8] {
    let mut t = [0u64; 8];
    for i in 0..4 {
        let mut carry = 0u128;
        for j in 0..4 {
            let cur = (a[i] as u128) * (b[j] as u128) + (t[i + j] as u128) + carry;
            t[i + j] = cur as u64;
            carry = cur >> 64;
        }
        let mut idx = i + 4;
        while carry != 0 {
            let cur = (t[idx] as u128) + carry;
            t[idx] = cur as u64;
            carry = cur >> 64;
            idx += 1;
        }
    }
    t
}

fn add_const(acc: &mut [u64; 5], off: usize, v: u64) {
    let mut carry = v as u128;
    for k in off..5 {
        let t = (acc[k] as u128) + carry;
        acc[k] = t as u64;
        carry = t >> 64;
        if carry == 0 {
            return;
        }
    }
    if carry != 0 {
        let fold = carry * (CK as u128);
        add_const(acc, off, fold as u64);
        let hi = (fold >> 64) as u64;
        if hi != 0 {
            add_const(acc, off + 1, hi);
        }
    }
}

fn reduce_mod_p(acc: &mut [u64; 5]) {
    while acc[4] != 0 {
        let a4 = acc[4];
        acc[4] = 0;
        let prod = (a4 as u128) * (CK as u128);
        add_const(acc, 0, prod as u64);
        let hi = (prod >> 64) as u64;
        if hi != 0 {
            add_const(acc, 1, hi);
        }
    }
    if ge_p(&[acc[0], acc[1], acc[2], acc[3]]) {
        let s = sub256(&[acc[0], acc[1], acc[2], acc[3]], &P).0;
        acc[..4].copy_from_slice(&s);
    }
}

pub fn mul_mod(a: &[u64; 4], b: &[u64; 4]) -> [u64; 4] {
    let raw = mul_raw(a, b);
    let mut acc = [raw[0], raw[1], raw[2], raw[3], 0u64];
    for i in 0..4 {
        let h = raw[4 + i];
        if h == 0 {
            continue;
        }
        let prod = (h as u128) * (CK as u128);
        add_const(&mut acc, i, prod as u64);
        let hi = (prod >> 64) as u64;
        if hi != 0 {
            add_const(&mut acc, i + 1, hi);
        }
    }
    reduce_mod_p(&mut acc);
    [acc[0], acc[1], acc[2], acc[3]]
}

pub fn sqr_mod(a: &[u64; 4]) -> [u64; 4] {
    mul_mod(a, a)
}

pub fn inv_mod(a: &[u64; 4]) -> [u64; 4] {
    let mut r = [1u64, 0, 0, 0];
    let mut base = *a;
    let p_minus_2: [u64; 4] = [
        0xFFFFFFFEFFFFFC2D,
        0xFFFFFFFFFFFFFFFF,
        0xFFFFFFFFFFFFFFFF,
        0xFFFFFFFFFFFFFFFF,
    ];
    for word in p_minus_2.iter() {
        for i in 0..64 {
            if (word >> i) & 1 == 1 {
                r = mul_mod(&r, &base);
            }
            base = sqr_mod(&base);
        }
    }
    r
}

pub fn batch_invert(zs: &mut [Fe]) {
    let n = zs.len();
    if n == 0 {
        return;
    }
    if n == 1 {
        zs[0] = inv_mod(&zs[0]);
        return;
    }
    let mut prefix = Vec::with_capacity(n);
    let mut acc = zs[0];
    prefix.push(acc);
    for i in 1..n {
        acc = mul_mod(&acc, &zs[i]);
        prefix.push(acc);
    }
    let mut inv = inv_mod(&acc);
    for i in (0..n).rev() {
        let zi = zs[i];
        let pprev = if i == 0 { ONE } else { prefix[i - 1] };
        zs[i] = mul_mod(&inv, &pprev);
        inv = mul_mod(&inv, &zi);
    }
}

fn is_zero(a: &[u64; 4]) -> bool {
    (a[0] | a[1] | a[2] | a[3]) == 0
}

fn point_double(p: &Jacobian) -> Jacobian {
    if is_zero(&p.z) {
        return INFINITY;
    }
    let x = p.x;
    let y = p.y;
    let z = p.z;

    let a = sqr_mod(&x);
    let b = sqr_mod(&y);
    let c = sqr_mod(&b);
    let t = sqr_mod(&add_mod(&x, &b));
    let t = sub_mod(&t, &a);
    let t = sub_mod(&t, &c);
    let d = add_mod(&t, &t);
    let t = add_mod(&a, &a);
    let e = add_mod(&t, &a);
    let f = sqr_mod(&e);

    let t = add_mod(&d, &d);
    let x3 = sub_mod(&f, &t);

    let t = sub_mod(&d, &x3);
    let t = mul_mod(&e, &t);
    let mut eight_c = c;
    eight_c = add_mod(&eight_c, &c);
    eight_c = add_mod(&eight_c, &eight_c);
    eight_c = add_mod(&eight_c, &eight_c);
    let y3 = sub_mod(&t, &eight_c);

    let t = mul_mod(&y, &z);
    let z3 = add_mod(&t, &t);

    Jacobian { x: x3, y: y3, z: z3 }
}

pub fn point_add(p: &Jacobian, gx: &[u64; 4], gy: &[u64; 4]) -> Jacobian {
    if is_zero(&p.z) {
        return Jacobian {
            x: *gx,
            y: *gy,
            z: ONE,
        };
    }
    let z1z1 = sqr_mod(&p.z);
    let u2 = mul_mod(gx, &z1z1);
    let s2 = mul_mod(gy, &mul_mod(&p.z, &z1z1));

    let h = sub_mod(&u2, &p.x);
    let r = sub_mod(&s2, &p.y);

    if is_zero(&h) {
        if is_zero(&r) {
            return point_double(p);
        }
        return INFINITY;
    }

    let hh = sqr_mod(&h);
    let hhh = mul_mod(&h, &hh);
    let v = mul_mod(&p.x, &hh);

    let t = sub_mod(&sqr_mod(&r), &hhh);
    let t = sub_mod(&t, &add_mod(&v, &v));
    let x3 = t;

    let t = sub_mod(&v, &x3);
    let t = mul_mod(&r, &t);
    let y3 = sub_mod(&t, &mul_mod(&p.y, &hhh));

    let z3 = mul_mod(&p.z, &h);

    Jacobian { x: x3, y: y3, z: z3 }
}

pub fn scalar_mult(s: &[u64; 4], gx: &[u64; 4], gy: &[u64; 4]) -> Jacobian {
    let mut acc = INFINITY;
    for i in (0..256).rev() {
        acc = point_double(&acc);
        let bit = (s[i / 64] >> (i % 64)) & 1;
        if bit == 1 {
            acc = point_add(&acc, gx, gy);
        }
    }
    acc
}

pub fn to_compressed(p: &Jacobian) -> [u8; 33] {
    let zi = inv_mod(&p.z);
    to_compressed_inv(p, &zi)
}

pub fn to_compressed_inv(p: &Jacobian, zi: &Fe) -> [u8; 33] {
    let zinv2 = mul_mod(zi, zi);
    let x = mul_mod(&p.x, &zinv2);
    let zinv3 = mul_mod(&zinv2, zi);
    let y = mul_mod(&p.y, &zinv3);
    let prefix = if y[0] & 1 == 1 { 0x03 } else { 0x02 };
    let mut out = [0u8; 33];
    out[0] = prefix;
    for i in 0..4 {
        out[1 + i * 8..1 + i * 8 + 8].copy_from_slice(&x[3 - i].to_be_bytes());
    }
    out
}

pub fn hash160_from_compressed(compressed: &[u8]) -> [u8; 20] {
    let h1 = Sha256::digest(compressed);
    let mut h1_bytes = [0u8; 32];
    h1_bytes.copy_from_slice(&h1);

    let mut hasher = ripemd::Ripemd160::new();
    hasher.update(&h1_bytes);
    let h2 = hasher.finalize();

    let mut out = [0u8; 20];
    out.copy_from_slice(&h2);
    out
}

pub fn hash160_of_n(n: u64) -> [u8; 20] {
    let s = scalar_from_u64(n);
    let p = scalar_mult(&s, &GX, &GY);
    let comp = to_compressed(&p);
    hash160_from_compressed(&comp)
}

pub fn bech32_polymod(values: &[u8]) -> u32 {
    let gen: [u32; 5] = [0x3b6a57b2, 0x26508e6d, 0x1ea119fa, 0x3d4233dd, 0x2a1462b3];
    let mut chk: u32 = 1;
    for v in values {
        let b = (chk >> 25) as u8;
        chk = (chk & 0x1ffffff) << 5 ^ (*v as u32);
        for i in 0..5 {
            if (b >> i) & 1 == 1 {
                chk ^= gen[i];
            }
        }
    }
    chk
}

fn bech32_hrp_expand(hrp: &[u8]) -> Vec<u8> {
    let mut e = Vec::with_capacity(hrp.len() * 2 + 1);
    for &b in hrp {
        e.push(b >> 5);
    }
    e.push(0);
    for &b in hrp {
        e.push(b & 31);
    }
    e
}

fn bech32_create_checksum(hrp: &[u8], data: &[u8]) -> [u8; 6] {
    let mut values = bech32_hrp_expand(hrp);
    values.extend_from_slice(data);
    values.extend_from_slice(&[0u8; 6]);
    let poly = bech32_polymod(&values) ^ 1;
    let mut checksum = [0u8; 6];
    for i in 0..6 {
        checksum[i] = ((poly >> 5 * (5 - i)) & 31) as u8;
    }
    checksum
}

fn bech32_encode(hrp: &[u8], data: &[u8]) -> String {
    let checksum = bech32_create_checksum(hrp, data);
    let mut combined = data.to_vec();
    combined.extend_from_slice(&checksum);
    let mut out = String::with_capacity(hrp.len() + 1 + combined.len() + 6);
    out.push_str(&String::from_utf8_lossy(hrp));
    out.push('1');
    for d in &combined {
        out.push(BECH32_CHARSET[*d as usize] as char);
    }
    out
}

fn convert_bits(data: &[u8], from_bits: u32, to_bits: u32, pad: bool) -> Result<Vec<u8>, ()> {
    let mut acc: u32 = 0;
    let mut bits: u32 = 0;
    let mut out = Vec::new();
    let maxv = (1u32 << to_bits) - 1;
    for &value in data {
        let v = value as u32;
        if (v >> from_bits) != 0 {
            return Err(());
        }
        acc = (acc << from_bits) | v;
        bits += from_bits;
        while bits >= to_bits {
            bits -= to_bits;
            out.push(((acc >> bits) & maxv) as u8);
        }
    }
    if pad {
        if bits > 0 {
            out.push(((acc << (to_bits - bits)) & maxv) as u8);
        }
    } else if bits >= from_bits || ((acc << (to_bits - bits)) & maxv) != 0 {
        return Err(());
    }
    Ok(out)
}

pub fn bech32_address(h160: &[u8; 20]) -> String {
    let mut data = vec![0u8]; // witness version 0
    data.extend_from_slice(&convert_bits(h160, 8, 5, true).unwrap());
    bech32_encode(b"bc", &data)
}

pub fn decode_bech32_address(addr: &str) -> Option<[u8; 20]> {
    let bytes = addr.as_bytes();
    if bytes.len() < 8 {
        return None;
    }
    let has_upper = bytes.iter().any(|b| b.is_ascii_uppercase());
    let has_lower = bytes.iter().any(|b| b.is_ascii_lowercase());
    if has_upper && has_lower {
        return None;
    }
    let lower = addr.to_ascii_lowercase();
    let b = lower.as_bytes();
    let pos = b.iter().rposition(|&c| c == b'1')?;
    if pos == 0 {
        return None;
    }
    let hrp = &lower[..pos];
    let data = &lower[pos + 1..];
    if data.len() < 6 {
        return None;
    }
    let mut values = Vec::with_capacity(data.len());
    for &c in data.as_bytes() {
        let v = BECH32_CHARSET.iter().position(|&x| x == c)? as u8;
        values.push(v);
    }
    let mut exp = bech32_hrp_expand(hrp.as_bytes());
    exp.extend_from_slice(&values);
    if bech32_polymod(&exp) != 1 {
        return None;
    }
    let payload = &values[..values.len() - 6];
    if payload.is_empty() {
        return None;
    }
    if payload[0] != 0 {
        return None;
    }
    let program = convert_bits(&payload[1..], 5, 8, false).ok()?;
    if program.len() != 20 {
        return None;
    }
    let mut h160 = [0u8; 20];
    h160.copy_from_slice(&program);
    Some(h160)
}
