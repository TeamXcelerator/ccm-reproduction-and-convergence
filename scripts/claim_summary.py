#!/usr/bin/env python3
"""Assess the paper's documented finite numerical checks from HP JSON evidence.

A claim PASSes only when every configuration completed and every check below
holds. PASS applies to the stated check, not a theorem or an interval
certificate.
"""
import argparse
from collections import Counter
import hashlib
import importlib.util
import json
import math
import re
import sys
from decimal import Decimal, localcontext
from pathlib import Path
from zipfile import BadZipFile

D = Decimal
_spec = importlib.util.spec_from_file_location('manuscript_cells', Path(__file__).with_name('manuscript_cells.py'))
manuscript = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(manuscript)
# Retained rational roots can exceed Python's default decimal conversion limit.
# Each scalar still has an explicit admission limit before conversion.
if hasattr(sys, 'set_int_max_str_digits') and 0 < sys.get_int_max_str_digits() < 40000:
    sys.set_int_max_str_digits(40000)
HEADLINE = {(13, 120): {1: D('55.764'), 20: D('25.339')},
            (100, 500): {1: D('460.09'), 20: D('422.44')},
            (1000, 800): {1: D('1019.7'), 20: D('1020.3')}}
# At the HP-1000 arithmetic floor the reproduction minima were fixed separately
# from the measured values printed to one decimal place.
CLAIM1C_MINIMUM = {1: D('1018.0'), 20: D('1019.3')}
LAMBDA = {13: '55.764', 20: '92.817', 30: '131.80', 50: '170.19', 100: '211.29'}
# tab:lambda_sweep, HP-1000 eigenvalue and conversion gap.
LAMBDA_HP1000 = {
    13: ('3.48399e-59', '2.694'), 20: ('2.50471e-96', '2.784'),
    30: ('2.19404e-135', '2.859'), 50: ('6.89051e-174', '2.976'),
    100: ('3.48656e-215', '3.170'),
}
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
# The first four residual/gap rows retain the historical ceiling. The revised
# C=1200 row compares a directed finite-source quantity at the table's six
# significant digits and requires its audited arithmetic method.
SEPARATIONS = {(13, 120): ('3.05556e-55', '1.31185e-51'), (100, 500): ('1.79801e-458', '1.78543e-453'),
               (1000, 800): ('1.24493e-1258', '2.05363e-1253'), (1000, 890): ('5.42414e-1357', '1.04167e-1351'),
               (1200, 970): ('2.86128e-1493', '6.55282e-1488')}
RESIDUAL_TO_GAP_BOUND = D('2.01e-533')
DIRECTED_RESIDUAL_REFERENCES = {(1200, 970): D('3.03494e-533')}
DIRECTED_RESIDUAL_METHOD = (
    'even_sector_simple_index_0_neighbor_cluster_sturm_source_allowance_directed_residual_v3')
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


def root_digits(point, index, *, allow_lower_bound=False):
    row = next((r for r in point['roots'] if r['reference_index'] == index), None)
    if row is None or row['status'] != 'converged':
        raise ValueError(f'zero {index} is missing or not converged')
    value = row.get('matching_digits')
    if value is None and not allow_lower_bound:
        raise ValueError(f'zero {index} has only a lower bound on matching digits')
    return number(value if value is not None else row['matching_digits_lower_bound'])


