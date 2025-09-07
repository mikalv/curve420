// src/schnorr.rs

use crate::curve::{EdwardsPoint, G, L};
use num_bigint::{BigUint, RandBigInt};
use num_traits::Zero;
use sha2::{Digest, Sha512};

/// --- Hjelpere ---

#[inline]
fn mod_l(x: &BigUint) -> BigUint {
    x % &*L
}

#[inline]
fn add_mod_l(a: &BigUint, b: &BigUint) -> BigUint {
    mod_l(&(a + b))
}

#[inline]
fn sub_mod_l(a: &BigUint, b: &BigUint) -> BigUint {
    if a >= b {
        a - b
    } else {
        &*L + a - b
    }
}

/// Hash(points || msg) -> BigUint mod L
pub fn hash_points_and_message(points: &[&EdwardsPoint], msg: &[u8]) -> BigUint {
    let mut h = Sha512::new();
    // Domain-separation så vi ikke kolliderer med andre protokoller
    h.update(b"ed420-schnorr-v1");
    for p in points {
        let (x, y) = p.coords();
        h.update(x.to_biguint().to_bytes_be());
        h.update(y.to_biguint().to_bytes_be());
    }
    h.update(msg);
    mod_l(&BigUint::from_bytes_be(&h.finalize()))
}

/// Hash(points || msg || info) -> BigUint mod L
pub fn hash_points_message_and_info(points: &[&EdwardsPoint], msg: &[u8], info: &[u8]) -> BigUint {
    let mut h = Sha512::new();
    // Domain-separation for partially blind signatures
    h.update(b"ed420-partially-blind-schnorr-v1");
    for p in points {
        let (x, y) = p.coords();
        h.update(x.to_biguint().to_bytes_be());
        h.update(y.to_biguint().to_bytes_be());
    }
    h.update(msg);
    h.update(info);
    mod_l(&BigUint::from_bytes_be(&h.finalize()))
}


/// --- Nøkler ---

pub fn keygen() -> (BigUint, EdwardsPoint) {
    let mut rng = rand::thread_rng();
    // Velg sk i [1, L-1]
    let mut sk = BigUint::zero();
    while sk.is_zero() {
        sk = rng.gen_biguint_below(&L);
    }
    let pk = &*G * sk.clone();
    (sk, pk)
}

/// --- Standard Schnorr ---

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    pub r: EdwardsPoint,
    pub s: BigUint,
}

pub fn sign(sk: &BigUint, pk: &EdwardsPoint, msg: &[u8]) -> Signature {
    let mut rng = rand::thread_rng();

    // Tilfeldig nonce k i [1, L-1]
    let mut k = BigUint::zero();
    while k.is_zero() {
        k = rng.gen_biguint_below(&L);
    }
    let r = &*G * k.clone();

    let e = hash_points_and_message(&[&r, pk], msg);
    let s = add_mod_l(&k, &(&e * sk)); // s = k + e*sk mod L

    Signature { r, s }
}

pub fn verify(pk: &EdwardsPoint, msg: &[u8], sig: &Signature) -> bool {
    // Sjekk basis-gyldighet
    if sig.s.is_zero() || &sig.s >= &*L {
        return false;
    }

    let e = hash_points_and_message(&[&sig.r, pk], msg);

    // s*G ?= R + e*PK
    let s_g = &*G * sig.s.clone();
    let r_plus_e_pk = &sig.r + &(pk * e);

    s_g == r_plus_e_pk
}

/// --- Blind Schnorr (simulert, 1‑runde API) ---
/// Protokoll: requester velger alpha, beta; R' = R + alpha*G + beta*PK
/// e' = H(R', PK, m); sender e_blinded = e' - beta; signer svarer s = k + e_blinded*sk
/// requester unblinder: s' = s + alpha; signaturen er (R', s')

pub struct BlindSignatureRequester {
    alpha: BigUint,
    beta: BigUint,
    pub signer_pk: EdwardsPoint,
    pub r_prime: EdwardsPoint,
    pub e_prime: BigUint,
}

