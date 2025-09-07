#!/usr/bin/env sage -q
# Export prime-certification targets (ℓ and odd(N_twist)) as decimal files

import os
import json
from sage.all import ZZ

with open('curve420.json','r') as f:
    J = json.load(f)

p = ZZ(J['field']['p'])
N = ZZ(J['group']['N'])
l = ZZ(J['group']['l'])
h = ZZ(J['group']['h'])

# Twist order and its odd part
N_twist = 2*p + 2 - N
v2 = (N_twist).valuation(2)
odd = N_twist >> v2

os.makedirs('proved/certs', exist_ok=True)

with open('proved/certs/l.txt','w') as f:
    f.write(str(int(l)) + "\n")

with open('proved/certs/odd_twist_odd.txt','w') as f:
    f.write(str(int(odd)) + "\n")

with open('proved/certs/README.targets.txt','w') as f:
    f.write(f"l bits: {int(l.nbits())}\n")
    f.write(f"N_twist v2: {int(v2)}\n")
    f.write(f"odd(N_twist) bits: {int(odd.nbits())}\n")

print('Wrote proved/certs/l.txt and odd_twist_odd.txt')

