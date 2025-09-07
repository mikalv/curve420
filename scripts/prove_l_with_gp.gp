\\ PARI/GP script: prove subgroup prime l with proof
\\ Usage: gp -q -f scripts/prove_l_with_gp.gp > proved/certs/l.proof.txt

print("== Curve420: PARI/GP primality check for l ==");
print("pari_version=", version());

\\ Read l from decimal file
Lstr = readstr("proved/certs/l.txt");
if(Lstr==NULL, error("Could not read proved/certs/l.txt"));
\\ Parse as integer (tolerate whitespace/newlines and multiple lines)
S = if(type(Lstr)==t_VEC, concat(Lstr), Lstr);
S = Str(S);
S = subst(S, " ", "");
S = subst(S, "\n", "");
S = subst(S, "\r", "");
n = eval(S);
if(type(n) != t_INT, error("Input is not an integer"));

bits = length(binary(n));
print("bits=", bits);

t0 = getwalltime();
isp = isprime(n);          \\ returns 1 when n is (provably) prime in GP
t1 = getwalltime();
print("isprime=", isp);
print("elapsed_ms=", t1 - t0);

if(isp!=1, print("WARNING: isprime(n) != 1 in this GP session"));