pub struct BlindSignatureSigner {
    sk: BigUint,
    k: BigUint,
    pub r: EdwardsPoint,
}

impl BlindSignatureSigner {
    pub fn new(sk: BigUint, _pk: EdwardsPoint) -> Self {
        let mut rng = rand::thread_rng();
        let mut k = BigUint::zero();
        while k.is_zero() {
            k = rng.gen_biguint_below(&L);
        }
        let r = &*G * k.clone();
        Self { sk, k, r }
    }

    pub fn sign(&self, e_blinded: &BigUint) -> BigUint {
        // s = k + e_blinded*sk mod L
        add_mod_l(&self.k, &(e_blinded * &self.sk))
    }
}

impl BlindSignatureRequester {
    pub fn new(signer_pk: EdwardsPoint, message: &[u8], r_from_signer: &EdwardsPoint) -> Self {
        let mut rng = rand::thread_rng();
        let mut alpha = BigUint::zero();
        let mut beta = BigUint::zero();
        while alpha.is_zero() {
            alpha = rng.gen_biguint_below(&L);
        }
        while beta.is_zero() {
            beta = rng.gen_biguint_below(&L);
        }

        let r_prime = r_from_signer + &(&*G * alpha.clone()) - &( &signer_pk * beta.clone() );
        let e_prime = hash_points_and_message(&[&r_prime, &signer_pk], message);

        Self { alpha, beta, signer_pk, r_prime, e_prime }
    }

    pub fn create_blinded_challenge(&self) -> BigUint {
        // e_blinded = e' - beta mod L
        sub_mod_l(&self.e_prime, &self.beta)
    }

    pub fn unblind_signature(&self, s_from_signer: &BigUint) -> Signature {
        // s' = s + alpha mod L
        let s_prime = add_mod_l(s_from_signer, &self.alpha);
        Signature { r: self.r_prime.clone(), s: s_prime }
    }
}

pub fn verify_blind_signature(pk: &EdwardsPoint, msg: &[u8], sig: &Signature) -> bool {
    // verifiser som vanlig Schnorr (med R' i stedet for R)
    verify(pk, msg, sig)
}


/// --- Partially Blind Schnorr ---

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartiallyBlindSignature {
    pub r: EdwardsPoint,
    pub s: BigUint,
    pub info: Vec<u8>,
}

pub struct PartiallyBlindSignatureRequester {
    alpha: BigUint,
    beta: BigUint,
    r_prime: EdwardsPoint,
    e_prime: BigUint,
    info: Vec<u8>,
}

impl PartiallyBlindSignatureRequester {
    pub fn new(signer_pk: EdwardsPoint, message: &[u8], info: &[u8], r_from_signer: &EdwardsPoint) -> Self {
        let mut rng = rand::thread_rng();
        let mut alpha = BigUint::zero();
        let mut beta = BigUint::zero();
        while alpha.is_zero() {
            alpha = rng.gen_biguint_below(&L);
        }
        while beta.is_zero() {
            beta = rng.gen_biguint_below(&L);
        }

        let r_prime = r_from_signer + &(&*G * alpha.clone()) - &( &signer_pk * beta.clone() );
        let e_prime = hash_points_message_and_info(&[&r_prime, &signer_pk], message, info);

        Self {
            alpha,
            beta,
            r_prime,
            e_prime,
            info: info.to_vec(),
        }
    }

    pub fn create_blinded_challenge(&self) -> BigUint {
        sub_mod_l(&self.e_prime, &self.beta)
    }

    pub fn unblind_signature(&self, s_from_signer: &BigUint) -> PartiallyBlindSignature {
        let s_prime = add_mod_l(s_from_signer, &self.alpha);
        PartiallyBlindSignature {
            r: self.r_prime.clone(),
            s: s_prime,
            info: self.info.clone(),
        }
    }
}

