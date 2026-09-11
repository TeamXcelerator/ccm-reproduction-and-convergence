"""Check retained response-channel sums and the one-sided prime-13 matrix jump."""
import argparse,hashlib,json
from pathlib import Path
from flint import arb,ctx


def main():
    if not __debug__:raise RuntimeError('Do not use -O')
    ap=argparse.ArgumentParser(description=__doc__)
    for k in ['prime-response','u-flow','output']:ap.add_argument('--'+k,type=Path,required=True)
    args=ap.parse_args();ctx.dps=1200
    p_raw=args.prime_response.read_bytes();u_raw=args.u_flow.read_bytes()
    p=json.loads(p_raw);u=json.loads(u_raw)
    for k in ['tau_content_digest','eigenpair_content_digest','root_range_content_digest','secular_source_content_digest','root_selection_digest']:
        assert p[k]==u[k]
    assert p['lambda_squared']==u['lambda_squared']=='13' and p['n_modes']==u['n_modes']==120
    channels={r['channel']:arb(r['fixed_pole_root_velocity_responses'][0]) for r in u['channels']}
    prime=sum((arb(r['root_velocity_responses'][0]) for r in p['events']),arb(0))
    prime_error=abs(prime-channels['tau_active_prime_aggregate'])
    fixed=sum((v for k,v in channels.items() if k!='tau_total'),arb(0))
    fixed_error=abs(fixed-channels['tau_total'])
    motion=arb(u['secular_pole_motion_root_velocity_responses'][0])
    combined=fixed+motion;reported=arb(u['total_moving_pole_root_velocity_responses'][0])
    total_error=abs(combined-reported)
    assert max(prime_error,fixed_error,total_error)<arb('1e-700')
    result={'status':'PASS','scope':'Numerical source-bound channel reconciliation; the sum does not independently validate every nonprime derivative.',
        'prime_response_sha256':hashlib.sha256(p_raw).hexdigest(),'u_flow_sha256':hashlib.sha256(u_raw).hexdigest(),
        'channels':{k:v.str(35) for k,v in channels.items()},'pole_motion':motion.str(35),
        'total_moving_pole_derivative':reported.str(35),'max_channel_sum_error':max(prime_error,fixed_error,total_error).str(25)}
    ctx.dps=120;N=120;x=arb(13).log();w=x/arb(13).sqrt();expected=-2*w/x
    rows=[]
    for exponent in [6,9,12]:
        h=arb(10)**(-exponent);right=x+h
        cs=[(2*arb.pi()*n*x/right).cos() for n in range(-N,N+1)]
        sn=[(2*arb.pi()*n*x/right).sin() for n in range(-N,N+1)]
        maximum=arb(0)
        for i,n in enumerate(range(-N,N+1)):
            for j,m in enumerate(range(n,N+1),start=i):
                kernel=2*(1-x/right)*cs[i] if n==m else (sn[j]-sn[i])/(arb.pi()*(n-m))
                slope=-w*kernel/h
                maximum=max(maximum,abs((slope-expected)/expected).upper())
        assert maximum<10*h
        rows.append({'right_step_exponent':exponent,'max_relative_entry_error':maximum.str(25),
                     'left_prime_contribution':'exactly zero below activation','status':'PASS'})
    result['prime13_edge_control']={'parameter':'u=log(C)','dimension':241,
        'right_derivative_every_entry':expected.str(35),'rows':rows,
        'scope':'One-sided activation of the individual prime-13 matrix component, with all other components excluded.'}
    args.output.write_text(json.dumps(result,indent=2)+'\n')
    print('PASS response sums; total',reported.str(25),flush=True)
    print('PASS edge entries',[(r['right_step_exponent'],r['max_relative_entry_error']) for r in rows],flush=True)


if __name__=='__main__':main()
