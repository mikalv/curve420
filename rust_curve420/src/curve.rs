use crate::field::FieldElement;
use num_bigint::BigUint;
use num_traits::{Num, One, Zero};
use std::ops::{Add, Mul, Neg, Sub};

lazy_static::lazy_static! {
    // Edwards parameters (published model): a = A + 2, d = A − 2
    pub static ref EDW_A: FieldElement = FieldElement::new(
        BigUint::from_str_radix(
            "763975519699500577645754547835125169481986463482154078046572648671788968290548038674290307302429817161505744408446033521089604",
            10
        ).unwrap()
    );

    pub static ref EDW_D: FieldElement = FieldElement::new(
        BigUint::from_str_radix(
            "763975519699500577645754547835125169481986463482154078046572648671788968290548038674290307302429817161505744408446033521089600",
            10
        ).unwrap()
    );

    // Edwards base point (x, y)
    pub static ref G: EdwardsPoint = EdwardsPoint::Affine(
        FieldElement::new(BigUint::from_str_radix(
            "2554519045303036994902077297242990796196199161457630080356703041833906288977089421513471756737913123939108844302244613830350009", 10
        ).unwrap()),
        FieldElement::new(BigUint::from_str_radix(
            "1554004282195909523747673681974014268960308454695342458183393593582942692590987497223833263666951454840260505456918987028153736", 10
        ).unwrap()),
    );

    // Prime subgroup order ℓ and cofactor h
    pub static ref L: BigUint = BigUint::from_str_radix(
        "338460656020607282663380637712778772392143197677711984273740183501508577674026655281164768623743539442603492250355597371718719", 10
    ).unwrap();

    pub static ref H: BigUint = BigUint::from(8u32);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EdwardsPoint {
    Infinity, // Represents the neutral element (0, 1)
    Affine(FieldElement, FieldElement),
}

impl EdwardsPoint {
    pub fn coords(&self) -> (FieldElement, FieldElement) {
        match self {
            EdwardsPoint::Affine(x, y) => (x.clone(), y.clone()),
            EdwardsPoint::Infinity => (FieldElement::zero(), FieldElement::one()),
        }
    }
}

// --- Point arithmetic for Twisted Edwards curves ---
// Edwards equation: a*x^2 + y^2 = 1 + d*x^2*y^2

fn point_add(p1: &EdwardsPoint, p2: &EdwardsPoint) -> EdwardsPoint {
    match (p1, p2) {
        (EdwardsPoint::Infinity, p) | (p, EdwardsPoint::Infinity) => p.clone(),
        (EdwardsPoint::Affine(x1, y1), EdwardsPoint::Affine(x2, y2)) => {
            // (x3, y3) = ((x1*y2 + y1*x2) / (1 + d*x1*x2*y1*y2),
            //             (y1*y2 - a*x1*x2) / (1 - d*x1*x2*y1*y2))

            // Intermediates
            let x1y2 = x1 * y2;
            let y1x2 = y1 * x2;
            let x1x2 = x1 * x2;
            let y1y2 = y1 * y2;

            let dxxyy = &*EDW_D * &x1x2 * &y1y2;

            // x3
            let x3_num = &x1y2 + &y1x2;
            let x3_den = FieldElement::one() + &dxxyy;
            let x3 = x3_num * x3_den.inv();

            // y3
            let y3_num = &y1y2 - (&*EDW_A * x1x2);
            let y3_den = FieldElement::one() - dxxyy;
            let y3 = y3_num * y3_den.inv();

            EdwardsPoint::Affine(x3, y3)
        }
    }
}

fn point_double(p: &EdwardsPoint) -> EdwardsPoint {
    if let EdwardsPoint::Affine(_x, _y) = p {
        point_add(p, p) // Re-use addition formula for doubling
    } else {
        EdwardsPoint::Infinity
    }
}

impl Add for EdwardsPoint {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        point_add(&self, &rhs)
    }
}

impl Add<&EdwardsPoint> for EdwardsPoint {
    type Output = EdwardsPoint;
    fn add(self, rhs: &EdwardsPoint) -> EdwardsPoint {
        point_add(&self, rhs)
    }
}

impl<'a, 'b> Add<&'b EdwardsPoint> for &'a EdwardsPoint {
    type Output = EdwardsPoint;
    fn add(self, rhs: &'b EdwardsPoint) -> EdwardsPoint {
        point_add(self, rhs)
    }
}

impl Neg for EdwardsPoint {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self {
            EdwardsPoint::Infinity => EdwardsPoint::Infinity,
            EdwardsPoint::Affine(x, y) => EdwardsPoint::Affine(-x, y),
        }
    }
}

impl<'a> Neg for &'a EdwardsPoint {
    type Output = EdwardsPoint;
    fn neg(self) -> Self::Output {
        match self {
            EdwardsPoint::Infinity => EdwardsPoint::Infinity,
            EdwardsPoint::Affine(x, y) => EdwardsPoint::Affine(-x.clone(), y.clone()),
        }
    }
}

impl Sub for EdwardsPoint {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        point_add(&self, &(-rhs))
    }
}

impl Sub<&EdwardsPoint> for EdwardsPoint {
    type Output = EdwardsPoint;
    fn sub(self, rhs: &EdwardsPoint) -> EdwardsPoint {
        point_add(&self, &(-rhs))
    }
}

impl<'a, 'b> Sub<&'b EdwardsPoint> for &'a EdwardsPoint {
    type Output = EdwardsPoint;
    fn sub(self, rhs: &'b EdwardsPoint) -> EdwardsPoint {
        point_add(self, &(-rhs))
    }
}

impl Mul<BigUint> for EdwardsPoint {
    type Output = Self;
    fn mul(self, scalar: BigUint) -> Self {
        let mut res = EdwardsPoint::Infinity;
        let mut temp = self;
        let mut s = scalar;

        while !s.is_zero() {
            if (&s & BigUint::one()) == BigUint::one() {
                res = res + temp.clone();
            }
            temp = point_double(&temp);
            s >>= 1;
        }
        res
    }
}

impl<'a> Mul<BigUint> for &'a EdwardsPoint {
    type Output = EdwardsPoint;
    fn mul(self, scalar: BigUint) -> EdwardsPoint {
        let mut res = EdwardsPoint::Infinity;
        let mut temp = self.clone();
        let mut s = scalar;

        while !s.is_zero() {
            if (&s & BigUint::one()) == BigUint::one() {
                res = res + temp.clone();
            }
            temp = point_double(&temp);
            s >>= 1;
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_point_on_curve() {
        let (gx, gy) = G.coords();
        let gx2 = &gx * &gx;
        let gy2 = &gy * &gy;
        let lhs = &*EDW_A * &gx2 + &gy2;
        let rhs = FieldElement::one() + &*EDW_D * &gx2 * &gy2;
        assert_eq!(lhs, rhs, "Base point G must be on the curve");
    }

    #[test]
    fn test_base_point_order() {
        // l*G should be the identity element (point at infinity)
        let l_times_g = &*G * L.clone();
        assert_eq!(l_times_g, EdwardsPoint::Affine(FieldElement::zero(), FieldElement::one()), "l*G must be the identity element");

        // h*G should not be the identity element
        let h_times_g = &*G * H.clone();
        assert_ne!(h_times_g, EdwardsPoint::Affine(FieldElement::zero(), FieldElement::one()), "h*G must not be the identity element");
    }
}
