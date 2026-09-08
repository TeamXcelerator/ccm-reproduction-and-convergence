#!/usr/bin/env python3
"""Assess Paper 1's documented finite numerical checks from HP JSON evidence.

PASS applies to the stated check, not a theorem or an interval certificate.
Capture completeness is reported independently and never changes a claim to PASS.
"""
import argparse
import json
from decimal import Decimal, localcontext
from pathlib import Path

D = Decimal
HEADLINE = {(13, 120): {1: D('55.764'), 20: D('25.339')},
            (100, 500): {1: D('460.09'), 20: D('422.44')},
            (1000, 800): {1: D('1019.0'), 20: D('1020.3')}}
LAMBDA = {13: '55.764', 20: '92.817', 30: '131.80', 50: '170.19', 100: '211.29'}
CRITICAL = {(50, 200): ('217.34', '207.24'), (50, 250): ('235.45', '225.42'),
            (100, 300): ('364.37', '354.03'), (100, 400): ('419.28', '409.04')}
EPSILON = {(13, 120): '3.48399e-59', (100, 500): '9.56481e-464', (1000, 800): '3.92215e-1264',
           (500, 630): '1.09117e-929', (600, 690): '2.22999e-1029', (700, 740): '9.45898e-1116',
           (800, 790): '1.35946e-1199', (1000, 890): '1.52095e-1362', (1200, 970): '6.79867e-1499'}
CONVERGENCE = dict(zip((10,20,30,40,50,60,80,100,120), (
    ('21.585','16.335','11.856','5.1626','3.4915'), ('35.795','32.261','30.011','26.504','24.870'),
    ('44.506','41.267','39.262','36.208','34.844'), ('50.294','47.152','45.225','42.304','41.015'),
    ('53.827','50.721','48.822','45.948','44.685'), ('55.299','52.203','50.312','47.451','46.195'),
    ('55.654','52.560','50.670','47.811','46.556'), ('55.735','52.641','50.751','47.892','46.637'),
    ('55.764','52.670','50.779','47.921','46.666'))))


def number(value):
    result = D(str(value))
    if not result.is_finite():
        raise ValueError('non-finite measurement')
    return result


def root_digits(point, index):
    row = next((r for r in point['roots'] if r['reference_index'] == index), None)
    if row is None or row['status'] != 'converged':
        raise ValueError(f'zero {index} is missing or not converged')
    value = row.get('matching_digits')
    return number(value if value is not None else row['matching_digits_lower_bound'])


def assess_point(claim, point):
    """Return explicit check rows; modified configurations without a baseline fail closed."""
    c, n = point['lambda_squared'], point['n_modes']
    checks = []
    if claim.startswith('claim4'):
        deviation = number(point['evenness_deviation'])
        natural, even = number(point['natural_eigenvalue']), number(point['even_eigenvalue'])
        checks.append(('evenness deviation < 1e-10', deviation >= 0 and deviation < D('1e-10')))
        checks.append(('both eigenvalues positive and relative difference < 1e-8', natural > 0 and even > 0 and abs(natural / even - 1) < D('1e-8')))
    elif claim.startswith(('claim1', 'claim8')):
        baseline = HEADLINE[(c,n)]
        # A guarded arithmetic floor is not reproducible to its last digit.
        allowance = D('1.0') if c == 1000 else D('0.05')
        for index, expected in baseline.items():
            observed = root_digits(point,index)
            checks.append((f'zero {index}: {observed:.5f} matching digits >= {expected-allowance}', observed >= expected-allowance))
    elif claim.startswith('claim2'):
        if n != 120:
            raise ValueError('published lambda sweep baseline requires N=120')
        observed, expected = root_digits(point,1), D(LAMBDA[c])
        checks.append((f'first-zero accuracy agrees with {expected} within 0.05 digit', abs(observed-expected) <= D('0.05')))
    elif claim.startswith('claim3'):
        for index, expected in zip((1,5), CRITICAL[(c,n)]):
            checks.append((f'zero {index} matching digits >= {D(expected)-D("0.05")}', root_digits(point,index) >= D(expected)-D('0.05')))
    elif claim.startswith('claim6'):
        observed, expected = number(point['epsilon_N']), D(EPSILON[(c,n)])
        checks.append(('positive epsilon_N agrees with the tabulated value within 5e-5 relative', observed > 0 and abs(observed/expected-1) <= D('5e-5')))
    elif claim.startswith('claim7'):
        if c != 13:
            raise ValueError('published N sweep baseline requires lambda_squared=13')
        for index, expected in enumerate(CONVERGENCE[n],1):
            checks.append((f'zero {index} agrees with {expected} within 0.05 digit', abs(root_digits(point,index)-D(expected)) <= D('0.05')))
    else:
        raise ValueError(f'no numerical acceptance policy for {claim}')
    return checks


