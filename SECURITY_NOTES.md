# Security Notes – Curve420

Generated: 2025-09-07T14:25:19.653835+00:00

## Summary
- Field size: 420 bits; subgroup prime ℓ: 418 bits; cofactor h = 8.
- Cofactor relation N = h·ℓ holds: yes; anomalous N = p: no.

## 9) Security Notes
- d non-square in Edwards a = −1 model: True.
- j-invariant: not 0 (True), not 1728 (True).
- MOV embedding degree: no k ≤ 1000 with ℓ | (p^k − 1).
- Cheon transparency: gcd(ℓ−1, p−1) = 2, gcd(ℓ−1, p+1) = 6.
- Twist security: v2(N_twist) = 2; odd(N_twist) probable prime: False.
- Twist factorization (bounded search): small factors found = [[401, 1]].
- Remaining cofactor: 410 bits; probable prime: False.
 - ℓ primality proof: see `proved/certs/l.proof.txt` (Sage `is_prime(proof=True)` log).

## Protocol Rules (recommended)
- ECDH (Montgomery u-only): perform on-curve decoding, reject out-of-range/non-canonical u, and check for all-zero shared secret.
- Signatures (Edwards): use cofactor-aware verification or prime-order encoding; if using cofactor-clearing, multiply by h = 8 where relevant.
- Encoding: specify endianness and canonical sign-bit; ensure decode/encode round-trip and reject non-canonical encodings.

## Reference Vectors
- ECDH (Montgomery u-only) and DH checks: see `proved/vectors.json` sections `montgomery_u_only` and `dh_checks`.
- Edwards Schnorr vectors: see `proved/vectors.json` section `edwards_schnorr`.
- Edwards Blind Schnorr vectors: see `proved/vectors.json` section `edwards_blind_schnorr`.
