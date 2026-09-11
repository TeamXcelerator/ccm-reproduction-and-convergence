"""Independent root-transfer replay using exact vector norms and full-space gaps.

Uses orbit hulls and an infinity-norm residual budget, rather than the primary
calculation's intersections, even-sector gap, and componentwise L2 budget.
"""
import argparse,hashlib,json,time
from decimal import Decimal
from pathlib import Path
from flint import acb,arb,arb_mat,ctx,fmpq


def main():
    if not __debug__:raise RuntimeError('Do not use -O')
    ap=argparse.ArgumentParser(description=__doc__)
    for name in ['matrix','state','sector-certificate','root-certificate','output']:
        ap.add_argument('--'+name,type=Path,required=True)
    args=ap.parse_args();ctx.dps=600;started=time.perf_counter()
    raw=args.matrix.read_bytes();a=json.loads(raw);sraw=args.state.read_bytes();s=json.loads(sraw)
    c=json.loads(args.sector_certificate.read_bytes());roots=json.loads(args.root_certificate.read_bytes())
    assert hashlib.sha256(raw).hexdigest()==c['source_sha256']==roots['matrix_sha256']
    assert hashlib.sha256(sraw).hexdigest()==roots['state_sha256']
    assert hashlib.sha256(args.sector_certificate.read_bytes()).hexdigest()==roots['sector_certificate_sha256']
    assert c['status']=='certified' and roots['status']=='PASS'
    N=a['N'];C=a['C'];dim=2*N+1
    v=[fmpq(*Decimal(x).as_integer_ratio()) for x in s['eigenvector']]
    assert len(v)==dim and len(a['tau'])==dim*dim
    exact_norm2=sum((x*x for x in v),fmpq(0));norm=arb(exact_norm2).sqrt()
    v=[arb(x)/norm for x in v]
    entries=[(fmpq(x['lower']),fmpq(x['upper'])) for x in a['tau']]
    def entry(i,j):
        lo=min(entries[i*dim+j][0],entries[j*dim+i][0]);hi=max(entries[i*dim+j][1],entries[j*dim+i][1])
        return arb((lo+hi)/2)+arb(0,arb((hi-lo)/2).upper())
    A=arb_mat([[entry(i,j) for j in range(dim)] for i in range(dim)])
    Aq=A*arb_mat([[x] for x in v])
    rho=sum((v[i]*Aq[i,0] for i in range(dim)),arb(0)).mid()
    full_gap=(min(arb(fmpq(r['lower'])) for r in c['enclosures'] if r['name']!='even_ground')-rho).lower()
    residual=(arb(dim).sqrt()*max((Aq[i,0]-rho*v[i]).abs_upper() for i in range(dim))).upper()
    assert full_gap>0 and residual/full_gap<1
    eta=(arb(2).sqrt()*residual/full_gap).upper()
    spacing=2*arb.pi()/arb(C).log();poles=[spacing*j for j in range(-N,N+1)]
    rows=[];previous=None
    for r in roots['rows']:
        t=arb(r['root_interval']);assert previous is None or previous<t.lower()
        previous=t.upper();assert all(not (t-p).contains(0) for p in poles)
        def enclosure(t,power):
            coeff=[(t-p)**(-power) for p in poles]
            central=sum((a*b for a,b in zip(v,coeff)),arb(0))
            # A different sufficient Cauchy bound: ||a||2 <= sqrt(d)*max |a_j|.
            budget=(eta*arb(dim).sqrt()*max(x.abs_upper() for x in coeff)).upper()
            return central+arb(0,budget)
        left=enclosure(t.lower(),1);right=enclosure(t.upper(),1)
        assert (left<0 and right>0) or (left>0 and right<0)
        assert not enclosure(t,2).contains(0)
        out={'bracket_ordinal':r['bracket_ordinal'],'unique_analytic_root':True}
        if 'reference_k' in r:
            ref=acb.zeta_zero(r['reference_k']).imag
            difference=t-ref;assert not difference.contains(0)
            depth=-(abs(difference)/ref).log()/arb(10).log()
            # Root intervals in the report are outward rounded to 90 digits.
            # Check the reported decimal precision rather than claiming nested
            # balls after reserializing a wider enclosure.
            assert abs(depth.mid()-arb(r['relative_matching_digits']).mid())<arb('1e-8')
            out.update({'reference_k':r['reference_k'],'matching_digits':depth.str(25)})
        rows.append(out)
    assert len(rows)==(10 if N==10 else 1)
    args.output.write_text(json.dumps({'status':'PASS','C':C,'N':N,'method':__doc__,
        'matrix_sha256':hashlib.sha256(raw).hexdigest(),'state_sha256':hashlib.sha256(sraw).hexdigest(),
        'sector_certificate_sha256':hashlib.sha256(args.sector_certificate.read_bytes()).hexdigest(),
        'root_certificate_sha256':hashlib.sha256(args.root_certificate.read_bytes()).hexdigest(),
        'full_space_vector_error_upper':eta.str(35),'rows':rows,'seconds':time.perf_counter()-started},indent=2)+'\n')
    print('PASS independent analytic roots',C,N,len(rows),flush=True)


if __name__=='__main__':main()
