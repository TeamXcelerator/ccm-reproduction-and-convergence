"""Scientific acceptance tests; run with python3 -m unittest discover -s tests."""
import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

spec = importlib.util.spec_from_file_location('claim_summary', Path(__file__).resolve().parents[1] / 'scripts/claim_summary.py')
summary = importlib.util.module_from_spec(spec)
spec.loader.exec_module(summary)


class ClaimSummaryTests(unittest.TestCase):
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