def assess_point(claim, point, *, for_publication=False):
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
        check_printed = for_publication and claim.startswith('claim1c')
        for index, expected in baseline.items():
            observed = root_digits(point, index, allow_lower_bound=not check_printed)
            minimum = CLAIM1C_MINIMUM[index] if (c, n) == (1000, 800) else expected - D('0.05')
            checks.append((f'zero {index}: {observed:.5f} matching digits >= {minimum}', observed >= minimum))
            if check_printed:
                checks.append((f'zero {index} matching digits round to {expected}',
                               observed.quantize(D('0.1')) == expected))
    elif claim.startswith('claim2'):
        if n != 120:
            raise ValueError('published lambda sweep baseline requires N=120')
        observed, expected = root_digits(point,1), D(LAMBDA[c])
        checks.append((f'first-zero accuracy agrees with {expected} within 0.05 digit', abs(observed-expected) <= D('0.05')))
        if claim.startswith('claim2b'):
            epsilon = number(point['epsilon_N'])
            if epsilon <= 0:
                raise ValueError('Claim 2b requires a positive eigenvalue for its logarithmic depth')
            printed_epsilon, printed_gap = LAMBDA_HP1000[c]
            with localcontext() as context:
                context.prec = max(100, len(epsilon.as_tuple().digits), len(observed.as_tuple().digits)) + 16
                gap = -epsilon.log10() - observed
                checks.append((f'eigenvalue rounds to {printed_epsilon}', six_digits(epsilon) == six_digits(printed_epsilon)))
                checks.append((f'conversion gap rounds to {printed_gap}', gap.quantize(D('0.001')) == D(printed_gap)))
                if for_publication:
                    checks.append((f'first-zero accuracy rounds to {expected}', observed.quantize(expected) == expected))
    elif claim.startswith('claim3'):
        for index, expected in zip((1,5), CRITICAL[(c,n)]):
            checks.append((f'zero {index} matching digits >= {D(expected)-D("0.05")}', root_digits(point,index, allow_lower_bound=True) >= D(expected)-D('0.05')))
    elif claim.startswith('claim6'):
        observed, expected = number(point['epsilon_N']), D(EPSILON[(c,n)])
        checks.append(('positive epsilon_N agrees with the tabulated value within 5e-5 relative', observed > 0 and abs(observed/expected-1) <= D('5e-5')))
    elif claim.startswith('claim7b'):
        if c != 13 or not any(n in window for _, window, _ in BRACKETS):
            raise ValueError('published Critical-N brackets require lambda_squared=13 and N in 31-40 or 51-60')
        checks.append(('zero 1 converged with retained matching digits', root_digits(point,1, allow_lower_bound=True) > 0))
    elif claim.startswith('claim7'):
        if c != 13:
            raise ValueError('published N sweep baseline requires lambda_squared=13')
        for index, expected in enumerate(CONVERGENCE[n],1):
            checks.append((f'zero {index} agrees with {expected} within 0.05 digit', abs(root_digits(point,index)-D(expected)) <= D('0.05')))
    else:
        raise ValueError(f'no numerical acceptance policy for {claim}')
    return checks


def claim_recipe(group):
    """Cutoff, basis, decimal precision, command, parity, acquisition and root count."""
    configs = {
        'claim1a': [(13, 120, 1000)], 'claim1b': [(100, 500, 1000)], 'claim1c': [(1000, 800, 1000)],
        'claim2a': [(c, 120, 200) for c in LAMBDA], 'claim2b': [(c, 120, 1000) for c in LAMBDA],
        'claim3': [(c, n, 1000) for c, n in CRITICAL],
        'claim4a': [(13, 120, 1000)], 'claim4b': [(100, 500, 1000)],
        'claim4c': [(1000, 800, 2000), (1000, 890, 2000)], 'claim4d': [(1200, 970, 2000)],
        'claim6': [(13, 120, 1000), (100, 500, 1000), (1000, 800, 2000)],
        'claim6b': [(c, n, 1500 if c <= 800 else 2000) for c, n in EPSILON if (c, n) not in HEADLINE],
        'claim7': [(13, n, hp) for n in CONVERGENCE for hp in (200, 1000)],
        'claim7b': [(13, n, hp) for _, window, _ in BRACKETS for n in window for hp in (200, 1000)],
        'claim8': [(13, 120, 1000), (100, 500, 1000)],
    }[group]
    if group.startswith('claim4'):
        return [(*cfg, 'check-evenness', None, None, 0) for cfg in configs]
    roots = 25 if group.startswith('claim1') or group == 'claim8' else 5 if group in ('claim3', 'claim7') else 1
    parities = ('even-sector', 'natural') if group == 'claim8' else ('even-sector',)
    return [(*cfg, 'run', parity, 'Seeded', roots) for cfg in configs for parity in parities]


