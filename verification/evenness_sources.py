"""Replay the two Claim 4 measurements from their cached finite eigenstates."""
import hashlib
import json
import os
import subprocess
from decimal import Decimal as D, localcontext
from pathlib import Path
from zipfile import ZipFile


def read(path):
    return json.loads(path.read_bytes())


def require(condition, message):
    if not condition:
        raise ValueError(message)


def cache_root():
    for key in ('XC_CACHE_ROOT', 'XC_TYPED_CACHE_ROOT'):
        if key in os.environ:
            require(os.environ[key], f'{key} is empty')
            return Path(os.environ[key])
    if os.getenv('XDG_CACHE_HOME'):
        return Path(os.environ['XDG_CACHE_HOME']) / 'xcelerator'
    if os.name == 'nt' and os.getenv('LOCALAPPDATA'):
        return Path(os.environ['LOCALAPPDATA']) / 'Xcelerator/cache'
    if os.getenv('HOME'):
        return Path(os.environ['HOME']) / '.cache/xcelerator'
    return Path('.xcelerator-cache')


def stream_digest(stream):
    digest = hashlib.sha256()
    for block in iter(lambda: stream.read(1 << 20), b''):
        digest.update(block)
    return digest.hexdigest()


def payload(cache, manifest, *, decode=json.loads):
    part, = manifest['objects']
    oid = part['content_digest']
    path = cache / 'objects/sha256' / oid[:2] / oid
    with path.open('rb') as source:
        require(stream_digest(source) == oid, 'damaged artifact package')
        source.seek(0)
        with ZipFile(source) as archive:
            require(archive.namelist() == ['payload.json']
                    and archive.getinfo('payload.json').file_size == manifest['size_bytes'],
                    'unexpected artifact package contents')
            with archive.open('payload.json') as stream:
                if decode is not None:
                    raw = stream.read()
                    require(hashlib.sha256(raw).hexdigest() == manifest['content_digest'], 'artifact payload digest mismatch')
                    return decode(raw)
                require(stream_digest(stream) == manifest['content_digest'], 'artifact payload digest mismatch')
    return None


def canonical_digest(value):
    canonical = json.dumps(value, sort_keys=True, ensure_ascii=False, separators=(',', ':'), allow_nan=False)
    return hashlib.sha256(canonical.encode('utf-8')).hexdigest()


def canonical_state_id(manifest, bits):
    # Portable payload identity under the Toolkit canonical JSON representation.
    descriptor = dict(schema_version=2, scalar_backend='rug_mpfr', precision_bits=bits,
                      scalar_representation='canonical-json-utf8-v1', dimensions=[],
                      endianness='not-applicable', special_value_encoding='decimal-string-or-json-number-v1',
                      ordered_items=[dict(normalized_path='payload.json', content_digest=manifest['content_digest'],
                                          size_bytes=manifest['size_bytes'])])
    return canonical_digest(descriptor)


def dyadic(text, bits):
    """Return the exact nearest binary source number as signed integer times 2**exponent."""
    num, den = D(text).as_integer_ratio()
    if not num:
        return 0, 0
    magnitude = abs(num)
    exponent = magnitude.bit_length() - den.bit_length()
    if magnitude << max(0, -exponent) < den << max(0, exponent):
        exponent -= 1
    shift = bits - 1 - exponent
    divisor = den << max(0, -shift)
    q, r = divmod(magnitude << max(0, shift), divisor)
    q += (2*r > divisor or (2*r == divisor and q % 2 == 1))
    return (q if num > 0 else -q), -shift


def verify_compact_payload(raw, digest, binary):
    executable = binary or os.getenv('BIN') or 'ccm-reproduction'
    try:
        result = subprocess.run([executable, 'verify-compact-sector-certificate'], input=raw, capture_output=True)
    except FileNotFoundError as error:
        raise ValueError('compact replay requires the Paper 1 binary; set BIN to its executable path') from error
    if result.returncode:
        detail = result.stderr.decode('utf-8', errors='replace').strip() or f'exit code {result.returncode}'
        raise ValueError(f'compact sector certificate replay failed: {detail}')
    require(result.stdout.strip() == digest.encode('ascii'), 'compact replay did not confirm the selected payload digest')


def compact_proof_digest(proof):
    return canonical_digest({k: v for k, v in proof.items() if k not in ('source_identity', 'producer_toolkit_version')})


