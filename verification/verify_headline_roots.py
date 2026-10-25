"""Direct interval derivative and endpoint replay, independent of Taylor bounds."""
from pathlib import Path
import argparse,hashlib,json,time
from flint import arb,acb,ctx

if not __debug__:
    raise RuntimeError('Verification requires assertions; do not use python -O')
ap=argparse.ArgumentParser()
ap.add_argument('--inputs',type=Path,required=True,help='Directory of decoded state JSON files named by manifest digest')
ap.add_argument('--output',type=Path,required=True)
args=ap.parse_args()
certificate=Path(__file__).with_name('headline-root-certificates.json')
report=json.loads(certificate.read_bytes());ctx.dps=3500;start=time.perf_counter()
reference=acb.zeta_zero(1).imag
rows=[]
for row in report['rows']:
    path=args.inputs/(row['state_manifest']+'.json');raw=path.read_bytes()
    assert hashlib.sha256(raw).hexdigest()==row['state_payload_sha256'];data=json.loads(raw)
    N=row['N'];C=row['C'];spacing=2*arb.pi()/arb(C).log();v=list(map(arb,data['eigenvector']))
    poles=[spacing*j for j in range(-N,N+1)]
    interval=arb(row['center'])+arb(row['result']['offset_enclosure'])
    assert all(not (interval-p).contains(0) for p in poles)
    derivative=-sum((w/(interval-p)**2 for p,w in zip(poles,v)),arb(0))
    assert not derivative.contains(0)
    def value(x):return sum((w/(x-p) for p,w in zip(poles,v)),arb(0))
    lo,hi=value(interval.lower()),value(interval.upper())
    assert (lo<0 and hi>0) or (hi<0 and lo>0)
    error=interval-reference;assert not error.contains(0)
    depth=-(abs(error)/abs(reference)).log()/arb(10).log()
    assert arb(row['relative_matching_digits_enclosure']).contains(depth)
    rows.append({'C':C,'N':N,'status':'PASS','unique_real_root':True,'signed_error_nonzero':True,'depth':depth.str(30)})
    print('PASS',C,N,depth.str(25),flush=True)
args.output.write_text(json.dumps({'status':'PASS','method':'Direct interval derivatives and opposite endpoint signs at 3500 dps, with fresh independent zeta reference',
    'certificate_sha256':hashlib.sha256(certificate.read_bytes()).hexdigest(),'rows':rows,'seconds':time.perf_counter()-start},indent=2)+'\n')