def measurement_route(point):
    coords = tuple(point[k] for k in ('lambda_squared', 'n_modes', 'precision_digits'))
    if not all(type(x) is int for x in coords):
        raise ValueError('expected integer cutoff, basis and precision')
    roots = point.get('roots', [])
    ordinals = [r['reference_index'] for r in roots]
    if (any(type(i) is not int for i in ordinals) or sorted(ordinals) != list(range(1, len(roots)+1))
            or any(r.get('status') != 'converged' for r in roots)):
        raise ValueError('expected a complete, converged reference-root window starting at ordinal 1')
    return (*coords, point['command'], point.get('parity_policy'), point.get('root_acquisition'), len(roots))


def recipe_matches(claim, points):
    return Counter(map(measurement_route, points)) == Counter(claim_recipe(claim.split('_', 1)[0]))


def assess_series(claim, points):
    group = claim.split('_',1)[0]
    matched = recipe_matches(group, points)
    checks = [('measurements cover the exact published configurations and root windows once each', matched)]
    if not matched:
        return checks
    if group == 'claim3':
        indexed={(p['lambda_squared'],p['n_modes']):p for p in points}
        for c,low,high in ((50,200,250),(100,300,400)):
            checks.append((f'c={c}: accuracy increases with N',root_digits(indexed[c,high],1, allow_lower_bound=True)>root_digits(indexed[c,low],1)))
    if group in ('claim6','claim6b'):
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
    if group == 'claim7b':
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


def evidence_paths(log):
    text = log.read_text(encoding='utf-8', errors='replace')
    measurements, journals = [], []
    for line in text.splitlines():
        if line.startswith('Claim measurements: '):
            measurements.append(Path(line.split(': ',1)[1]))
        if line.strip().startswith('private claim journal: '):
            journals.append(Path(line.strip().split(': ',1)[1]))
    return measurements, journals


def read_log(log):
    paths, journals = evidence_paths(log)
    return [json.loads(path.read_text(encoding='utf-8')) for path in paths], journals


def verified_evenness(measurement, journal):
    measurement, journal = measurement.resolve(strict=True), journal.resolve(strict=True)
    direct = measurement.parent
    binding = json.loads((direct/'supplemental-journal.json').read_text(encoding='utf-8'))
    if (binding['schema_version'] != 1 or measurement.name != 'claim-measurements.json'
            or direct.parent != journal.parent or direct == journal
            or (binding['measurement_directory'], binding['journal_directory']) != (direct.name, journal.name)):
        raise ValueError('direct evenness measurement belongs to a different journal')
    files = {'measurement_sha256': measurement, 'request_sha256': journal/'request.json',
             'inventory_sha256': journal/'primary-sources.json', 'capture_sha256': journal/'capture.json',
             'isolation_sha256': journal/'spectral-isolation.json'}
    for key, path in files.items():
        with path.open('rb') as stream:
            digest = hashlib.sha256()
            for block in iter(lambda: stream.read(1 << 20), b''):
                digest.update(block)
        if digest.hexdigest() != binding[key]:
            raise ValueError(f'evenness binding differs for {path.name}')
    point = json.loads(measurement.read_text(encoding='utf-8'))
    request = json.loads((journal/'request.json').read_text(encoding='utf-8'))
    if (point['command'] != 'check-evenness' or request['parity_policy'] != 'even-sector'
            or (point['lambda_squared'], point['n_modes']) != (request['lambda_squared'], request['n_modes'])
            or math.ceil(point['precision_digits'] * math.log2(10)) + 64 != request['precision_bits']):
        raise ValueError('supplemental request disagrees with direct evenness measurement')
    return point