def assess_series(claim, points):
    checks = []
    group = claim.split('_',1)[0]
    expected = {
        'claim1a': [(13,120)], 'claim1b': [(100,500)], 'claim1c': [(1000,800)],
        'claim2a': [(c,120) for c in LAMBDA], 'claim2b': [(c,120) for c in LAMBDA],
        'claim3': list(CRITICAL), 'claim4a': [(13,120)], 'claim4b': [(100,500)],
        'claim4c': [(1000,800),(1000,890)], 'claim4d': [(1200,970)],
        'claim6': [(13,120),(100,500),(1000,800)],
        'claim6b': [(500,630),(600,690),(700,740),(800,790),(1000,890),(1200,970)],
        'claim7': [(13,n) for n in CONVERGENCE for _ in range(2)],
        'claim8': [(13,120),(13,120),(100,500),(100,500)],
    }.get(group)
    actual=[(p['lambda_squared'],p['n_modes']) for p in points]
    checks.append(('all configurations for this individual claim are present exactly once per planned route',expected is not None and sorted(actual)==sorted(expected)))
    if group == 'claim3' and sorted(actual)==sorted(expected):
        indexed={(p['lambda_squared'],p['n_modes']):p for p in points}
        for c,low,high in ((50,200,250),(100,300,400)):
            checks.append((f'c={c}: accuracy increases with N',root_digits(indexed[c,high],1)>root_digits(indexed[c,low],1)))
    if group in ('claim6','claim6b') and sorted(actual)==sorted(expected):
        ordered=sorted(points,key=lambda p:p['lambda_squared'])
        checks.append(('epsilon_N decreases across the published series',all(number(a['epsilon_N'])>number(b['epsilon_N'])>0 for a,b in zip(ordered,ordered[1:]))))
    if claim.startswith('claim8'):
        for c,n in ((13,120),(100,500)):
            pair = [p for p in points if (p['lambda_squared'],p['n_modes']) == (c,n)]
            if len(pair) != 2 or {p['parity_policy'] for p in pair} != {'even-sector','natural'}:
                checks.append((f'c={c}: both parity routes present',False)); continue
            a,b = pair
            same = a['precision_digits']==b['precision_digits'] and a['root_acquisition']==b['root_acquisition']
            ea,eb = number(a['epsilon_N']),number(b['epsilon_N'])
            checks.append((f'c={c}: matched-policy eigenvalues agree within 1e-20 relative',same and ea>0 and eb>0 and abs(ea/eb-1)<D('1e-20')))
            checks.append((f'c={c}: root accuracy agrees across parity routes within 0.05 digit',same and all(abs(root_digits(a,i)-root_digits(b,i))<=D('0.05') for i in (1,20))))
    if claim.startswith('claim7'):
        for n in CONVERGENCE:
            pair=[p for p in points if p['n_modes']==n]
            valid=len(pair)==2 and {p['precision_digits'] for p in pair}=={200,1000}
            checks.append((f'N={n}: HP-200/1000 accuracy agrees within 0.05 digit', valid and all(abs(root_digits(pair[0],i)-root_digits(pair[1],i))<=D('0.05') for i in range(1,6))))
    return checks


