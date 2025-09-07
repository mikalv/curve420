#!/usr/bin/env sage -q
# Generate Edwards Schnorr test vectors using Curve420 parameters.

import os
import json
import hashlib
from sage.all import GF, ZZ, EllipticCurve

with open('curve420.json','r') as f:
    J = json.load(f)

p = ZZ(J['field']['p'])
A = ZZ(J['montgomery']['A'])
N = ZZ(J['group']['N'])
l = ZZ(J['group']['l'])

Fp = GF(p)
Af = Fp(A)
E = EllipticCurve(Fp, [0, Af, 0, 1, 0])

u0 = Fp(ZZ(J['montgomery']['base_point']['u']))
v0 = Fp(ZZ(J['montgomery']['base_point']['v']))
G = E(u0, v0)

def fe(x):
    return Fp(x)

def map_mont_to_ed(u, v):
    x = u / v
    y = (u - 1) / (u + 1)
    return x, y

def ed_encode_x_signy(x, y):
    # 53-byte little endian of x; top bit of last byte is sign(y) = y mod 2
    xb = int(x).to_bytes(53, 'little')
    ysign = (int(y) & 1) & 0x01
    b = bytearray(xb)
    b[-1] = (b[-1] & 0x7F) | (ysign << 7)
    return bytes(b)

def H_to_scalar(*chunks):
    h = hashlib.shake_128()
    for c in chunks:
        if isinstance(c, bytes):
            h.update(c)
        elif isinstance(c, str):
            h.update(c.encode())
        else:
            h.update(str(c).encode())
    r = int.from_bytes(h.digest(64), 'big') % int(l)
    if r == 0:
        r = 1
    return ZZ(r)

def schnorr_sign(sk, msg):
    # Public key
    P = ZZ(sk) * G
    ux, uy = P[0], P[1]
    xP, yP = map_mont_to_ed(ux, uy)
    encP = ed_encode_x_signy(xP, yP)
    # Deterministic nonce
    r = H_to_scalar('schnorr-nonce', sk, msg)
    R = ZZ(r) * G
    uR, vR = R[0], R[1]
    xR, yR = map_mont_to_ed(uR, vR)
    encR = ed_encode_x_signy(xR, yR)
    # Challenge
    e = H_to_scalar('schnorr-chal', encR, encP, msg)
    s = (r + e * ZZ(sk)) % l
    # Verify: s*G ?= R + e*P
    lhs = ZZ(s) * G
    rhs = R + ZZ(e) * P
    ok = (lhs == rhs)
    return {
        'sk': str(int(sk)),
        'pk_x': str(int(xP)),
        'pk_y': str(int(yP)),
        'R_x': str(int(xR)),
        'R_y': str(int(yR)),
        'e': str(int(e)),
        's': str(int(s)),
        'verify': bool(ok),
    }

vectors = {
    'edwards_schnorr': [],
}

msgs = [b'', b'test', b'Curve420', b'Interop\x00example']
for i, m in enumerate(msgs, start=1):
    sk = H_to_scalar('schnorr-sk', i)
    sig = schnorr_sign(sk, m)
    sig['msg_hex'] = m.hex()
    # Checks
    # on-curve and order-ℓ checks for pk
    P = ZZ(int(sig['sk'])) * G
    on_curve = (P in E)
    order_ok = ((ZZ(l) * P).is_zero())
    sig['on_curve'] = bool(on_curve)
    sig['order_l'] = bool(order_ok)
    vectors['edwards_schnorr'].append(sig)

os.makedirs('proved', exist_ok=True)

# Merge into proved/vectors.json if exists
out_path = 'proved/vectors.json'
if os.path.exists(out_path):
    with open(out_path, 'r') as f:
        base = json.load(f)
else:
    base = {}
base.update(vectors)
with open(out_path, 'w') as f:
    json.dump(base, f, indent=2)

print('Updated proved/vectors.json with edwards_schnorr vectors')