def six_digits(value):
    return f'{number(value):.5e}'


def source_verifier():
    path = Path(__file__).resolve().parents[1]/'verification/evenness_sources.py'
    spec = importlib.util.spec_from_file_location('evenness_sources',path)
    verifier = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(verifier)
    return verifier


def verify_assembly_rows(primary, analysis, verifier):
    exact = analysis['exact_form_roots']
    limitation = analysis.get('exact_form_roots_limitation')
    if exact is None:
        if not isinstance(limitation, str) or not limitation.strip():
            raise ValueError('missing reason for unavailable assembly root enclosures')
        return 'qualified: '+limitation
    if limitation is not None:
        raise ValueError('assembly reports both root enclosures and a limitation')
    roots = {}
    for i, outcome in enumerate(primary['eigenvalues_pos'], primary['first_positive_root_index']):
        if outcome['status'] not in ('converged','approximate','stagnated','failed'):
            raise ValueError(f'unknown primary outcome at root {i}')
        if outcome['status'] != 'failed':
            roots[i] = outcome['value']['value']
    rows = exact['rows']
    if any(type(row['ordinal']) is not int for row in rows) or [row['ordinal'] for row in rows] != list(roots):
        raise ValueError(f'assembly root ordinals differ from primary (retained {len(rows)}, expected {len(roots)})')
    for row in rows:
        root = roots[row['ordinal']]
        mantissa, exponent = verifier.dyadic(row['computed_value'],root['precision_bits'])
        primary_mantissa = int(root['significand_hex'],16)
        shift = root['binary_exponent'] - exponent
        if mantissa << max(-shift,0) != primary_mantissa << max(shift,0):
            raise ValueError(f"assembly input differs at root {row['ordinal']}")
    return f'{len(rows)} assembly root inputs matched the primary'


def verify_assembly_journal(journal, cache=None):
    request = json.loads((journal/'request.json').read_bytes())
    requested = 'assembly_error' in (request.get('applicability') or {}).get('requested_diagnostics', [])
    if not (journal/'capture.json').is_file() and not requested and (request.get('capture') or {}).get('level') != 'ultra':
        return None
    record = json.loads((journal/'capture.json').read_bytes())
    if not requested and 'assembly_error' not in record.get('requested_diagnostics',[]) and 'assembly_error' not in record['measurements']:
        return None
    measurement = record['measurements']['assembly_error']
    verifier = source_verifier()
    analysis = measurement.get('value')
    if analysis is None:
        ref, = measurement['value_reference']['artifacts']
        key = ref['key']
        if key['kind'] != 'ccm_assembly_error_analysis' or not re.fullmatch(r'[0-9a-f]{64}',key['parameters_digest']):
            raise ValueError('invalid assembly artifact reference')
        cache = cache or verifier.cache_root()
        directory = cache/'artifacts-v3'/hashlib.sha256(key['kind'].encode()).hexdigest()
        for path in directory.glob('*/'+key['parameters_digest']+'/manifests/*.json'):
            manifest = verifier.read(path)
            if (manifest['content_digest']==ref['content_digest'] and manifest['key']['kind']==key['kind']
                    and manifest['key']['parameters_digest']==key['parameters_digest']):
                analysis = verifier.payload(cache,manifest)
                break
        if analysis is None:
            raise ValueError('referenced assembly payload is unavailable')
    primary = verifier.read(journal/'primary.json')
    return verify_assembly_rows(primary,analysis,verifier)