pub fn verify_partially_blind_signature(pk: &EdwardsPoint, msg: &[u8], sig: &PartiallyBlindSignature) -> bool {
    if sig.s.is_zero() || &sig.s >= &*L {
        return false;
    }

    let e = hash_points_message_and_info(&[&sig.r, pk], msg, &sig.info);

    let s_g = &*G * sig.s.clone();
    let r_plus_e_pk = &sig.r + &(pk * e);

    s_g == r_plus_e_pk
}


/// --- Tester ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_verify_ok() {
        let (sk, pk) = keygen();
        let m = b"This is a test message for the ed420 rust library.";
        let sig = sign(&sk, &pk, m);
        assert!(verify(&pk, m, &sig), "Signature should be valid for the correct message.");
    }

    #[test]
    fn test_verify_tampered_message_fails() {
        let (sk, pk) = keygen();
        let m = b"This is a test message for the ed420 rust library.";
        let sig = sign(&sk, &pk, m);
        let bad = b"This is a different message.";
        assert!(!verify(&pk, bad, &sig), "Signature should be invalid for a tampered message.");
    }

    #[test]
    fn test_verify_wrong_public_key_fails() {
        let (sk1, pk1) = keygen();
        let (_sk2, pk2) = keygen();
        let m = b"A message to be signed.";
        let sig = sign(&sk1, &pk1, m);
        assert!(!verify(&pk2, m, &sig), "Signature should be invalid with a different public key.");
    }

    #[test]
    fn test_blind_signature_ok() {
        // 1) signer / requester
        let (signer_sk, signer_pk) = keygen();
        let message = b"This is the message to be blindly signed.";

        // 2) signer nonce commit
        let signer = BlindSignatureSigner::new(signer_sk, signer_pk.clone());

        // 3) requester: blind challenge
        let requester = BlindSignatureRequester::new(signer_pk.clone(), message, &signer.r);
        let e_blinded = requester.create_blinded_challenge();

        // 4) signer svarer
        let s_signed = signer.sign(&e_blinded);

        // 5) requester unblinder
        let final_sig = requester.unblind_signature(&s_signed);

        // 6) verifiser
        assert!(
            verify_blind_signature(&signer_pk, message, &final_sig),
            "The blind signature should be valid."
        );

        // 7) Negativ test
        let tampered = b"some other message";
        assert!(
            !verify_blind_signature(&signer_pk, tampered, &final_sig),
            "Blind signature must fail on different message."
        );
    }

    #[test]
    fn test_partially_blind_signature_ok() {
        // 1) Setup
        let (signer_sk, signer_pk) = keygen();
        let message = b"Blinded part of the message.";
        let info = b"Public info, known to signer.";

        // 2) Signer commits to nonce R
        let signer = BlindSignatureSigner::new(signer_sk, signer_pk.clone());

        // 3) Requester creates blinded challenge
        let requester = PartiallyBlindSignatureRequester::new(signer_pk.clone(), message, info, &signer.r);
        let e_blinded = requester.create_blinded_challenge();

        // 4) Signer signs the blinded challenge
        let s_signed = signer.sign(&e_blinded);

        // 5) Requester unblinds the signature
        let final_sig = requester.unblind_signature(&s_signed);

        // 6) Verification
        assert!(
            verify_partially_blind_signature(&signer_pk, message, &final_sig),
            "The partially blind signature should be valid."
        );

        // 7) Negative tests
        let tampered_message = b"different blinded part";
        assert!(
            !verify_partially_blind_signature(&signer_pk, tampered_message, &final_sig),
            "Partially blind signature must fail for different message."
        );

        let tampered_info_sig = PartiallyBlindSignature {
            info: b"different public info".to_vec(),
            ..final_sig.clone()
        };
        assert!(
            !verify_partially_blind_signature(&signer_pk, message, &tampered_info_sig),
            "Partially blind signature must fail for different info."
        );
    }
}
