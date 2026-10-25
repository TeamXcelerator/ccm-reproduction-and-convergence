"""Independent SymPy exact root counts and Arb enclosure replay."""
import json,time
from pathlib import Path
from decimal import Decimal
import sympy as sp
from flint import arb, fmpq, ctx
import argparse, hashlib

if not __debug__:
    raise RuntimeError('Verification requires assertions; do not use python -O')
sha=lambda data:hashlib.sha256(data).hexdigest()
ap=argparse.ArgumentParser()
ap.add_argument('--input',type=Path,required=True)
ap.add_argument('--census',type=Path,required=True)
ap.add_argument('--output',type=Path,required=True)
args=ap.parse_args()

start=time.perf_counter()
result=json.loads(args.census.read_text())
path=args.input
assert sha(path.read_bytes())==result['input_sha256']
obj=json.loads(path.read_text()); N=obj['n_modes']
assert int(obj['lambda_squared'])==13 and N==10 and len(obj['eigenvector'])==21
x=sp.Symbol('s'); v=[sp.Rational(s) for s in obj['eigenvector']]
assert all(v[N-j]==v[N+j] for j in range(N+1))
den=sp.Poly.from_list([1],x,domain=sp.QQ)
for j in range(1,N+1): den*=sp.Poly(x-j*j,x)
num=v[N]*den
for j in range(1,N+1): num+=2*v[N+j]*sp.Poly(x,x)*den.exquo(sp.Poly(x-j*j,x))
assert num.degree()==N and sp.gcd(num,den).degree()==0
assert num.count_roots(0,sp.oo)==N
assert num.count_roots(-sp.oo,0)==0
assert sp.gcd(num,num.diff()).degree()==0
ctx.dps=180; spacing=2*arb.pi()/arb(13).log()
previous=None; rows=[]
for row in result['rows']:
    a,b=sp.Rational(row['s_lower']),sp.Rational(row['s_upper'])
    assert a<b and (previous is None or previous<a)
    previous=b
    assert num.count_roots(a,b)==1
    assert num.eval(a)*num.eval(b)<0
    lo,hi=arb(fmpq(str(a))),arb(fmpq(str(b)))
    center=(lo+hi)/2; s=center+arb(0,(hi-lo)/2); nu=spacing*s.sqrt()
    assert arb(row['nu_interval']).contains(nu)
    jlo=s.sqrt().lower().floor().unique_fmpz(); jhi=s.sqrt().upper().floor().unique_fmpz()
    assert jlo==jhi==row['carrier_index_floor']
    assert row['positive_full_operator_ordinal']==row['movable_rank']+max(0,int(jlo)-N)
    rows.append({'movable_rank':row['movable_rank'],'unique_simple_root':True,'opposite_endpoint_signs':True,'enclosure_replays':True,'carrier_ordinal_replays':True})
out={'status':'PASS','independent_engine':'SymPy exact rational count_roots; Arb at 180 decimal digits',
     'input_sha256':sha(path.read_bytes()),'census_sha256':sha(args.census.read_bytes()),
     'rows':rows,'wall_seconds':time.perf_counter()-start}
args.output.write_text(json.dumps(out,indent=2)+'\n')
print('PASS: all 10 root intervals, total counts, squarefreeness, pole exclusions, and full positive ordinals; seconds',out['wall_seconds'])
