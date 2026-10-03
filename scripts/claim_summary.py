#!/usr/bin/env python3
"""Assess the paper's documented finite numerical checks from HP JSON evidence.

PASS applies to the stated check, not a theorem or an interval certificate.
Capture completeness is reported independently and never changes a claim to PASS.
"""
import argparse
import hashlib
import json
import re
import sys
from fractions import Fraction
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

# Preserve the historical request set for retrospective journal review.
ULTRA_V6_ADDITIONS = {
    'state_geometry', 'indexed_transform', 'operator_energy', 'root_band',
    'reference_projection', 'compactness', 'weighted_reference_projection',
    'signed_transform', 'arithmetic_energy_full', 'directional_response_full',
    'weighted_tail', 'spectral_cluster_full', 'resolution_budget',
    'energy_allowance', 'complex_transform', 'root_transport', 'operator_cluster',
    'finite_section_transfer', 'tail_operator', 'observable_budget',
    'capture_preflight', 'consistency', 'configuration_comparison',
    'band_reconstruction', 'transform_enclosure',
}

# Toolkit 0.16.0 (Ultra capture plan v7) requests three further diagnostics.
ULTRA_V7_ADDITIONS = {'assembly_error', 'checkpoint_spectra', 'target_comparison'}
ULTRA_V8_ADDITIONS = {
    'constrained_l1_fit', 'trial_vector_energy', 'trial_vector_parity',
    'finite_root_budget', 'directional_error_bound', 'finite_tail_bound',
    'dimension_precision_budget', 'normalization_error_bound',
    'continuous_l1_bound', 'spectral_cluster_bound', 'indexed_prolate_comparison',
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
    plan = request.get('capture', {})
    if (plan.get('level') == 'ultra'
            and plan.get('semantics') == 'ccm-measurement-capture-plan-v8'):
        checkpoints = plan.get('prefix_checkpoint_dimensions', [])
        if (not checkpoints or any(type(n) is not int or n < 1 for n in checkpoints)
                or len(checkpoints) != len(set(checkpoints))):
            raise ValueError('Ultra v8 checkpoint request is missing or invalid')
        expected = (CLAIM8_NATURAL_REQUESTS | ULTRA_V6_ADDITIONS | ULTRA_V7_ADDITIONS
                    | ULTRA_V8_ADDITIONS | {'prime_power_response', 'u_flow_response'}
                    | {f'prefix_checkpoint_{n}' for n in checkpoints})
        policy = applicability.get('policy')
        if policy == 'full':
            allowed_exclusions = set()
        elif policy == 'claim8-natural':
            allowed_exclusions = {claim8_natural_checkpoint(request), 'prime_power_response', 'u_flow_response'}
        elif policy == 'claim1c-hp1000':
            if ((request.get('lambda_squared'), request.get('n_modes'), request.get('precision_bits'),
                 request.get('parity_policy')) != (1000, 800, 3386, 'even-sector')
                    or checkpoints != [801] or plan.get('prefix_working_precision_bits') is not None):
                raise ValueError('claim1c-hp1000 requires its frozen configuration')
            allowed_exclusions = {'prime_power_response', 'u_flow_response', 'prefix_ladder',
                                  'prefix_checkpoint_801', 'target_distance', 'distance_resolution',
                                  'target_residual_analysis', 'deviation_decomposition'}
        else:
            raise ValueError('unknown Ultra v8 applicability policy')
        if (set(excluded) != allowed_exclusions
                or len(requested) != len(set(requested))
                or set(requested) != expected - allowed_exclusions
                or request.get('applicability') != applicability
                or (policy == 'full' and record.get('resolved_plan', {}).get('capture') != plan)
                or (policy != 'full' and record.get('resolved_plan', {}).get('applicability') != applicability)
                or set(record['receipt']['outcomes']) != set(requested)
                or not set(requested) <= set(outcomes)):
            raise ValueError('Ultra v8 request, applicability and receipt must retain every applicable diagnostic')
        missing = ULTRA_V8_ADDITIONS - set(record['receipt']['outcomes'])
        if missing:
            raise ValueError(f'Ultra v8 receipt omitted requested diagnostics: {sorted(missing)}')
        missing_values = {name for name, row in record['receipt']['outcomes'].items()
                          if row['status'] == 'completed'} - set(record.get('measurements', {}))
        if missing_values:
            raise ValueError(f'completed diagnostics have no retained measurement: {sorted(missing_values)}')
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
        expected = CLAIM8_NATURAL_REQUESTS
        if request.get('toolkit_release') == '0.15.1':
            expected = expected | ULTRA_V6_ADDITIONS
        elif request.get('toolkit_release') == '0.16.0':
            expected = expected | ULTRA_V6_ADDITIONS | ULTRA_V7_ADDITIONS
            if request['capture'].get('semantics') == 'ccm-measurement-capture-plan-v8':
                expected |= ULTRA_V8_ADDITIONS
        if (set(excluded) != {checkpoint, 'prime_power_response', 'u_flow_response'}
                or set(requested) != expected):
            raise ValueError('claim8-natural must retain every applicable diagnostic for its toolkit version')
    return applicability


# Measurements reviewed below. Toolkit 0.16.0 may record their values by
# reference; the harness then saves the resolved values beside the receipt.
REVIEWED_MEASUREMENTS = {'distance_resolution', 'prefix_ladder', 'retained_reduction'} | ULTRA_V8_ADDITIONS


def numerical_review(record, applicability=None, resolved=None, coverage_notes=None):
    """Known numerical rejection fields remain visible even after successful capture."""
    review=[]
    notes = coverage_notes if coverage_notes is not None else []
    for name, measurement in record['measurements'].items():
        if 'value' in measurement:
            values=measurement['value']
        elif name not in REVIEWED_MEASUREMENTS:
            continue
        elif name in (resolved or {}):
            values=resolved[name]
        else:
            raise ValueError(f'{name}: value recorded by reference was not resolved')
        values=values if isinstance(values,list) else [values]
        for value in values:
            if name in ULTRA_V8_ADDITIONS:
                data = value['data']
                if value['kind'] != 'ccm_finite_diagnostic_analysis' or data['diagnostic'] != name:
                    raise ValueError(f'{name}: diagnostic identity mismatch')
                if data['outcome'] not in ('computed', 'missing_input', 'not_applicable', 'blocked',
                                            'awaiting_source', 'awaiting_cohort'):
                    raise ValueError(f'{name}: unknown diagnostic outcome')
                if data.get('input_preparation_error'):
                    review.append(f'{name}: optional input preparation failed: {data["input_preparation_error"]}')
                if data['outcome'] != 'computed':
                    notes.append(f'{name}: {data["outcome"]}: {data.get("reason", "reason unavailable")}')
                for row in data['rows']:
                    if row['outcome'] == 'premise_not_verified':
                        reason = data.get('reason')
                        for details in (row.get('result'), data.get('result')):
                            if not reason and isinstance(details, dict):
                                reason = details.get('reason')
                        review.append(f'{name}/{row["label"]}: premise not verified; {reason or "see retained premise details"}')
                    elif row['outcome'] in ('computed_not_certified', 'unresolved', 'missing_input',
                                            'conditional_energy_bound', 'conditional_finite_bound', 'finite_interpolant_enclosures',
                                            'blocked', 'awaiting_source', 'awaiting_cohort', 'not_applicable'):
                        details = row.get('result')
                        reason = details.get('reason') if isinstance(details, dict) else None
                        notes.append(f'{name}/{row["label"]}: {row["outcome"]}; {reason or "inspect retained scope and premises"}')
                    elif row['outcome'] not in ('certified_finite_enclosure', 'point_measurement'):
                        raise ValueError(f'{name}: unknown diagnostic row outcome')
            if name=='distance_resolution':
                # Toolkit 0.16.0 wraps the refinement rows in retained_evidence
                # and adds overall verdicts; earlier values hold the rows directly.
                for entry in value.get('retained_evidence', [value]):
                    for row in entry['refinements']:
                        if not row['tolerance_met']:
                            review.append(f'{name}: {row["quadrature_rule"]}/{row["grid_variable"]} at Q={row["final_resolution"]} did not meet tolerance')
                for verdict in ('refinement_ladder_tolerance_met', 'reported_resolution_tolerance_met'):
                    if value.get(verdict) is False:
                        review.append(f'{name}: {verdict} is false')
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


def numerical_coverage(records):
    """Keep the Toolkit assessment intact; missing rows never become successes."""
    result = {}
    for record in records:
        for name, coverage in record.get('numerical_coverage', {}).items():
            if name in result and result[name] != coverage:
                raise ValueError(f'conflicting numerical coverage for {name}')
            result[name] = coverage
    return result


def _stored_fraction(value):
    if (not isinstance(value, str) or len(value) > 32768
            or re.fullmatch(r'-?[0-9]+(?:/[1-9][0-9]*)?', value) is None):
        raise ValueError('invalid or oversized exact retained scalar')
    return Fraction(value)


def _portable_fraction(value):
    exponent = value['binary_exponent']
    significand = value['significand_hex']
    if (type(exponent) is not int or abs(exponent) > 65536
            or not isinstance(significand, str) or len(significand) > 16384):
        raise ValueError('portable scalar exceeds cohort admission budget')
    integer = int(significand, 16)
    return Fraction(integer << exponent) if exponent >= 0 else Fraction(integer, 1 << -exponent)


def read_convergence_observation(directory, request):
    """Join the source-derived snapshot to the retained primary, not printed digits."""
    path = directory / 'convergence-observation.json'
    if not path.exists():
        if request.get('convergence_dependency_policy'):
            raise ValueError('required source-derived convergence observation is missing')
        return None  # historical journals remain historical, never fabricated
    if path.stat().st_size > 64 * 1024 * 1024:
        raise ValueError('cohort observation exceeds admission budget')
    snapshot = json.loads(path.read_text())
    primary = json.loads((directory / 'primary.json').read_text())
    sources = json.loads((directory / 'primary-sources.json').read_text())
    build = json.loads((directory / 'build.json').read_text())
    if (snapshot['schema_version'] != 1
            or snapshot['n_modes'] != request['n_modes']
            or snapshot['precision_bits'] != request['precision_bits']
            or snapshot['precision_bits'] != primary['precision_bits']
            or Fraction(snapshot['lambda_squared']) != Fraction(str(request['lambda_squared']))):
        raise ValueError('cohort observation differs from primary configuration')
    for field, kinds in [('source_eigenpair', {'ccm_weil_eigenpair'}),
                         ('source_roots', {'ccm_root_refinement', 'ccm_root_discovery_window'})]:
        matching = [m for m in sources if m['key']['kind'] in kinds]
        if snapshot[field] is None and not matching:
            continue
        if len(matching) != 1 or matching[0]['content_digest'] != snapshot[field]:
            raise ValueError(f'cohort {field} source join mismatch')
    first = primary['first_positive_root_index']
    expected = primary['eigenvalues_pos']
    if (type(first) is not int or first < 1 or len(snapshot['roots']) != len(expected)
            or (expected and snapshot['root_precision_bits'] != primary['precision_bits'])):
        raise ValueError('cohort root window/precision mismatch')
    for offset, (row, original) in enumerate(zip(snapshot['roots'], expected)):
        if row['ordinal'] != first + offset or row['source_status'] != original['status']:
            raise ValueError('cohort root ordinal/status join mismatch')
        if row['value'] is None:
            if row['source_status'] != 'failed':
                raise ValueError('cohort nonfailed root has no value')
        elif _stored_fraction(row['value']) != _portable_fraction(original['value']['value']):
            raise ValueError('cohort root differs from exact retained primary value')
    if snapshot['global_ordinal_certified'] is not False:
        raise ValueError('cohort point snapshot cannot grant an ordinal certificate')
    if not isinstance(build['cargo_lock'], str) or not build['cargo_lock']:
        raise ValueError('cohort build identity missing')
    snapshot['model_id'] = hashlib.sha256(json.dumps({
        'cargo_lock': build['cargo_lock'], 'selection_policy': snapshot['selection_policy'],
        'acquisition': snapshot['acquisition'],
        'assembly_recipe': request.get('assembly_policy', {}).get('recipe', 'historical_unspecified')},
        sort_keys=True, separators=(',', ':')).encode()).hexdigest()
    snapshot['assembly_policy'] = request.get('assembly_policy', {'recipe': 'historical_unspecified'})
    snapshot['journal'] = str(directory)
    snapshot['observation_sha256'] = hashlib.sha256(path.read_bytes()).hexdigest()
    return snapshot


def convergence_cohorts(observations):
    """Observed changes across completed independent N/P configurations only.

    Root labels join the reports; neither point differences nor this join prove
    convergence, global indexing, error to zeta, or an a priori N/P budget.
    """
    groups = {}
    duplicates = 0
    seen = {}
    unmatched = []
    for observation in observations:
        key = (observation['model_id'], str(Fraction(observation['lambda_squared'])))
        identity = (observation['model_id'], observation['source_eigenpair'], observation['source_roots'])
        immutable = {k: v for k, v in observation.items() if k not in ('journal', 'observation_sha256')}
        if identity in seen:
            if seen[identity] != immutable:
                raise ValueError('conflicting observations of the same retained source')
            duplicates += 1
            continue
        seen[identity] = immutable
        if not observation.get('selection_policy') or not isinstance(observation.get('acquisition'), dict):
            unmatched.append({'source_eigenpair': observation['source_eigenpair'],
                              'source_roots': observation['source_roots'],
                              'reason': 'selection or root acquisition metadata unavailable'})
            continue
        labels = [r['ordinal'] for r in observation['roots']]
        if len(labels) != len(set(labels)) or any(type(k) is not int or k < 1 for k in labels):
            raise ValueError('invalid or duplicate cohort root labels')
        groups.setdefault(key, []).append(observation)
    result = []
    for (model, cutoff), members in sorted(groups.items()):
        configs = {}
        for point in members:
            configs.setdefault((point['n_modes'], point['precision_bits']), []).append(point)
        pairs = set()
        for axis in ('dimension', 'precision'):
            fixed = 1 if axis == 'dimension' else 0
            varying = 1 - fixed
            for value in sorted({c[fixed] for c in configs}):
                ladder = sorted((c for c in configs if c[fixed] == value), key=lambda c: c[varying])
                pairs.update((axis, left, right) for left, right in zip(ladder, ladder[1:]))
        comparisons = []
        for axis, left_config, right_config in sorted(pairs):
            for left in sorted(configs[left_config], key=lambda s: str(s['source_roots'])):
                for right in sorted(configs[right_config], key=lambda s: str(s['source_roots'])):
                    roots_left = {r['ordinal']: r for r in left['roots']}
                    roots_right = {r['ordinal']: r for r in right['roots']}
                    rows = []
                    for ordinal in sorted(roots_left.keys() | roots_right.keys()):
                        a, b = roots_left.get(ordinal), roots_right.get(ordinal)
                        if any(r is None or r['source_status'] != 'converged' or r['value'] is None for r in (a, b)):
                            rows.append({'ordinal': ordinal, 'status': 'unavailable',
                                         'left_status': a['source_status'] if a else 'not_requested',
                                         'right_status': b['source_status'] if b else 'not_requested'})
                            continue
                        change = abs(_stored_fraction(a['value']) - _stored_fraction(b['value']))
                        with localcontext() as context:
                            context.prec = 50
                            digits = str(-(D(change.numerator) / D(change.denominator)).log10()) if change else None
                        rows.append({'ordinal': ordinal, 'status': 'observed_point_change',
                                     'absolute_change_exact': str(change), 'decimal_stability_estimate': digits,
                                     'equal_stored_values': change == 0, 'certified_error_to_limit': None})
                    comparisons.append({'axis': axis,
                                        'axis_scope': 'requested configuration change; derived quadrature may also change, so this is not an isolated arithmetic or truncation error estimate',
                                        'left': {'N': left_config[0], 'P': left_config[1], 'source_roots': left['source_roots'], 'assembly_policy': left.get('assembly_policy')},
                                        'right': {'N': right_config[0], 'P': right_config[1], 'source_roots': right['source_roots'], 'assembly_policy': right.get('assembly_policy')}, 'roots': rows})
        result.append({'model_id': model, 'lambda_squared': cutoff, 'configuration_count': len(configs),
                       'status': 'observed_comparisons' if comparisons else 'awaiting_cohort',
                       'reference_assisted': any(m['acquisition'].get('reference_seeds_used') is not False for m in members),
                       'sources': [{'eigenpair': m['source_eigenpair'], 'roots': m['source_roots'],
                                    'observation_sha256': m.get('observation_sha256')} for m in members],
                       'comparisons': comparisons})
    return {'semantics': 'retained-independent-N-P-comparisons-v1',
            'status': 'observed_comparisons' if any(g['comparisons'] for g in result) else 'awaiting_cohort',
            'scope': 'exact changes between stored converged root points at the same cutoff, build, selection and acquisition policy; declared ordinal joins only',
            'duplicates_ignored': duplicates, 'groups': result, 'unmatched_sources': unmatched, 'selected_N_or_P': None,
            'analytic_obligations': ['infinite-mode truncation', 'cutoff-to-target error', 'branch/index matching for a limiting theorem']}


def input_preparation_review(directory, request, review, notes):
    """Assess the optional post-acquisition phase without upgrading its science."""
    policy = request.get('private_input_preparation', {})
    if not policy.get('configured'):
        return None
    path = directory / 'input-preparation-status.json'
    if not path.is_file():
        review.append('private input preparation: configured phase has no retained status')
        return {'status': 'missing'}
    status = json.loads(path.read_text())
    if status.get('status') != 'prepared':
        review.append('private input preparation: failed; inspect retained preparation status')
    elif status.get('source_join_validated') is not True or status.get('primary_recomputed') is not False:
        review.append('private input preparation: source join or acquisition contract not established')
    else:
        prepared_path = directory / 'prepared-research-inputs.json'
        if (not prepared_path.is_file() or prepared_path.stat().st_size > 64 * 1024 * 1024
                or hashlib.sha256(prepared_path.read_bytes()).hexdigest() != status.get('input_sha256')):
            raise ValueError('private prepared input missing or digest mismatch')
        component = status.get('component_preparation')
        if component != 'retained_independent_components_prepared':
            notes.append(f'private input preparation: component evidence {component or "unavailable"}')
        assessment_path = directory / 'prepared-research-assessment.json'
        if assessment_path.is_file():
            if assessment_path.stat().st_size > 1024 * 1024:
                raise ValueError('private preparation assessment exceeds its size limit')
            assessment = json.loads(assessment_path.read_text())
            if (assessment.get('schema_version') != 1
                    or re.fullmatch(r'[0-9a-f]{64}', str(assessment.get('input_sha256'))) is None
                    or assessment.get('input_sha256') != status.get('input_sha256')
                    or not isinstance(assessment.get('coverage_notes'), list)
                    or not all(isinstance(s, str) and s.strip() for s in assessment['coverage_notes'])):
                raise ValueError('private preparation assessment does not match its input')
            evidence = assessment.get('evidence')
            if (not isinstance(evidence, str) or Path(evidence).name != evidence
                    or evidence in ('.', '..')):
                raise ValueError('private preparation assessment has an invalid evidence path')
            evidence_path = directory/evidence
            if not evidence_path.is_file() or evidence_path.stat().st_size > 64 * 1024 * 1024:
                raise ValueError('private preparation assessment evidence missing or oversized')
            evidence_bytes = evidence_path.read_bytes()
            if len(evidence_bytes) > 64 * 1024 * 1024 or hashlib.sha256(evidence_bytes).hexdigest() != assessment.get('evidence_sha256'):
                raise ValueError('private preparation assessment evidence digest mismatch')
            target = request.get('research_inputs', {}).get('XC_TARGET_SPEC_FILE', {})
            if target.get('status') == 'supplied':
                source_hash = json.loads(evidence_bytes).get('source_file_sha256')
                if (re.fullmatch(r'[0-9a-f]{64}', str(target.get('sha256'))) is None
                        or source_hash != target['sha256']):
                    raise ValueError('private prepared target differs from the source recorded before acquisition')
            if assessment.get('resolution_status') not in ('absolute_bounds_only', 'unassessed'):
                raise ValueError('unrecognized private preparation resolution assessment')
            notes.append('private source energy: absolute bounds retained; small-energy resolution is not established')
            notes.extend(assessment['coverage_notes'])
            status['qualification_assessment'] = assessment
        else:
            if request.get('research_inputs', {}).get('XC_TARGET_SPEC_FILE', {}).get('status') == 'supplied':
                review.append('private input preparation: target binding evidence was not retained')
            notes.append('private input preparation: numerical precision assessment was not retained')
    return status


def summarize(claim, logs):
    checks, points, captures, observations = [], [], [], []
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
                coverage_notes=[]
                records=[]
                resolved=[]
                for filename in ('capture.json','capture-explicit.json'):
                    path=directory/filename
                    if path.exists():
                        record=json.loads(path.read_text())
                        outcomes.update(record['receipt']['outcomes'])
                        records.append(record)
                        values=path.with_name(path.stem+'-values.json')
                        resolved.append(json.loads(values.read_text()) if values.exists() else {})
                request = json.loads((directory/'request.json').read_text())
                preparation = input_preparation_review(directory, request, review, coverage_notes)
                observation = read_convergence_observation(directory, request)
                if observation is not None:
                    observations.append(observation)
                applicability = validate_applicability(state, request, records[0], outcomes)
                for record, values in zip(records, resolved):
                    review.extend(numerical_review(record, applicability, values, coverage_notes))
                excluded = applicability.get('excluded_diagnostics', {})
                complete = state.get('capture_complete',False) and bool(outcomes) and all(o['status']=='completed' for o in outcomes.values())
                captures.append({'directory':str(directory),'complete':complete,'outcomes':outcomes,'numerical_review':review,
                                 'policy':applicability.get('policy','full'),'excluded_diagnostics':excluded,
                                 'coverage_notes':coverage_notes, 'numerical_coverage':numerical_coverage(records),
                                 'input_preparation':preparation})
        except (OSError, ValueError, KeyError, IndexError, ArithmeticError) as error:
            checks.append((f'{log.name}: missing/unassessed evidence: {error}',False))
    try:
        checks.extend(assess_series(claim,points))
    except (ValueError, KeyError, ArithmeticError) as error:
        checks.append((f'series could not be assessed: {error}',False))
    passed=bool(checks) and all(ok for _,ok in checks)
    try:
        cohort = convergence_cohorts(observations)
    except (ValueError, KeyError, ArithmeticError) as error:
        checks.append((f'convergence cohort could not be assessed: {error}', False))
        passed = False
        cohort = {'status': 'failed', 'reason': str(error)}
    return {'claim':claim,'passed':passed,'checks':[{'description':name,'passed':ok} for name,ok in checks],
            'captures':captures,'convergence_cohorts':cohort}


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--claim')
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--cohort-only', action='store_true', help=argparse.SUPPRESS)
    parser.add_argument('--cohort-journal', action='append', default=[], type=Path, help=argparse.SUPPRESS)
    parser.add_argument('logs',nargs='*',type=Path)
    args=parser.parse_args()
    if args.cohort_only:
        if not args.cohort_journal or args.logs or args.claim:
            parser.error('--cohort-only requires journal directories and no claim/logs')
        observations = []
        for directory in args.cohort_journal:
            request = json.loads((directory / 'request.json').read_text())
            observation = read_convergence_observation(directory, request)
            if observation is None:
                raise ValueError(f'{directory}: no retained convergence observation')
            observations.append(observation)
        result = convergence_cohorts(observations)
        args.output.write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8')
        print(f'Cohort: {result["status"]}; point changes do not certify error to a limit. {args.output}')
        return 0
    if not args.claim or not args.logs or args.cohort_journal:
        parser.error('a claim summary requires --claim and log paths')
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
        for item in capture['coverage_notes']:
            print(f'[COVERAGE NOTE] {item}')
        for name, coverage in capture['numerical_coverage'].items():
            print(f'[COVERAGE] {name}: {coverage["outcome"]}; '
                  f'{coverage["resolved_rows"]} resolved, {coverage["qualified_rows"]} qualified, '
                  f'{coverage["unresolved_rows"]} unresolved of {coverage["retained_rows"]} retained rows')
            if coverage.get('reason'):
                print(f'  {coverage["reason"]}')
    print(f'Overall numerical checks: {"PASS" if result["passed"] else "FAIL"}')
    print(f'Convergence cohort: {result["convergence_cohorts"]["status"]}; observed changes are not analytic error bounds.')
    print('These are finite numerical reproduction checks; certificates and capture completeness are separate.')
    args.output.write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')
    print(f'Summary: {args.output}')
    return 0 if result['passed'] else 1


if __name__=='__main__':
    raise SystemExit(main())
