"""Numerical centered-perturbation test of retained prime-power responses.

This is an independent high-precision experiment, not an interval certificate.
The perturbed family is A+t D_power with observation poles fixed. It does
not remove a prime and is not the full cutoff derivative.
"""
import argparse
import hashlib
import json
import time
from pathlib import Path
from flint import arb, arb_mat, ctx, fmpq


def point(x):
    return x.mid()


def norm(v):
    return sum((x*x for x in v), arb(0)).sqrt()


def unit(v):
    size = norm(v)
    return [point(x/size) for x in v]


def main():
    if not __debug__:
        raise RuntimeError('Do not use -O')
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--matrix',type=Path,required=True)
    ap.add_argument('--state',type=Path,required=True)
    ap.add_argument('--response',type=Path,required=True)
    ap.add_argument('--output',type=Path,required=True)
    args=ap.parse_args();ctx.dps=500;started=time.perf_counter()
    raw=args.matrix.read_bytes();data=json.loads(raw)
    state_raw=args.state.read_bytes();state=json.loads(state_raw)
    response_raw=args.response.read_bytes();response=json.loads(response_raw)
    N=data['N'];C=data['C'];d=2*N+1
    assert C==13 and N==120 and response['n_modes']==state['n_modes']==N
    assert response['eigenpair_content_digest']==hashlib.sha256(state_raw).hexdigest()
    full=[point(arb((fmpq(x['lower'])+fmpq(x['upper']))/2)) for x in data['tau']]
    rt2=arb(2).sqrt();L=arb(C).log();pi=arb.pi()

    def reduce(entry):
        return [[point(entry(N,N) if i==j==0 else
                       rt2*entry(N,N+j) if i==0 else
                       rt2*entry(N+i,N) if j==0 else
                       entry(N+i,N+j)+entry(N+i,N-j)) for j in range(N+1)] for i in range(N+1)]
    B=reduce(lambda i,j:full[i*d+j])
    v=[arb(x) for x in state['eigenvector']]
    q=unit([v[N]]+[rt2*v[N+j] for j in range(1,N+1)])
    rho=arb(state['eigenvalue']).mid()
    reference_root=arb(response['roots'][0]['value']).mid()
    poles=[2*pi*j/L for j in range(N+1)]

    def solve(matrix):
        # Newton on the isolated eigenpair, using a different augmented solve
        # than the retained response producer. Approximate FLINT LU is explicit.
        z=q[:];value=rho
        Bm=arb_mat(matrix)
        for _ in range(6):
            residual=Bm*arb_mat([[x] for x in z])-arb_mat([[value*x] for x in z])
            augmented=[[(matrix[i][j]-(value if i==j else 0)) for j in range(N+1)]+[-z[i]] for i in range(N+1)]
            augmented.append(z+[arb(0)])
            rhs=arb_mat([[-residual[i,0]] for i in range(N+1)]+[[arb(0)]])
            correction=arb_mat(augmented).solve(rhs,algorithm='approx')
            z=unit([z[i]+correction[i,0] for i in range(N+1)])
            value=point(value+correction[N+1,0])
        residual=Bm*arb_mat([[x] for x in z])-arb_mat([[value*x] for x in z])
        residual_norm=norm([residual[i,0].mid() for i in range(N+1)])
        overlap=abs(sum((a*b for a,b in zip(z,q)),arb(0)))
        assert overlap>arb('0.99999999999999999999') and residual_norm<arb('1e-220')
        t=reference_root
        def secular(t):
            value=z[0]/t;derivative=-z[0]/(t*t)
            for j in range(1,N+1):
                weight=z[j]/rt2
                value+=weight*(1/(t-poles[j])+1/(t+poles[j]))
                derivative-=weight*(1/(t-poles[j])**2+1/(t+poles[j])**2)
            return value,derivative
        for _ in range(8):
            value_h,derivative=secular(t)
            correction=value_h/derivative;t=point(t-correction)
            if abs(correction)<arb('1e-270'):break
        else:raise AssertionError('Root did not converge')
        return t,value,residual_norm

    results=[]
    for event in response['events']:
        power=event['power'];prime=event['prime'];x=arb(power).log();w=arb(prime).log()/arb(power).sqrt()
        cos=[(2*pi*j*x/L).cos() for j in range(-N,N+1)]
        sin=[(2*pi*j*x/L).sin() for j in range(-N,N+1)]
        def direction(i,j):
            n=i-N;m=j-N
            if n==m:
                return -2*w*x/L**2*(cos[i]+2*pi*n*(1-x/L)*sin[i])
            return -2*w*x/L**2*(n*cos[i]-m*cos[j])/(n-m)
        D=reduce(direction)
        prediction=arb(event['root_velocity_responses'][0])
        eigen_prediction=arb(event['eigenvalue_velocity_response'])
        local=[]
        for exponent in [90,100,110]:
            h=arb(10)**(-exponent)
            plus=solve([[point(B[i][j]+h*D[i][j]) for j in range(N+1)] for i in range(N+1)])
            minus=solve([[point(B[i][j]-h*D[i][j]) for j in range(N+1)] for i in range(N+1)])
            estimate=(plus[0]-minus[0])/(2*h)
            eigen_estimate=(plus[1]-minus[1])/(2*h)
            error=abs((estimate-prediction)/prediction)
            eigen_error=abs((eigen_estimate-eigen_prediction)/eigen_prediction)
            assert error<arb('1e-12') and eigen_error<arb('1e-12')
            local.append({'step_exponent':exponent,'centered_root_derivative':estimate.str(30),
                          'root_relative_difference':error.str(20),'eigenvalue_relative_difference':eigen_error.str(20),
                          'max_eigenpair_residual':max(plus[2],minus[2]).str(20),'status':'PASS'})
            print('PASS prime',power,'step',exponent,'relative difference',error.str(10),flush=True)
        results.append({'power':power,'prime':prime,'event_edge':event['observation_is_event_edge'],
                        'retained_root_derivative':prediction.str(30),'steps':local})
        args.output.write_text(json.dumps({'status':'IN_PROGRESS','rows':results},indent=2)+'\n')
    out={'status':'PASS','scope':__doc__,'C':C,'N':N,'dps':500,'step_exponents':[90,100,110],
         'matrix_sha256':hashlib.sha256(raw).hexdigest(),'state_sha256':hashlib.sha256(state_raw).hexdigest(),
         'response_sha256':hashlib.sha256(response_raw).hexdigest(),'rows':results,'seconds':time.perf_counter()-started}
    args.output.write_text(json.dumps(out,indent=2)+'\n')


if __name__=='__main__':main()
