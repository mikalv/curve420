# Sage security checks for Curve420

This folder contains a self-contained Sage script that verifies the security notes in CONSISTENCY_CHECKLIST.md §9.

## How to run

- Prereq: SageMath installed and available as `sage` in your PATH.
- Run: `sage -q SAGE/verify_security.sage`
- Outputs a human-readable summary and writes a JSON report to `proved/security_report.json`.

## What is checked

- d non-square in the a = -1 Edwards model (complete addition formulas).
- j-invariant is not 0 or 1728 (no special CM endomorphisms).
- MOV embedding degree lower bound: no small k ≤ 200 with ℓ | (p^k − 1).
- Twist security: computes N_twist, 2-adic valuation, and odd-part probable primality.
- Cofactor consistency and clearing rules summary.

## Extras (optional)

- CM discriminant summary: reports Δ = t^2 − 4p, fundamental discriminant D, and conductor f.
- Class number (optional): set `CM_CLASSNO=1` to compute class number h(Δ) via PARI (can be slow).
- MOV bound control: set `MOV_MAX_K` (default 200) to extend the embedding-degree search.
- Cheon transparency: reports `gcd(l−1, p−1)` and `gcd(l−1, p+1)`.
- Twist odd-part proof: set `TWIST_PROOF=1` to use `is_prime(proof=True)` for `odd(N_twist)`; may be slow.
- Twist factorization (bounded transparency):
  - Trial division over primes ≤ `TWIST_TRIAL_BOUND` (default 200000).
  - Bounded Pollard Rho attempts via `TWIST_RHO_ATTEMPTS` (default 4) and `TWIST_RHO_ITERS` (default 25000) per attempt.
  - Reports found small factors, remaining cofactor bit length, and whether remainder is probable prime.

## Notes

- The Edwards parameters in `curve420.json` use the isomorphic form `a = A + 2`, `d = A − 2`. The script converts to the a = −1 model to test non-squareness of `d` relevant for complete addition.
- You can adjust the MOV bound by setting the environment variable `MOV_MAX_K` (default 200), e.g. `MOV_MAX_K=500 sage -q SAGE/verify_security.sage`.

## Invalid-curve/twist input corpus

- Generate a small corpus for ECDH/encoding negative tests: `sage -q SAGE/generate_invalid_corpus.sage`.
- Writes `proved/invalid_corpus.json` with samples of Montgomery `u` that are:
  - special values (0, 1, −1),
  - roots of the 2-torsion polynomial,
  - random `u` where RHS is a quadratic residue (on-curve-like),
  - random `u` where RHS is a non-residue (twist-like).

## Test vectors

- Generate basic ECDH vectors (Montgomery u-only): `sage -q SAGE/generate_vectors.sage`
- Generate Edwards Schnorr vectors: `sage -q SAGE/generate_schnorr_vectors.sage`
- Generate Edwards Blind Schnorr vectors: `sage -q SAGE/generate_blind_schnorr_vectors.sage`
- Or run all at once: `make vectors`

Output written to `proved/vectors.json`:
- `montgomery_u_only`: 8 deterministiske nøkler (sk, pk_u)
- `dh_checks`: 4 DH-par med delt u og match=true
- `edwards_schnorr`: (sk, pk_x, pk_y, R_x, R_y, e, s, msg_hex, verify=true)
- `edwards_blind_schnorr`: (signer_sk, signer_pk_x/y, Rprime_x/y, e_prime, s_prime, msg_hex, verify=true)

## Ristretto (experimental)

- Export constants (a = −1 view): `make ristretto-constants` → `proved/ristretto_constants.json`
- Provisional vectors (current encoding uses Edwards x + sign(y)): `make ristretto-vectors` → `proved/ristretto_vectors.json`
- Note: The Rust crate includes a working round-trip with the provisional encoding. Final Ristretto encode/decode will replace this and vectors will be updated accordingly.