def verify_run_compact_certificate(cache, cert, request, analysis, binary):
    requested_source = request['sector_gap_certificate_storage']['reconstruction_source']['source_tree_digest']
    require(isinstance(requested_source, str) and len(requested_source) == 64
            and all(ch in '0123456789abcdef' for ch in requested_source),
            'invalid reconstruction source digest in request')
    raw = payload(cache, cert, decode=bytes)
    proof = json.loads(raw)
    identity = json.loads(cert['tags']['xc.semantic_key.v1'])
    params = identity['resolved_mathematical_parameters']
    require(canonical_digest(identity) == cert['key']['parameters_digest']
            and identity['schema_version'] == 1
            and identity['artifact_kind'] == 'ccm_sector_gap_certificate'
            and identity['mathematical_semantics_version'] == identity['algorithm_semantics'] == proof['reconstruction_semantics']
            and params['representation'] == 'compact_analytic_replay_v1'
            and all(proof[k] == v for k, v in params.items() if k != 'representation'),
            'compact certificate differs from its semantic key')
    guides = identity['source_data_identities']
    spectra = [guides['ccm_even_sector_spectrum'], guides['ccm_odd_sector_spectrum']]
    gap = guides['ccm_sector_gap']
    sector = analysis['value'][0]
    deps = analysis['source_dependencies']
    require(spectra == [proof['guide_even_spectrum_content_digest'], proof['guide_odd_spectrum_content_digest']]
            and spectra == [sector['even_spectrum_content_digest'], sector['odd_spectrum_content_digest']]
            and gap == proof['guide_sector_gap_content_digest']
            and sorted(spectra) == sorted(d['content_digest'] for d in deps if d['key']['kind'] == 'ccm_sector_spectrum')
            and [gap] == [d['content_digest'] for d in deps if d['key']['kind'] == 'ccm_sector_gap'],
            'compact certificate differs from the recorded sector sources')
    require((proof['integer_cutoff_c'], proof['n_modes']) == (request['lambda_squared'], request['n_modes'])
            and proof['precision_bits'] >= request['precision_bits']
            and proof['reconstruction_semantics'] == 'ccm-analytic-sector-gap-reassembly-counts-v1'
            and proof['full_positive'] + proof['full_negative'] == 2*proof['n_modes']+1
            and proof['certifies_finite_ground_state_simple'] is True, 'incomplete compact sector certificate')
    verify_compact_payload(raw, cert['content_digest'], binary)
    return compact_proof_digest(proof)


