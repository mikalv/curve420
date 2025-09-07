#!/usr/bin/env sage -q
# Verify Curve420 security notes (CONSISTENCY_CHECKLIST.md §9)

import os
import json
from sage.all import (
    GF, ZZ, EllipticCurve, valuation, kronecker_symbol, pari
)

# Compatibility: implement fundamental discriminant without relying on
# version-specific module paths.
def _fundamental_discriminant_and_conductor(Delta):
    Delta = ZZ(Delta)
    if Delta == 0:
        return ZZ(0), ZZ(0)
    d0 = Delta.squarefree_part()
    # For squarefree d0, Disc(Q(√d0)) = d0 if d0 ≡ 1 (mod 4), else 4*d0
    if d0 % 4 == 1:
        D = d0
    else:
        D = 4*d0
    f2 = Delta // D
    # Conductor f with Delta = D * f^2 (take positive f)
    f = ZZ(abs(f2)).isqrt()
    return ZZ(D), ZZ(f)

# Load canonical params
VERBOSE = os.environ.get('VERBOSE', '0') not in ('0', '')
if VERBOSE:
    print('[verify] Loading curve420.json ...')
with open('curve420.json', 'r') as f:
    J = json.load(f)

p = ZZ(J['field']['p'])
A = ZZ(J['montgomery']['A'])
N = ZZ(J['group']['N'])
h = ZZ(J['group']['h'])
l = ZZ(J['group']['l'])

Fp = GF(p)
Af = Fp(A)

# Montgomery curve (B = 1): y^2 = x^3 + A x^2 + x
if VERBOSE:
    print('[verify] Building Montgomery curve and basic invariants ...')
E = EllipticCurve(Fp, [0, Af, 0, 1, 0])

# Trace and twist order
t = p + 1 - N
N_twist = 2*p + 2 - N
v2_twist = valuation(N_twist, 2)
odd_twist = N_twist >> v2_twist

# 1) d non-square in the a = -1 Edwards model
if VERBOSE:
    print('[verify] Edwards a=-1 d non-square check ...')
# Using the relation d_m1 = (2 - A)/(A + 2) in Fp when a = -1
num = (ZZ(2) - A) % p
den = (A + 2) % p
den_inv = pow(den, -1, p)
d_m1 = (num * den_inv) % p
# Legendre symbol: -1 means non-square (quadratic non-residue)
leg_d = kronecker_symbol(ZZ(d_m1), p)
d_m1_is_nonsquare = (leg_d == -1)

# 2) j-invariant != 0, 1728
if VERBOSE:
    print('[verify] j-invariant check ...')
# For Montgomery y^2 = x^3 + A x^2 + x (B = 1): j = 256*(A^2 - 3)^3 / (A^2 - 4)
A2 = Af*Af
num_j = Fp(256) * (A2 - 3)**3
den_j = (A2 - 4)
j_inv = num_j / den_j
j_is_0 = (j_inv == 0)
j_is_1728 = (j_inv == Fp(1728))

# 3) Embedding degree lower bound (MOV): search for small k
max_k = int(os.environ.get('MOV_MAX_K', '200'))
if VERBOSE:
    print(f'[verify] MOV search up to k={max_k} ...')
found_k = None
for k in range(1, max_k + 1):
    if pow(int(p), k, int(l)) == 1:
        found_k = k
        break
mov_lower_bound_ok = (found_k is None)

# 3b) Extra transparency for Cheon-like considerations
gcd_lm1_pm1 = {
    'gcd(l-1, p-1)': int(ZZ(l-1).gcd(p-1)),
    'gcd(l-1, p+1)': int(ZZ(l-1).gcd(p+1)),
}

# 4) Twist security: odd part of N_twist should be large prime (or probable prime)
# Quick probable primality (proof=False), fast and sufficient for a report
twist_proof = os.environ.get('TWIST_PROOF', '0') not in ('0', '')
if VERBOSE:
    print(f"[verify] Twist odd-part primality (proof={'True' if twist_proof else 'False'}) ...")
odd_twist_probable_prime = ZZ(odd_twist).is_prime(proof=(True if twist_proof else False))

# 5) Cofactor checks
cofactor_ok = (N == h * l)
not_anomalous = (N != p)
v2N = valuation(N, 2)
h2 = ZZ(1) << v2N

