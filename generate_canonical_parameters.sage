# Sage script to generate the canonical parameters and base point for Curve420
import json
import hashlib
from sage.all import GF, EllipticCurve, ZZ, set_random_seed

# 1. Load parameters from hash_5769.json
with open('hash_5769.json', 'r') as f:
    hit = json.load(f)

p = (1 << 420) - 335
Fp = GF(p)
A = Fp(int(hit["A"]))
N = int(hit["N"])
v2 = int(hit["v2"])
M = int(hit["M"])
idx = hit.get("index","?")

# 2. Derive Edwards parameters from A
a = A + 2
d = A - 2

print("--- Derived Edwards Parameters ---")
print(f"a = {a}")
print(f"d = {d}")

# 3. Create the Montgomery curve (B=1)
E = EllipticCurve(Fp, [0, A, 0, 1, 0])

# 4. Determine group order l
l = ZZ(M)
assert l.is_prime()
h = ZZ(1 << v2)

print("\n--- Group Properties ---")
print(f"p bits = {p.bit_length()}")
print(f"l bits = {int(l).bit_length()} (prime? {ZZ(l).is_prime()})")
print(f"N = {N}, h = {h}, l = {l}")

# 5. Find a generator on the Montgomery curve deterministically
O = E(0)
scale = ZZ(N // l)
seed = f"Curve420:hash:{idx}:mont"
set_random_seed(int.from_bytes(hashlib.shake_128(seed.encode()).digest(16),'big'))

Gm = None
for _ in range(1024):
    R = E.random_point()
    H = scale * R
    if H != O and (ZZ(l) * H) == O:
        Gm = H
        break
if Gm is None:
    raise RuntimeError("Could not find a point of order l on the Montgomery curve.")

u, v = Fp(Gm[0]), Fp(Gm[1])

# 6. Map to Edwards coordinates
x = u / v
y = (u - 1) / (u + 1)

# 7. Verify the Edwards equation a*x^2 + y^2 = 1 + d*x^2*y^2
lhs = a*x*x + y*y
rhs = Fp(1) + d*x*x*y*y
assert lhs == rhs

print("\n--- Canonical Base Points ---")
print("Montgomery (u,v):")
print(f"u = {int(u)}")
print(f"v = {int(v)}")
print("\nEdwards (x,y):")
print(f"x = {int(x)}")
print(f"y = {int(y)}")

# 8. Save parameters to a new json file
params = {
    "field": {"p": str(p), "bits": p.bit_length()},
    "group": {"N": str(N), "h": str(h), "l": str(l)},
    "montgomery": {
        "equation": "y^2 = x^3 + A*x^2 + x",
        "A": str(int(A)),
        "base_point": {"u": str(int(u)), "v": str(int(v))}
    },
    "edwards": {
        "equation": "a*x^2 + y^2 = 1 + d*x^2*y^2",
        "a": str(int(a)),
        "d": str(int(d)),
        "base_point": {"x": str(int(x)), "y": str(int(y))}
    }
}

with open('curve420_canonical.json', 'w') as f:
    json.dump(params, f, indent=4)

print("\nSuccessfully generated canonical parameters and saved to curve420_canonical.json")
