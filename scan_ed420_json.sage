# scan_ed420_json.sage
# Bruk: sage scan_ed420_json.sage fam=frac lo=1 hi=1000 small=1000000 accept=2,3 p=... outdir=hits jsonl=1
import sys, hashlib, json, os
from sage.all import *

# ---- args ----
args = dict(fam="frac", lo=1, hi=1000, small=10**6, p=(1<<420)-335, accept="2,3", outdir="", jsonl=0)
for a in sys.argv[1:]:
    k,v = a.split("=",1)
    if k in ("lo","hi","small","p","jsonl"):
        v = int(v)
    if k == "accept":
        v = [int(x) for x in v.split(",") if x.strip()]
    args[k]=v

p = args["p"]; Fp = GF(p); a = Fp(-1)

if isinstance(args["accept"], str):
    args["accept"] = [int(x) for x in args["accept"].split(",") if x.strip()]

ACCEPT = set(args["accept"])
OUTDIR = args["outdir"]
JSONL  = bool(args["jsonl"])

if OUTDIR:
    os.makedirs(OUTDIR, exist_ok=True)

def pyify(x):
    # Konverterer Sage-typer rekursivt til rene Python-typer
    from sage.all import Integer as SageInteger  # funker i Sage-py
    try:
        from sage.rings.integer import Integer as SRInteger
        SI = (SageInteger, SRInteger)
    except Exception:
        SI = (int,)  # fallback

    if x is None or isinstance(x, (bool, int, float, str)):
        return x
    if isinstance(x, (list, tuple)):
        return [pyify(t) for t in x]
    if isinstance(x, dict):
        return {pyify(k): pyify(v) for k, v in x.items()}
    # forsøk int()-konvertering
    try:
        return int(x)
    except Exception:
        # fallback til streng
        return str(x)

def legendre_is_nonsquare(z):
    # True hvis z er IKKE-kvadrat
    return pow(int(z), (p-1)//2, p) == p-1

def ok_d(d):
    if d in (Fp(0),Fp(1),Fp(-1)): return False
    if (a - d) == 0: return False
    return legendre_is_nonsquare(d)

def mont_curve_from_d(d):
    # TE (a=-1,d) -> Montgomery y^2 = x^3 + A x^2 + x (B=1-variant)
    A = 2*(a + d)/(a - d)
    # For effektiv SEA i Sage: generelle Weierstrass-koeff (y^2 = x^3 + (A) x^2 + x)
    Em = EllipticCurve(Fp, [0, A, 0, 1, 0])
    return Em, A

from sage.all import pari

def screen_candidate(d, fam, index, small_bound):
    rec = {
        "fam": fam,
        "index": index,
        "p": f"2^420 - 335",
        "a": -1,
        "d": int(d),              # alltid ta med d
    }
    if not ok_d(d):
        rec.update(status="bad_d", reason="d square or singular")
        return rec

    E, A = mont_curve_from_d(d)
    N = E.cardinality()
    v2 = valuation(N, 2)
    M = N >> v2
    rec.update({
        "status": "tested",
        "A": int(A),
        "B": 1,
        "N": int(N),
        "v2": int(v2),
        "h_min": 1<<v2,
        "M": int(M),              # legg til M
        "M_bits": int(Integer(M).nbits())
    })

    if v2 not in ACCEPT:
        rec.update(status="bad_cofactor_v2", reason=f"v2={v2} not in {sorted(ACCEPT)}")
        return rec

    # småfaktor-sjekk på M
    for q in prime_range(3, small_bound):
        if M % q == 0:
            rec.update(status="smallfactor", small_factor=int(q))
            return rec

    # BPSW (pseudoprime) via PARI
    try:
        if not bool(pari(int(M)).ispseudoprime()):
            rec.update(status="bpsw_composite")
            return rec
    except Exception as e:
        rec.update(status="error_pari", error=str(e))
        return rec

    rec.update(status="ok")
    return rec

fam = args["fam"]; lo=args["lo"]; hi=args["hi"]
print(json.dumps(pyify({
    "p_bits": int(p.nbits()),
    "p_mod_4": int(p%4),
    "fam": fam,
    "range": [lo, hi],
    "accept_v2": sorted(ACCEPT)
}), separators=(",",":")))
sys.stdout.flush()

hits = []
def emit(obj, fname_hint=None):
    pobj = pyify(obj)

    # Alltid print til stdout hvis du kjører med --jsonl (for logging/debugging)
    if JSONL:
        print(json.dumps(pobj, separators=(",",":")))

    # Bare skriv til fil hvis status er "ok" (dvs. treffer accept=2,3, bpsw prime, osv.)
    if OUTDIR and pobj.get("status") == "ok":
        tag = pobj.get("fam", "unk")
        idx = pobj.get("index", "x")
        fname = fname_hint or f"{tag}_{idx}.json"
        path = os.path.join(OUTDIR, fname)
        with open(path, "w") as f:
            json.dump(pobj, f, separators=(",",":"))

if fam == "frac":
    for k in range(lo, hi+1):
        d = (-k) * (Fp(k+1))**(-1)
        rec = screen_candidate(d, "frac", int(k), args["small"])
        emit(rec, fname_hint=f"frac_{k}.json")
        if rec["status"]=="ok":
            hits.append(rec)
elif fam == "neg":
    for m in range(lo, hi+1):
        d = Fp(-m)
        rec = screen_candidate(d, "neg", int(m), args["small"])
        emit(rec, fname_hint=f"neg_{m}.json")
        if rec["status"]=="ok":
            hits.append(rec)
elif fam == "hash":
    for i in range(lo, hi+1):
        h = int.from_bytes(hashlib.shake_128(str(i).encode()).digest(64),"big") % p
        d = Fp(- (h*h) * (Fp(1) / (h*h + 1)))
        rec = screen_candidate(d, "hash", int(i), args["small"])
        rec["hash_source"] = "shake128(i)"
        emit(rec, fname_hint=f"hash_{i}.json")
        if rec["status"]=="ok":
            hits.append(rec)
else:
    emit({"status":"error","reason":"Unknown fam","fam":fam})

# Oppsummering til slutt
print(json.dumps({
    "summary":"done",
    "ok_count": len(hits)
}, separators=(",",":")))