def read_log(log):
    text = log.read_text(encoding='utf-8', errors='replace')
    measurements, journals = [], []
    for line in text.splitlines():
        if line.startswith('Claim measurements: '):
            measurements.append(json.loads(Path(line.split(': ',1)[1]).read_text(encoding='utf-8')))
        if line.strip().startswith('private claim journal: '):
            journals.append(Path(line.strip().split(': ',1)[1]))
    return measurements, journals


CLAIM8_NATURAL_REQUESTS = {
    'deviation_decomposition', 'distance_profile', 'distance_resolution', 'evenness',
    'prefix_ladder', 'retained_reduction', 'root_conditioning', 'sector_analysis',
    'target_distance', 'target_residual_analysis',
}


def claim8_natural_checkpoint(request):
    """Frozen applicability, derived from the request rather than a failed outcome."""
    c, n = request['lambda_squared'], request['n_modes']
    if ((c, n) not in ((13, 120), (100, 500)) or request['precision_bits'] != 3386
            or request['parity_policy'] != 'natural'
            or request['capture']['level'] != 'ultra'
            or request['capture']['prefix_checkpoint_dimensions'] != [n + 1]
            or request['capture'].get('prefix_working_precision_bits') is not None):
        raise ValueError('claim8-natural requires the frozen Ultra natural configuration')
    return f'prefix_checkpoint_{n + 1}'


def validate_applicability(state, request, record, outcomes):
    applicability = state.get('applicability', {})
    excluded = applicability.get('excluded_diagnostics', {})
    requested = applicability.get('requested_diagnostics', [])
    if excluded:
        if (request.get('applicability') != applicability
                or record.get('resolved_plan', {}).get('applicability') != applicability
                or set(excluded) & set(outcomes)
                or not all(isinstance(reason, str) and reason.strip() for reason in excluded.values())
                or not requested or set(requested) != set(record['receipt']['outcomes'])
                or set(requested) - set(outcomes)):
            raise ValueError('capture applicability does not match the retained request and receipt')
    if applicability.get('policy') == 'claim8-natural':
        checkpoint = claim8_natural_checkpoint(request)
        if (set(excluded) != {checkpoint, 'prime_power_response', 'u_flow_response'}
                or set(requested) != CLAIM8_NATURAL_REQUESTS):
            raise ValueError('claim8-natural must retain all ten applicable diagnostics')
    return applicability


def numerical_review(record, applicability=None):
    """Known numerical rejection fields remain visible even after successful capture."""
    review=[]
    for name, measurement in record['measurements'].items():
        values=measurement['value']
        values=values if isinstance(values,list) else [values]
        for value in values:
            if name=='distance_resolution':
                for row in value['refinements']:
                    if not row['tolerance_met']:
                        review.append(f'{name}: {row["quadrature_rule"]}/{row["grid_variable"]} at Q={row["final_resolution"]} did not meet tolerance')
            elif name=='retained_reduction' and not value['checks_passed']:
                review.append('retained_reduction: stored-matrix checks failed')
            elif name=='prefix_ladder':
                if value['ladder']['stopped'] is not None:
                    review.append(f'prefix_ladder: stopped before completion: {value["ladder"]["stopped"]}')
                if applicability and applicability.get('policy') == 'claim8-natural':
                    expected = {name for name in applicability['excluded_diagnostics'] if name.startswith('prefix_checkpoint_')}
                    actual = [f'prefix_checkpoint_{checkpoint["dimension"]}' for checkpoint in value['checkpoints']]
                    if set(actual) != expected or len(actual) != len(expected):
                        review.append('prefix_ladder: missing or unexpected innovation checkpoint')
                for checkpoint in value['checkpoints']:
                    # An innovation-only export is accepted only when the caller
                    # has validated the predeclared Claim 8 eigenstate exclusion.
                    if (applicability and applicability.get('policy') == 'claim8-natural'
                            and f'prefix_checkpoint_{checkpoint["dimension"]}' in applicability['excluded_diagnostics']
                            and checkpoint['status'] == 'innovation_export_passed_eigenpair_not_supplied'
                            and checkpoint.get('eigenpair_source') is None):
                        continue
                    if checkpoint['status']!='export_checks_passed':
                        review.append(f'prefix checkpoint {checkpoint["dimension"]}: {checkpoint["status"]}')
    return review


