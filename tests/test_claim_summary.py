"""Scientific acceptance tests; run with python3 -m unittest discover -s tests."""
import importlib.util
import json
import tempfile
import unittest
from decimal import Decimal, localcontext
from pathlib import Path

spec = importlib.util.spec_from_file_location('claim_summary', Path(__file__).resolve().parents[1] / 'scripts/claim_summary.py')
summary = importlib.util.module_from_spec(spec)
spec.loader.exec_module(summary)


def root_point(n, digits, precision):
    return {'lambda_squared': 13, 'n_modes': n, 'precision_digits': precision,
            'roots': [{'reference_index': 1, 'status': 'converged', 'matching_digits': digits}]}


def bracket_digits():
    """First-zero digits shaped like tab:critical_brackets."""
    values = {n: f'{45 + 0.5 * (n - 31):.6f}' for n in range(31, 39)}
    values.update({39: '49.745584', 40: '50.293684'})
    values.update({n: f'{52 + 0.5 * (n - 51):.6f}' for n in range(51, 55)})
    values.update({55: '54.708183', 56: '55.011854'})
    values.update({n: f'{55.1 + 0.1 * (n - 57):.6f}' for n in range(57, 61)})
    return values


def bracket_points(values=None, hp200=None):
    values = values or bracket_digits()
    hp200 = hp200 or values
    return ([root_point(n, d, 200) for n, d in hp200.items()]
            + [root_point(n, d, 1000) for n, d in values.items()])


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
        self.assertTrue(all(not ok for _, ok in summary.assess_series('claim7b_critical_brackets', [])))

    def test_evenness_requires_positive_agreeing_states(self):
        point = {'lambda_squared': 13, 'n_modes': 120, 'precision_digits': 1000, 'evenness_deviation': '1e-900',
                 'natural_eigenvalue': '3.48399e-59', 'even_eigenvalue': '3.48399e-59'}
        self.assertTrue(all(ok for _, ok in summary.assess_point('claim4a_lambda13', point)))
        point['natural_eigenvalue'] = '-3.48399e-59'
        self.assertFalse(all(ok for _, ok in summary.assess_point('claim4a_lambda13', point)))

    def test_evenness_requires_each_published_precision(self):
        for claim, c, n, precision in (('claim4a', 13, 120, 1000), ('claim4b', 100, 500, 1000),
                                      ('claim4c', 1000, 800, 2000), ('claim4c', 1000, 890, 2000),
                                      ('claim4d', 1200, 970, 2000)):
            point = {'lambda_squared': c, 'n_modes': n, 'precision_digits': precision,
                     'evenness_deviation': '1e-900', 'natural_eigenvalue': '1e-1500',
                     'even_eigenvalue': '1e-1500'}
            with self.subTest(claim=claim, n=n):
                self.assertTrue(all(ok for _, ok in summary.assess_point(claim, point)))
                for invalid in (200, 1000 if precision == 2000 else 2000, None):
                    point['precision_digits'] = invalid
                    self.assertFalse(all(ok for _, ok in summary.assess_point(claim, point)))

    def test_evenness_difference_retains_subcontext_digits(self):
        natural = '1.' + '0' * 961 + '1e-59'
        point = {'lambda_squared': 13, 'n_modes': 120, 'precision_digits': 1000,
                 'evenness_deviation': '1e-966', 'natural_eigenvalue': natural, 'even_eigenvalue': '1e-59'}
        with localcontext() as context:
            context.prec = 28
            relative = summary.evenness_relative_difference(Decimal(natural), Decimal('1e-59'))
            checks = summary.assess_point('claim4a', point)
            self.assertEqual(context.prec, 28)
        self.assertEqual(relative, Decimal('1e-962'))
        self.assertTrue(all(ok for _, ok in checks))
        self.assertIn('1.00000e-962', ' '.join(description for description, _ in checks))

    def test_evenness_relative_difference_preserves_strict_threshold(self):
        point = {'lambda_squared': 13, 'n_modes': 120, 'precision_digits': 1000,
                 'evenness_deviation': '1e-966', 'even_eigenvalue': '1'}
        for natural, accepted in (('1.000000009' + '9' * 1000, True),
                                  ('1.00000001', False), ('1.00000001' + '0' * 1000 + '1', False)):
            with self.subTest(accepted=accepted, length=len(natural)):
                point['natural_eigenvalue'] = natural
                self.assertEqual(all(ok for _, ok in summary.assess_point('claim4a', point)), accepted)
        point['even_eigenvalue'] = '0'
        self.assertFalse(all(ok for _, ok in summary.assess_point('claim4a', point)))

    def test_evenness_admission_bounds_decimal_work(self):
        for invalid in ('1' * (summary.EVENNESS_SCALAR_LIMIT + 1), '1e40001', '1e-40001', 'NaN'):
            with self.subTest(length=len(invalid)), self.assertRaises(ValueError):
                summary.evenness_number(invalid)

    def test_wrong_or_missing_configuration_does_not_pass_a_claim(self):
        point = {'lambda_squared': 100, 'n_modes': 500}
        self.assertFalse(summary.assess_series('claim1a_lambda13', [point])[0][1])

    def test_critical_brackets_pass_on_the_published_crossings(self):
        points = bracket_points()
        point_checks = [ok for p in points for _, ok in summary.assess_point('claim7b_critical_brackets', p)]
        self.assertTrue(point_checks and all(point_checks))
        checks = summary.assess_series('claim7b_critical_brackets', points)
        self.assertTrue(all(ok for _, ok in checks), [d for d, ok in checks if not ok])
        # 2 precisions x (2 crossings + 4 printed values) + presence, both routes and agreement.
        self.assertEqual(len(checks), 15)

    def test_critical_brackets_fail_on_a_moved_crossing_value_or_disagreement(self):
        moved = bracket_digits()
        moved[39] = '50.000001'
        self.assertFalse(all(ok for _, ok in summary.assess_series('claim7b_critical_brackets', bracket_points(moved))))
        late = bracket_digits()
        late[57] = '54.999999'
        self.assertFalse(all(ok for _, ok in summary.assess_series('claim7b_critical_brackets', bracket_points(late))))
        printed = bracket_digits()
        printed[56] = '55.011855'
        self.assertFalse(all(ok for _, ok in summary.assess_series('claim7b_critical_brackets', bracket_points(printed))))
        drift = bracket_digits()
        drift[33] = drift[33] + '0' * 143 + '1'  # 1e-150 above the HP-1000 value, exactly
        self.assertFalse(all(ok for _, ok in summary.assess_series(
            'claim7b_critical_brackets', bracket_points(hp200=drift))))
        self.assertFalse(summary.assess_series('claim7b_critical_brackets', bracket_points()[1:])[0][1])

    def test_critical_brackets_reject_other_configurations(self):
        with self.assertRaises(ValueError):
            summary.assess_point('claim7b_critical_brackets', root_point(45, '50.1', 200))

    def write_evenness_run(self, root, gap=None, isolation=None, binary_exit='0'):
        """A claim4a log, its status file, measurements and journal."""
        measurements = root / 'evenness.json'
        measurements.write_text(json.dumps({'command': 'check-evenness', 'lambda_squared': 13, 'n_modes': 120,
                                            'precision_digits': 1000, 'evenness_deviation': '2.1e-966',
                                            'natural_eigenvalue': '3.48398819933e-59',
                                            'even_eigenvalue': '3.48398819933e-59'}))
        journal = root / 'journal'
        journal.mkdir()
        (journal / 'request.json').write_text(json.dumps({'lambda_squared': 13, 'n_modes': 120}))
        gap = gap or {'lambda_even': '3.48398819933e-59', 'lambda_difference': '3.05556499870e-55',
                      'even_simplicity_margin': '1.31185424973e-51'}
        (journal / 'capture.json').write_text(json.dumps({
            'receipt': {'outcomes': {'evenness': {'status': 'completed'}, 'sector_analysis': {'status': 'completed'}}},
            'measurements': {'sector_analysis': {'value': [gap]}}}))
        if isolation is not False:
            (journal / 'spectral-isolation.json').write_text(json.dumps(
                {'selected_state_residual_to_gap_upper_bound': isolation or '1.13209418744e-968'}))
        log = root / 'claim.log'
        log.write_text(f'Claim measurements: {measurements}\n  private claim journal: {journal}\n')
        log.with_suffix('.status').write_text(f'binary_exit={binary_exit}\nlog_exit=0\n')
        return log

    def test_evenness_claim_passes_with_the_published_separations(self):
        with tempfile.TemporaryDirectory() as directory:
            log = self.write_evenness_run(Path(directory))
            result = summary.summarize('claim4a_lambda13', [log])
            self.assertTrue(result['passed'], [c for c in result['checks'] if not c['passed']])
            described = ' '.join(c['description'] for c in result['checks'])
            for text in ('o0-e0 rounds to 3.05556e-55', 'e1-e0 rounds to 1.31185e-51', 'residual / same-sector gap'):
                self.assertIn(text, described)

    def test_evenness_claim_fails_on_wrong_separation_large_ratio_or_missing_isolation(self):
        cases = [
            {'gap': {'lambda_even': '3.48398819933e-59', 'lambda_difference': '3.05566e-55',
                     'even_simplicity_margin': '1.31185424973e-51'}},
            {'gap': {'lambda_even': '3.48398819933e-59', 'lambda_difference': '3.05556499870e-55',
                     'even_simplicity_margin': '-1.31185424973e-51'}},
            {'isolation': '2.02e-533'},
            {'isolation': False},
            {'binary_exit': '1'},
        ]
        for case in cases:
            with self.subTest(case=case), tempfile.TemporaryDirectory() as directory:
                log = self.write_evenness_run(Path(directory), **case)
                self.assertFalse(summary.summarize('claim4a_lambda13', [log])['passed'])

    def test_evenness_claim_requires_its_sector_analysis_to_complete(self):
        with tempfile.TemporaryDirectory() as directory:
            log = self.write_evenness_run(Path(directory))
            capture = Path(directory) / 'journal' / 'capture.json'
            record = json.loads(capture.read_text())
            record['receipt']['outcomes']['sector_analysis'] = {'status': 'failed'}
            capture.write_text(json.dumps(record))
            self.assertFalse(summary.summarize('claim4a_lambda13', [log])['passed'])


if __name__ == '__main__':
    unittest.main()
