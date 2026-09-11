"""Independent Arb shifted-inertia replay of analytic Toolkit Tau intervals.

All operations are interval operations. Definite LDLT pivot signs certify
inertia; guide values only choose shifts. Reflection conclusions explicitly
use the exact form's centrosymmetry. No normalized sqrt(2) basis is required.
"""
import argparse,hashlib,json,time
from decimal import Decimal
from pathlib import Path
from flint import arb,fmpq,ctx

def box(a,b):
    return (a+b)/2+arb(0,((b-a)/2).abs_upper())

def inertia(matrix,shift,mass):
    d=len(matrix); L=[[arb(0) for _ in range(d)] for _ in range(d)]; piv=[]
    started=time.perf_counter()
    for i in range(d):
        for j in range(i):
            L[i][j]=(matrix[i][j]-sum((L[i][k]*piv[k]*L[j][k] for k in range(j)),arb(0)))/piv[j]
        p=matrix[i][i]-shift*mass[i]-sum((L[i][k]**2*piv[k] for k in range(i)),arb(0))
        if p.contains(0):
            return {'status':'unresolved','pivot':i,'pivot_ball':p.str(30),'seconds':time.perf_counter()-started}
        piv.append(p); L[i][i]=arb(1)
    return {'status':'certified','negative':sum(p<0 for p in piv),'positive':sum(p>0 for p in piv),
            'pivots':[p.str(45) for p in piv],'seconds':time.perf_counter()-started}

def main():
    if not __debug__:
        raise RuntimeError('Verification requires assertions; do not use python -O')
    ap=argparse.ArgumentParser();ap.add_argument('--input',type=Path,required=True);ap.add_argument('--output',type=Path,required=True);ap.add_argument('--dps',type=int,default=400)
    args=ap.parse_args();ctx.dps=args.dps; start=time.perf_counter()
    raw=args.input.read_bytes(); data=json.loads(raw); N=data['N']; dim=2*N+1
    interval=[(fmpq(x['lower']),fmpq(x['upper'])) for x in data['tau']]
    # Exact symmetric and reflection-orbit intersections, justified by the
    # defining closed-form symmetries. Check all intersections explicitly.
    def canonical(i,j,reflection=True):
        indices={(i,j),(j,i)}
        if reflection: indices|={(dim-1-i,dim-1-j),(dim-1-j,dim-1-i)}
        lo=max(interval[a*dim+b][0] for a,b in indices); hi=min(interval[a*dim+b][1] for a,b in indices)
        assert lo<=hi
        return box(arb(lo),arb(hi))
    full=[[canonical(i,j,False) for j in range(dim)] for i in range(dim)]
    A=[[canonical(i,j) for j in range(dim)] for i in range(dim)]
    even=[[arb(0) for _ in range(N+1)] for _ in range(N+1)]
    even[0][0]=A[N][N]
    for j in range(1,N+1): even[0][j]=even[j][0]=2*A[N][N+j]
    for i in range(1,N+1):
        for j in range(1,N+1): even[i][j]=2*(A[N+i][N+j]+A[N+i][N-j])
    odd=[[2*(A[N+i][N+j]-A[N+i][N-j]) for j in range(1,N+1)] for i in range(1,N+1)]
    assert data['C']==13 and N in (10,120), 'This control qualifies C=13 at N=10 or 120'
    g=json.loads((Path(__file__).parent/'guides.json').read_text())[str(N)]
    result={'scope':'Analytic finite cutoff-free CCM matrix; reflection-sector conclusions assume exact centrosymmetry of the closed-form matrix.',
            'C':13,'N':N,'source_sha256':hashlib.sha256(raw).hexdigest(),'source_precision_bits':data['precision_bits'],
            'verification_dps':args.dps,'guide_manifest':g['manifest'],'method':'Arb interval LDLT without pivoting; generalized parity bases have exact diagonal mass 1,2,... or 2,...',
            'raw_full_inertia':inertia(full,arb(0),[1]*dim),'enclosures':[]}
    print('full',N,result['raw_full_inertia']['status'],flush=True)
    for name,guide,index,matrix,mass in [
        ('even_ground',arb(g['result']['lambda_even']),0,even,[1]+[2]*N),
        ('even_first_excited',arb(g['result']['lambda_even'])+arb(g['result']['even_simplicity_margin']),1,even,[1]+[2]*N),
        ('odd_ground',arb(g['result']['lambda_odd']),0,odd,[2]*N)]:
        # Decimal endpoints interpreted as exact rationals; guides do not prove bounds.
        a,b=Decimal(guide.mid().str(35,radius=False)).as_integer_ratio()
        center=fmpq(a,b); rel=fmpq(1,10**6)
        lo=center*(1-rel);hi=center*(1+rel)
        lower=inertia(matrix,arb(lo),mass); upper=inertia(matrix,arb(hi),mass)
        ok=lower['status']==upper['status']=='certified' and lower['negative']==index and upper['negative']==index+1
        result['enclosures'].append({'name':name,'lower':str(lo),'upper':str(hi),'index':index,'certified':ok,'lower_inertia':lower,'upper_inertia':upper})
        print(name,N,ok,flush=True)
    e0,e1,o0=result['enclosures'];g1=fmpq(e1['lower'])-fmpq(e0['upper']);go=fmpq(o0['lower'])-fmpq(e0['upper'])
    result['even_gap_lower']=str(g1);result['odd_even_gap_lower']=str(go)
    result['status']='certified' if all(x['certified'] for x in result['enclosures']) and g1>0 and go>0 and result['raw_full_inertia'].get('positive')==dim else 'unresolved'
    result['seconds']=time.perf_counter()-start;args.output.write_text(json.dumps(result,indent=2)+'\n')
    print('PASS' if result['status']=='certified' else 'FAIL',result['seconds'],flush=True)
    if result['status']!='certified':
        raise SystemExit(1)

if __name__=='__main__': main()
