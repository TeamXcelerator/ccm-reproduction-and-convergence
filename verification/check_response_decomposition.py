"""Check the Archimedean action derivative, response-channel sums, and prime-13 jump."""
import argparse,hashlib,json
from pathlib import Path
from flint import acb, arb, arb_mat, ctx


def check_archimedean_action(state, response):
    """Differentiate the defining integral at fixed normalized Fourier coordinates."""
    n = state["n_modes"]
    assert response["n_modes"] == n and state["lambda_squared"] == response["lambda_squared"]
    assert len(state["eigenvector"]) == 2*n + 1
    vec = [arb(x) for x in state["eigenvector"]]
    norm = sum(x*x for x in vec).sqrt()
    vec = [x/norm for x in vec]
    channel, = [row for row in response["channels"] if row["channel"] == "tau_archimedean"]
    expected = list(map(arb, channel["tau_velocity_action_on_state"]))
    assert len(expected) == len(vec)
    u = arb(state["lambda_squared"]).log()
    step = arb("1e-24")
    lo, hi = u-step, u+step
    rows = []
    for order in (24, 32):
        # FLINT generates the rule independently at each order. Each panel covers at most
        # one period of the highest frequency. Difference the original integral,
        # independently of the derivative formulas and tables in the producer.
        panels = max(n, 4)
        sine = [arb(0) for _ in range(n+1)]
        diagonal = [arb(0) for _ in range(n+1)]
        tail = ((hi/2).tanh().log() - (lo/2).tanh().log())/(2*step)
        rule = [arb.legendre_p_root(order, k, weight=True) for k in range(order)]
        for panel in range(panels):
            for node, weight in rule:
                s = (2*panel+1+node)/(2*panels)
                a, b = lo/(2*(lo*s).sinh()), hi/(2*(hi*s).sinh())
                scale = weight/(4*panels*step)
                kernel = scale*(b*(hi*s/2).exp()-a*(lo*s/2).exp())
                constant = scale*(b-a)
                rotation = acb(0, 2*arb.pi()*s).exp()
                phase = acb(1)
                for k in range(n+1):
                    sine[k] += kernel*phase.imag
                    diagonal[k] += kernel*(1-s)*phase.real-constant
                    phase *= rotation
        indices = range(-n, n+1)
        def signed_sine(k):
            return -sine[-k] if k < 0 else sine[k]
        def entry(i, j):
            if i == j:
                return -2*diagonal[abs(i)]-tail
            return (signed_sine(i)-signed_sine(j))/(arb.pi()*(i-j))
        action = [sum(entry(i,j)*v for j,v in zip(indices, vec)) for i in indices]
        difference = max(abs(a-b).upper() for a,b in zip(action, expected))
        eigen_error = abs(sum(a*v for a,v in zip(action,vec))
                          - arb(channel["eigenvalue_velocity_response"])).upper()
        tolerance = arb("1e-26")
        if not (difference < tolerance and eigen_error < tolerance):
            raise ValueError(f"Archimedean derivative mismatch at order {order}: action {difference}, eigenvalue {eigen_error}")
        rows.append({"order":order, "panels":panels, "maximum_action_error":difference.str(14),
                     "eigenvalue_error":eigen_error.str(14)})
    return {"step":"1e-24", "absolute_tolerance":"1e-26", "rows":rows}


def check_state_and_tangents(state, response, tau):
    """Solve the differentiated, normalized eigenproblem from the retained finite matrix."""
    dimension = 2 * state["n_modes"] + 1
    if (tau["n_modes"] != state["n_modes"] or tau["lambda_squared"] != state["lambda_squared"]
            or tau["precision_bits"] != state["precision_bits"]
            or len(tau["entries"]) != dimension**2 or len(state["eigenvector"]) != dimension):
        raise ValueError("Tau matrix and eigenstate configuration differ")
    matrix = arb_mat(dimension, dimension, list(map(arb, tau["entries"])))
    values = list(map(arb, state["eigenvector"]))
    norm = sum(v*v for v in values).sqrt()
    x = [v/norm for v in values]
    eigenvalue = arb(state["eigenvalue"])
    product = matrix * arb_mat(dimension, 1, x)
    residual = max(abs(product[i, 0] - eigenvalue*x[i]).upper() for i in range(dimension))
    tolerance = arb("1e-700")
    if not residual < tolerance:
        raise ValueError(f"retained eigenpair residual exceeds tolerance: {residual}")
    bordered = arb_mat(dimension+1, dimension+1)
    for i in range(dimension):
        for j in range(dimension):
            bordered[i, j] = matrix[i, j] - (eigenvalue if i == j else 0)
        bordered[i, dimension] = bordered[dimension, i] = x[i]
    channels = response["channels"]
    names = [c["channel"] for c in channels]
    if len(names) != 4 or set(names) != {"tau_pole", "tau_archimedean", "tau_active_prime_aggregate", "tau_total"}:
        raise ValueError("unexpected Claim 1a response channel inventory")
    rhs = arb_mat(dimension+1, len(channels))
    for j, channel in enumerate(channels):
        action = list(map(arb, channel["tau_velocity_action_on_state"]))
        if len(action) != dimension:
            raise ValueError("channel action dimension differs from the state")
        velocity = sum(v*a for v, a in zip(x, action))
        if not abs(velocity-arb(channel["eigenvalue_velocity_response"])).upper() < tolerance:
            raise ValueError(f"eigenvalue derivative mismatch for {names[j]}")
        for i, a in enumerate(action):
            rhs[i, j] = velocity*x[i]-a
    solutions = bordered.solve(rhs)
    tangents, errors = {}, {}
    for j, channel in enumerate(channels):
        tangent = [solutions[i, j] for i in range(dimension)]
        reported = list(map(arb, channel["l2_eigenvector_velocity_response"]))
        if len(reported) != dimension:
            raise ValueError("channel tangent dimension differs from the state")
        error = max(abs(a-b).upper() for a, b in zip(tangent, reported))
        if not error < tolerance:
            raise ValueError(f"independent eigenvector tangent mismatch for {names[j]}: {error}")
        tangents[names[j]] = tangent
        errors[names[j]] = error.str(12)
    return tangents, {"maximum_eigenpair_residual": residual.str(12), "maximum_tangent_errors": errors,
                      "absolute_tolerance": "1e-700", "source": "retained Tau, eigenstate and channel actions"}


