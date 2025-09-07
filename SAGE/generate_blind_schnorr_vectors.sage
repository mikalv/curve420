#!/usr/bin/env sage -q
# Generate Edwards Blind Schnorr test vectors (deterministic, illustrative)

import os
import json
import hashlib
from sage.all import GF, ZZ, EllipticCurve

with open('curve420.json','r') as f:
    J = json.load(f)

p = ZZ(J['field']['p'])
A = ZZ(J['montgomery']['A'])
l = ZZ(J['group']['l'])

Fp = GF(p)
Af = Fp(A)
E = EllipticCurve(Fp, [0, Af, 0, 1, 0])

u0 = Fp(ZZ(J['montgomery']['base_point']['u']))
v0 = Fp(ZZ(J['montgomery']['base_point']['v']))
G = E(u0, v0)

def map_mont_to_ed(u, v):
    x = u / v
    y = (u - 1) / (u + 1)
    return x, y

def ed_encode_x_signy(x, y):
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

def vector_for_message(msg: bytes, idx: int):
    # Signer secret/public
    sk = H_to_scalar('blind-signer-sk', idx)
    P = ZZ(sk) * G
    Px, Py = map_mont_to_ed(P[0], P[1])
    encP = ed_encode_x_signy(Px, Py)

    # Signer nonce
    k = H_to_scalar('blind-nonce', idx)
    R = ZZ(k) * G

    # Requester blinds
    a = H_to_scalar('blind-a', idx)
    b = H_to_scalar('blind-b', idx)
    Rprime = R + ZZ(a) * G + ZZ(b) * P
    Rx, Ry = map_mont_to_ed(Rprime[0], Rprime[1])
    encR = ed_encode_x_signy(Rx, Ry)

    # Challenge and responses
    eprime = H_to_scalar('blind-chal', encR, encP, msg)
    e = (eprime + b) % l
    s = (k + e * sk) % l
    sprime = (s + a) % l

    # Verify: s'G == R' + e'P
    ok = (ZZ(sprime) * G == Rprime + ZZ(eprime) * P)

    return {
        'signer_sk': str(int(sk)),
        'signer_pk_x': str(int(Px)),
        'signer_pk_y': str(int(Py)),
        'Rprime_x': str(int(Rx)),
        'Rprime_y': str(int(Ry)),
        'e_prime': str(int(eprime)),
        's_prime': str(int(sprime)),
        'verify': bool(ok),
        'msg_hex': msg.hex(),
    }

vectors = {
    'edwards_blind_schnorr': []
}

msgs = [b'', b'test', b'Curve420', b'Blind\x00example']
for i, m in enumerate(msgs, start=1):
    vectors['edwards_blind_schnorr'].append(vector_for_message(m, i))

os.makedirs('proved', exist_ok=True)
out_path = 'proved/vectors.json'

base = {}
if os.path.exists(out_path):
    with open(out_path, 'r') as f:
        base = json.load(f)
base.update(vectors)
with open(out_path, 'w') as f:
    json.dump(base, f, indent=2)

print('Updated proved/vectors.json with edwards_blind_schnorr vectors')

