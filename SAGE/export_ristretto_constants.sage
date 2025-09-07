#!/usr/bin/env sage -q
# Export core constants for Ristretto-style encoding over Curve420

import os, json
from sage.all import GF, ZZ

with open('curve420.json','r') as f:
    J = json.load(f)

p = ZZ(J['field']['p'])
A = ZZ(J['montgomery']['A'])
Fp = GF(p)
Af = Fp(A)

# a = -1 view: d' = (2 - A)/(A + 2)
numer = (Fp(2) - Af)
denom = (Af + Fp(2))
d_m1 = numer / denom

# sqrt(-1) exists since p ≡ 1 (mod 4)
sqrt_m1 = Fp(-1).sqrt()

os.makedirs('proved', exist_ok=True)
out = {
    'p_bits': int(p.nbits()),
    'A': str(int(A)),
    'edwards_a_minus_1_d': str(int(d_m1)),
    'sqrt_m1': str(int(sqrt_m1)),
}
with open('proved/ristretto_constants.json','w') as f:
    json.dump(out, f, indent=2)

print('Wrote proved/ristretto_constants.json')

