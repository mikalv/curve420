#!/usr/bin/env sage -q
# Generate provisional Ristretto420 vectors using current encoding (Edwards x + sign(y)).

import json, os, hashlib
from sage.all import GF, ZZ, EllipticCurve

with open('curve420.json','r') as f:
    J = json.load(f)

p = ZZ(J['field']['p'])
A = ZZ(J['montgomery']['A'])
Fp = GF(p)
Af = Fp(A)
E = EllipticCurve(Fp, [0, Af, 0, 1, 0])

u0 = Fp(ZZ(J['montgomery']['base_point']['u']))
v0 = Fp(ZZ(J['montgomery']['base_point']['v']))
Gm = E(u0, v0)

def mont_to_ed(P):
    u, v = P[0], P[1]
    x = u / v
    y = (u - 1) / (u + 1)
    return x, y

def enc_edwards_x_signy(x, y):
    # 53-byte little-endian of x; MSB of last byte = parity(y)
    xb = int(x).to_bytes(53, 'little')
    ysign = (int(y) & 1) & 0x01
    b = bytearray(xb)
    b[-1] = (b[-1] & 0x7F) | (ysign << 7)
    return bytes(b).hex()

vectors = {
    'note': 'Provisional Ristretto encoding = Edwards x + sign(y); will be updated when final codec lands.',
    'points': []
}

def derive_scalar(tag, i):
    m = hashlib.shake_128(f'Curve420:ristretto:{tag}:{i}'.encode()).digest(64)
    return ZZ(int.from_bytes(m, 'big'))

L = ZZ(J['group']['l'])
for i in range(1, 9):
    k = derive_scalar('sk', i) % L
    if k == 0:
        k = 1
    Pm = ZZ(k) * Gm
    x, y = mont_to_ed(Pm)
    enc = enc_edwards_x_signy(x, y)
    vectors['points'].append({
        'i': i,
        'k': str(int(k)),
        'x': str(int(x)),
        'y': str(int(y)),
        'encoding_hex': enc,
    })

os.makedirs('proved', exist_ok=True)
with open('proved/ristretto_vectors.json', 'w') as f:
    json.dump(vectors, f, indent=2)

print('Wrote proved/ristretto_vectors.json')

