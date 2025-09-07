use num_bigint::BigUint;
use num_traits::{Num, One, Zero};
use std::ops::{Add, Sub, Mul, Neg};

lazy_static::lazy_static! {
    // p = 2^420 - 335
    static ref P: BigUint = BigUint::from_str_radix("2707685248164858261307045101702230179137145581421695874189921465443966120903931272499975005961073806735733604454495675614232241", 10).unwrap();
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldElement(pub BigUint);

// --- Constructors and Helpers ---
impl FieldElement {
    pub fn new(n: BigUint) -> Self { FieldElement(n % &*P) }
    pub fn from_hex(hex: &str) -> Self { FieldElement(BigUint::from_str_radix(hex, 16).unwrap()) }
    pub fn zero() -> Self { FieldElement(BigUint::zero()) }
    pub fn one() -> Self { FieldElement(BigUint::one()) }
    pub fn inv(&self) -> Self {
        let p_minus_2 = &*P - BigUint::from(2u32);
        FieldElement(self.0.modpow(&p_minus_2, &*P))
    }
    pub fn to_biguint(&self) -> BigUint { self.0.clone() }
    pub fn is_zero(&self) -> bool { self.0.is_zero() }
    pub fn is_one(&self) -> bool { self.0.is_one() }
    /// Exponentiate in the field: self^e mod p
    pub fn pow_big(&self, e: &BigUint) -> Self { FieldElement(self.0.modpow(e, &*P)) }
    /// Legendre symbol (self|p): returns 0 if 0, 1 if quadratic residue, -1 otherwise
    pub fn legendre(&self) -> i8 {
        if self.is_zero() { return 0; }
        let exp = (&*P - BigUint::one()) >> 1; // (p-1)/2
        let r = self.0.modpow(&exp, &*P);
        if r.is_zero() { 0 }
        else if r == BigUint::one() { 1 } else { -1 }
    }

    /// Square root using Tonelli–Shanks. Returns Some(root) if self is a square, else None.
    pub fn sqrt(&self) -> Option<Self> {
        if self.is_zero() { return Some(FieldElement::zero()); }
        // Check Legendre symbol == 1
        if self.legendre() != 1 { return None; }

        // Factor p-1 = q * 2^s with q odd
        let mut q = &*P - BigUint::one();
        let mut s: u32 = 0;
        while (&q & BigUint::one()) == BigUint::zero() {
            q >>= 1;
            s += 1;
        }

        // Find a quadratic non-residue z
        let mut z_val = BigUint::from(2u32);
        let z = loop {
            let z_fe = FieldElement::new(z_val.clone());
            if z_fe.legendre() == -1 { break z_fe; }
            z_val += BigUint::one();
        };

        // c = z^q, t = a^q, r = a^{(q+1)/2}
        let c = z.pow_big(&q);
        let t = self.pow_big(&q);
        let mut r = self.pow_big(&((&q + BigUint::one()) >> 1));
        let mut t_cur = t;
        let mut c_cur = c;
        let mut m = s;

        while !t_cur.is_one() {
            // Find smallest i in [1, m) such that t^{2^i} == 1
            let mut i = 1u32;
            let mut t2i = t_cur.clone();
            while i < m {
                t2i = &t2i * &t2i; // square
                if t2i.is_one() { break; }
                i += 1;
            }
            // b = c^{2^{m-i-1}}
            let mut e = m - i - 1;
            let mut b = c_cur.clone();
            while e > 0 {
                b = &b * &b;
                e -= 1;
            }
            // Update r, t, c, m
            r = r * b.clone();
            let b2 = &b * &b;
            t_cur = t_cur * b2.clone();
            c_cur = b2;
            m = i;
        }
        Some(r)
    }

    /// Parse a little-endian byte slice into a canonical field element (< p).
    pub fn from_le_bytes_canonical(bytes: &[u8]) -> Option<Self> {
        let n = BigUint::from_bytes_le(bytes);
        if &n < &*P { Some(FieldElement(n)) } else { None }
    }

    /// Serialize to little-endian with fixed length (pads/truncates as needed).
    pub fn to_le_bytes_len(&self, len: usize) -> Vec<u8> {
        let mut b = self.0.to_bytes_le();
        b.resize(len, 0u8);
        b
    }

    /// Constant-time like equality (best-effort; BigUint isn't strictly constant-time)
    pub fn ct_eq(&self, other: &Self) -> bool { self.0 == other.0 }

    /// Parity bit (least significant bit) of the canonical representative
    pub fn parity(&self) -> u8 { self.0.bit(0) as u8 }
}

// --- Trait Implementations ---

// Helper for safe modular subtraction
fn sub_mod(a: &BigUint, b: &BigUint, p: &BigUint) -> BigUint {
    if a >= b {
        a - b
    } else {
        a + p - b
    }
}

// Negation
impl Neg for FieldElement {
    type Output = Self;
    fn neg(self) -> Self { FieldElement(sub_mod(&P, &self.0, &P)) }
}
impl Neg for &FieldElement {
    type Output = FieldElement;
    fn neg(self) -> FieldElement { FieldElement(sub_mod(&P, &self.0, &P)) }
}

// Addition
impl Add for FieldElement {
    type Output = Self;
    fn add(self, rhs: Self) -> Self { FieldElement((self.0 + rhs.0) % &*P) }
}
impl Add<&FieldElement> for FieldElement {
    type Output = FieldElement;
    fn add(self, rhs: &FieldElement) -> FieldElement { FieldElement((self.0 + &rhs.0) % &*P) }
}
impl Add<FieldElement> for &FieldElement {
    type Output = FieldElement;
    fn add(self, rhs: FieldElement) -> FieldElement { FieldElement((&self.0 + rhs.0) % &*P) }
}
impl Add<&FieldElement> for &FieldElement {
    type Output = FieldElement;
    fn add(self, rhs: &FieldElement) -> FieldElement { FieldElement((&self.0 + &rhs.0) % &*P) }
}

// Subtraction
impl Sub for FieldElement {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self { FieldElement(sub_mod(&self.0, &rhs.0, &P)) }
}
impl Sub<&FieldElement> for FieldElement {
    type Output = FieldElement;
    fn sub(self, rhs: &FieldElement) -> FieldElement { FieldElement(sub_mod(&self.0, &rhs.0, &P)) }
}
impl Sub<FieldElement> for &FieldElement {
    type Output = FieldElement;
    fn sub(self, rhs: FieldElement) -> FieldElement { FieldElement(sub_mod(&self.0, &rhs.0, &P)) }
}
impl Sub<&FieldElement> for &FieldElement {
    type Output = FieldElement;
    fn sub(self, rhs: &FieldElement) -> FieldElement { FieldElement(sub_mod(&self.0, &rhs.0, &P)) }
}

// Multiplication
impl Mul for FieldElement {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self { FieldElement((self.0 * rhs.0) % &*P) }
}
impl Mul<&FieldElement> for FieldElement {
    type Output = FieldElement;
    fn mul(self, rhs: &FieldElement) -> FieldElement { FieldElement((self.0 * &rhs.0) % &*P) }
}
impl Mul<FieldElement> for &FieldElement {
    type Output = FieldElement;
    fn mul(self, rhs: FieldElement) -> FieldElement { FieldElement((&self.0 * rhs.0) % &*P) }
}
impl Mul<&FieldElement> for &FieldElement {
    type Output = FieldElement;
    fn mul(self, rhs: &FieldElement) -> FieldElement { FieldElement((&self.0 * &rhs.0) % &*P) }
}
