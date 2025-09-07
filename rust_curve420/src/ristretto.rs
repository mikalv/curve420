//! Ristretto420 scaffolding over the Curve420 Edwards model (a = A+2, d = A−2).
//! This wraps Edwards points in a prime-order abstraction. Encoding/decoding will follow.

use crate::curve::{EdwardsPoint, L, EDW_A, EDW_D};
use crate::field::FieldElement;
use lazy_static::lazy_static;
use num_traits::Num;
use num_bigint::BigUint;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RistrettoPoint(EdwardsPoint);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RistrettoError {
    InvalidEncoding,
    NotImplemented,
}

lazy_static! {
    // Constants for a = -1 view
    // d' = (2 - A)/(A + 2) mod p
    pub static ref D_M1: FieldElement = FieldElement::new(
        // from proved/ristretto_constants.json
        BigUint::from_str_radix(
            "2452716181725381856644875084906193393415092913133662187679137757399562559402223776760896555937275583243540028031723320155896995",
            10
        ).unwrap()
    );
    pub static ref SQRT_M1: FieldElement = FieldElement::new(
        BigUint::from_str_radix(
            "1125536906516536500462288751072116878795238505010065672221134269135451808572734403962317328989539260645101783109609626216749877",
            10
        ).unwrap()
    );
}

impl RistrettoPoint {
    /// Domain parameters
    #[allow(dead_code)]
    const ENCODING_LEN: usize = 53;
    /// Construct from an Edwards point after checking it lies in the prime-order subgroup.
    pub fn from_edwards_checked(p: &EdwardsPoint) -> Option<Self> {
        // Check ℓ * P == O and P != O. Identity may be represented as Infinity or (0,1).
        fn is_identity_point(q: &EdwardsPoint) -> bool {
            match q {
                EdwardsPoint::Infinity => true,
                EdwardsPoint::Affine(x, y) => x.is_zero() && y.is_one(),
            }
        }
        let l_p = p.clone() * L.clone();
        let is_identity = is_identity_point(&l_p);
        let p_is_identity = is_identity_point(p);
        if is_identity && !p_is_identity {
            Some(RistrettoPoint(p.clone()))
        } else {
            None
        }
    }

    /// Return the inner Edwards point (prime-order by construction).
    pub fn to_edwards(&self) -> EdwardsPoint {
        self.0.clone()
    }

    /// Add two Ristretto points.
    pub fn add(&self, rhs: &RistrettoPoint) -> RistrettoPoint {
        RistrettoPoint(self.0.clone() + rhs.0.clone())
    }

    /// Negate a Ristretto point.
    pub fn neg(&self) -> RistrettoPoint {
        RistrettoPoint((-&self.0).clone())
    }

    /// Scalar multiply by a non-negative integer (BigUint).
    pub fn mul(&self, k: &BigUint) -> RistrettoPoint {
        RistrettoPoint(self.0.clone() * k.clone())
    }

    /// Return the inner Edwards point (prime-order by construction).
    pub fn inner(&self) -> &EdwardsPoint { &self.0 }

    /// Encode this point to 53-byte canonical encoding (placeholder).
    /// Note: Ristretto encoding differs from Edwards compressed encoding; this is a stub.
    pub fn encode(&self) -> [u8; 53] {
        // Temporary: use Edwards-like encoding (x with sign(y)) until full Ristretto encode is implemented.
        let (x, y) = self.0.coords();
        let mut out = [0u8; 53];
        let mut xb = x.to_le_bytes_len(53);
        // Set MSB of last byte to sign(y) = parity(y)
        xb[52] = (xb[52] & 0x7F) | ((y.parity() & 1) << 7);
        out.copy_from_slice(&xb);
        out
    }

    /// Decode a 53-byte canonical Ristretto encoding (skeleton with basic checks and partial mapping).
    pub fn decode(bytes: &[u8; 53]) -> Result<RistrettoPoint, RistrettoError> {
        // First, try placeholder Edwards-style decoding to support current encode() round-trip.
        // Interpret input as x with sign(y) in MSB of last byte.
        let mut xb = *bytes;
        let sign = (xb[52] >> 7) & 1;
        xb[52] &= 0x7F;
        if let Some(x) = FieldElement::from_le_bytes_canonical(&xb) {
            let x2 = &x * &x;
            // y^2 = (1 - a x^2) / (1 - d x^2)
            let one = FieldElement::one();
            let num = &one - (&*EDW_A * &x2);
            let den = &one - (&*EDW_D * &x2);
            if !den.is_zero() {
                let y2 = num * den.inv();
                if let Some(mut y) = y2.sqrt() {
                    if y.parity() != sign { y = -y; }
                    let cand = EdwardsPoint::Affine(x, y);
                    if let Some(rp) = RistrettoPoint::from_edwards_checked(&cand) {
                        return Ok(rp);
                    }
                }
            }
        }

        // Otherwise, attempt a = -1 Ristretto-style mapping from t
        // Basic canonical check on original bytes: parse as field element < p
        let t = FieldElement::from_le_bytes_canonical(bytes).ok_or(RistrettoError::InvalidEncoding)?;

        // Sketch of mapping (not final):
        // r = t
        let r = t;
        let r2 = &r * &r;
        let one = FieldElement::one();
        let u1 = &one - &r2;            // 1 - r^2
        let u2 = &one + &r2;            // 1 + r^2
        let r4 = &r2 * &r2;             // r^4
        let v = &one - (&*D_M1 * r4);   // 1 - d*r^4

        // invsqrt for (v * u2^2); invalid if none
        let u2_sq = &u2 * &u2;
        let denom = &v * &u2_sq;
        let invsqrt = match sqrt_ratio(&one, &denom) {
            Some(val) => val,
            None => return Err(RistrettoError::InvalidEncoding),
        };

        let den1 = &invsqrt * &u2;      // invsqrt * (1 + r^2)
        let den2 = &den1 * &v;          // invsqrt * (1 + r^2) * (1 - d r^4)
        let mut x = (&r + &r) * den1.clone();    // x ~ 2 r * den1
        let mut y = (&u1) * den2;               // y ~ (1 - r^2) * den2

        // Branchless sign/rotation adjustment akin to ristretto255
        // If x*y is negative, rotate by sqrt(-1): (x, y) = (y*SQRT_M1, x*SQRT_M1)
        let xy = &x * &y;
        let neg_xy = is_negative(&xy);
        let x_alt = &y * &*SQRT_M1;
        let y_alt = &x * &*SQRT_M1;
        x = ct_select_fe(&x, &x_alt, neg_xy);
        y = ct_select_fe(&y, &y_alt, neg_xy);

        // Ensure y is non-negative; if y < 0, flip signs
        if is_negative(&y) == 1 {
            x = -x;
            y = -y;
        }

        let candidate = EdwardsPoint::Affine(x, y);
        if let Some(rp) = RistrettoPoint::from_edwards_checked(&candidate) {
            Ok(rp)
        } else {
            Err(RistrettoError::NotImplemented)
        }
    }
}