# 5b) Basepoint sanity, mapping round-trip, and cofactor-clearing sanity
Ge_ok = None
Gm_ok = None
h2_G_not_zero = None
roundtrip_ok = None
try:
    u0 = Fp(ZZ(J['montgomery']['base_point']['u']))
    v0 = Fp(ZZ(J['montgomery']['base_point']['v']))
    x0 = Fp(ZZ(J['edwards']['base_point']['x']))
    y0 = Fp(ZZ(J['edwards']['base_point']['y']))
    Gm = E(u0, v0)
    # Check ℓ * Gm == O
    Gm_ok = ((ZZ(l) * Gm).is_zero())
    # Map to Edwards coordinates via frozen mapping
    xe = u0 / v0
    ye = (u0 - 1) / (u0 + 1)
    Ge_ok = (xe == x0 and ye == y0)
    # Round-trip back to Montgomery
    um = (1 + ye) / (1 - ye)
    vm = um / xe
    roundtrip_ok = (um == u0 and vm == v0)
    # Cofactor-clearing sanity: h2 * G != O for generator of order ℓ
    h2_G_not_zero = ( (ZZ(h2) * Gm).is_zero() == False )
except Exception:
    pass

# 6) CM discriminant summary
if VERBOSE:
    print('[verify] CM discriminant decomposition ...')