def check_root_velocities(state, response, tangents):
    """Implicitly differentiate sum_j v_j/(r-p_j)=0 using retained state tangents."""
    n = state["n_modes"]
    length = arb(state["lambda_squared"]).log()
    vec = list(map(arb, state["eigenvector"]))
    norm = sum(x*x for x in vec).sqrt()
    weights = [x/norm for x in vec]
    poles = [2*arb.pi()*j/length for j in range(-n, n+1)]
    if len(weights) != len(poles):
        raise ValueError("state length does not match the Fourier modes")
    tolerance = arb("1e-700")
    errors = []
    if len(response["roots"]) != 25:
        raise ValueError("expected all 25 Claim 1a roots")
    roots = list(map(lambda row: arb(row["value"]), response["roots"]))
    if not all(a < b for a, b in zip(roots, roots[1:])):
        raise ValueError("expected distinct increasing roots")
    for i, row in enumerate(response["roots"]):
        if row["status"] != "converged":
            raise ValueError("expected a converged reference root")
        root = arb(row["value"])
        inverse = [1/(root-pole) for pole in poles]
        denominator = sum(w*d*d for w,d in zip(weights,inverse))
        if denominator.contains(0):
            raise ValueError("secular slope unresolved at the reference root")
        root_step = abs(sum(w*d for w, d in zip(weights, inverse))/denominator).upper()
        if not root_step < tolerance:
            raise ValueError(f"secular equation mismatch at root {i+1}: {root_step}")
        motion = sum(w*d*d*(-pole/length) for w,d,pole in zip(weights,inverse,poles))/denominator
        errors.append(abs(motion-arb(response["secular_pole_motion_root_velocity_responses"][i])).upper())
        for channel in response["channels"]:
            tangent = tangents[channel["channel"]]
            if len(tangent) != len(poles):
                raise ValueError("state tangent length differs from the number of poles")
            fixed = sum(v*d for v,d in zip(tangent,inverse))/denominator
            errors.append(abs(fixed-arb(channel["fixed_pole_root_velocity_responses"][i])).upper())
            if channel["channel"] == "tau_total":
                total = arb(response["total_moving_pole_root_velocity_responses"][i])
                errors.append(abs(fixed+motion-total).upper())
    if not errors or not (max(errors) < tolerance):
        raise ValueError(f"implicit secular root derivative mismatch: {max(errors, default=None)}")
    return {"roots_checked":len(response["roots"]), "maximum_absolute_error":max(errors).str(12),
            "absolute_tolerance":"1e-700", "source":"retained eigenvector and independently solved channel tangents"}


def main():
    if not __debug__:raise RuntimeError('Do not use -O')
    ap=argparse.ArgumentParser(description=__doc__)
    for k in ['prime-response','u-flow','state','output']:ap.add_argument('--'+k,type=Path,required=True)
    ap.add_argument('--tau', type=Path, help='Retained Tau payload (default: tau.json beside the eigenstate)')
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
    state_raw = args.state.read_bytes()
    assert hashlib.sha256(state_raw).hexdigest() == u["eigenpair_content_digest"]
    tau_raw = (args.tau or args.state.with_name("tau.json")).read_bytes()
    if hashlib.sha256(tau_raw).hexdigest() != u["tau_content_digest"]:
        raise ValueError("Tau payload digest does not match the response sources")
    result["tau_sha256"] = u["tau_content_digest"]
    tangents, result["bordered_eigenproblem"] = check_state_and_tangents(json.loads(state_raw), u, json.loads(tau_raw))
    result["secular_derivative_control"] = check_root_velocities(json.loads(state_raw), u, tangents)
    ctx.dps = 110
    result["archimedean_derivative_control"] = check_archimedean_action(json.loads(state_raw), u)
    result["state_sha256"] = hashlib.sha256(state_raw).hexdigest()
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