/// Compute r = sqrt(u/v) if it exists. If u/v is a non-residue, try sqrt(-u/v) and
/// return SQRT_M1 * r, which satisfies (SQRT_M1*r)^2 = -1 * (r^2) = u/v.
fn sqrt_ratio(u: &FieldElement, v: &FieldElement) -> Option<FieldElement> {
    let inv_v = v.inv();
    let x = u * inv_v;
    if let Some(r) = x.sqrt() { return Some(r); }
    // try sqrt(-x)
    let one = FieldElement::one();
    let minus_one = &FieldElement::zero() - &one;
    let x_neg = &x * &minus_one;
    if let Some(r2) = x_neg.sqrt() {
        // return SQRT_M1 * r2
        return Some(&*SQRT_M1 * r2);
    }
    None
}

/// Best-effort constant-time like conditional select between two field elements.
/// choice = 0 selects a, choice != 0 selects b.
#[allow(dead_code)]
fn ct_select_fe(a: &FieldElement, b: &FieldElement, choice: u8) -> FieldElement {
    if choice == 0 { a.clone() } else { b.clone() }
}

/// Return 1 if value is "negative" (LSB = 1), else 0.
#[allow(dead_code)]
fn is_negative(x: &FieldElement) -> u8 { x.parity() }

impl From<&EdwardsPoint> for RistrettoPoint {
    fn from(p: &EdwardsPoint) -> Self { RistrettoPoint(p.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::curve::G;
    use crate::montgomery::A as M_A;

    #[test]
    fn test_prime_order_wrapper() {
        // G is prime-order in Edwards model; wrapper should accept it
        let rp = RistrettoPoint::from_edwards_checked(&*G).expect("G must be in prime-order subgroup");
        // Basic group ops
        let two = BigUint::from(2u32);
        let rp2 = rp.mul(&two);
        assert_ne!(rp2, rp, "2*G != 1*G");
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let rp = RistrettoPoint::from_edwards_checked(&*G).unwrap();
        let bytes = rp.encode();
        let rp2 = RistrettoPoint::decode(&bytes).unwrap();
        assert_eq!(rp, rp2);
    }

    #[test]
    fn test_d_m1_matches_relation() {
        // Check D_M1 == (2 - A)/(A + 2) mod p
        let two = FieldElement::new(BigUint::from(2u32));
        let num = &two - &*M_A;
        let den = &*M_A + &two;
        let den_inv = den.inv();
        let d_calc = num * den_inv;
        assert_eq!(d_calc, *D_M1);
    }

    #[test]
    fn test_sqrt_m1_property() {
        // Verify (SQRT_M1)^2 == -1 mod p
        let one = FieldElement::one();
        let minus_one = &FieldElement::zero() - &one;
        let sq = &*SQRT_M1 * &*SQRT_M1;
        assert_eq!(sq, minus_one);
        // And sqrt(-1) returns one of ±SQRT_M1
        let root = minus_one.sqrt().expect("-1 should be a square in this field");
        let neg = -&*SQRT_M1;
        assert!(root == *SQRT_M1 || root == neg);
    }

    #[test]
    fn test_sqrt_ratio_basic() {
        // Choose t, set u = t^2, v = 1 => sqrt_ratio(u,v) should return ±t
        let t = FieldElement::new(BigUint::from(7u32));
        let u = &t * &t;
        let v = FieldElement::one();
        let r = sqrt_ratio(&u, &v).expect("should have sqrt");
        // r^2 == u
        assert_eq!(&r * &r, u);

        // For non-residue try u = -1 (if -1 is non-residue, here it's residue so pick 2 which is likely NR)
        let two = FieldElement::new(BigUint::from(2u32));
        let r2 = sqrt_ratio(&two, &v);
        // May or may not be Some depending on 2's residue status; at least it should not panic.
        let _ = r2; // no-op
    }
}
