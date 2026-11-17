"""Portable regression tests for claim measurement admission."""
import copy
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import Mock, patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import claim_summary as summary


class ClaimRecipeTests(unittest.TestCase):
    def test_assembly_exact_form_rows_belong_to_the_retained_primary_window(self):
        root = dict(precision_bits=4, significand_hex='a',binary_exponent=-3)
        primary = dict(first_positive_root_index=3, eigenvalues_pos=[
            dict(status='converged',value=dict(value=root)),dict(status='failed')])
        row = dict(ordinal=3,computed_value='1.25')
        verifier = summary.source_verifier()
        def check(rows):
            return summary.verify_assembly_rows(primary,dict(exact_form_roots=dict(rows=rows)),verifier)
        self.assertIn('1 assembly root',check([row]))
        for rows in ([],[row,row],[dict(row,ordinal=4)], [dict(row,computed_value='1.5')]):
            with self.subTest(rows=rows),self.assertRaises(ValueError):
                check(rows)
        self.assertIn('qualified',summary.verify_assembly_rows(primary,dict(
            exact_form_roots=None,exact_form_roots_limitation='unresolved separation'),verifier))
        for exact, limitation in ((None, '  '), (None, 1), (dict(rows=[row]), 'unresolved separation')):
            with self.subTest(exact=exact, limitation=limitation), self.assertRaises(ValueError):
                summary.verify_assembly_rows(primary, dict(exact_form_roots=exact,
                                             exact_form_roots_limitation=limitation), verifier)

    def test_ultra_cannot_silently_drop_a_requested_assembly_diagnostic(self):
        with tempfile.TemporaryDirectory() as directory:
            journal = Path(directory)
            request = dict(capture=dict(level='ultra'),applicability=dict(requested_diagnostics=['assembly_error']))
            (journal/'request.json').write_text(json.dumps(request),encoding='utf-8')
            with self.assertRaises(FileNotFoundError):
                summary.verify_assembly_journal(journal)
            (journal/'capture.json').write_text('{"measurements":{}}',encoding='utf-8')
            with self.assertRaises(KeyError):
                summary.verify_assembly_journal(journal)

    def test_direct_and_historical_tables_keep_their_own_reference_pairs(self):
        verifier = Mock()
        verifier.replay.side_effect = lambda point, journal, cache, reference, binary: reference
        point = dict(lambda_squared=13,n_modes=120)
        with patch('importlib.util.module_from_spec',return_value=verifier), \
                patch('importlib.util.spec_from_file_location'):
            original = summary.verify_evenness_sources(point,Path('.'),Path('.'))
            current = summary.verify_evenness_sources(point,Path('.'),Path('.'),reference_table='evenness')
            with self.assertRaisesRegex(ValueError,'unknown reference table'):
                summary.verify_evenness_sources(point,Path('.'),Path('.'),reference_table='other')
        self.assertNotIn('evenness_reference',original)
        revised = current['evenness_reference']
        self.assertNotEqual(revised['even_state_payload_sha256'],original['even_state_payload_sha256'])
        self.assertEqual(format(summary.D(revised['relative_eigenvalue_difference']),'.3e'),'4.501e-963')

    def test_review_rejects_values_which_pass_the_numerical_tolerance(self):
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp)
            log = directory/'run.log'
            measurement = directory/'claim-measurements.json'
            evidence = f'Claim measurements: {measurement}\n'
            log.write_text(evidence + f'private claim journal: {directory}\n', encoding='utf-8')
            (directory/'request.json').write_text('{"capture":{"level":"claim"}}', encoding='utf-8')
            log.with_suffix('.status').write_text('binary_exit=0\nlog_exit=0\n', encoding='utf-8')
            roots = [dict(reference_index=i,status='converged',matching_digits='25.339') for i in range(1,26)]
            point = dict(command='run', root_acquisition='Seeded', parity_policy='even-sector',
                         lambda_squared=13,n_modes=120,precision_digits=1000,epsilon_N='3.48399e-59',roots=roots)
            for value, expected in [('55.7644','MATCH'), ('55.744','DIFFERS'), (None,'INCOMPLETE')]:
                roots[0].update(matching_digits=value, matching_digits_lower_bound='55.764')
                measurement.write_text(json.dumps(point), encoding='utf-8')
                for publication in (False,True):
                    report = summary.summarize('claim1a',[log],for_publication=publication)
                    self.assertTrue(report['numerical_checks_passed'])
                    self.assertEqual(report['manuscript_printed_values'],expected)
                    self.assertEqual(report['passed'],expected=='MATCH')
            roots[0]['matching_digits'] = '55.7644'
            measurement.write_text(json.dumps(point), encoding='utf-8')
            other = directory/'other'
            other.mkdir()
            (other/measurement.name).write_bytes(measurement.read_bytes())
            (other/'request.json').write_bytes((directory/'request.json').read_bytes())
            for journals in ([], [other], [directory, directory]):
                with self.subTest(journals=journals):
                    log.write_text(evidence + ''.join(f'private claim journal: {path}\n' for path in journals),
                                   encoding='utf-8')
                    report = summary.summarize('claim1a', [log], for_publication=True)
                    self.assertFalse(report['passed'])
                    self.assertTrue(any('recorded claim journal' in check['description'] for check in report['checks']))

    def test_a_printed_inequality_cannot_be_read_as_a_point_value(self):
        with self.assertRaises(ValueError):
            summary.manuscript.printed_number('$>55.764$')

    def test_zero_even_eigenvalue_leaves_the_relative_difference_unassessed(self):
        point = dict(lambda_squared=13, n_modes=120, precision_digits=1000,
                     natural_eigenvalue='1', even_eigenvalue='0', evenness_deviation='0')
        table = summary.manuscript.tables(Path(__file__).resolve().parents[1]/'paper.tex')
        rows = summary.manuscript.compare_point('4a', point, [point], None, table,
                                                summary.root_digits, summary.evenness_relative_difference)
        relative, = [row for row in rows if row['metric'] == 'eigenvalue_relative_difference']
        self.assertEqual(relative['verdict'], 'UNASSESSED')

    def test_scientific_notation_preserves_significant_digits_across_decades(self):
        row = dict(printed=r'$1.000 \times 10^{-4}$', expected='1.000e-4')
        rounded = summary.manuscript.round_printed
        self.assertNotEqual(rounded(summary.D('9.998e-5'),row), summary.D(row['expected']))
        self.assertEqual(rounded(summary.D('9.9996e-5'),row), summary.D(row['expected']))

    def test_expected_measurement_coverage_for_every_route(self):
        table = summary.manuscript.tables(Path(summary.__file__).resolve().parents[1]/'paper.tex')
        total = 0
        for claim, count in summary.manuscript.CELL_COUNTS.items():
            with self.subTest(claim=claim):
                selected = set()
                for c,n,p,_,parity,_,_ in summary.claim_recipe('claim'+claim):
                    point=dict(lambda_squared=c,n_modes=n,precision_digits=p)
                    rows=summary.manuscript.expected_rows(claim,point,table)
                    self.assertEqual(len(rows),summary.manuscript.invocation_cell_count(claim,point))
                    selected.update((c,n,p,parity,r['metric'],r['root_index']) for r in rows)
                self.assertEqual(len(selected),count)
                total += count
        self.assertEqual(total,210)

    def test_only_the_four_historical_residuals_can_be_qualified(self):
        cells = summary.manuscript
        table = cells.tables(Path(summary.__file__).resolve().parents[1]/'paper.tex')
        exceptions = set()
        for claim in cells.CELL_COUNTS:
            rows = []
            for c, n, p, _, parity, _, _ in summary.claim_recipe('claim'+claim):
                point = dict(lambda_squared=c, n_modes=n, precision_digits=p)
                for row in cells.expected_rows(claim, point, table):
                    historical = (row['scope'] == 'historical_nearest_point_vs_current_directed_bound')
                    if historical:
                        exceptions.add((claim, c, n, row['metric']))
                    rows.append(dict(row, cutoff=c, modes=n, precision_digits=p, parity=parity,
                                     verdict='DIFFERENT_QUANTITY' if historical else 'MATCH',
                                     matches_printed_value=not historical))
            verdict = 'QUALIFIED_MATCH' if any(r['verdict']=='DIFFERENT_QUANTITY' for r in rows) else 'MATCH'
            self.assertEqual(cells.printed_verdict(claim, rows, complete=True), verdict)
            for i, row in enumerate(rows):
                if row['verdict'] == 'MATCH':
                    with self.subTest(claim=claim, row=i):
                        altered = dict(row, verdict='DIFFERS', matches_printed_value=False)
                        changed = rows[:i] + [altered] + rows[i+1:]
                        self.assertEqual(cells.printed_verdict(claim, changed, complete=True), 'DIFFERS')
                        altered['verdict'] = 'DIFFERENT_QUANTITY'
                        self.assertEqual(cells.printed_verdict(claim, changed, complete=True), 'INCOMPLETE')
        self.assertEqual(exceptions, {(g, c, n, 'residual_gap') for g, c, n in
                                      [('4a',13,120), ('4b',100,500), ('4c',1000,800), ('4c',1000,890)]})

    def setUp(self):
        self.points = [
            dict(command='run', lambda_squared=c, n_modes=120, precision_digits=1000,
                 parity_policy='even-sector', root_acquisition='Seeded',
                 roots=[dict(reference_index=1, status='converged', matching_digits=digits)])
            for c, digits in summary.LAMBDA.items()
        ]

    def test_precision_selects_the_correct_claim(self):
        self.assertTrue(summary.recipe_matches('claim2b', self.points))
        for point in self.points:
            point['precision_digits'] = 200
        self.assertTrue(summary.recipe_matches('claim2a', self.points))
        self.assertFalse(summary.recipe_matches('claim2b', self.points))

    def test_lower_bound_cannot_establish_two_sided_agreement(self):
        point = self.points[0]
        row = point['roots'][0]
        row['matching_digits_lower_bound'] = row.pop('matching_digits')
        with self.assertRaisesRegex(ValueError, 'only a lower bound'):
            summary.assess_point('claim2b', point)
        self.assertEqual(summary.root_digits(point, 1, allow_lower_bound=True),
                         summary.D(row['matching_digits_lower_bound']))

    def test_equal_values_do_not_replace_a_missing_route(self):
        for key, value in [('command', 'check-evenness'), ('root_acquisition', 'Independent'),
                           ('parity_policy', 'natural'), ('roots', [])]:
            with self.subTest(key=key):
                changed = copy.deepcopy(self.points)
                changed[0][key] = value
                self.assertFalse(summary.recipe_matches('claim2b', changed))
        duplicates = self.points[:-1] + [self.points[0]]
        self.assertFalse(summary.recipe_matches('claim2b', duplicates))

    def test_claim2b_subtracts_before_rounding(self):
        for c, epsilon, digits, displayed in [
            (50, '6.89051e-174', '170.185718719500', '170.19'),
            (100, '3.48656e-215', '211.287122255125', '211.29'),
        ]:
            with self.subTest(cutoff=c):
                point = next(p for p in self.points if p['lambda_squared'] == c)
                point['epsilon_N'] = epsilon
                point['roots'][0]['matching_digits'] = digits
                for publish in (False, True):
                    self.assertTrue(all(ok for _, ok in summary.assess_point('claim2b', point, for_publication=publish)))
                point['roots'][0]['matching_digits'] = displayed
                failures = [name for name, ok in summary.assess_point('claim2b', point) if not ok]
                self.assertEqual(failures, [f'conversion gap rounds to {summary.LAMBDA_HP1000[c][1]}'])

    def test_claim2b_checks_the_eigenvalue_and_printed_root_individually(self):
        point = self.points[0]
        point['epsilon_N'] = '3.48400e-59'
        point['roots'][0]['matching_digits'] = '55.763557878621'
        failures = [name for name, ok in summary.assess_point('claim2b', point) if not ok]
        self.assertEqual(failures, ['eigenvalue rounds to 3.48399e-59'])
        point['epsilon_N'] = '3.48399e-59'
        point['roots'][0]['matching_digits'] = '55.76349'
        self.assertTrue(all(ok for _, ok in summary.assess_point('claim2b', point)))
        failures = [name for name, ok in summary.assess_point('claim2b', point, for_publication=True) if not ok]
        self.assertEqual(failures, ['first-zero accuracy rounds to 55.764'])

    def test_root_window_requires_the_requested_ordinals(self):
        row = self.points[0]['roots'][0]
        row['reference_index'] = 2
        with self.assertRaises(ValueError):
            summary.recipe_matches('claim2b', self.points)
        row.update(reference_index=1, status='failed')
        with self.assertRaises(ValueError):
            summary.recipe_matches('claim2b', self.points)

    def test_claim1c_minima_and_printed_publication_values(self):
        point = dict(lambda_squared=1000, n_modes=800, roots=[
            dict(reference_index=1, status='converged', matching_digits='1019.7472033678021508'),
            dict(reference_index=20, status='converged', matching_digits='1020.3093939621565547')])
        def passes(publish):
            return all(ok for _, ok in summary.assess_point('claim1c_lambda1000', point,
                                                            for_publication=publish))
        self.assertTrue(passes(True))
        first, twentieth = point['roots']
        twentieth['matching_digits'] = '1020.2'
        self.assertFalse(passes(True))
        twentieth['matching_digits'] = '1020.3093939621565547'
        first['matching_digits'] = '1019.0049653964920486'
        self.assertTrue(passes(False))
        self.assertFalse(passes(True))
        first['matching_digits'] = '1018.0'
        self.assertTrue(passes(False))
        first['matching_digits'] = '1017.99999'
        self.assertFalse(passes(False))
        del first['matching_digits']
        first['matching_digits_lower_bound'] = '1019.7'
        self.assertTrue(passes(False))
        with self.assertRaisesRegex(ValueError, 'only a lower bound'):
            passes(True)


if __name__ == '__main__':
    unittest.main()
