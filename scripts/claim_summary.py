#!/usr/bin/env python3
"""Assess the paper's documented finite numerical checks from HP JSON evidence.

A claim PASSes only when every configuration completed and every check below
holds. PASS applies to the stated check, not a theorem or an interval
certificate.
"""
import argparse
import json
import sys
from decimal import Decimal, localcontext
from pathlib import Path

D = Decimal
# Retained rational roots can exceed Python's default decimal conversion limit.
# Each scalar still has an explicit admission limit before conversion.
if hasattr(sys, 'set_int_max_str_digits') and 0 < sys.get_int_max_str_digits() < 40000:
    sys.set_int_max_str_digits(40000)
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


# tab:critical_brackets: first-zero digits adjacent to each crossing, and the
# stated bound on the HP-200/HP-1000 difference over the 40 computations.
BRACKETS = ((50, range(31, 41), 40), (55, range(51, 61), 56))
BRACKET_VALUES = {39: '49.745584', 40: '50.293684', 55: '54.708183', 56: '55.011854'}
BRACKET_PRECISION_DIFFERENCE = D('3.18e-160')
# tab:sector_separations: o0-e0 and e1-e0 as printed (six significant digits).
# The residual/gap column is a precision-floor quantity; the text bounds it.
SEPARATIONS = {(13, 120): ('3.05556e-55', '1.31185e-51'), (100, 500): ('1.79801e-458', '1.78543e-453'),
               (1000, 800): ('1.24493e-1258', '2.05363e-1253'), (1000, 890): ('5.42414e-1357', '1.04167e-1351'),
               (1200, 970): ('2.86128e-1493', '6.55282e-1488')}
RESIDUAL_TO_GAP_BOUND = D('2.01e-533')
EVENNESS_PRECISIONS = {(13, 120): 1000, (100, 500): 1000,
                      (1000, 800): 2000, (1000, 890): 2000, (1200, 970): 2000}
EVENNESS_SCALAR_LIMIT = 40000


def number(value):
    result = D(str(value))
    if not result.is_finite():
        raise ValueError('non-finite measurement')
    return result


def evenness_number(value):
    """Bound retained scalar size before allocating its decimal coefficient."""
    text = str(value)
    if len(text) > EVENNESS_SCALAR_LIMIT:
        raise ValueError('evenness measurement exceeds the scalar admission limit')
    result = number(text)
    if abs(result.adjusted()) > EVENNESS_SCALAR_LIMIT:
        raise ValueError('evenness measurement exponent exceeds the admission limit')
    return result


def evenness_relative_difference(natural, even):
    if even == 0:
        return None
    with localcontext() as context:
        context.prec = max(100, len(natural.as_tuple().digits), len(even.as_tuple().digits)) + 16
        # Subtract the exact retained coefficients before dividing. Dividing
        # first and subtracting one erases differences below the context floor.
        return abs(natural - even) / abs(even)


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
        expected_precision = EVENNESS_PRECISIONS[(c, n)]
        checks.append((f'published precision HP-{expected_precision}', point.get('precision_digits') == expected_precision))
        deviation = evenness_number(point['evenness_deviation'])
        natural, even = evenness_number(point['natural_eigenvalue']), evenness_number(point['even_eigenvalue'])
        relative = evenness_relative_difference(natural, even)
        relative_text = f'{relative:.5e}' if relative is not None else 'undefined'
        checks.append(('evenness deviation < 1e-10', deviation >= 0 and deviation < D('1e-10')))
        checks.append((f'both eigenvalues positive and relative difference {relative_text} < 1e-8', natural > 0 and even > 0 and relative < D('1e-8')))
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
    elif claim.startswith('claim7b'):
        if c != 13 or not any(n in window for _, window, _ in BRACKETS):
            raise ValueError('published Critical-N brackets require lambda_squared=13 and N in 31-40 or 51-60')
        checks.append(('zero 1 converged with retained matching digits', root_digits(point,1) > 0))
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
        'claim7b': [(13,n) for _, window, _ in BRACKETS for n in window for _ in range(2)],
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
    if group == 'claim7b' and sorted(actual)==sorted(expected):
        by_precision = {}
        for p in points:
            by_precision.setdefault(p['precision_digits'], {})[p['n_modes']] = root_digits(p,1)
        checks.append(('both HP-200 and HP-1000 present', sorted(by_precision)==[200,1000]))
        for digits, values in sorted(by_precision.items()):
            for target, window, first in BRACKETS:
                below = all(values[n] < target for n in window if n < first)
                meets = all(values[n] >= target for n in window if n >= first)
                checks.append((f'HP-{digits}: {target} digits first reached at N={first} within {window.start}-{window.stop-1}', below and meets))
            for n, printed in BRACKET_VALUES.items():
                checks.append((f'HP-{digits}: N={n} first-zero digits round to {printed}', f'{values[n]:.6f}'==printed))
        if sorted(by_precision)==[200,1000]:
            difference = max(abs(by_precision[200][n]-by_precision[1000][n]) for n in by_precision[200])
            checks.append((f'HP-200/HP-1000 first-zero digits differ by less than {BRACKET_PRECISION_DIFFERENCE:.2e}', difference < BRACKET_PRECISION_DIFFERENCE))
    if claim.startswith('claim7') and group != 'claim7b':
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


