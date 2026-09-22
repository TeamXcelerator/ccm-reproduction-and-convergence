"""Input selection must not bind a sweep point to another point's research data."""
import importlib.util
import tempfile
import unittest
from pathlib import Path

spec = importlib.util.spec_from_file_location('claim_inputs', Path(__file__).resolve().parents[1] / 'scripts/claim_inputs.py')
inputs = importlib.util.module_from_spec(spec)
spec.loader.exec_module(inputs)


class InputSelectionTests(unittest.TestCase):
    def test_target_selection_tracks_cutoff_and_precision_and_requires_a_file(self):
        with tempfile.TemporaryDirectory() as tmp:
            env = {'XC_TARGET_SPEC_DIR': tmp}
            for c, digits in ((13, 200), (13, 1000), (100, 1000)):
                path = Path(tmp) / f'c{c}-d{digits}.json'
                path.write_text('{}')
                selected = inputs.selected_environment(['run', '--lambda-sq', str(c), '--precision-digits', str(digits)], env)
                self.assertEqual(Path(selected['XC_TARGET_SPEC_FILE']), path.resolve())
            self.assertNotIn('XC_TARGET_SPEC_FILE', env)
            with self.assertRaises(FileNotFoundError):
                inputs.selected_environment(['run', '--lambda-sq', '50'], env)
            with self.assertRaises(ValueError):
                inputs.selected_environment([], dict(env, XC_TARGET_SPEC_FILE='old.json'))

    def test_exact_reference_precedes_cutoff_fallback_and_no_state_fallback(self):
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp)
            cutoff = directory / 'c13.json'
            exact = directory / 'c13-n120-d1000-natural.json'
            cutoff.write_text('{}')
            env = {'XC_RESEARCH_REFERENCE_DIR': tmp, 'XC_RESEARCH_INPUTS_DIR': tmp}
            args = ['run', '--lambda-sq', '13', '--n-modes', '120', '--precision-digits', '1000', '--no-force-even']
            selected = inputs.selected_environment(args, env)
            self.assertEqual(Path(selected['XC_RESEARCH_REFERENCE_FILE']), cutoff.resolve())
            self.assertNotIn('XC_RESEARCH_INPUTS_FILE', selected)
            exact.write_text('{}')
            selected = inputs.selected_environment(args, env)
            self.assertEqual(Path(selected['XC_RESEARCH_REFERENCE_FILE']), exact.resolve())
            self.assertEqual(Path(selected['XC_RESEARCH_INPUTS_FILE']), exact.resolve())
            selected = inputs.selected_environment(args[:-1], env)
            self.assertNotIn('XC_RESEARCH_INPUTS_FILE', selected)
            self.assertNotIn('XC_RESEARCH_REFERENCE_FILE', env)

    def test_missing_cutoff_does_not_reuse_another_reference(self):
        with tempfile.TemporaryDirectory() as tmp:
            (Path(tmp) / 'c13.json').write_text('{}')
            env = {'XC_RESEARCH_REFERENCE_DIR': tmp}
            self.assertNotIn('XC_RESEARCH_REFERENCE_FILE', inputs.selected_environment(['run', '--lambda-sq', '100'], env))

    def test_ambiguous_configuration_is_rejected(self):
        with self.assertRaises(ValueError):
            inputs.selected_environment([], {'XC_RESEARCH_REFERENCE_DIR': '.', 'XC_RESEARCH_REFERENCE_FILE': 'target.json'})
        with self.assertRaises(ValueError):
            inputs.selected_environment(['--lambda-sq', '../13'], {})


if __name__ == '__main__':
    unittest.main()
