# curve420 (Rust)

A minimal, self-contained Rust implementation of Curve420 with both Montgomery (u-only, X25519-style ECDH) and Twisted Edwards (signatures) models, matching the frozen specification in `SPEC.md` and constants in `curve420.json`.

## Modules

- `field`: prime field arithmetic over p = 2^420 − 335 (53-byte little-endian).
- `montgomery`: Montgomery form for ECDH; base u and parameter A match `curve420.json`.
- `curve` (Edwards): Twisted Edwards with a = A + 2, d = A − 2; base point (x, y) matches `curve420.json`.
- `schnorr`: Deterministic Schnorr and blind/partially-blind Schnorr over the Edwards form (prime-order subgroup ℓ; cofactor h = 8).

## Parameters (source of truth)

- All constants are frozen in `curve420.json` / `curve420.yaml` and restated in `SPEC.md`.
- Edwards parameters used here:
  - `EDW_A = A + 2`
  - `EDW_D = A − 2`
- Basepoints (Montgomery u; Edwards x,y) exactly match the JSON.

## Build & Test

- Run tests:
  - `cargo test`
- Examples:
  - Diffie-Hellman (Montgomery): `cargo run --example diffie_hellman`
  - Schnorr signature (Edwards): `cargo run --example schnorr_signature`
  - Blind/partially blind Schnorr: `cargo run --example blind_signature`, `cargo run --example partially_blind_signature`

## Mapping (Montgomery ↔ Edwards)

- Frozen mapping (unscaled, B = 1):
  - To Edwards: `x = u / v`, `y = (u − 1) / (u + 1)`
  - To Montgomery: `u = (1 + y) / (1 − y)`, `v = u / x`
- A unit test checks that the published Montgomery basepoint (u, v) maps to the published Edwards (x, y) and round-trips back.

## Encoding

- Field elements: 53-byte little-endian, canonical.
- Montgomery ECDH: encode u only (X25519-style). Applications must reject out-of-range/non-canonical inputs and check for an all-zero shared secret.
- Edwards signatures: encode x with the sign bit of y as the MSB of the last byte (sign(y) = y mod 2). Encodings must be canonical and decode/encode must round-trip.

## Vectors & Interop

- Deterministic vectors are generated under `proved/` at the repo root:
  - ECDH keys and DH checks: `proved/vectors.json` → `montgomery_u_only`, `dh_checks`
  - Schnorr and blind Schnorr: `proved/vectors.json` → `edwards_schnorr`, `edwards_blind_schnorr`
- The crate’s tests verify on-curve, subgroup order, and mapping; examples can be extended to cross-check against these vectors.

## Security Notes (summary)

- ℓ is a 418-bit prime; h = 8; N = h·ℓ.
- `d' = (2 − A)/(A + 2)` is non-square in Fp → complete Edwards addition (a = −1 view used for proofs).
- j ≠ 0, 1728; no small MOV embedding degree found up to k = 1000.
- Twist order analyzed; invalid-curve/twist corpus available in `proved/invalid_corpus.json`.

## Provenance & Reproducibility

- Scripts to reproduce parameters, generate vectors, and produce security reports are in `SAGE/` at the repo root.
- Make targets:
  - `make prove-security-strong`, `make vectors`, `make security-notes`

## License

Apache-2.0

## Quick example

```rust
use curve420::{montgomery::{G_MONT, montgomery_ladder}, schnorr};
use num_bigint::BigUint;

fn main() {
    // ECDH (Montgomery u-only)
    let alice_sk = BigUint::from(123u32);
    let bob_sk   = BigUint::from(456u32);
    let alice_pk = montgomery_ladder(&G_MONT, &alice_sk);
    let bob_pk   = montgomery_ladder(&G_MONT, &bob_sk);
    let alice_shared = montgomery_ladder(&bob_pk, &alice_sk);
    let bob_shared   = montgomery_ladder(&alice_pk, &bob_sk);
    assert_eq!(alice_shared, bob_shared);

    // Schnorr (Edwards)
    let (sk, pk) = schnorr::keygen();
    let msg = b"hello curve420";
    let sig = schnorr::sign(&sk, &pk, msg);
    assert!(schnorr::verify(&pk, msg, &sig));
}
```
