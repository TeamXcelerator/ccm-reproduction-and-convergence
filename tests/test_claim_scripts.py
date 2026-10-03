"""Exercise claim orchestration with a deliberately failing test executable."""
import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


@unittest.skipIf(os.name == 'nt', 'claim shell integration runs on Linux/WSL')
class ClaimScriptTests(unittest.TestCase):
    def exercise(self, script, *arguments, extra_env=None):
        with tempfile.TemporaryDirectory(prefix='paper claim test ') as temporary:
            base = Path(temporary)
            binary = base / 'test binary'
            binary.write_text('''#!/usr/bin/env python3
import json, os, sys
from pathlib import Path
p=Path(os.environ['TEST_CALLS'])
with p.open('a') as f: f.write(json.dumps(sys.argv[1:])+'\\n')
with Path(os.environ['TEST_CALLS']+'.env').open('a') as f:
    f.write(json.dumps({k:v for k,v in os.environ.items() if k.startswith('XC_')})+'\\n')
print('intentional test executable: no scientific measurements')
raise SystemExit(1 if len(p.read_text().splitlines())==1 else 0)
''', encoding='utf-8')
            binary.chmod(0o700)
            env = {k:v for k,v in os.environ.items() if not k.startswith('XC_')}
            env.update(BIN=str(binary), CLAIM_LOG_ROOT=str(base/'logs'), TEST_CALLS=str(base/'calls.jsonl'), **(extra_env or {}))
            result = subprocess.run(['bash', str(ROOT/'scripts'/script), '--capture-output', str(base/'journals'), *arguments], cwd=ROOT, env=env, capture_output=True, text=True, timeout=30)
            calls_path = base/'calls.jsonl'
            calls = [json.loads(line) for line in calls_path.read_text().splitlines()] if calls_path.exists() else []
            env_path = base/'calls.jsonl.env'
            self.environments = [json.loads(line) for line in env_path.read_text().splitlines()] if env_path.exists() else []
            self.created_logs = (base/'logs').exists()
            self.created_journals = (base/'journals').exists()
            return result, calls

    def test_preflight_checks_each_real_case_without_inputs_logs_or_publication(self):
        result, calls = self.exercise('claim2a_hp200.sh', '--preflight-only', '--publish', extra_env={
            'XC_TARGET_SPEC_DIR': '/must-not-read-targets',
            'XC_RESEARCH_PREPARER': 'invalid-on-purpose',
            'XC_PUBLISH_EXECUTE': 'true', 'XC_PUBLISH_TARGET': 'both'})
        self.assertEqual(len(calls), 5)
        self.assertEqual(result.returncode, 1)  # Deliberately rejected first case.
        self.assertTrue(all('--preflight-only' in call for call in calls))
        self.assertFalse(self.created_logs)
        self.assertFalse(self.created_journals)
        self.assertNotIn('Overall numerical checks:', result.stdout)
        for environment in self.environments:
            self.assertEqual(environment['XC_PUBLISH_TARGET'], 'none')
            self.assertEqual(environment['XC_PUBLISH_EXECUTE'], 'false')

    def test_preflight_preserves_claim1b_configuration(self):
        result, calls = self.exercise('claim1b_lambda100.sh', '--preflight-only')
        self.assertEqual(len(calls), 1, result.stderr)
        call = calls[0]
        for name, value in [('--lambda-sq', '100'), ('--n-modes', '500'),
                            ('--precision-digits', '1000'), ('--top', '25'),
                            ('--research-capture', 'ultra'), ('--parity-policy', 'even-sector')]:
            self.assertEqual(call[call.index(name)+1], value)

    def test_failed_case_does_not_skip_later_cases_and_summary_fails(self):
        result, calls = self.exercise('claim2a_hp200.sh')
        self.assertEqual(len(calls), 5)
        self.assertEqual(result.returncode, 1)
        self.assertIn('Overall numerical checks: FAIL', result.stdout)
        for call in calls:
            self.assertEqual(call[call.index('--research-capture')+1], 'ultra')

    def test_natural_comparison_keeps_ultra_and_its_primary_policy(self):
        result, calls = self.exercise('claim8_natural_eigenvector.sh')
        self.assertEqual(len(calls), 4)
        self.assertEqual(sum('--no-force-even' in call for call in calls), 2)
        self.assertTrue(all(call[call.index('--research-capture')+1]=='ultra' for call in calls))
        for call in calls:
            if '--no-force-even' in call:
                self.assertEqual(call[call.index('--capture-policy')+1], 'claim8-natural')
            else:
                self.assertNotIn('--capture-policy', call)
        self.assertEqual(result.returncode, 1)

    def test_claim8_fixes_its_two_routes_despite_generic_parity_override(self):
        for policy in ('natural', 'adaptive-even'):
            result, calls = self.exercise('claim8_natural_eigenvector.sh', '--preflight-only',
                                         '--parity-policy', policy)
            self.assertEqual(len(calls), 4, result.stderr)
            for call in calls:
                self.assertEqual(call[call.index('--parity-policy')+1], 'even-sector')
            self.assertEqual(['--no-force-even' in call for call in calls], [False, True, False, True])

    def test_every_claim_preserves_its_complete_configuration_grid(self):
        expected = {
            'claim1a_lambda13.sh': [(13,120,1000)], 'claim1b_lambda100.sh': [(100,500,1000)],
            'claim1c_lambda1000.sh': [(1000,800,1000)],
            'claim2a_hp200.sh': [(c,120,200) for c in (13,20,30,50,100)],
            'claim2b_hp1000.sh': [(c,120,1000) for c in (13,20,30,50,100)],
            'claim3_critical_n.sh': [(50,200,1000),(50,250,1000),(100,300,1000),(100,400,1000)],
            'claim4a_lambda13.sh': [(13,120,1000)], 'claim4b_lambda100.sh': [(100,500,1000)],
            'claim4c_lambda1000.sh': [(1000,800,2000),(1000,890,2000)],
            'claim4d_lambda1200.sh': [(1200,970,2000)],
            'claim6_eps_n.sh': [(13,120,1000),(100,500,1000),(1000,800,2000)],
            'claim6b_eps_n_abovefloor.sh': [(500,630,1500),(600,690,1500),(700,740,1500),
                                         (800,790,1500),(1000,890,2000),(1200,970,2000)],
            'claim7_convergence_n.sh': [(13,n,p) for p in (200,1000) for n in (10,20,30,40,50,60,80,100,120)],
            'claim8_natural_eigenvector.sh': [(13,120,1000)]*2+[(100,500,1000)]*2,
        }
        self.assertEqual(set(expected), {p.name for p in (ROOT/'scripts').glob('claim[1-8]*.sh')})
        for script, grid in expected.items():
            result, calls = self.exercise(script, '--preflight-only')
            actual = [tuple(int(call[call.index(arg)+1]) for arg in
                      ('--lambda-sq','--n-modes','--precision-digits')) for call in calls]
            self.assertEqual(actual, grid, script)
            self.assertEqual(result.returncode, 1, script)  # First mock invocation fails.
            self.assertTrue(all(call[call.index('--research-capture')+1]=='ultra' for call in calls))
            self.assertFalse(self.created_logs)
            self.assertFalse(self.created_journals)

    def test_claim1c_selects_the_documented_policy_before_execution(self):
        _, calls = self.exercise('claim1c_lambda1000.sh')
        self.assertEqual(len(calls), 1)
        self.assertEqual(calls[0][calls[0].index('--capture-policy')+1], 'claim1c-hp1000')
        _, calls = self.exercise('claim1b_lambda100.sh')
        self.assertNotIn('--capture-policy', calls[0])

    def test_run_claims_are_certified_by_default_and_evenness_claims_are_not(self):
        _, calls = self.exercise('claim1a_lambda13.sh')
        self.assertEqual(calls[0][calls[0].index('--root-validation')+1], 'certified')
        self.assertEqual(calls[0].count('--capture-sector-gap-certificate'), 1)
        _, calls = self.exercise('claim1a_lambda13.sh', '--capture-sector-gap-certificate')
        self.assertEqual(calls[0].count('--capture-sector-gap-certificate'), 1)
        _, calls = self.exercise('claim1a_lambda13.sh', '--no-certification')
        self.assertEqual(calls[0][calls[0].index('--root-validation')+1], 'off')
        self.assertNotIn('--capture-sector-gap-certificate', calls[0])
        _, calls = self.exercise('claim4a_lambda13.sh')
        self.assertTrue(calls)
        for call in calls:
            self.assertNotIn('--root-validation', call)
            self.assertNotIn('--capture-sector-gap-certificate', call)

    def test_publication_routes_claim1a_to_both_and_other_claims_to_private(self):
        scripts = sorted((ROOT / 'scripts').glob('claim[1-8]*.sh'))
        self.assertEqual(len(scripts), 14)
        for path in scripts:
            script = path.name
            target = 'both' if script == 'claim1a_lambda13.sh' else 'private'
            result, calls = self.exercise(script, '--publish-plan', extra_env={'XC_PUBLISH_TARGET': 'public'})
            self.assertTrue(calls, result.stderr)
            for environment in self.environments:
                self.assertEqual(environment['XC_PUBLISH_TARGET'], target, script)
                self.assertEqual(environment['XC_RUN_PROFILE'], 'author')
                self.assertEqual(environment['XC_PUBLISH_EXECUTE'], 'false')
        self.exercise('claim1b_lambda100.sh')
        for environment in self.environments:
            self.assertEqual(environment['XC_PUBLISH_TARGET'], 'none')
            self.assertEqual(environment['XC_PUBLISH_EXECUTE'], 'false')
            self.assertNotIn('XC_RUN_PROFILE', environment)

    def test_inherited_publication_cannot_bypass_direct_script_policy(self):
        for script in ('claim1a_lambda13.sh', 'claim1b_lambda100.sh'):
            result, calls = self.exercise(script, extra_env={
                'XC_RUN_PROFILE': 'author', 'XC_PUBLISH_TARGET': 'both',
                'XC_PUBLISH_EXECUTE': 'true', 'PUBLICATION': 'none'})
            self.assertTrue(calls, result.stderr)
            for environment in self.environments:
                self.assertEqual(environment['XC_PUBLISH_TARGET'], 'none')
                self.assertEqual(environment['XC_PUBLISH_EXECUTE'], 'false')

    def test_ultra_prepares_each_configured_target_and_honors_explicit_opt_out(self):
        for script in ('claim1a_lambda13.sh', 'claim1b_lambda100.sh', 'claim7_convergence_n.sh'):
            result, calls = self.exercise(script, extra_env={'XC_TARGET_SPEC_FILE': '/private/runtime-target.json'})
            self.assertTrue(calls, result.stderr)
            for environment in self.environments:
                self.assertEqual(environment['XC_RESEARCH_PREPARE_TARGET_REFERENCE'], '1')
        self.exercise('claim1a_lambda13.sh', extra_env={
            'XC_TARGET_SPEC_FILE': '/private/runtime-target.json',
            'XC_RESEARCH_PREPARE_TARGET_REFERENCE': '0'})
        self.assertEqual(self.environments[0]['XC_RESEARCH_PREPARE_TARGET_REFERENCE'], '0')
        self.exercise('claim1a_lambda13.sh')
        self.assertNotIn('XC_RESEARCH_PREPARE_TARGET_REFERENCE', self.environments[0])
        self.exercise('claim1a_lambda13.sh', '--research-capture', 'claim',
                      extra_env={'XC_TARGET_SPEC_FILE': '/private/runtime-target.json'})
        self.assertNotIn('XC_RESEARCH_PREPARE_TARGET_REFERENCE', self.environments[0])

    def test_publish_requires_the_author_commit_email(self):
        result, calls = self.exercise('claim1a_lambda13.sh', '--publish')
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(calls, [])
        self.assertIn('XC_PUBLISH_AUTHOR_EMAIL', result.stderr)
        _, calls = self.exercise('claim1a_lambda13.sh', '--publish',
                                 extra_env={'XC_PUBLISH_AUTHOR_EMAIL': 'author@example.invalid'})
        self.assertTrue(calls)
        self.assertEqual(self.environments[0]['XC_PUBLISH_EXECUTE'], 'true')
        self.assertEqual(self.environments[0]['XC_PUBLISH_TARGET'], 'both')


if __name__ == '__main__':
    unittest.main()
