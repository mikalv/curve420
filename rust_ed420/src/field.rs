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
