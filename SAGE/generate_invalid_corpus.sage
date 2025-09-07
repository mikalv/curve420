#!/usr/bin/env sage -q
# Generate invalid-curve/twist input corpus for Montgomery u-encodings

import json
from sage.all import GF, ZZ

with open('curve420.json','r') as f:
    J = json.load(f)

p = ZZ(J['field']['p'])
A = ZZ(J['montgomery']['A'])
Fp = GF(p)
Af = Fp(A)

# For Montgomery y^2 = x^3 + A x^2 + x, define RHS(u) and check residue/non-residue status.
def rhs(u):
    u = Fp(u)
    return u*(u*u + Af*u + 1)

def is_residue(z):
    # 0 counts as residue (v = 0)
    if z == 0:
        return True
    return z.is_square()

special = [0, 1, int(-Fp(1))]

# 2-torsion u-roots: solve u*(u^2 + A u + 1) = 0
two_torsion = []
poly_coeffs = [1, int(Af), 1]  # u^2 + A u + 1
# Find roots of quadratic over Fp (handles repeated or no roots gracefully)
disc = Af*Af - 4
if disc.is_square():
    sqrt_disc = disc.sqrt()
    r1 = (-Af + sqrt_disc) / 2
    r2 = (-Af - sqrt_disc) / 2
    two_torsion = [0, int(r1), int(r2)]
else:
    two_torsion = [0]

# Random samples by residue status
def sample_us(kind, limit):
    out = []
    i = 0
    u = Fp(2)
    while len(out) < limit and i < 20000:
        z = rhs(u)
        if (kind == 'residue' and is_residue(z)) or (kind == 'nonresidue' and not is_residue(z)):
            out.append(int(u))
        u += 1
        i += 1
    return out

residues = sample_us('residue', 64)
nonresidues = sample_us('nonresidue', 64)

corpus = {
    'field_bits': int(p.nbits()),
    'p': str(p),
    'A': str(A),
    'special_u': special,
    'two_torsion_u': two_torsion,
    'residue_u_samples': residues,
    'nonresidue_u_samples': nonresidues,
    'notes': 'Residue u likely maps to on-curve; non-residue to quadratic twist. Use for ECDH negative tests, plus all-zero shared check.'
}

import os
os.makedirs('proved', exist_ok=True)
with open('proved/invalid_corpus.json','w') as f:
    json.dump(corpus, f, indent=2)

print('Wrote proved/invalid_corpus.json')