def sector_budget(journal, cache, by_key, binary=None):
    request = read(journal/'request.json')
    storage = request.get('sector_gap_certificate_storage')
    ultra = request.get('capture', {}).get('level') == 'ultra'
    if ultra or storage:
        selected = 'sector_gap_certificate' in request['explicit_diagnostics']
        included = 'finite_root_budget' in request['applicability']['requested_diagnostics']
        require(selected == bool(storage) and included == (ultra and selected),
                'sector certificate and root-budget selections disagree')
    if not storage:
        return None
    require(storage['format'] in ('compact', 'full'), 'unknown sector certificate storage')
    main = read(journal/'capture.json')
    extra = read(journal/'capture-explicit.json')
    require(extra['resolved_plan']['paper_capture'] == request, 'sector certificate belongs to another request')
    def bound(record, name, kind):
        ref, = record['measurements'][name]['value_reference']['artifacts']
        key = ref['key']
        m = by_key[(ref['content_digest'], kind, key['parameters_digest'])]
        require(key['kind'] == kind and key['logical_key'] in
                (m['key']['logical_key'], 'identity/' + key['parameters_digest'])
                and record['receipt']['outcomes'][name]['status'] == 'completed',
                'incomplete or substituted ' + name)
        return m
    cert = bound(extra, 'sector_gap_certificate', 'ccm_sector_gap_certificate')
    coords = (request['lambda_squared'], request['n_modes'])
    if storage['format'] == 'compact':
        proof_digest = verify_run_compact_certificate(cache, cert, request, main['measurements']['sector_analysis'], binary)
    else:
        payload(cache, cert, decode=None)
        identity = json.loads(cert['tags']['xc.semantic_key.v1'])
        params = identity['resolved_mathematical_parameters']
        require(identity['mathematical_semantics_version'] == 'ccm-cutoff-free-sector-gap-certificate-mpfr-selected-v5'
                and (int(params['lambda_squared']), params['n_modes']) == coords,
                'full sector certificate differs from its request')
    if not ultra:
        return dict(certificate=cert['content_digest'], storage=storage['format'])
    budget = bound(main, 'finite_root_budget', 'ccm_finite_diagnostic_analysis')
    artifact = payload(cache, budget)
    inputs = json.loads(budget['tags']['xc.semantic_key.v1'])['resolved_mathematical_parameters']['inputs']
    data = artifact['data']
    report = data['result']['report']
    certificate_digest = cert['content_digest']
    cert_kind = 'ccm_sector_gap_certificate'
    sources = read(journal/'primary-sources.json')
    if report['semantics'] == 'ccm-analytic-local-root-budget-compact-v2':
        require(storage['format'] == 'compact' and
                [p.get('meaning') for p in inputs if p['kind'] == cert_kind] == ['compact-sector-gap-proof-v1'],
                'unexpected compact proof identity')
        certificate_digest = proof_digest
        eigenpair, = [m['content_digest'] for m in sources if m['key']['kind'] == 'ccm_weil_eigenpair']
        binding = [report['semantics'], eigenpair, proof_digest, data['result']['request']]
        require(report['source_digest'] == eigenpair and report['request_digest'] == canonical_digest(binding),
                'compact root-budget request binding mismatch')
        parent, = [d for d in budget['dependencies'] if d['key']['kind'] == cert_kind]
        require(parent['key']['parameters_digest'] == cert['key']['parameters_digest'],
                'root budget cites a different sector certificate key')
        if parent['content_digest'] != cert['content_digest']:
            producer = by_key[(parent['content_digest'], cert_kind, parent['key']['parameters_digest'])]
            require(compact_proof_digest(payload(cache, producer)) == proof_digest,
                    'root budget cites a different compact proof')
    else:
        expected = ('ccm-analytic-local-root-budget-compact-v1' if storage['format'] == 'compact'
                    else 'ccm-analytic-local-root-budget-v1')
        require(report['semantics'] == expected, 'unknown root-budget certificate binding')
    for kind in ('ccm_weil_eigenpair','ccm_root_refinement','ccm_secular_source','ccm_sector_gap_certificate'):
        required = [m['content_digest'] for m in sources if m['key']['kind'] == kind] if kind != cert_kind else [certificate_digest]
        require(len(required) == 1 and required == [p['content_digest'] for p in inputs if p['kind'] == kind],
                'finite root budget source mismatch for ' + kind)
    require(data['diagnostic'] == 'finite_root_budget' and report['matrix_certificate_digest'] == certificate_digest
            and (int(report['lambda_squared']), report['n_modes']) == coords, 'foreign root-budget report')
    observation = read(journal/'convergence-observation.json')
    root_source, = [m for m in sources if m['key']['kind'] == 'ccm_root_refinement']
    window = json.loads(root_source['tags']['xc.semantic_key.v1'])['resolved_mathematical_parameters']
    require(observation['source_roots'] == root_source['content_digest'], 'observation identifies another root source')
    first, count = window['first_root_index'], window['root_count']
    roots = observation['roots']
    indices = [r['ordinal'] for r in roots]
    rows = data['rows']
    require(count > 0 and len(indices) == count and all(i == first + k for k, i in enumerate(indices))
            and all(r['source_status'] == 'converged' for r in roots)
            and indices == [w['requested_index'] for w in data['result']['request']['windows']]
            and indices == [r['requested_index'] for r in report['rows']]
            and [str(i) for i in indices] == [r['label'] for r in rows]
            and [r['result'] for r in rows] == report['rows'], 'root budget does not cover the retained window')
    require(all(r['outcome'] == ('certified_finite_enclosure' if r['result']['status'] == 'certified_local'
                                 else 'unresolved') for r in rows),
            'root-budget row outcome disagrees with its certificate result')
    certified = sum(r['outcome'] == 'certified_finite_enclosure' for r in rows)
    return dict(certificate=cert['content_digest'], budget=budget['content_digest'], storage=storage['format'],
                retained=len(rows), certified=certified, unresolved=len(rows)-certified)


