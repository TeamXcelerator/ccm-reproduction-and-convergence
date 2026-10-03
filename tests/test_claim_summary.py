"""Scientific acceptance tests; run with python3 -m unittest discover -s tests."""
import importlib.util
import json
import tempfile
import unittest
from copy import deepcopy
from pathlib import Path

spec = importlib.util.spec_from_file_location('claim_summary', Path(__file__).resolve().parents[1] / 'scripts/claim_summary.py')
summary = importlib.util.module_from_spec(spec)
spec.loader.exec_module(summary)


class ClaimSummaryTests(unittest.TestCase):
    def test_post_acquisition_preparation_is_source_checked_and_cap_is_coverage(self):
        request={'private_input_preparation':{'configured':True}}
        with tempfile.TemporaryDirectory() as temp:
            directory=Path(temp)
            review,notes=[],[]
            summary.input_preparation_review(directory,request,review,notes)
            self.assertEqual(len(review),1)
            value={'status':'prepared','source_join_validated':True,'primary_recomputed':False,
                   'component_preparation':'resource_blocked'}
            prepared=b'{"input":"fixture"}'
            (directory/'prepared-research-inputs.json').write_bytes(prepared)
            value['input_sha256']=summary.hashlib.sha256(prepared).hexdigest()
            (directory/'input-preparation-status.json').write_text(json.dumps(value))
            review,notes=[],[]
            summary.input_preparation_review(directory,request,review,notes)
            self.assertFalse(review)
            self.assertIn('resource_blocked',notes[0])
            (directory/'input-preparation-status.json').write_text(json.dumps(value))
            evidence=b'{"bounds":"retained"}'
            (directory/'bounds.json').write_bytes(evidence)
            assessment={'schema_version':1,'input_sha256':value['input_sha256'],'coverage_notes':['precision unresolved'],
                        'resolution_status':'absolute_bounds_only',
                        'evidence':'bounds.json','evidence_sha256':summary.hashlib.sha256(evidence).hexdigest()}
            (directory/'prepared-research-assessment.json').write_text(json.dumps(assessment))
            summary.input_preparation_review(directory,request,review,notes)
            self.assertIn('precision unresolved',notes)
            request['research_inputs']={'XC_TARGET_SPEC_FILE':{'status':'supplied','sha256':'a'*64}}
            with self.assertRaisesRegex(ValueError,'before acquisition'):
                summary.input_preparation_review(directory,request,review,notes)
            bound_evidence=json.dumps({'bounds':'retained','source_file_sha256':'a'*64}).encode()
            (directory/'bounds.json').write_bytes(bound_evidence)
            assessment['evidence_sha256']=summary.hashlib.sha256(bound_evidence).hexdigest()
            (directory/'prepared-research-assessment.json').write_text(json.dumps(assessment))
            summary.input_preparation_review(directory,request,review,notes)
            evidence=bound_evidence
            (directory/'bounds.json').write_bytes(b'changed')
            with self.assertRaisesRegex(ValueError,'digest mismatch'):
                summary.input_preparation_review(directory,request,review,notes)
            (directory/'bounds.json').write_bytes(evidence)
            (directory/'prepared-research-inputs.json').write_bytes(b'changed')
            with self.assertRaisesRegex(ValueError,'prepared input'):
                summary.input_preparation_review(directory,request,review,notes)
            value['source_join_validated']=False
            (directory/'input-preparation-status.json').write_text(json.dumps(value))
            summary.input_preparation_review(directory,request,review,notes)
            self.assertEqual(len(review),1)

    @staticmethod
    def observation(n=8, p=231, model='same-build-and-selection', cutoff='5', delta='0'):
        return {'schema_version': 1, 'model_id': model, 'source_eigenpair': f'e-{n}-{p}',
                'source_roots': f'r-{n}-{p}', 'n_modes': n, 'precision_bits': p,
                'lambda_squared': cutoff, 'selection_policy': 'even',
                'acquisition': {'reference_seeds_used': True},
                'roots': [{'ordinal': k, 'source_status': 'converged',
                           'value': str(summary.Fraction(k) + summary.Fraction(delta))} for k in range(1, 26)]}

    def test_energy_extension_rows_retain_conditional_scope(self):
        for name,outcome in [('finite_tail_bound','conditional_energy_bound'),
                             ('trial_vector_energy','finite_interpolant_enclosures')]:
            value={'kind':'ccm_finite_diagnostic_analysis','data':{'diagnostic':name,'outcome':'computed',
                   'rows':[{'label':'extension','outcome':outcome,'result':None}]}}
            notes=[]
            review=summary.numerical_review({'measurements':{name:{'value':value}}},coverage_notes=notes)
            self.assertFalse(review)
            self.assertEqual(len(notes),1)
            self.assertIn(outcome,notes[0])

    def test_supplement_keeps_blocked_primary_visible(self):
        value={'kind':'ccm_finite_diagnostic_analysis','data':{'diagnostic':'finite_tail_bound',
               'outcome':'computed','rows':[
                   {'label':'primary_analysis','outcome':'blocked','result':{'reason':'automatic proof exhausted its budget'}},
                   {'label':'projection_energy','outcome':'conditional_energy_bound','result':None}]}}
        notes=[]
        review=summary.numerical_review({'measurements':{'finite_tail_bound':{'value':value}}},coverage_notes=notes)
        self.assertFalse(review)
        self.assertEqual(len(notes),2)
        self.assertIn('automatic proof exhausted its budget',notes[0])

    def test_cohort_waits_for_actual_configurations_and_deduplicates_replay(self):
        a = self.observation()
        b = deepcopy(a)
        b['journal'] = 'replay-directory'
        result = summary.convergence_cohorts([a, b])
        self.assertEqual(result['status'], 'awaiting_cohort')
        self.assertEqual(result['duplicates_ignored'], 1)
        self.assertIsNone(result['selected_N_or_P'])
        b['roots'][0]['value'] = '99'
        with self.assertRaisesRegex(ValueError, 'conflicting'):
            summary.convergence_cohorts([a, b])

    def test_cohort_keeps_dimension_precision_and_all_root_labels_separate(self):
        result = summary.convergence_cohorts([
            self.observation(), self.observation(n=12, delta='1/1000'),
            self.observation(p=400, delta='1/1000000')])
        pairs = result['groups'][0]['comparisons']
        self.assertEqual({p['axis'] for p in pairs}, {'dimension', 'precision'})
        self.assertEqual(len(pairs), 2)
        for pair in pairs:
            self.assertEqual([r['ordinal'] for r in pair['roots']], list(range(1, 26)))
            expected = '1/1000' if pair['axis'] == 'dimension' else '1/1000000'
            for row in pair['roots']:
                self.assertEqual(row['absolute_change_exact'], expected)
                self.assertIsNone(row['certified_error_to_limit'])
        self.assertTrue(result['groups'][0]['reference_assisted'])

    def test_cohort_does_not_join_different_cutoffs_or_policies(self):
        result = summary.convergence_cohorts([
            self.observation(), self.observation(n=12, cutoff='9'),
            self.observation(n=12, model='natural-parity'),
            self.observation(n=12, model='independent-discovery'),
            self.observation(n=12, model='different-toolkit-build')])
        self.assertEqual(len(result['groups']), 5)
        self.assertEqual(result['status'], 'awaiting_cohort')

    def test_cohort_never_fills_missing_roots_or_calls_agreement_a_proof(self):
        a, b = self.observation(), self.observation(n=12)
        b['roots'][0].update(source_status='failed', value=None)
        b['roots'][1].update(source_status='stagnated')
        b['roots'].pop()
        rows = summary.convergence_cohorts([a, b])['groups'][0]['comparisons'][0]['roots']
        for index in [0, 1, 24]:
            self.assertEqual(rows[index]['status'], 'unavailable')
            self.assertNotIn('absolute_change_exact', rows[index])
        self.assertTrue(rows[2]['equal_stored_values'])
        self.assertIsNone(rows[2]['decimal_stability_estimate'])
        self.assertIsNone(rows[2]['certified_error_to_limit'])

    def test_source_observation_is_joined_to_exact_primary_roots(self):
        with tempfile.TemporaryDirectory() as name:
            directory = Path(name)
            snapshot = self.observation()
            snapshot.update(root_precision_bits=231, global_ordinal_certified=False)
            primary = {'precision_bits': 231, 'first_positive_root_index': 1,
                       'eigenvalues_pos': [{'status': 'converged', 'value': {'value': {
                           'significand_hex': format(k, 'x'), 'binary_exponent': 0}}} for k in range(1, 26)]}
            sources = [{'key': {'kind': kind}, 'content_digest': digest} for kind, digest in [
                ('ccm_weil_eigenpair', snapshot['source_eigenpair']), ('ccm_root_refinement', snapshot['source_roots'])]]
            request = {'n_modes': 8, 'precision_bits': 231, 'lambda_squared': 5,
                       'convergence_dependency_policy': {'semantics': 'retained_then_cohort_v1'}}
            for file, data in [('primary.json', primary), ('primary-sources.json', sources),
                               ('build.json', {'cargo_lock': 'fixed dependency identity'})]:
                (directory / file).write_text(json.dumps(data))
            path = directory / 'convergence-observation.json'
            with self.assertRaisesRegex(ValueError, 'missing'):
                summary.read_convergence_observation(directory, request)
            path.write_text(json.dumps(snapshot))
            self.assertEqual(len(summary.read_convergence_observation(directory, request)['roots']), 25)
            for mutation in ['value', 'ordinal', 'status', 'source', 'precision', 'certificate']:
                bad = deepcopy(snapshot)
                if mutation == 'value': bad['roots'][0]['value'] = '100'
                elif mutation == 'ordinal': bad['roots'][0]['ordinal'] = 2
                elif mutation == 'status': bad['roots'][0]['source_status'] = 'failed'
                elif mutation == 'source': bad['source_roots'] = 'unrelated'
                elif mutation == 'precision': bad['root_precision_bits'] = 128
                else: bad['global_ordinal_certified'] = True
                path.write_text(json.dumps(bad))
                with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                    summary.read_convergence_observation(directory, request)

    def test_cohort_large_exact_scalars_are_bounded_and_missing_policy_is_retained(self):
        exact = '1' + '0' * 5000
        self.assertEqual(str(summary._stored_fraction(exact)), exact)
        for invalid in ['1e999999999', '1/0', 'x', '1' * 32769]:
            with self.assertRaises(ValueError): summary._stored_fraction(invalid)
        a = self.observation()
        a['acquisition'] = None
        result = summary.convergence_cohorts([a])
        self.assertEqual(result['status'], 'awaiting_cohort')
        self.assertEqual(len(result['unmatched_sources']), 1)

    def test_deferred_work_is_coverage_not_a_false_numerical_failure(self):
        name = 'dimension_precision_budget'
        record = {'measurements': {name: {'value_reference': {'artifacts': []}}}}
        for outcome in ('awaiting_source', 'awaiting_cohort'):
            notes = []
            value = {'kind': 'ccm_finite_diagnostic_analysis', 'data': {
                'diagnostic': name, 'outcome': outcome, 'reason': 'dependency pending', 'rows': []}}
            self.assertEqual(summary.numerical_review(record, resolved={name: value}, coverage_notes=notes), [])
            self.assertIn(outcome, notes[0])

    def test_full_ultra_v8_cannot_silently_drop_any_group_or_value(self):
        names = (summary.CLAIM8_NATURAL_REQUESTS | summary.ULTRA_V6_ADDITIONS
                 | summary.ULTRA_V7_ADDITIONS | summary.ULTRA_V8_ADDITIONS
                 | {'prefix_checkpoint_121', 'prime_power_response', 'u_flow_response'})
        self.assertEqual(len(names), 52)
        policy = {'policy': 'full', 'requested_diagnostics': sorted(names), 'excluded_diagnostics': {}}
        request = {'capture': {'level': 'ultra', 'semantics': 'ccm-measurement-capture-plan-v8',
                              'prefix_checkpoint_dimensions': [121]}, 'applicability': policy}
        outcomes = {name: {'status': 'completed'} for name in names}
        record = {'receipt': {'outcomes': outcomes}, 'resolved_plan': {'capture': request['capture']},
                  'measurements': {name: {} for name in names}}
        state = {'applicability': policy}
        summary.validate_applicability(state, request, record, outcomes)
        for name in names:
            for field in ('outcomes', 'values', 'request'):
                with self.subTest(name=name, field=field):
                    bad_state, bad_request, bad_record = deepcopy((state, request, record))
                    if field == 'outcomes': del bad_record['receipt']['outcomes'][name]
                    elif field == 'values': del bad_record['measurements'][name]
                    else: bad_request['applicability']['requested_diagnostics'].remove(name)
                    with self.assertRaises(ValueError):
                        summary.validate_applicability(bad_state, bad_request, bad_record, bad_record['receipt']['outcomes'])
        # Recording a failure is valid evidence; omitting a request is not.
        record['receipt']['outcomes']['trial_vector_energy'] = {'status': 'failed'}
        del record['measurements']['trial_vector_energy']
        summary.validate_applicability(state, request, record, record['receipt']['outcomes'])

    def test_v8_new_values_are_required_and_missing_premises_remain_visible(self):
        for name in summary.ULTRA_V8_ADDITIONS:
            record = {'measurements': {name: {'value_reference': {'artifacts': []}}}}
            with self.assertRaises(ValueError):
                summary.numerical_review(record)
            value = {'kind': 'ccm_finite_diagnostic_analysis', 'data': {
                'diagnostic': name, 'outcome': 'missing_input',
                'reason': 'declared analytic bounds unavailable', 'rows': []}}
            notes = []
            review = summary.numerical_review(record, resolved={name: value}, coverage_notes=notes)
            self.assertEqual(review, [])
            self.assertIn('missing_input', notes[0])
            value['data'].update(outcome='computed', rows=[{
                'label': 'finite', 'outcome': 'certified_finite_enclosure'}])
            self.assertEqual(summary.numerical_review(record, resolved={name: value}), [])
            value['data']['rows'][0]['outcome'] = 'unresolved'
            notes = []
            self.assertEqual(summary.numerical_review(record, resolved={name: value}, coverage_notes=notes), [])
            self.assertIn('unresolved', notes[0])
            value['data']['rows'][0]['outcome'] = 'premise_not_verified'
            self.assertIn('premise not verified', summary.numerical_review(record, resolved={name: value})[0])
            value['data']['rows'][0]['outcome'] = 'invented'
            with self.assertRaises(ValueError):
                summary.numerical_review(record, resolved={name: value})

    def test_v8_claim1c_keeps_only_its_eight_frozen_exclusions(self):
        names = (summary.CLAIM8_NATURAL_REQUESTS | summary.ULTRA_V6_ADDITIONS
                 | summary.ULTRA_V7_ADDITIONS | summary.ULTRA_V8_ADDITIONS
                 | {'prefix_checkpoint_801', 'prime_power_response', 'u_flow_response'})
        excluded = {'prime_power_response', 'u_flow_response', 'prefix_ladder', 'prefix_checkpoint_801',
                    'target_distance', 'distance_resolution', 'target_residual_analysis', 'deviation_decomposition'}
        policy = {'policy': 'claim1c-hp1000', 'requested_diagnostics': sorted(names-excluded),
                  'excluded_diagnostics': {name: 'frozen numerical limit' for name in excluded}}
        request = {'lambda_squared': 1000, 'n_modes': 800, 'precision_bits': 3386, 'parity_policy': 'even-sector',
                   'capture': {'level': 'ultra', 'semantics': 'ccm-measurement-capture-plan-v8',
                               'prefix_checkpoint_dimensions': [801]}, 'applicability': policy}
        outcomes = {name: {'status': 'completed'} for name in names-excluded}
        self.assertEqual(len(outcomes), 44)
        record = {'receipt': {'outcomes': outcomes}, 'resolved_plan': {'applicability': policy},
                  'measurements': {name: {} for name in outcomes}}
        summary.validate_applicability({'applicability': policy}, request, record, outcomes)
        request['precision_bits'] = 6712
        with self.assertRaises(ValueError):
            summary.validate_applicability({'applicability': policy}, request, record, outcomes)

    def test_premise_review_preserves_nested_reason(self):
        name = 'normalization_error_bound'
        record = {'measurements': {name: {'value_reference': {'artifacts': []}}}}
        reason = 'center normalizer is zero'
        for location in ('data', 'data_result', 'row_result', 'absent'):
            with self.subTest(location=location):
                row = {'label': 'finite', 'outcome': 'premise_not_verified', 'result': None}
                data = {'diagnostic': name, 'outcome': 'computed', 'reason': None,
                        'result': None, 'rows': [row]}
                if location == 'data':
                    data['reason'] = reason
                elif location == 'data_result':
                    data['result'] = {'reason': reason}
                elif location == 'row_result':
                    row['result'] = {'reason': reason}
                value = {'kind': 'ccm_finite_diagnostic_analysis', 'data': data}
                notes = []
                review = summary.numerical_review(record, resolved={name: value}, coverage_notes=notes)
                self.assertEqual(notes, [])
                self.assertEqual(len(review), 1)
                expected = reason if location != 'absent' else 'see retained premise details'
                self.assertEqual(review[0], f'{name}/finite: premise not verified; {expected}')

    def test_v8_requires_every_new_request_without_changing_v7(self):
        names = summary.CLAIM8_NATURAL_REQUESTS | summary.ULTRA_V6_ADDITIONS | summary.ULTRA_V7_ADDITIONS
        policy = {'policy': 'claim8-natural', 'requested_diagnostics': sorted(names),
                  'excluded_diagnostics': {name: 'requires even primary' for name in
                      ('prefix_checkpoint_121', 'prime_power_response', 'u_flow_response')}}
        request = {'toolkit_release': '0.16.0', 'lambda_squared': 13, 'n_modes': 120,
                   'precision_bits': 3386, 'parity_policy': 'natural',
                   'capture': {'level': 'ultra', 'prefix_checkpoint_dimensions': [121],
                               'semantics': 'ccm-measurement-capture-plan-v7'}, 'applicability': policy}
        outcomes = {name: {'status': 'completed'} for name in names}
        record = {'resolved_plan': {'applicability': policy}, 'receipt': {'outcomes': outcomes}}
        summary.validate_applicability({'applicability': policy}, request, record, outcomes)
        request['capture']['semantics'] = 'ccm-measurement-capture-plan-v8'
        with self.assertRaises(ValueError):
            summary.validate_applicability({'applicability': policy}, request, record, outcomes)
        names |= summary.ULTRA_V8_ADDITIONS
        policy['requested_diagnostics'] = sorted(names)
        outcomes.update({name: {'status': 'completed'} for name in summary.ULTRA_V8_ADDITIONS})
        record['measurements'] = {name: {} for name in names}
        summary.validate_applicability({'applicability': policy}, request, record, outcomes)
    def test_tiny_epsilon_keeps_sign_and_magnitude(self):
        point = {'lambda_squared': 1000, 'n_modes': 800, 'epsilon_N': '3.92215e-1264'}
        self.assertTrue(all(ok for _, ok in summary.assess_point('claim6_eps_n', point)))
        for invalid in ('0', '-3.92215e-1264', '3.92215e-1263'):
            point['epsilon_N'] = invalid
            self.assertFalse(all(ok for _, ok in summary.assess_point('claim6_eps_n', point)))

    def test_missing_or_stagnated_root_cannot_pass(self):
        point = {'lambda_squared': 13, 'n_modes': 120, 'roots': []}
        with self.assertRaises(ValueError):
            summary.assess_point('claim1a_lambda13', point)
        point['roots'] = [{'reference_index': 1, 'status': 'stagnated', 'matching_digits': '55.764'}]
        with self.assertRaises(ValueError):
            summary.assess_point('claim1a_lambda13', point)

    def test_low_accuracy_fails_headline(self):
        point = {'lambda_squared': 13, 'n_modes': 120, 'roots': [
            {'reference_index': 1, 'status': 'converged', 'matching_digits': '20'},
            {'reference_index': 20, 'status': 'converged', 'matching_digits': '25.339'}]}
        checks = summary.assess_point('claim1a_lambda13', point)
        self.assertFalse(checks[0][1])
        self.assertTrue(checks[1][1])

    def test_nan_is_rejected(self):
        for invalid in ('NaN', 'Infinity', '-Infinity'):
            with self.assertRaises(ValueError):
                summary.number(invalid)

    def test_missing_files_fail_closed(self):
        with tempfile.TemporaryDirectory() as directory:
            result = summary.summarize('claim1a_lambda13', [Path(directory) / 'absent.log'])
            self.assertFalse(result['passed'])

    def test_series_needs_both_parities_and_precisions(self):
        self.assertTrue(all(not ok for _, ok in summary.assess_series('claim8_natural_eigenvector', [])))
        self.assertTrue(all(not ok for _, ok in summary.assess_series('claim7_convergence_n', [])))

    def test_evenness_requires_positive_agreeing_states(self):
        point = {'lambda_squared': 13, 'n_modes': 120, 'evenness_deviation': '1e-900',
                 'natural_eigenvalue': '3.48399e-59', 'even_eigenvalue': '3.48399e-59'}
        self.assertTrue(all(ok for _, ok in summary.assess_point('claim4a_lambda13', point)))
        point['natural_eigenvalue'] = '-3.48399e-59'
        self.assertFalse(all(ok for _, ok in summary.assess_point('claim4a_lambda13', point)))

    def test_completed_capture_still_reports_numerical_nonacceptance(self):
        record = {'measurements': {'distance_resolution': {'value': [{'refinements': [
            {'tolerance_met': False, 'quadrature_rule': 'trapezoid', 'grid_variable': 'uniform_u', 'final_resolution': 128}
        ]}]}, 'retained_reduction': {'value': {'checks_passed': False}}}}
        review = summary.numerical_review(record)
        self.assertEqual(len(review), 2)
        self.assertIn('did not meet tolerance', review[0])

    def test_ultra_v6_requires_new_diagnostics_on_natural_route(self):
        names = summary.CLAIM8_NATURAL_REQUESTS | summary.ULTRA_V6_ADDITIONS
        policy = {'policy':'claim8-natural', 'requested_diagnostics':sorted(names),
                  'excluded_diagnostics':{name:'requires even primary' for name in
                      ('prefix_checkpoint_121','prime_power_response','u_flow_response')}}
        request = {'toolkit_release':'0.15.1', 'lambda_squared':13, 'n_modes':120,
                   'precision_bits':3386,'parity_policy':'natural',
                   'capture':{'level':'ultra','prefix_checkpoint_dimensions':[121]}, 'applicability':policy}
        outcomes = {name:{'status':'completed'} for name in names}
        record = {'resolved_plan':{'applicability':policy}, 'receipt':{'outcomes':outcomes}}
        summary.validate_applicability({'applicability':policy}, request, record, outcomes)
        outcomes.pop('transform_enclosure')
        policy['requested_diagnostics'].remove('transform_enclosure')
        with self.assertRaises(ValueError):
            summary.validate_applicability({'applicability':policy}, request, record, outcomes)

    def test_ultra_v7_requires_toolkit_0_16_diagnostics_on_natural_route(self):
        names = summary.CLAIM8_NATURAL_REQUESTS | summary.ULTRA_V6_ADDITIONS | summary.ULTRA_V7_ADDITIONS
        policy = {'policy':'claim8-natural', 'requested_diagnostics':sorted(names),
                  'excluded_diagnostics':{name:'requires even primary' for name in
                      ('prefix_checkpoint_121','prime_power_response','u_flow_response')}}
        request = {'toolkit_release':'0.16.0', 'lambda_squared':13, 'n_modes':120,
                   'precision_bits':3386,'parity_policy':'natural',
                   'capture':{'level':'ultra','prefix_checkpoint_dimensions':[121]}, 'applicability':policy}
        outcomes = {name:{'status':'completed'} for name in names}
        record = {'resolved_plan':{'applicability':policy}, 'receipt':{'outcomes':outcomes}}
        summary.validate_applicability({'applicability':policy}, request, record, outcomes)
        outcomes.pop('checkpoint_spectra')
        policy['requested_diagnostics'].remove('checkpoint_spectra')
        with self.assertRaises(ValueError):
            summary.validate_applicability({'applicability':policy}, request, record, outcomes)

    def test_distance_resolution_rows_are_reviewed_in_the_toolkit_0_16_layout(self):
        row = {'tolerance_met': False, 'quadrature_rule': 'trapezoid', 'grid_variable': 'uniform_u', 'final_resolution': 8000}
        value = {'refinement_ladder_tolerance_met': False, 'reported_resolution_tolerance_met': True,
                 'retained_evidence': [{'refinements': [row]}]}
        review = summary.numerical_review({'measurements': {'distance_resolution': {'value': value}}})
        self.assertEqual(len(review), 2)
        self.assertIn('did not meet tolerance', review[0])
        self.assertIn('refinement_ladder_tolerance_met', review[1])
        row['tolerance_met'] = True
        value['refinement_ladder_tolerance_met'] = True
        self.assertEqual(summary.numerical_review({'measurements': {'distance_resolution': {'value': value}}}), [])

    def test_referenced_values_are_reviewed_only_when_resolved(self):
        reference = {'value_reference': {'artifacts': [], 'array': False}, 'source_dependencies': []}
        record = {'measurements': {'root_conditioning': reference, 'retained_reduction': reference}}
        with self.assertRaises(ValueError):
            summary.numerical_review(record)
        review = summary.numerical_review(record, resolved={'retained_reduction': {'checks_passed': False}})
        self.assertEqual(review, ['retained_reduction: stored-matrix checks failed'])
        self.assertEqual(summary.numerical_review(record, resolved={'retained_reduction': {'checks_passed': True}}), [])

    def test_numerical_coverage_preserves_unresolved_rows_and_rejects_conflicts(self):
        coverage = {'outcome':'partial_unresolved', 'resolved_rows':1, 'qualified_rows':2,
                    'unresolved_rows':3, 'retained_rows':6, 'expected_rows':8}
        record = {'numerical_coverage':{'transform_enclosure':coverage}}
        self.assertEqual(summary.numerical_coverage([record])['transform_enclosure'], coverage)
        with self.assertRaises(ValueError):
            summary.numerical_coverage([record, {'numerical_coverage':{'transform_enclosure':{'outcome':'point_measurement'}}}])

    def test_wrong_or_missing_configuration_does_not_pass_a_claim(self):
        point = {'lambda_squared': 100, 'n_modes': 500}
        self.assertFalse(summary.assess_series('claim1a_lambda13', [point])[0][1])

    def test_innovation_only_export_needs_validated_claim8_applicability(self):
        checkpoint = {'dimension':121, 'status':'innovation_export_passed_eigenpair_not_supplied', 'eigenpair_source':None}
        record = {'measurements':{'prefix_ladder':{'value':{'ladder':{'stopped':None}, 'checkpoints':[checkpoint]}}}}
        policy = {'policy':'claim8-natural', 'requested_diagnostics':sorted(summary.CLAIM8_NATURAL_REQUESTS),
                  'excluded_diagnostics':{name:'unsupported for this primary' for name in
                      ('prefix_checkpoint_121', 'prime_power_response', 'u_flow_response')}}
        request = {'lambda_squared':13, 'n_modes':120, 'precision_bits':3386, 'parity_policy':'natural',
                   'capture':{'level':'ultra', 'prefix_checkpoint_dimensions':[121]}, 'applicability':policy}
        outcomes = {name:{'status':'completed'} for name in summary.CLAIM8_NATURAL_REQUESTS}
        record.update(resolved_plan={'applicability':policy}, receipt={'outcomes':outcomes})
        validated = summary.validate_applicability({'applicability':policy}, request, record, outcomes)
        self.assertEqual(summary.numerical_review(record, validated), [])
        self.assertEqual(len(summary.numerical_review(record)), 1)
        for status in ('unresolved_prefix', 'export_checks_failed'):
            checkpoint['status'] = status
            self.assertEqual(len(summary.numerical_review(record, validated)), 1)
        checkpoint['status'] = 'innovation_export_passed_eigenpair_not_supplied'
        checkpoint['eigenpair_source'] = {'unexpected':'state'}
        self.assertEqual(len(summary.numerical_review(record, validated)), 1)
        request['parity_policy'] = 'even-sector'
        with self.assertRaises(ValueError):
            summary.validate_applicability({'applicability':policy}, request, record, outcomes)

    def test_claim8_cannot_exclude_an_additional_failure(self):
        policy = {'policy':'claim8-natural', 'requested_diagnostics':sorted(summary.CLAIM8_NATURAL_REQUESTS - {'retained_reduction'}),
                  'excluded_diagnostics':{name:'excluded' for name in
                      ('prefix_checkpoint_121', 'prime_power_response', 'u_flow_response', 'retained_reduction')}}
        request = {'lambda_squared':13, 'n_modes':120, 'precision_bits':3386, 'parity_policy':'natural',
                   'capture':{'level':'ultra', 'prefix_checkpoint_dimensions':[121]}, 'applicability':policy}
        outcomes = {name:{'status':'completed'} for name in policy['requested_diagnostics']}
        record = {'resolved_plan':{'applicability':policy}, 'receipt':{'outcomes':outcomes}}
        with self.assertRaises(ValueError):
            summary.validate_applicability({'applicability':policy}, request, record, outcomes)

    def test_exclusions_require_matching_predeclared_request_and_receipt(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            log = root/'claim.log'
            point = {'lambda_squared':1000, 'n_modes':800, 'precision_digits':1000, 'roots':[
                {'reference_index':1, 'status':'converged', 'matching_digits':'1019.0'},
                {'reference_index':20, 'status':'converged', 'matching_digits':'1020.3'}]}
            (root/'claim-measurements.json').write_text(json.dumps(point))
            log.write_text(f'private claim journal: {root}\nClaim measurements: {root / "claim-measurements.json"}\n')
            log.with_suffix('.status').write_text('binary_exit=0\nlog_exit=0\n')
            policy = {'policy':'claim1c-hp1000', 'requested_diagnostics':['root_conditioning'],
                      'excluded_diagnostics':{'u_flow_response':'requires an isolated state'}}
            (root/'status.json').write_text(json.dumps({'capture_complete':True,'applicability':policy}))
            (root/'request.json').write_text(json.dumps({'applicability':policy}))
            record = {'resolved_plan':{'applicability':policy},'receipt':{'outcomes':{'root_conditioning':{'status':'completed'}}},'measurements':{}}
            (root/'capture.json').write_text(json.dumps(record))
            result = summary.summarize('claim1c_lambda1000', [log])
            self.assertTrue(result['passed'])
            self.assertTrue(result['captures'][0]['complete'])
            self.assertEqual(result['captures'][0]['excluded_diagnostics'], policy['excluded_diagnostics'])
            record['receipt']['outcomes']['root_conditioning'] = {'status':'failed','reason':'unexpected error'}
            (root/'capture.json').write_text(json.dumps(record))
            result = summary.summarize('claim1c_lambda1000', [log])
            self.assertFalse(result['captures'][0]['complete'])
            record['receipt']['outcomes']['u_flow_response'] = {'status':'failed','reason':'historical failure'}
            (root/'capture.json').write_text(json.dumps(record))
            self.assertFalse(summary.summarize('claim1c_lambda1000', [log])['passed'])


if __name__ == '__main__':
    unittest.main()