Delta = ZZ(t)*ZZ(t) - 4*ZZ(p)  # negative for ordinary curves
D_fund, f_cond = _fundamental_discriminant_and_conductor(Delta)
f2 = (Delta // D_fund)
cm_info = {
    'Delta': str(Delta),
    'Delta_bits': int(abs(Delta).nbits()),
    'D_fund': str(D_fund),
    'D_fund_bits': int(abs(D_fund).nbits()),
    'conductor_f': int(f_cond),
}

# Optional: class number of order with discriminant Delta (can be expensive)
try_classno = os.environ.get('CM_CLASSNO', '0') not in ('0', '')
class_no = None
if try_classno:
    if VERBOSE:
        print('[verify] PARI class number qfbclassno(Δ) ... (can be slow)')
    try:
        class_no = int(pari(int(Delta)).qfbclassno())
    except Exception:
        class_no = None
cm_info['class_number_optional'] = (None if class_no is None else int(class_no))

# 4b) Try partial factorization of odd_twist for transparency
trial_bound = int(os.environ.get('TWIST_TRIAL_BOUND', '200000'))
rho_attempts = int(os.environ.get('TWIST_RHO_ATTEMPTS', '4'))
rho_iters = int(os.environ.get('TWIST_RHO_ITERS', '25000'))

if VERBOSE:
    print(f"[verify] Twist factorization: trial primes ≤ {trial_bound}, rho attempts={rho_attempts}, iters={rho_iters} ...")

def small_factor_trial(n, bound):
    from sage.arith.all import prime_range
    factors = []
    n = ZZ(n)
    for p in prime_range(bound+1):
        if n % p == 0:
            e = 0
            while n % p == 0:
                n //= p
                e += 1
            factors.append((int(p), int(e)))
        if n == 1:
            break
    return n, factors

def rho_find_factor(n, max_iters):
    # Simple Pollard's Rho
    from random import randrange
    n = ZZ(n)
    if n % 2 == 0:
        return ZZ(2)
    for _ in range(rho_attempts):
        x = ZZ(randrange(2, n-1))
        c = ZZ(randrange(1, n-1))
        y = x
        d = ZZ(1)
        i = 0
        while d == 1 and i < max_iters:
            x = (x*x + c) % n
            y = (y*y + c) % n
            y = (y*y + c) % n
            d = (x - y).abs().gcd(n)
            i += 1
        if 1 < d < n:
            return d
    return None

remaining, small_factors = small_factor_trial(odd_twist, trial_bound)
remaining = ZZ(remaining)

rho_factor = None
if remaining != 1 and not remaining.is_prime(proof=False):
    rho_factor = rho_find_factor(remaining, rho_iters)
    if rho_factor is not None:
        # split remaining
        q = remaining // rho_factor
        # normalize so smaller first
        if q < rho_factor:
            rho_factor, q = q, rho_factor
        # accumulate small_factors with rho_factor (exponent 1 by construction)
        small_factors.append((int(rho_factor), 1))
        remaining = ZZ(q)

remaining_probable_prime = (remaining == 1) or remaining.is_prime(proof=False)
remaining_bits = (0 if remaining == 1 else int(remaining.nbits()))

twist_factorization = {
    'trial_bound': int(trial_bound),
    'rho_attempts': int(rho_attempts),
    'rho_iters': int(rho_iters),
    'small_factors': small_factors,
    'rho_factor_found': (None if rho_factor is None else int(rho_factor)),
    'remaining': str(int(remaining)),
    'remaining_bits': int(remaining_bits),
    'remaining_probable_prime': bool(remaining_probable_prime),
}

# Summarize
summary = {
    'field_bits': int(p.nbits()),
    'N_bits': int(N.nbits()),
    'l_bits': int(l.nbits()),
    'p': str(p),
    'A': str(A),
    'N': str(N),
    'h': str(h),
    'l': str(l),
    'trace_t': str(t),
    'N_twist': str(N_twist),
    'v2_twist': int(v2_twist),
    'odd_twist': str(odd_twist),
    'odd_twist_probable_prime': bool(odd_twist_probable_prime),
    'edwards_a_minus_1_d': str(d_m1),
    'edwards_a_minus_1_d_nonsquare': bool(d_m1_is_nonsquare),
    'j_invariant_fp': str(int(j_inv)),
    'j_is_0': bool(j_is_0),
    'j_is_1728': bool(j_is_1728),
    'mov_small_k_found': (None if found_k is None else int(found_k)),
    'mov_k_search_max': int(max_k),
    'gcd_lm1_pm1': gcd_lm1_pm1,
    'cofactor_relation_ok': bool(cofactor_ok),
    'not_anomalous_N_ne_p': bool(not_anomalous),
    'cm': cm_info,
    'twist_factorization': twist_factorization,
    'v2N': int(v2N),
    'h2': str(int(h2)),
    'basepoint_checks': {
        'l_times_Gm_is_O': (None if Gm_ok is None else bool(Gm_ok)),
        'mapping_matches_json': (None if Ge_ok is None else bool(Ge_ok)),
        'roundtrip_ok': (None if roundtrip_ok is None else bool(roundtrip_ok)),
        'h2_times_G_not_zero': (None if h2_G_not_zero is None else bool(h2_G_not_zero)),
    },
}

# Print human-readable report
print('--- Curve420 Security Report ---')
print(f"Field bits: {summary['field_bits']}")
print(f"Group order N bits: {summary['N_bits']} | Prime l bits: {summary['l_bits']}")
print(f"Trace t = p + 1 - N: {summary['trace_t']}")
print()
print('Complete Edwards (a = -1) check:')
print(f"  d = (2 - A)/(A + 2) mod p is non-square? {summary['edwards_a_minus_1_d_nonsquare']}")
print()
print('j-invariant sanity:')
print(f"  j == 0? {summary['j_is_0']} | j == 1728? {summary['j_is_1728']}")
print()
print('MOV embedding degree lower bound:')
if found_k is None:
    print(f"  No k ≤ {max_k} with l | (p^k - 1) found (good)")
else:
    print(f"  Found small embedding degree k = {found_k} (bad; consider revisiting)")
print()
print('Cheon-related transparency:')
print(f"  gcd(l-1, p-1) = {gcd_lm1_pm1['gcd(l-1, p-1)']}")
print(f"  gcd(l-1, p+1) = {gcd_lm1_pm1['gcd(l-1, p+1)']}")
print()
print('Twist security:')
print(f"  N_twist 2-adic valuation v2 = {summary['v2_twist']}")
print(f"  odd(N_twist) probable prime? {summary['odd_twist_probable_prime']}")
print(f"  small factors found (trial): {twist_factorization['small_factors']}")
if twist_factorization['rho_factor_found'] is not None:
    print(f"  rho factor found: {twist_factorization['rho_factor_found']}")
print(f"  remaining cofactor bits: {twist_factorization['remaining_bits']}, probable prime? {twist_factorization['remaining_probable_prime']}")
print()
print('Cofactor & anomalous checks:')
print(f"  N == h * l ? {summary['cofactor_relation_ok']}")
print(f"  N != p (not anomalous)? {summary['not_anomalous_N_ne_p']}")
print(f"  v2(N) = {summary['v2N']} | h2 = {summary['h2']}")
if summary['basepoint_checks']['l_times_Gm_is_O'] is not None:
    print('Basepoint & mapping checks:')
    print(f"  l * G_M == O ? {summary['basepoint_checks']['l_times_Gm_is_O']}")
    print(f"  Mapping G_M -> G_E matches JSON? {summary['basepoint_checks']['mapping_matches_json']}")
    print(f"  Round-trip G_M <-> G_E ok? {summary['basepoint_checks']['roundtrip_ok']}")
    print(f"  h2 * G (cofactor sanity) != O ? {summary['basepoint_checks']['h2_times_G_not_zero']}")
print()
print('CM discriminant summary:')
print(f"  Δ = t^2 - 4p = {cm_info['Delta']} (bits={cm_info['Delta_bits']})")
print(f"  Fundamental discriminant D = {cm_info['D_fund']} (bits={cm_info['D_fund_bits']}), conductor f = {cm_info['conductor_f']}")
if class_no is not None:
    print(f"  Class number h(Δ) = {class_no}")

# Write machine-readable report
os.makedirs('proved', exist_ok=True)
with open('proved/security_report.json', 'w') as f:
    json.dump(summary, f, indent=2)

print('\nWrote proved/security_report.json')
