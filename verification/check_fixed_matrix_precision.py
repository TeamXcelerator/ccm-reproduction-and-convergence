"""Round one fixed C=13,N=10 matrix, then solve each rounded problem at 220 dps.

This separates input rounding from matrix recomputation and solver precision.
Outcomes at deliberately inadequate precisions are retained as observations.
"""
import argparse,hashlib,json,time
from pathlib import Path
import mpmath as mp


def main():
    if not __debug__:raise RuntimeError('Do not use -O')
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--matrix',type=Path,required=True)
    ap.add_argument('--output',type=Path,required=True)
    args=ap.parse_args();raw=args.matrix.read_bytes();data=json.loads(raw)
    N=data['N'];C=data['C'];assert C==13 and N==10;d=2*N+1
    mp.mp.dps=220;started=time.perf_counter()
    def rational(s):
        a,sep,b=s.partition('/');return mp.mpf(a)/mp.mpf(b) if sep else mp.mpf(a)
    original=mp.matrix(d,d)
    for i,row in enumerate(data['tau']):original[i//d,i%d]=(rational(row['lower'])+rational(row['upper']))/2
    reference_values,reference_vectors=mp.eigsy(original)
    q0=reference_vectors[:,0];lambda0=reference_values[0]
    gamma=mp.im(mp.zetazero(1));spacing=2*mp.pi/mp.log(C)
    def root(v):
        def f(t):return sum(v[j]/(t-spacing*(j-N)) for j in range(d))
        def df(t):return -sum(v[j]/(t-spacing*(j-N))**2 for j in range(d))
        return mp.findroot(f,gamma,df=df,solver='newton',tol=mp.mpf('1e-170'),maxsteps=30)
    base_root=root(q0);rows=[]
    for bits in [24,53,64,80,96,128,192,256,384]:
        with mp.workprec(bits):rounded=mp.matrix([[(+original[i,j]) for j in range(d)] for i in range(d)])
        values,vectors=mp.eigsy(rounded);v=vectors[:,0]
        overlap=abs(sum(a*b for a,b in zip(v,q0)))
        parity=mp.sqrt(sum((v[i]-v[d-1-i])**2 for i in range(d)))
        residual=mp.norm(rounded*v-values[0]*v)
        assert residual<mp.mpf('1e-180')
        row={'matrix_bits':bits,'solver_dps':220,'lambda_min':mp.nstr(values[0],30),
             'lambda_relative_error':mp.nstr(abs((values[0]-lambda0)/lambda0),15),
             'ground_overlap':mp.nstr(overlap,25),'parity_defect':mp.nstr(parity,15),
             'solve_residual':mp.nstr(residual,15)}
        if parity>mp.mpf('1e-100'):
            row['root_assessment']='not_applicable_odd_ground'
        else:
            try:
                t=root(v)
                row.update({'root_assessment':'converged','first_root':mp.nstr(t,70),
                            'matching_digits':mp.nstr(-mp.log10(abs(t-gamma)/abs(gamma)),20),
                            'difference_from_fixed_matrix_root':mp.nstr(abs(t-base_root),15)})
            except (ValueError,ZeroDivisionError):row['root_assessment']='unresolved'
        rows.append(row);print('precision',bits,row['lambda_min'],row.get('matching_digits',row['root_assessment']),flush=True)
    assert all(r['root_assessment']=='converged' and mp.mpf(r['matching_digits'])>21.58 for r in rows if r['matrix_bits']>=192)
    output={'status':'PASS','scope':__doc__,'C':C,'N':N,'matrix_sha256':hashlib.sha256(raw).hexdigest(),
            'reference_eigenvalue':mp.nstr(lambda0,45),'reference_first_root':mp.nstr(base_root,70),
            'rows':rows,'seconds':time.perf_counter()-started}
    args.output.write_text(json.dumps(output,indent=2)+'\n')


if __name__=='__main__':main()