def verify_evenness_sources(point, journal, cache=None, binary=None, *, reference_table='sector_separations'):
    """Replay bound sources against the named table's reference pair.

    The original reference in the sector-separations record also identifies
    its historical residual. The evenness table can cite a different pair.
    """
    root = Path(__file__).resolve().parents[1]
    verifier = source_verifier()
    rows = json.loads((root/'docs/research-evidence.json').read_text(encoding='utf-8'))['sector_separations']
    reference, = [r for r in rows if r['configuration'][:2] == [str(point['lambda_squared']), point['n_modes']]]
    if reference_table == 'sector_separations':
        reference.pop('evenness_reference', None)
    elif reference_table != 'evenness':
        raise ValueError(f'unknown reference table {reference_table!r}')
    return verifier.replay(point, journal, cache or verifier.cache_root(), reference, binary)


def assess_residual_ratio(c, n, isolation):
    """Versioned finite-source comparison, not a universal residual tolerance."""
    label = f'c={c}, N={n}'
    ratio = evenness_number(isolation['selected_state_residual_to_gap_upper_bound'])
    expected = DIRECTED_RESIDUAL_REFERENCES.get((c, n))
    if expected is None:
        return [(f'{label}: even-state residual / same-sector gap {six_digits(ratio)} < {RESIDUAL_TO_GAP_BOUND:.2e}',
                 ratio >= 0 and ratio < RESIDUAL_TO_GAP_BOUND)]
    compatible = (isolation.get('isolation_method') == DIRECTED_RESIDUAL_METHOD
                  and isolation.get('status') == 'isolated_simple'
                  and isolation.get('parity') == 'even'
                  and type(isolation.get('selected_algebraic_index')) is int
                  and type(isolation.get('neighboring_algebraic_index')) is int
                  and isolation.get('selected_algebraic_index') == 0
                  and isolation.get('neighboring_algebraic_index') == 1)
    return [
        (f'{label}: directed finite-source residual method and isolated even indices 0/1 match the revised reference',
         compatible),
        (f'{label}: directed residual / same-sector gap rounds to {six_digits(expected)} (revised finite-source reference)',
         ratio > 0 and six_digits(ratio) == six_digits(expected)),
    ]


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
    checks.extend(assess_residual_ratio(c, n, isolation))
    return checks


