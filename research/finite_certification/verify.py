"""Second inertia implementation: symmetric Schur updates with wider input hulls."""
import argparse,hashlib,json,time
from pathlib import Path
from flint import arb,fmpq,ctx

def main():
    if not __debug__:
        raise RuntimeError('Verification requires assertions; do not use python -O')
    ap=argparse.ArgumentParser();ap.add_argument('--input',type=Path,required=True);ap.add_argument('--certificate',type=Path,required=True);ap.add_argument('--output',type=Path,required=True)
    a=ap.parse_args();raw=a.input.read_bytes();source=json.loads(raw);cert=json.loads(a.certificate.read_bytes());ctx.dps=500
    assert hashlib.sha256(raw).hexdigest()==cert['source_sha256']
    assert cert['status']=='certified' and source['C']==cert['C']==13 and source['N']==cert['N']
    N=source['N'];d=2*N+1;intervals=[(fmpq(t['lower']),fmpq(t['upper'])) for t in source['tau']]
    def entry(i,j,parity):
        cells=[(i,j),(j,i)]
        if parity:cells += [(d-1-i,d-1-j),(d-1-j,d-1-i)]
        lo=min(intervals[x*d+y][0] for x,y in cells);hi=max(intervals[x*d+y][1] for x,y in cells)
        return arb((lo+hi)/2)+arb(0,arb((hi-lo)/2).upper())
    full=[[entry(i,j,False) for j in range(d)] for i in range(d)]
    even_basis=[[(N,1)]]+[[(N+j,1),(N-j,1)] for j in range(1,N+1)]
    odd_basis=[[(N+j,1),(N-j,-1)] for j in range(1,N+1)]
    def restrict(basis):
        return [[sum((si*sj*entry(i,j,True) for i,si in vi for j,sj in vj),arb(0)) for vj in basis] for vi in basis]
    even,odd=restrict(even_basis),restrict(odd_basis)
    def count(matrix,shift,mass):
        B=[row.copy() for row in matrix]; negative=0
        for i in range(len(B)):B[i][i]-=shift*mass[i]
        for k in range(len(B)):
            p=B[k][k];assert not p.contains(0),(k,p)
            negative+=bool(p<0)
            for i in range(k+1,len(B)):
                factor=B[i][k]/p
                for j in range(i,len(B)):
                    B[j][i]-=factor*B[j][k];B[i][j]=B[j][i]
        return negative
    started=time.perf_counter();assert count(full,arb(0),[1]*d)==0
    results=[]
    for row in cert['enclosures']:
        matrix,mass=(odd,[2]*N) if row['name']=='odd_ground' else (even,[1]+[2]*N)
        lo,hi=fmpq(row['lower']),fmpq(row['upper']);assert lo<hi
        lower=count(matrix,arb(lo),mass);upper=count(matrix,arb(hi),mass)
        assert (lower,upper)==(row['index'],row['index']+1)
        results.append({'name':row['name'],'lower_negative':lower,'upper_negative':upper,'pass':True})
    e0,e1,o0=cert['enclosures']
    assert fmpq(e0['upper'])<min(fmpq(e1['lower']),fmpq(o0['lower']))
    output={'status':'PASS','method':'Independent Arb Schur-update inertia at 500 digits, using orbit hulls instead of intersections and direct basis contractions',
            'source_sha256':cert['source_sha256'],'certificate_sha256':hashlib.sha256(a.certificate.read_bytes()).hexdigest(),
            'C':13,'N':N,'positive_definite':True,'simple_even_ground':True,'parity_premise':cert['scope'],
            'enclosures':results,'seconds':time.perf_counter()-started}
    a.output.write_text(json.dumps(output,indent=2)+'\n');print('PASS',N,output['seconds'])

if __name__=='__main__':main()