def summarize(claim, logs):
    checks, points, captures = [], [], []
    for log in logs:
        try:
            measurement, journals = read_log(log)
            status = dict(line.split('=',1) for line in log.with_suffix('.status').read_text().splitlines())
            checks.append((f'{log.name}: process completed with retained measurements',status.get('binary_exit')=='0' and status.get('log_exit')=='0' and bool(measurement)))
            for point in measurement:
                points.append(point)
                label=f'c={point["lambda_squared"]}, N={point["n_modes"]}, HP-{point["precision_digits"]}'
                checks.extend((f'{label}: {description}',passed) for description,passed in assess_point(claim,point))
            for directory in journals:
                state=json.loads((directory/'status.json').read_text())
                outcomes={}
                review=[]
                records=[]
                for filename in ('capture.json','capture-explicit.json'):
                    path=directory/filename
                    if path.exists():
                        record=json.loads(path.read_text())
                        outcomes.update(record['receipt']['outcomes'])
                        records.append(record)
                request = json.loads((directory/'request.json').read_text())
                applicability = validate_applicability(state, request, records[0], outcomes)
                for record in records:
                    review.extend(numerical_review(record, applicability))
                excluded = applicability.get('excluded_diagnostics', {})
                complete = state.get('capture_complete',False) and bool(outcomes) and all(o['status']=='completed' for o in outcomes.values())
                captures.append({'directory':str(directory),'complete':complete,'outcomes':outcomes,'numerical_review':review,
                                 'policy':applicability.get('policy','full'),'excluded_diagnostics':excluded})
        except (OSError, ValueError, KeyError, IndexError, ArithmeticError) as error:
            checks.append((f'{log.name}: missing/unassessed evidence: {error}',False))
    try:
        checks.extend(assess_series(claim,points))
    except (ValueError, KeyError, ArithmeticError) as error:
        checks.append((f'series could not be assessed: {error}',False))
    passed=bool(checks) and all(ok for _,ok in checks)
    return {'claim':claim,'passed':passed,'checks':[{'description':name,'passed':ok} for name,ok in checks],'captures':captures}


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--claim',required=True)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('logs',nargs='+',type=Path)
    args=parser.parse_args()
    with localcontext() as context:
        context.prec=100  # ratios/logarithmic digit counts; original HP scalars stay exact strings
        result=summarize(args.claim,args.logs)
    print(f'\n=== {args.claim}: numerical claim summary ===')
    for row in result['checks']:
        print(f'[{"PASS" if row["passed"] else "FAIL"}] {row["description"]}')
    for capture in result['captures']:
        incomplete=[f'{name}: {outcome["status"]}: {outcome.get("reason", "reason unavailable")}' for name,outcome in capture['outcomes'].items() if outcome['status']!='completed']
        print(f'[DATA {"COMPLETE" if capture["complete"] else "INCOMPLETE"}] {capture["directory"]}')
        if capture['excluded_diagnostics']:
            completed=sum(o['status']=='completed' for o in capture['outcomes'].values())
            print(f'  Policy {capture["policy"]}: {completed}/{len(capture["outcomes"])} requested diagnostics completed; {len(capture["excluded_diagnostics"])} excluded before execution')
            for name, reason in capture['excluded_diagnostics'].items():
                print(f'[EXCLUDED] {name}: {reason}')
        for item in incomplete:
            print(f'  {item}')
        for item in capture['numerical_review']:
            print(f'[DATA REVIEW] {item}')
    print(f'Overall numerical checks: {"PASS" if result["passed"] else "FAIL"}')
    print('These are finite numerical reproduction checks; certificates and capture completeness are separate.')
    args.output.write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')
    print(f'Summary: {args.output}')
    return 0 if result['passed'] else 1


if __name__=='__main__':
    raise SystemExit(main())
