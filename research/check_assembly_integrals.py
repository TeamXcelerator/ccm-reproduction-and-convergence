"""Independent numerical integration of CCM (3.14), (3.16), and (4.4).

Compare all entries of the C=13, N=10 matrix with analytic interval assembly.
Tanh-sinh quadrature is a numerical cross-check, not a rigorous enclosure.
"""
import argparse
import hashlib
import json
import time
from pathlib import Path
import mpmath as mp


def main():
    if not __debug__:raise RuntimeError('Do not use -O')
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--matrix',type=Path,required=True)
    ap.add_argument('--output',type=Path,required=True)
    args=ap.parse_args();raw=args.matrix.read_bytes();data=json.loads(raw)
    C=data['C'];N=data['N'];d=2*N+1;assert C==13 and N==10
    powers=[(2,2),(3,3),(4,2),(5,5),(7,7),(8,2),(9,3),(11,11),(13,13)]
    started=time.perf_counter();results=[]
    for digits in (60,90):
        mp.mp.dps=digits;L=mp.log(C);full={};maxdiff=mp.mpf(0);zero=None
        for n in range(-N,N+1):
            for m in range(n,N+1):
                key=min((n,m),(-m,-n))
                if key not in full:
                    w0=mp.mpf(2) if n==m else mp.mpf(0)
                    def omega(y):
                        if n==m:return 2*(1-y/L)*mp.cos(2*mp.pi*n*y/L)
                        return (mp.sin(2*mp.pi*m*y/L)-mp.sin(2*mp.pi*n*y/L))/(mp.pi*(n-m))
                    def arch(y):
                        if y==0:return mp.mpf('0.5')-1/L if n==m else mp.mpf(-1)
                        return (mp.exp(y/2)*omega(y)-w0)/(2*mp.sinh(y))
                    partition=[L*j/4 for j in range(5)]
                    pole=mp.quad(lambda y:2*mp.cosh(y/2)*omega(y),partition)
                    # Keep the rational cutoff factor exact in the logarithm.
                    wr=mp.quad(arch,partition)+w0/2*(mp.euler+mp.log(4*mp.pi*mp.mpf(C-1)/(C+1)))
                    wp=sum(mp.log(p)/mp.sqrt(k)*omega(mp.log(k)) for k,p in powers)
                    full[key]=pole-wr-wp
                    if n==m==0:zero={'pole':mp.nstr(pole,45),'archimedean':mp.nstr(wr,45),'prime':mp.nstr(wp,45),'tau':mp.nstr(full[key],45)}
                value=full[key];entry=data['tau'][(n+N)*d+m+N]
                exact=lambda s:mp.mpf(s.split('/')[0])/mp.mpf(s.split('/')[1]) if '/' in s else mp.mpf(s)
                midpoint=(exact(entry['lower'])+exact(entry['upper']))/2
                maxdiff=max(maxdiff,abs(value-midpoint))
            print('integration',digits,'row',n,flush=True)
        assert maxdiff<mp.power(10,-digits+10)
        matrix=mp.matrix([[full[min((min(n,m),max(n,m)),(-max(n,m),-min(n,m))) ]
                           for m in range(-N,N+1)] for n in range(-N,N+1)])
        eigenvalues=mp.eigsy(matrix,eigvals_only=True)
        even=mp.matrix(N+1);odd=mp.matrix(N)
        for i in range(N+1):
            for j in range(N+1):
                even[i,j]=(matrix[N,N] if i==j==0 else mp.sqrt(2)*matrix[N,N+j] if i==0
                           else mp.sqrt(2)*matrix[N+i,N] if j==0 else matrix[N+i,N+j]+matrix[N+i,N-j])
        for i in range(N):
            for j in range(N):odd[i,j]=matrix[N+i+1,N+j+1]-matrix[N+i+1,N-j-1]
        ev=mp.eigsy(even,eigvals_only=True);ov=mp.eigsy(odd,eigvals_only=True)
        assert 0<ev[0]<min(ev[1],ov[0]) and abs(eigenvalues[0]-ev[0])<mp.power(10,-digits+5)
        results.append({'dps':digits,'dimension':d,'entries_covered':d*d,'independent_integrals':len(full),
                        'max_absolute_entry_difference':mp.nstr(maxdiff,25),'zero_mode_components':zero,
                        'even_ground':mp.nstr(ev[0],40),'odd_ground':mp.nstr(ov[0],40),
                        'next_even':mp.nstr(ev[1],40),
                        'matrix_point_values':[mp.nstr(matrix[i,j],digits-5) for i in range(d) for j in range(d)],
                        'status':'PASS'})
        args.output.write_text(json.dumps({'status':'IN_PROGRESS','rows':results},indent=2)+'\n')
    args.output.write_text(json.dumps({'status':'PASS','C':C,'N':N,'scope':__doc__,
        'primary_source':'https://arxiv.org/html/2511.22755v1#S4.SS3',
        'matrix_sha256':hashlib.sha256(raw).hexdigest(),'rows':results,'seconds':time.perf_counter()-started},indent=2)+'\n')
    print('PASS independent integral assembly',[(r['dps'],r['max_absolute_entry_difference']) for r in results],flush=True)


if __name__=='__main__':main()
