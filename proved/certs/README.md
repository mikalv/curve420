# ECPP certificates

## Targets

- Subgroup prime ℓ (418-bit): `proved/certs/l.txt`
- Odd part of twist order odd(N_twist) (~410-bit): `proved/certs/odd_twist_odd.txt`

## How to export targets

- Run: `make cert-targets` (writes the two decimal files and a small info file)

## Paths to obtain certificates

- Primo (Atkin–Morain ECPP): GUI/CLI tool widely used to generate verifiable certificates (.cert).
  1) Load the decimal number from `l.txt` (and optionally `odd_twist_odd.txt`).
  2) Start certification; save resulting certificate files into this folder.
  3) Use Primo’s verify mode to check the .cert files.

- PARI/GP (if your version supports certificate objects):
  - Set `default(proof, 3)` and use `primecert(n)` to get a certificate object, then `write()` it. Verify with `checkprime(n, cert)`.
  - If `primecert` is unavailable, fall back to Primo.

## Local GP proof log (no external .cert)
  - `make prove-l-with-sage` → writes the same `proved/certs/l.proof.txt` using Sage’s `is_prime(proof=True)`.
  - Recommended on macOS/ARM if pari-galpol behaves inconsistently.

## Publication
  - Add certificate files to the release bundle and reference tool versions used.
  - Keep a note of runtime, CPU, and software versions for reproducibility.