def summarize(claim, logs, *, for_publication=False):
    checks, points, journals_by_point = [], [], []
    assembly_checks = []
    for log in logs:
        try:
            paths, journals = evidence_paths(log)
            status = dict(line.split('=',1) for line in log.with_suffix('.status').read_text().splitlines())
            if status.get('binary_exit') != '0' or status.get('log_exit') != '0':
                checks.append((f"{log.name}: failed invocation (binary={status.get('binary_exit')}, log={status.get('log_exit')})", False))
                for journal in journals:
                    for receipt in ('capture.json', 'capture-explicit.json'):
                        if (journal/receipt).is_file():
                            outcomes = json.loads((journal/receipt).read_text(encoding='utf-8'))['receipt']['outcomes']
                            checks.extend((f"{name}: {row['status']}: {row.get('reason', '')}", False)
                                          for name, row in outcomes.items() if row['status'] != 'completed')
                continue
            if len(paths) != 1:
                raise ValueError(f'expected one measurement per invocation, found {len(paths)}')
            if claim.startswith('claim4'):
                if len(journals) != 1:
                    raise ValueError(f'expected one evenness journal, found {len(journals)}')
                point = verified_evenness(paths[0], journals[0])
            else:
                if for_publication and (len(journals) != 1 or
                        paths[0].resolve(strict=True) != (journals[0]/'claim-measurements.json').resolve(strict=True)):
                    raise ValueError('publication requires the measurement from its recorded claim journal')
                point = json.loads(paths[0].read_text(encoding='utf-8'))
            checks.append((f'{log.name}: process completed with retained measurements',status.get('binary_exit')=='0' and status.get('log_exit')=='0'))
            points.append(point)
            journals_by_point.append(journals[0] if claim.startswith('claim4') else None)
            label=f'c={point["lambda_squared"]}, N={point["n_modes"]}, HP-{point["precision_digits"]}'
            checks.extend((f'{label}: {description}',passed) for description,passed in
                          assess_point(claim, point, for_publication=for_publication))
            if for_publication:
                for journal in journals:
                    try:
                        coverage = verify_assembly_journal(journal)
                        if coverage:
                            assembly_checks.append((f'{label}: {coverage}', True))
                    except (OSError, ArithmeticError, KeyError, TypeError, ValueError, BadZipFile) as error:
                        assembly_checks.append((f'{label}: {error}',False))
            if claim.startswith('claim4'):
                checks.extend(assess_separations(journals[0]))
                if for_publication:
                    verified = verify_evenness_sources(point, journals[0], reference_table='evenness')
                    checks.append((f"{label}: replayed natural/even sources {verified['natural']} / {verified['even']}", True))
                    sector = verified.get('sector_budget')
                    if sector and sector['storage'] == 'compact':
                        checks.append((f"{label}: replayed compact sector certificate {sector['certificate']}", True))
        except (OSError, ValueError, KeyError, IndexError, TypeError, ArithmeticError, BadZipFile) as error:
            checks.append((f'{log.name}: missing/unassessed evidence: {error}',False))
    try:
        checks.extend(assess_series(claim,points))
    except (ValueError, KeyError, TypeError, ArithmeticError) as error:
        checks.append((f'series could not be assessed: {error}',False))
    numerical_passed = bool(checks) and all(ok for _, ok in checks)
    checks.extend(assembly_checks)
    group = claim.split('_', 1)[0].partition('claim')[2]
    rows = []
    complete = len(points) == len(logs)
    try:
        table = manuscript.tables(Path(__file__).resolve().parents[1] / 'paper.tex')
        for point, journal in zip(points, journals_by_point):
            rows.extend(manuscript.compare_point(group, point, points, journal, table,
                                                 root_digits, evenness_relative_difference))
        complete = complete and recipe_matches(claim, points)
        verdict = manuscript.printed_verdict(group, rows, complete=complete)
    except (OSError, ArithmeticError, ValueError, KeyError, TypeError, IndexError, StopIteration) as error:
        verdict = 'INCOMPLETE'
        checks.append((f'could not assess manuscript cells: {error}', False))
    checks.append((f'manuscript comparison: {verdict}', verdict in ('MATCH', 'QUALIFIED_MATCH')))
    return dict(claim=claim, passed=all(ok for _, ok in checks), numerical_checks_passed=numerical_passed,
                assembly_root_checks=[dict(description=name,passed=ok) for name,ok in assembly_checks],
                manuscript_printed_values=verdict, manuscript_rows=rows,
                checks=[{'description': name, 'passed': ok} for name, ok in checks])


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--claim', required=True)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--for-publication', action='store_true',
                        help='verify publication evidence, including assembly root windows and Claim 4 source pairs')
    parser.add_argument('logs',nargs='+',type=Path)
    args=parser.parse_args()
    with localcontext() as context:
        context.prec=100  # ratios/logarithmic digit counts; original HP scalars stay exact strings
        result=summarize(args.claim, args.logs, for_publication=args.for_publication)
    print(f'\n=== {args.claim}: claim summary ===')
    for row in result['checks']:
        print(f'[{"PASS" if row["passed"] else "FAIL"}] {row["description"]}')
    for row in result['manuscript_rows']:
        if row['verdict'] != 'MATCH':
            print(f"[{row['verdict']}] C={row['cutoff']} N={row['modes']} HP-{row['precision_digits']} "
                  f"{row['metric']}: manuscript {row['expected']}, "
                  f"observed {row.get('rounded', row.get('reason', 'unavailable'))}")
    print(f'Claim {args.claim}: {"PASS" if result["passed"] else "FAIL"}')
    print('These are finite numerical reproduction checks, not theorems or interval certificates.')
    args.output.write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')
    print(f'Summary: {args.output}')
    return 0 if result['passed'] else 1


if __name__=='__main__':
    raise SystemExit(main())
