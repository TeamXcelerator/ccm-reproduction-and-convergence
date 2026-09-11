"""Exact rational, reference-free census of the retained C=13,N=10 source."""
import json,time,hashlib
from decimal import Decimal
from flint import fmpq, fmpq_poly, arb, ctx
import argparse
from pathlib import Path

def exact_decimal(s):
    a,b=Decimal(s).as_integer_ratio(); return fmpq(a,b)

def normalized(p):
    # Scaling by a POSITIVE constant preserves the Sturm signs.
    return p/abs(p.leading_coefficient()) if p else p

def sign(v): return 1 if v>0 else -1 if v<0 else 0

def main():
    start=time.perf_counter()
    if not __debug__:
        raise RuntimeError('Verification requires assertions; do not use python -O')
    ap=argparse.ArgumentParser()
    ap.add_argument('--input',type=Path,required=True)
    ap.add_argument('--output',type=Path,required=True)
    args=ap.parse_args(); raw=args.input.read_bytes(); o=json.loads(raw)
    assert int(o['lambda_squared'])==13 and o['n_modes']==10
    N=o['n_modes']; v=list(map(exact_decimal,o['eigenvector']))
    assert len(v)==2*N+1
    assert all(v[N-j]==v[N+j] for j in range(N+1))
    x=fmpq_poly([0,1]); denominator=fmpq_poly([1])
    for j in range(1,N+1): denominator*=x-j*j
    numerator=v[N]*denominator
    for j in range(1,N+1):
        quotient,remainder=divmod(denominator,x-j*j); assert not remainder
        numerator+=2*v[N+j]*x*quotient
    assert numerator.degree()==N
    assert numerator.gcd(denominator).degree()==0
    assert numerator.gcd(numerator.derivative()).degree()==0
    seq=[normalized(numerator),normalized(numerator.derivative())]
    while seq[-1].degree()>0:
        seq.append(normalized(-(seq[-2]%seq[-1])))
    def variations(point):
        if point is None: signs=[sign(p.leading_coefficient()) for p in seq]
        elif point=='-inf': signs=[sign(p.leading_coefficient())*(-1)**p.degree() for p in seq]
        else: signs=[sign(p(point)) for p in seq]
        signs=[s for s in signs if s]
        return sum(a!=b for a,b in zip(signs,signs[1:]))
    v0=variations(fmpq(0)); vinf=variations(None); vneg=variations('-inf')
    assert v0-vinf==N and vneg-v0==0
    right=fmpq(N*N*4)
    while variations(right)!=vinf: right*=2
    pending=[(fmpq(0),right,v0,vinf)]; brackets=[]; target=fmpq(1,10**70)
    while pending:
        lo,hi,vl,vh=pending.pop()
        if vl==vh: continue
        if vl-vh==1 and hi-lo<target:
            brackets.append((lo,hi)); continue
        mid=(lo+hi)/2
        assert numerator(mid)!=0, 'An exact dyadic root needs separate endpoint handling'
        vm=variations(mid)
        pending.extend([(lo,mid,vl,vm),(mid,hi,vm,vh)])
    brackets.sort(); assert len(brackets)==N
    ctx.dps=120
    h=2*arb.pi()/arb(o['lambda_squared']).log(); rows=[]
    for i,(lo,hi) in enumerate(brackets,1):
        s=(arb(lo)+arb(hi))/2+arb(0,arb(hi-lo)/2)
        t=h*s.sqrt(); y=s.sqrt()
        # Outward endpoints must agree on carrier lattice count.
        jlo=int(str(y.lower().floor().unique_fmpz()))
        jhi=int(str(y.upper().floor().unique_fmpz()))
        assert jlo==jhi
        tail_before=max(0,jlo-N)
        rows.append({'movable_rank':i,'s_lower':str(lo),'s_upper':str(hi),
                     'nu_interval':t.str(65),'carrier_index_floor':jlo,
                     'untouched_positive_carriers_below':tail_before,
                     'positive_full_operator_ordinal':i+tail_before})
    # Reference values enter only AFTER all roots and ordinals are isolated.
    import mpmath as mp
    mp.mp.dps=100
    for row in rows[:5]:
        k=row['movable_rank']; lo=mp.mpf(row['s_lower']); hi=mp.mpf(row['s_upper'])
        center=2*mp.pi/mp.log(13)*mp.sqrt((lo+hi)/2); ref=mp.im(mp.zetazero(k))
        row['reference_k_after_census']=k
        row['relative_matching_digits']=mp.nstr(-mp.log10(abs(center-ref)/abs(ref)),15)
    result={'scope':'Exact decimal retained vector; exact rational numerator census; Arb enclosure of pi/log(C). No analytic eigenstate enclosure.',
            'input_sha256':hashlib.sha256(raw).hexdigest(),
            'configuration':{'C':13,'N':10,'precision_bits':o['precision_bits']},
            'degree':N,'positive_simple_roots':N,'negative_roots':0,'pole_cancellations':0,
            'zero_is_numerator_root':bool(numerator(0)==0),'sturm_variations':{'negative_infinity':vneg,'zero':v0,'positive_infinity':vinf},
            'rows':rows,'wall_seconds':time.perf_counter()-start}
    args.output.write_text(json.dumps(result,indent=2)+'\n')
    for row in rows:
        print(row['movable_rank'],row['nu_interval'],row['positive_full_operator_ordinal'],row.get('relative_matching_digits',''))
    print('PASS: exact root census; seconds',result['wall_seconds'])

if __name__=='__main__': main()
