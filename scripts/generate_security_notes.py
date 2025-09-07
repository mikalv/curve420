#!/usr/bin/env python3
import json
from datetime import datetime, timezone

def load_json(path):
    with open(path, 'r') as f:
        return json.load(f)

R = load_json('proved/security_report.json')
C = load_json('curve420.json')

p_bits = R.get('field_bits')
l_bits = R.get('l_bits')
h = int(C['group']['h'])

d_ok = R.get('edwards_a_minus_1_d_nonsquare')
j0 = R.get('j_is_0')
j1728 = R.get('j_is_1728')
mov_found = R.get('mov_small_k_found')
mov_max = R.get('mov_k_search_max')
g1 = R.get('gcd_lm1_pm1', {}).get('gcd(l-1, p-1)')
g2 = R.get('gcd_lm1_pm1', {}).get('gcd(l-1, p+1)')
v2_twist = R.get('v2_twist')
odd_twist_prp = R.get('odd_twist_probable_prime')
tw_fac = R.get('twist_factorization', {})
nf_ok = R.get('cofactor_relation_ok')
not_anom = R.get('not_anomalous_N_ne_p')

lines = []
lines.append('# Security Notes – Curve420')
lines.append('')
lines.append(f'Generated: {datetime.now(timezone.utc).isoformat()}')
lines.append('')
lines.append('## Summary')
lines.append(f'- Field size: {p_bits} bits; subgroup prime ℓ: {l_bits} bits; cofactor h = {h}.')
lines.append(f'- Cofactor relation N = h·ℓ holds: {"yes" if nf_ok else "no"}; anomalous N = p: {"no" if not_anom else "no"}.')
lines.append('')
lines.append('## 9) Security Notes')
lines.append(f'- d non-square in Edwards a = −1 model: {"True" if d_ok else "False"}.')
lines.append(f'- j-invariant: not 0 ({"True" if not j0 else "False"}), not 1728 ({"True" if not j1728 else "False"}).')
if mov_found is None:
    lines.append(f'- MOV embedding degree: no k ≤ {mov_max} with ℓ | (p^k − 1).')
else:
    lines.append(f'- MOV embedding degree: found k = {mov_found} (≤ {mov_max}).')
lines.append(f'- Cheon transparency: gcd(ℓ−1, p−1) = {g1}, gcd(ℓ−1, p+1) = {g2}.')
lines.append(f'- Twist security: v2(N_twist) = {v2_twist}; odd(N_twist) probable prime: {"True" if odd_twist_prp else "False"}.')

small_factors = tw_fac.get('small_factors')
rem_bits = tw_fac.get('remaining_bits')
rem_prp = tw_fac.get('remaining_probable_prime')
rho_found = tw_fac.get('rho_factor_found')
if small_factors is not None:
    if small_factors:
        lines.append(f'- Twist factorization (bounded search): small factors found = {small_factors}.')
    else:
        lines.append(f'- Twist factorization (bounded search): no small factors ≤ {tw_fac.get("trial_bound")} found.')
    if rho_found is not None:
        lines.append(f'- Pollard Rho found factor: {rho_found}.')
    lines.append(f'- Remaining cofactor: {rem_bits} bits; probable prime: {"True" if rem_prp else "False"}.')

lines.append('')
lines.append('## Protocol Rules (recommended)')
lines.append('- ECDH (Montgomery u-only): perform on-curve decoding, reject out-of-range/non-canonical u, and check for all-zero shared secret.')
lines.append(f'- Signatures (Edwards): use cofactor-aware verification or prime-order encoding; if using cofactor-clearing, multiply by h = {h} where relevant.')
lines.append('- Encoding: specify endianness and canonical sign-bit; ensure decode/encode round-trip and reject non-canonical encodings.')

with open('SECURITY_NOTES.md', 'w') as f:
    f.write('\n'.join(lines) + '\n')

print('Wrote SECURITY_NOTES.md')