def six_digits(value):
    return f'{number(value):.5e}'


def assess_separations(directory):
    """tab:sector_separations for one evenness journal."""
    request = json.loads((directory/'request.json').read_text())
    c, n = int(request['lambda_squared']), int(request['n_modes'])
    label = f'c={c}, N={n}'
    if (c, n) not in SEPARATIONS:
        raise ValueError(f'{label}: no published sector separations for this configuration')
    record = json.loads((directory/'capture.json').read_text())
    outcomes = record['receipt']['outcomes']
    checks = [(f'{label}: {name} completed', outcomes.get(name, {}).get('status')=='completed')
              for name in ('evenness', 'sector_analysis')]
    gap = record['measurements']['sector_analysis']['value'][0]
    lambda_even = number(gap['lambda_even'])
    difference, margin = number(gap['lambda_difference']), number(gap['even_simplicity_margin'])
    odd_expected, even_expected = SEPARATIONS[(c, n)]
    checks.append((f'{label}: e0 > 0, o0-e0 > 0 and e1-e0 > 0', lambda_even > 0 and difference > 0 and margin > 0))
    checks.append((f'{label}: o0-e0 rounds to {odd_expected}', six_digits(difference)==six_digits(odd_expected)))
    checks.append((f'{label}: e1-e0 rounds to {even_expected}', six_digits(margin)==six_digits(even_expected)))
    isolation = json.loads((directory/'spectral-isolation.json').read_text())
    ratio = number(isolation['selected_state_residual_to_gap_upper_bound'])
    checks.append((f'{label}: even-state residual / same-sector gap {six_digits(ratio)} < {RESIDUAL_TO_GAP_BOUND:.2e}',
                   ratio >= 0 and ratio < RESIDUAL_TO_GAP_BOUND))
    return checks


def summarize(claim, logs):
    checks, points = [], []
    for log in logs:
        try:
            measurement, journals = read_log(log)
            status = dict(line.split('=',1) for line in log.with_suffix('.status').read_text().splitlines())
            checks.append((f'{log.name}: process completed with retained measurements',status.get('binary_exit')=='0' and status.get('log_exit')=='0' and bool(measurement)))
            for point in measurement:
                points.append(point)
                label=f'c={point["lambda_squared"]}, N={point["n_modes"]}, HP-{point["precision_digits"]}'
                checks.extend((f'{label}: {description}',passed) for description,passed in assess_point(claim,point))
            if claim.startswith('claim4'):
                if len(journals) != 1:
                    raise ValueError(f'expected one evenness journal, found {len(journals)}')
                checks.extend(assess_separations(journals[0]))
        except (OSError, ValueError, KeyError, IndexError, TypeError, ArithmeticError) as error:
            checks.append((f'{log.name}: missing/unassessed evidence: {error}',False))
    try:
        checks.extend(assess_series(claim,points))
    except (ValueError, KeyError, ArithmeticError) as error:
        checks.append((f'series could not be assessed: {error}',False))
    passed=bool(checks) and all(ok for _,ok in checks)
    return {'claim':claim,'passed':passed,'checks':[{'description':name,'passed':ok} for name,ok in checks]}


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--claim', required=True)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('logs',nargs='+',type=Path)
    args=parser.parse_args()
    with localcontext() as context:
        context.prec=100  # ratios/logarithmic digit counts; original HP scalars stay exact strings
        result=summarize(args.claim,args.logs)
    print(f'\n=== {args.claim}: claim summary ===')
    for row in result['checks']:
        print(f'[{"PASS" if row["passed"] else "FAIL"}] {row["description"]}')
    print(f'Claim {args.claim}: {"PASS" if result["passed"] else "FAIL"}')
    print('These are finite numerical reproduction checks, not theorems or interval certificates.')
    args.output.write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')
    print(f'Summary: {args.output}')
    return 0 if result['passed'] else 1


if __name__=='__main__':
    raise SystemExit(main())