def replay(point, journal, cache, reference, binary=None):
    """Accept only the pair joined by the validation record and the bound even primary."""
    c, n = point['lambda_squared'], point['n_modes']
    bits = read(journal / 'request.json')['precision_bits']
    require(reference['configuration'] == [str(c), n, bits], 'unexpected reference configuration')
    reference = reference.get('evenness_reference', reference)
    primary, = [m for m in read(journal / 'primary-sources.json') if m['key']['kind'] == 'ccm_weil_eigenpair']
    manifests, by_key, candidates = {}, {}, set()
    for path in (cache / 'artifacts-v3').rglob('manifests/*.json'):
        m = read(path)
        manifests[m['content_digest']] = m
        by_key[(m['content_digest'], m['key']['kind'], m['key']['parameters_digest'])] = m
        if m['key']['logical_key'] == f'ccm/evenness/{c}/{n}/{bits}' and m['key']['kind'] == 'ccm_validation_record':
            key = json.loads(m['tags']['xc.semantic_key.v1'])['resolved_mathematical_parameters']
            if key['forced_eigenpair'] != primary['content_digest']:
                continue
            record = payload(cache, m)
            if (record['claim_scope'] == 'exact_stored_tau' and
                    all(D(record[a]) == D(point[b]) for a, b in (
                        ('evenness_deviation', 'evenness_deviation'), ('natural_eigenvalue', 'natural_eigenvalue'),
                        ('forced_eigenvalue', 'even_eigenvalue')))):
                candidates.add((key['natural_eigenpair'], key['forced_eigenpair']))
    require(len(candidates) == 1, 'direct measurement has no unique source pair matching its even primary')
    pair, = candidates
    require(all(manifests[h]['key']['kind'] == 'ccm_weil_eigenpair' for h in pair), 'unexpected pair source type')
    data = [payload(cache, manifests[h]) for h in pair]
    eigenvalues = [D(point[k]) for k in ('natural_eigenvalue', 'even_eigenvalue')]
    for ix, state in enumerate(data):
        require((state['lambda_squared'], state['n_modes'], state['precision_bits']) == (str(c), n, bits)
                and state['force_even'] is bool(ix) and len(state['eigenvector']) == 2*n+1
                and D(state['eigenvalue']) == eigenvalues[ix] > 0, 'mismatched eigenstate or eigenvalue')
    with localcontext() as ctx:
        ctx.prec = max(len(x.as_tuple().digits) for x in eigenvalues) + 40
        components = [dyadic(x, bits) for x in data[0]['eigenvector']]
        least = min(e for m, e in components if m)
        v = [m << (e-least) if m else 0 for m, e in components]
        square = sum(x*x for x in v)
        defect = (D(sum((x-y)**2 for x, y in zip(v, reversed(v)))) / D(square)).sqrt()
        reported = D(point['evenness_deviation'])
        require(abs(defect-reported) <= defect * D('1e-20'), 'reflection defect does not replay from natural state')
        gap = read(journal / 'capture.json')['measurements']['sector_analysis']['value'][0]
        odd_distance = D(gap['lambda_odd']) - eigenvalues[0]
        require(odd_distance > 0, 'measured odd spectrum does not separate the natural state')
        residuals = [D(state['inverse_iteration']['final_relative_residual_norm']) for state in data]
        require(all(r.is_finite() and 0 <= r < D(2)**(32-bits) for r in residuals),
                'source residual exceeds the Toolkit precision-floor criterion')
        # The stored infinity residual is scaled by the maximum vector component.
        # Convert to a two-norm estimate; sqrt(d) is also safe for the projected even basis.
        dim_scale = D(2*n+1).sqrt()
        parity_limit = 2 * dim_scale * residuals[0] * D(max(map(abs, v))) / D(square).sqrt() / odd_distance
        relative = abs(eigenvalues[0]-eigenvalues[1]) / eigenvalues[1]
        relative_limit = dim_scale * sum(residuals) / eigenvalues[1]
        require(defect <= parity_limit and relative <= relative_limit,
                'state pair exceeds its finite residual-based parity or eigenvalue estimate')
    ids = [canonical_state_id(manifests[h], bits) for h in pair]
    natural_reference = ids[0] == reference['natural_state_payload_sha256']
    pair_reference = natural_reference and ids[1] == reference['even_state_payload_sha256']
    require(max(defect, relative) < D('1e-200'), 'source pair does not reproduce hundreds of digits of agreement')
    if natural_reference:
        require(format(reported, '.3e') == format(D(reference['parity_defect_recomputed']), '.3e'),
                'original natural state disagrees with its reference reflection defect')
    if pair_reference:
        require(format(relative, '.3e') == format(D(reference['relative_eigenvalue_difference']), '.3e'),
                'original pair disagrees with its reference relative difference')
    return dict(natural=ids[0], even=ids[1], natural_reference=natural_reference, pair_reference=pair_reference,
                sector_budget=sector_budget(journal, cache, by_key, binary),
                reference_defect=reference['parity_defect_recomputed'],
                reference_relative=reference['relative_eigenvalue_difference'],
                parity_estimate=str(parity_limit), eigenvalue_relative_estimate=str(relative_limit),
                reflection_replay=format(defect, '.20e'), relative_difference=format(relative, '.20e'))
