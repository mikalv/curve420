#!/usr/bin/env sage -q
# Generate basic ECDH vectors for Curve420 (Montgomery u-only)

import json
import hashlib
from sage.all import GF, ZZ, EllipticCurve, set_random_seed

with open('curve420.json','r') as f:
    J = json.load(f)

p = ZZ(J['field']['p'])
A = ZZ(J['montgomery']['A'])
N = ZZ(J['group']['N'])
l = ZZ(J['group']['l'])
h = ZZ(J['group']['h'])

Fp = GF(p)
Af = Fp(A)
E = EllipticCurve(Fp, [0, Af, 0, 1, 0])

u0 = Fp(int(J['montgomery']['base_point']['u']))
v0 = Fp(int(J['montgomery']['base_point']['v']))
G = E(u0, v0)

def derive_scalar(tag, i):
    m = hashlib.shake_128(f'Curve420:vectors:{tag}:{i}'.encode()).digest(64)
    k = int.from_bytes(m, 'big') % int(l)
    # Avoid 0 and reduce to [1, l-1]
    if k == 0:
        k = 1
    return ZZ(k)

def encode_le(n, length):
    return int(n).to_bytes(length, 'little')

def fe_to_int(x):
    return int(Fp(x))

def point_mul_u(k, P):
    Q = ZZ(k) * P
    return fe_to_int(Q[0])

vectors = {
    'field_bits': int(p.nbits()),
    'byte_length': 53,
    'montgomery_u_only': [],
}

# Generate 8 keypairs and 4 shared secrets
keys = []
for i in range(1, 9):
    sk = derive_scalar('ecdh-sk', i)
    P = ZZ(sk) * G
    pk_u = fe_to_int(P[0])
    keys.append({'i': i, 'sk': str(int(sk)), 'pk_u': str(pk_u)})
vectors['montgomery_u_only'] = keys

# Produce a few DH shared secrets using pairs (1,2), (3,4), (5,6), (7,8)
dh_pairs = []
for a, b in [(1,2), (3,4), (5,6), (7,8)]:
    ska = ZZ(int(keys[a-1]['sk']))
    skb = ZZ(int(keys[b-1]['sk']))
    Pa = ZZ(ska) * G
    Pb = ZZ(skb) * G
    # Shared u computed via ska * Pb (full point), take u-coordinate
    S_ab_u = fe_to_int((ska * Pb)[0])
    S_ba_u = fe_to_int((skb * Pa)[0])
    dh_pairs.append({
        'a': a,
        'b': b,
        'shared_u_ab': str(S_ab_u),
        'shared_u_ba': str(S_ba_u),
        'match': bool(S_ab_u == S_ba_u),
    })
vectors['dh_checks'] = dh_pairs

import os
os.makedirs('proved', exist_ok=True)
with open('proved/vectors.json','w') as f:
    json.dump(vectors, f, indent=2, default=str)

print('Wrote proved/vectors.json')
