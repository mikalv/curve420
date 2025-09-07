#!/usr/bin/env sage -python
import os
import time
from datetime import datetime, timezone
from sage.all import ZZ, version

def main():
    in_path = os.path.join('proved', 'certs', 'l.txt')
    out_path = os.path.join('proved', 'certs', 'l.proof.txt')
    with open(in_path, 'r') as f:
        n = ZZ(int(f.read().strip()))
    t0 = time.perf_counter()
    isp = n.is_prime(proof=True)
    t1 = time.perf_counter()
    os.makedirs(os.path.dirname(out_path), exist_ok=True)
    with open(out_path, 'w') as out:
        out.write('== Curve420: SageMath primality proof for l ==\n')
        out.write(f'timestamp={datetime.now(timezone.utc).isoformat()}\n')
        out.write(f'sage_version={version()}\n')
        out.write(f'bits={n.nbits()}\n')
        out.write(f'is_prime_proof={isp}\n')
        out.write(f'elapsed_seconds={t1 - t0:.6f}\n')
    print(f'Wrote {out_path}')

if __name__ == '__main__':
    main()

