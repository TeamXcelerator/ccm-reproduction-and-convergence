import json
import hashlib
import io
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch
from zipfile import ZipFile

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'verification'))
import evenness_sources as verifier


class EvennessSourcesTests(unittest.TestCase):
    def test_compact_proof_digest_binds_ordered_proof_values_across_producers(self):
        proof = dict(source_identity={'build_identity': 'A'}, producer_toolkit_version='A',
                     windows=[{'upper': '2', 'lower': '1'}, {'upper': '4', 'lower': '3'}])
        # Independent canonical bytes preserve the order of the two windows.
        encoded = b'{"windows":[{"lower":"1","upper":"2"},{"lower":"3","upper":"4"}]}'
        expected = hashlib.sha256(encoded).hexdigest()
        self.assertEqual(verifier.compact_proof_digest(proof), expected)
        proof.update(source_identity={'build_identity': 'B'}, producer_toolkit_version='B')
        self.assertEqual(verifier.compact_proof_digest(proof), expected)
        proof['windows'].reverse()
        self.assertNotEqual(verifier.compact_proof_digest(proof), expected)

    def test_typed_cache_reader_authenticates_the_package_and_its_payload(self):
        with tempfile.TemporaryDirectory() as temp:
            cache = Path(temp)
            raw = b'{"test":1}'
            buffer = io.BytesIO()
            with ZipFile(buffer, 'w') as archive:
                archive.writestr('payload.json', raw)
            package = buffer.getvalue()
            oid = hashlib.sha256(package).hexdigest()
            path = cache/'objects/sha256'/oid[:2]/oid
            path.parent.mkdir(parents=True)
            path.write_bytes(package)
            manifest = dict(objects=[{'content_digest': oid}], size_bytes=len(raw),
                            content_digest=hashlib.sha256(raw).hexdigest())
            self.assertEqual(verifier.payload(cache, manifest), {'test': 1})
            with self.assertRaisesRegex(ValueError, 'contents'):
                verifier.payload(cache, dict(manifest, size_bytes=len(raw)+1))
            with self.assertRaisesRegex(ValueError, 'payload digest'):
                verifier.payload(cache, dict(manifest, content_digest='0'*64), decode=None)
            path.write_bytes(package + b'changed')
            with self.assertRaisesRegex(ValueError, 'damaged artifact package'):
                verifier.payload(cache, manifest)

    def test_root_budget_is_bound_to_the_requested_certificate_and_retained_ordinals(self):
        with tempfile.TemporaryDirectory() as temp:
            journal = Path(temp)
            def save(name, doc):
                (journal/name).write_text(json.dumps(doc), encoding='utf-8')
            def source(kind, digest):
                return dict(key=dict(kind=kind, logical_key=kind, parameters_digest=digest), content_digest=digest)
            def capture(name, ref):
                return dict(receipt={'outcomes': {name: {'status': 'completed'}}},
                            measurements={name: {'value_reference': {'artifacts': [ref]}}})
            request = dict(lambda_squared=13, n_modes=1, precision_bits=256, capture={'level': 'ultra'},
                           explicit_diagnostics=['sector_gap_certificate'], sector_gap_certificate_storage={
                               'format': 'compact', 'reconstruction_source': {'source_tree_digest': 'a'*64}},
                           applicability={'requested_diagnostics': ['finite_root_budget']})
            save('request.json', request)
            cert = source('ccm_sector_gap_certificate', 'c')
            semantics = 'ccm-analytic-sector-gap-reassembly-counts-v1'
            guides = dict(ccm_even_sector_spectrum='e', ccm_odd_sector_spectrum='o', ccm_sector_gap='g')
            identity = dict(schema_version=1, artifact_kind='ccm_sector_gap_certificate',
                            mathematical_semantics_version=semantics, algorithm_semantics=semantics,
                            resolved_mathematical_parameters=dict(integer_cutoff_c=13, n_modes=1, precision_bits=320,
                                                                 representation='compact_analytic_replay_v1'),
                            source_data_identities=guides)
            key = json.dumps(identity, sort_keys=True, separators=(',', ':'))
            cert['key']['parameters_digest'] = hashlib.sha256(key.encode()).hexdigest()
            cert['tags'] = {'xc.semantic_key.v1': key}
            budget = source('ccm_finite_diagnostic_analysis', 'b')
            primary = [source(kind, str(i)) for i, kind in enumerate(
                ['ccm_weil_eigenpair', 'ccm_root_refinement', 'ccm_secular_source'])]
            primary[1]['tags'] = {'xc.semantic_key.v1': json.dumps({'resolved_mathematical_parameters':
                                                                      {'first_root_index': 1, 'root_count': 1}})}
            inputs = [dict(kind=p['key']['kind'], content_digest=p['content_digest']) for p in [*primary, cert]]
            budget['tags'] = {'xc.semantic_key.v1': json.dumps({'resolved_mathematical_parameters': {'inputs': inputs}})}
            extra = capture('sector_gap_certificate', cert)
            extra['resolved_plan'] = {'paper_capture': request}
            save('capture-explicit.json', extra)
            main = capture('finite_root_budget', budget)
            dependencies = [source('ccm_sector_spectrum', d) for d in ('e', 'o')] + [source('ccm_sector_gap', 'g')]
            sector = {'even_spectrum_content_digest': 'e', 'odd_spectrum_content_digest': 'o'}
            main['measurements']['sector_analysis'] = {
                'source_dependencies': dependencies, 'value': [sector, {'parity': 'even'}, {'parity': 'odd'}]}
            save('capture.json', main)
            save('primary-sources.json', primary)
            save('convergence-observation.json', {'source_roots': '1', 'roots': [dict(ordinal=1, source_status='converged')]})
            row = dict(requested_index=1, status='certified_local')
            report = dict(semantics='ccm-analytic-local-root-budget-compact-v1', matrix_certificate_digest='c',
                          lambda_squared=13, n_modes=1, rows=[row])
            budget_data = dict(diagnostic='finite_root_budget', result=dict(
                request={'windows': [row]}, report=report),
                rows=[dict(label='1', outcome='certified_finite_enclosure', result=row)])
            objects = dict(c=dict(integer_cutoff_c=13, n_modes=1, precision_bits=320,
                                  source_identity={'source_tree_digest': 'd'*64},
                                  reconstruction_semantics=semantics, guide_even_spectrum_content_digest='e',
                                  guide_odd_spectrum_content_digest='o', guide_sector_gap_content_digest='g',
                                  full_positive=3, full_negative=0, certifies_finite_ground_state_simple=True),
                           b={'data': budget_data})
            index = {(m['content_digest'], m['key']['kind'], m['key']['parameters_digest']): m for m in (cert,budget)}
            def load(_, manifest, **kw):
                data = objects[manifest['content_digest']]
                return json.dumps(data).encode() if kw.get('decode') is bytes else data
            with patch.object(verifier, 'payload', side_effect=load), patch.object(verifier, 'verify_compact_payload') as replay:
                checked = verifier.sector_budget(journal, journal, index, 'selected-binary')
                replay.assert_called_once_with(json.dumps(objects['c']).encode(), 'c', 'selected-binary')
                self.assertEqual((checked['certified'], checked['retained'], checked['unresolved']), (1, 1, 0))
                objects['c']['n_modes'] = 2
                with self.assertRaisesRegex(ValueError, 'semantic key'):
                    verifier.sector_budget(journal, journal, index)
                objects['c']['n_modes'] = 1
                dependencies[0]['content_digest'] = 'foreign'
                save('capture.json', main)
                with self.assertRaisesRegex(ValueError, 'sector sources'):
                    verifier.sector_budget(journal, journal, index)
                dependencies[0]['content_digest'] = 'e'
                sector.update(even_spectrum_content_digest='o', odd_spectrum_content_digest='e')
                save('capture.json', main)
                with self.assertRaisesRegex(ValueError, 'sector sources'):
                    verifier.sector_budget(journal, journal, index)
                sector.update(even_spectrum_content_digest='e', odd_spectrum_content_digest='o')
                save('capture.json', main)
                row['status'] = 'unresolved'
                with self.assertRaisesRegex(ValueError, 'row outcome'):
                    verifier.sector_budget(journal, journal, index)
                row['status'] = 'certified_local'
                report['matrix_certificate_digest'] = 'unrelated'
                with self.assertRaisesRegex(ValueError, 'foreign root-budget'):
                    verifier.sector_budget(journal, journal, index)
                report['matrix_certificate_digest'] = 'c'
                original_tag = budget['tags']['xc.semantic_key.v1']
                proof_digest = verifier.compact_proof_digest(objects['c'])
                report.update(semantics='ccm-analytic-local-root-budget-compact-v2', matrix_certificate_digest=proof_digest)
                report['source_digest'] = primary[0]['content_digest']
                binding = [report['semantics'], report['source_digest'], proof_digest, budget_data['result']['request']]
                report['request_digest'] = verifier.canonical_digest(binding)
                inputs[-1].update(content_digest=proof_digest, meaning='compact-sector-gap-proof-v1')
                budget['tags']['xc.semantic_key.v1'] = json.dumps({'resolved_mathematical_parameters': {'inputs': inputs}})
                alternate = dict(cert, content_digest='alternate')
                budget['dependencies'] = [alternate]
                index[('alternate', cert['key']['kind'], cert['key']['parameters_digest'])] = alternate
                objects['alternate'] = dict(objects['c'], source_identity={'source_tree_digest': 'f'*64})
                self.assertEqual(verifier.sector_budget(journal, journal, index)['certified'], 1)
                budget_data['result']['request']['precision_bits'] = 512
                with self.assertRaisesRegex(ValueError, 'request binding'):
                    verifier.sector_budget(journal, journal, index)
                del budget_data['result']['request']['precision_bits']
                objects['alternate']['full_positive'] = 2
                with self.assertRaisesRegex(ValueError, 'different compact proof'):
                    verifier.sector_budget(journal, journal, index)
                budget['tags']['xc.semantic_key.v1'] = original_tag
                report.update(semantics='ccm-analytic-local-root-budget-compact-v1', matrix_certificate_digest='c')
                save('convergence-observation.json', {'source_roots': 'wrong', 'roots': [dict(ordinal=1, source_status='converged')]})
                with self.assertRaisesRegex(ValueError, 'another root source'):
                    verifier.sector_budget(journal, journal, index)
                save('convergence-observation.json', {'source_roots': '1', 'roots': [dict(ordinal=2, source_status='converged')]})
                with self.assertRaisesRegex(ValueError, 'retained window'):
                    verifier.sector_budget(journal, journal, index)
                for level in ('gap', 'maximum'):
                    request['capture']['level'] = level
                    request['applicability']['requested_diagnostics'] = []
                    save('request.json', request)
                    save('capture-explicit.json', extra)
                    self.assertEqual(verifier.sector_budget(journal, journal, index),
                                     {'certificate': 'c', 'storage': 'compact'})
                request['sector_gap_certificate_storage'] = {'format': 'full'}
                cert['tags'] = {'xc.semantic_key.v1': json.dumps({
                    'mathematical_semantics_version': 'ccm-cutoff-free-sector-gap-certificate-mpfr-selected-v5',
                    'resolved_mathematical_parameters': {'lambda_squared':'13', 'n_modes':1}})}
                save('request.json', request)
                save('capture-explicit.json', extra)
                self.assertEqual(verifier.sector_budget(journal, journal, index), {'certificate':'c', 'storage':'full'})
                request['lambda_squared'] = 20
                save('request.json', request)
                save('capture-explicit.json', extra)
                with self.assertRaisesRegex(ValueError, 'full sector certificate'):
                    verifier.sector_budget(journal, journal, index)
                request['lambda_squared'] = 13
                request['capture']['level'] = 'ultra'
                request['applicability']['requested_diagnostics'] = ['finite_root_budget']
                save('request.json', request)
            extra['receipt']['outcomes']['sector_gap_certificate']['status'] = 'blocked'
            save('capture-explicit.json', extra)
            with self.assertRaisesRegex(ValueError, 'incomplete or substituted'):
                verifier.sector_budget(journal, journal, index)
            request.pop('sector_gap_certificate_storage')
            save('request.json', request)
            with self.assertRaisesRegex(ValueError, 'selections disagree'):
                verifier.sector_budget(journal, journal, index)

    def test_compact_replay_requires_success_and_a_receipt_for_the_exact_input(self):
        from subprocess import CompletedProcess
        raw = b'{"proof":"fixture"}'
        digest = hashlib.sha256(raw).hexdigest()
        with patch.object(verifier.subprocess, 'run') as command:
            command.return_value = CompletedProcess([], 0, digest.encode() + b'\n', b'')
            verifier.verify_compact_payload(raw, digest, 'selected-binary')
            command.assert_called_once_with(['selected-binary', 'verify-compact-sector-certificate'],
                                            input=raw, capture_output=True)
            command.return_value = CompletedProcess([], 0, b'0'*64, b'')
            with self.assertRaisesRegex(ValueError, 'payload digest'):
                verifier.verify_compact_payload(raw, digest, 'selected-binary')
            command.return_value = CompletedProcess([], 1, digest.encode(), b'inconsistent matrix')
            with self.assertRaisesRegex(ValueError, 'inconsistent matrix'):
                verifier.verify_compact_payload(raw, digest, 'selected-binary')

    def test_dyadic_rounding_uses_ties_to_even_for_both_signs(self):
        for text, expected in [('1.125', (4, -2)), ('1.375', (6, -2)), ('-1.375', (-6, -2))]:
            self.assertEqual(verifier.dyadic(text, 3), expected)

    def test_pair_replay_checks_each_source_and_both_reference_columns(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            inventory = root / 'artifacts-v3' / 'test' / 'manifests'
            inventory.mkdir(parents=True)
            def save(path, value):
                path.write_text(json.dumps(value), encoding='utf-8')
            def manifest(h, kind, logical):
                return {'content_digest': h, 'size_bytes': 100,
                        'key': {'kind': kind, 'logical_key': logical, 'parameters_digest': h}}
            a, b = 'a'*64, 'b'*64
            left, right = [manifest(h, 'ccm_weil_eigenpair', h) for h in (a,b)]
            validation = manifest('c'*64, 'ccm_validation_record', 'ccm/evenness/13/1/1024')
            validation['tags'] = {'xc.semantic_key.v1': json.dumps({'resolved_mathematical_parameters':
                                    {'natural_eigenpair': a, 'forced_eigenpair': b}})}
            for m in (left, right, validation):
                save(inventory / (m['content_digest'] + '.json'), m)
            save(root/'primary-sources.json', [right])
            save(root/'request.json', {'precision_bits': 1024})
            save(root/'capture.json', {'measurements': {'sector_analysis': {'value': [{'lambda_odd': '2'}]}}})
            point = {'lambda_squared': 13, 'n_modes': 1, 'evenness_deviation': '0',
                     'natural_eigenvalue': '1', 'even_eigenvalue': '1'}
            reference = {'configuration': ['13', 1, 1024], 'parity_defect_recomputed': '0',
                         'relative_eigenvalue_difference': '0',
                         'natural_state_payload_sha256': verifier.canonical_state_id(left, 1024),
                         'even_state_payload_sha256': verifier.canonical_state_id(right, 1024)}
            state = {'lambda_squared': '13', 'n_modes': 1, 'precision_bits': 1024, 'eigenvalue': '1',
                     'eigenvector': ['1', '2', '1'], 'inverse_iteration': {'final_relative_residual_norm': '0'}}
            records = {a: dict(state, force_even=False), b: dict(state, force_even=True),
                       'c'*64: {'claim_scope': 'exact_stored_tau', 'natural_eigenvalue': '1',
                                'forced_eigenvalue': '1', 'evenness_deviation': '0'}}
            with patch.object(verifier, 'payload', side_effect=lambda _, m: records[m['content_digest']]):
                self.assertTrue(verifier.replay(point, root, root, reference)['pair_reference'])
                historical = dict(reference, natural_state_payload_sha256='f'*64,
                                  even_state_payload_sha256='e'*64)
                revised = dict(historical, evenness_reference=reference)
                self.assertTrue(verifier.replay(point, root, root, revised)['pair_reference'])
                revised['evenness_reference'] = dict(reference, relative_eigenvalue_difference='1e-250')
                with self.assertRaisesRegex(ValueError, 'reference relative'):
                    verifier.replay(point, root, root, revised)
                with self.assertRaisesRegex(ValueError, 'reference relative'):
                    verifier.replay(point, root, root, dict(reference, relative_eigenvalue_difference='1e-250'))
                with self.assertRaisesRegex(ValueError, 'reference reflection'):
                    verifier.replay(point, root, root, dict(reference, parity_defect_recomputed='1e-250'))
                records[a]['eigenvector'] = ['2','2','1']
                with self.assertRaisesRegex(ValueError, 'reflection defect'):
                    verifier.replay(point, root, root, reference)
                save(root/'primary-sources.json', [left])
                with self.assertRaisesRegex(ValueError, 'unique source pair'):
                    verifier.replay(point, root, root, reference)
