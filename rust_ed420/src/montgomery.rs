// Montgomery form utilities (u-only ECDH and basepoint/mapping helpers)

use crate::field::FieldElement;
use num_bigint::BigUint;
use num_traits::{Num, One};

lazy_static::lazy_static! {
    // Montgomery parameter A (B = 1)
    pub static ref A: FieldElement = FieldElement::new(
        BigUint::from_str_radix(
            "763975519699500577645754547835125169481986463482154078046572648671788968290548038674290307302429817161505744408446033521089602",
            10
        ).unwrap()
    );

    // Montgomery base point u-coordinate
    pub static ref G_MONT: MontgomeryPoint = MontgomeryPoint(FieldElement::new(
        BigUint::from_str_radix(
            "1887066872174968132246224128199266266323489104588603923691363826518154582291788366769852665419756146257203683605002692187211605",
            10
        ).unwrap()
    ));

    // Montgomery base point v-coordinate (for mapping tests)
    pub static ref G_MONT_V: FieldElement = FieldElement::new(
        BigUint::from_str_radix(
            "1615823937666138581405149982946858036615132278772287171232550469704961695279457501113588538572409066758954677368118289169060562",
            10
        ).unwrap()
    );
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MontgomeryPoint(pub FieldElement);

// Projective coordinate representation for Montgomery ladder
#[derive(Debug, Clone)]
struct ProjPoint {
    x: FieldElement,
    z: FieldElement,
}

impl ProjPoint {
    fn to_montgomery(&self) -> MontgomeryPoint {
        if self.z.is_zero() {
            return MontgomeryPoint(FieldElement::zero());
        }
        MontgomeryPoint(&self.x * self.z.inv())
    }
}

// Conditional swap used by the ladder (constant-time conditional move)
fn cswap(p1: &mut ProjPoint, p2: &mut ProjPoint, b: bool) {
    if b {
        std::mem::swap(&mut p1.x, &mut p2.x);
        std::mem::swap(&mut p1.z, &mut p2.z);
    }
}

pub fn montgomery_ladder(p: &MontgomeryPoint, k: &BigUint) -> MontgomeryPoint {
    let mut p0 = ProjPoint { x: FieldElement::one(), z: FieldElement::zero() }; // Point at infinity
    let mut p1 = ProjPoint { x: p.0.clone(), z: FieldElement::one() };
    
    for i in (0..k.bits()).rev() {
        let bit = (k >> i) & BigUint::one() == BigUint::one();
        cswap(&mut p0, &mut p1, bit);
        
        // Differential addition and doubling
        let (doubled, added) = ladder_step(&p0, &p1, &p.0);
        p0 = doubled;
        p1 = added;

        cswap(&mut p0, &mut p1, bit);
    }

    p0.to_montgomery()
}

// Returns (2*p0, p0+p1)
fn ladder_step(p0: &ProjPoint, p1: &ProjPoint, base_x: &FieldElement) -> (ProjPoint, ProjPoint) {
    let x0 = &p0.x;
    let z0 = &p0.z;
    let x1 = &p1.x;
    let z1 = &p1.z;

    // Addition: p0 + p1
    let v0 = x0 + z0;
    let v1 = x0 - z0;
    let v2 = x1 + z1;
    let v3 = x1 - z1;

    let v4 = &v0 * &v3;
    let v5 = &v1 * &v2;

    let v0_ = &v4 + &v5;
    let v1_ = &v4 - &v5;

    let x_added = &v0_ * &v0_;
    let z_added = base_x * &v1_ * &v1_;

    // Doubling: 2 * p0
    let x0_sq = x0 * x0;
    let z0_sq = z0 * z0;
    
    let x_doubled = (&x0_sq - &z0_sq) * (&x0_sq - &z0_sq);
    
    let four_x0_z0 = &(&v0 * &v0) - &(&v1 * &v1);
    // Note: we parameterize doubling directly by A (not precomputing a24 = (A+2)/4)
    let t = &x0_sq + &(&*A * x0 * z0) + &z0_sq;
    let z_doubled = &four_x0_z0 * &t;
    
    (ProjPoint{x: x_doubled, z: z_doubled}, ProjPoint{x: x_added, z: z_added})
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::curve::{L, G as EDW_G};
    use num_bigint::RandBigInt;

    #[test]
    fn test_diffie_hellman() {
        let mut rng = rand::thread_rng();

        // Alice
        let a_sk = rng.gen_biguint_below(&L);
        let a_pk = montgomery_ladder(&G_MONT, &a_sk);

        // Bob
        let b_sk = rng.gen_biguint_below(&L);
        let b_pk = montgomery_ladder(&G_MONT, &b_sk);

        assert_ne!(a_pk, b_pk);

        // Shared secrets
        let shared_secret_a = montgomery_ladder(&b_pk, &a_sk);
        let shared_secret_b = montgomery_ladder(&a_pk, &b_sk);

        assert_eq!(shared_secret_a, shared_secret_b);
    }

    #[test]
    fn test_mapping_basepoints_between_models() {
        // Edwards from Montgomery: x = u/v, y = (u-1)/(u+1)
        let u = &G_MONT.0;
        let v = &*G_MONT_V;
        let x = u * v.inv();
        let one = FieldElement::one();
        let y = (u - &one) * (u + &one).inv();

        let (gx, gy) = EDW_G.coords();
        assert_eq!(x, gx, "Mapped x must match Edwards base x");
        assert_eq!(y, gy, "Mapped y must match Edwards base y");

        // Back to Montgomery: u = (1+y)/(1-y), v = u/x
        let um = (&one + &y) * (&one - &y).inv();
        let vm = &um * x.inv();
        assert_eq!(um, *u, "Round-trip u must match");
        assert_eq!(vm, *v, "Round-trip v must match");
    }
}
