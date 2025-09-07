# Curve420 – Consistency & Reproducibility Checklist

This checklist locks down how parameters and base points are **chosen, represented, and verified** so anyone can reproduce the exact same results.

---

## 1) Freeze the Field & Models
- [X] **Prime field:** `p = 2^420 - 335` (write both the formula and the decimal string).
- [X] **Montgomery model:** `v^2 = u^3 + A u^2 + u` with `B = 1`.
 - [X] **Edwards model:** `a x^2 + y^2 = 1 + d x^2 y^2` with `a = A + 2`, `d = A - 2` in Fp (frozen in SPEC.md and curve420.json).
- [X] **Relation between A and d (B=1, a=-1):**
  - [X] `d = (2 - A) / (A + 2)` in Fp (document this explicitly).
  - [X] Include the exact decimal strings of `A` and `d`.

---

## 2) Canonical Basepoint Selection (Deterministic)

- **Seeded RNG method (recommended for transparency)**
  - [X] **Seed string (exact):** e.g. `Curve420:hash:5769:A=<A_dec>:d=<d_dec>`.
  - [X] Derive RNG state with `SHAKE128(seed) -> 16 bytes`, then `set_random_seed(...)`.
  - [X] On Montgomery curve `E(Fp)`, loop:
    - [X] Draw `R = E.random_point()`.
    - [X] Compute `H = (N/ℓ) * R` (cofactor clearing into the ℓ-subgroup).
    - [X] If `H != O` and `ℓ * H == O`, set basepoint `G_M = H` and stop.
  - [X] Map `G_M = (u,v)` to Edwards using the **correct mapping** (see §3).


---

## 3) Canonical Montgomery ↔ Edwards Mapping
- [X] Use unscaled mapping with `a = A + 2`, `d = A - 2`, `B = 1`.
  - [X] To Edwards: `x = u / v`, `y = (u - 1) / (u + 1)`.
  - [X] To Montgomery: `u = (1 + y) / (1 - y)`, `v = u / x`.
  - [X] Round-trip validated for published base points (SPEC.md; proved/security_report.json).
  - [X] No `c` used (not applicable under this convention).

---

## 4) Group Order & Cofactors
- [X] Publish exact `N`, `v2`, `h2 = 2^v2`, and prime `ℓ` (decimal strings).
- [X] State the **implementation cofactor** `h_impl = h2 * s` (here `s = 1`).
- [X] Provide ECPP/Primo certificate for `ℓ` (or at least BPSW+ECPP evidence).
- [X] Document quadratic twist order `N_twist`, `v2_twist`, and BPSW/ECPP status.

---

## 5) Canonical Basepoint Values
- [X] Final **Montgomery basepoint** `G_M = (u, v)` (decimal strings).
- [X] Final **Edwards basepoint** `G_E = (x, y)` (decimal strings).
- [X] Verify and publish:
  - [X] Edwards equation holds: `a x^2 + y^2 == 1 + d x^2 y^2 (mod p)`.
  - [X] `ℓ * G_E = O` and `ℓ * G_M = O`.
  - [X] `h2 * G_E != O` (cofactor clearing sanity) and same for `G_M` if applicable.
  - [X] Round-trip mapping `G_M ↔ G_E` works with the published mapping.

---

## 6) Serialization & Encoding
- [X] Specify **endianness** and **byte length** (e.g., 53-byte little-endian).
- [X] **Montgomery (DH):** encode `u` only (X25519-style), define invalid encodings.
- [X] **Edwards (signatures):** encode `x` plus sign bit of `y` (or the inverse); freeze the rule.
- [X] State canonical sign rule (e.g., “use even `y`”/“use lexicographically smaller”).

---

## 7) Reference Vectors & Interop
- [ ] Provide self-contained test vectors:
  - [X] **DH:** (sk, pk, shared) triplets.
  - [X] **Schnorr:** (sk, pk, msg, R, s) with verification passing.
  - [X] **Blind Schnorr:** end-to-end (commit, blind, sign, unblind, verify).
- [X] Include an **on-curve** check and an **order-ℓ** check in vectors.
- [X] Publish both **Montgomery** and **Edwards** vectors.

---

## 8) Transparency & Scripts
- [X] Publish the **exact scripts** used to:
  - [X] Scan candidates and select the curve/parameters.
  - [X] Compute `A`, `d`, `c`, the basepoints, and group order factors.
  - [X] Generate test vectors (DH/Schnorr/blind).
- [X] Scripts must be deterministic given only public inputs (seeds/strings).
- [X] Include a `Makefile` or one-line commands to re-generate all artifacts.

---

## 9) Security Notes
- [X] Record: `d` is non-square in Fp → complete Edwards addition (a = -1).
- [X] Record: `j ≠ 0, 1728`.
- [X] Record: embedding degree lower bound (no small MOV).
- [X] Document twist security (no easy twist attacks).
- [X] State cofactor-clearing rules for all protocols.

---

## 10) Versioning & Change Control
- [X] Assign a **version tag** to the frozen parameters (e.g., `curve420-v1`).
- [X] Note that changing the basepoint breaks interop; future updates must bump version.

---

## 11) Final Publication Bundle
- [X] `curve420.json` / `curve420.yaml` with all constants as **decimal strings**.
- [X] `README.md` (spec summary) + `SPEC.md` (full details).
- [X] `CONSISTENCY_CHECKLIST.md` (this file).
- [X] `SAGE/` and `RUST/` directories with reference code and vectors.
- [ ] Optional: ECPP certificate files for `ℓ` and (optionally) twist’s odd part.
