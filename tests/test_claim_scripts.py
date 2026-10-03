"""Exercise claim orchestration with a deliberately failing test executable."""
import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

GRIDS = {
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
    'claim7b_critical_brackets.sh': [(13,n,p) for p in (200,1000) for n in (*range(31,41), *range(51,61))],
    'claim8_natural_eigenvector.sh': [(13,120,1000)]*2+[(100,500,1000)]*2,
}
EVENNESS = {'claim4a_lambda13.sh', 'claim4b_lambda100.sh', 'claim4c_lambda1000.sh', 'claim4d_lambda1200.sh'}
# Declared Toolkit working budgets (GiB): the real-binary admission requirement
# of every configuration plus about 10%. Kept equal to the README memory column.
BUDGETS = {
    'claim1a_lambda13.sh': 16, 'claim1b_lambda100.sh': 24, 'claim1c_lambda1000.sh': 56,
    'claim2a_hp200.sh': 16, 'claim2b_hp1000.sh': 16, 'claim3_critical_n.sh': 16,
    'claim4a_lambda13.sh': 16, 'claim4b_lambda100.sh': 48, 'claim4c_lambda1000.sh': 256,
    'claim4d_lambda1200.sh': 304, 'claim6_eps_n.sh': 88, 'claim6b_eps_n_abovefloor.sh': 128,
    'claim7_convergence_n.sh': 16, 'claim7b_critical_brackets.sh': 16, 'claim8_natural_eigenvector.sh': 48,
}
HOSTILE_RESEARCH_ENVIRONMENT = {
    'XC_RESEARCH_WORKING_BYTES': '1', 'XC_RESEARCH_OUTPUT_BYTES': '0', 'XC_RESEARCH_CHECKPOINT_BYTES': '0',
    'XC_RESEARCH_ROOT_BLOCK_ROWS': '0', 'XC_RESEARCH_PREPARER': 'x', 'XC_RESEARCH_PREPARE_TARGET_REFERENCE': '1',
    'XC_RESEARCH_REFERENCE_FILE': '/nonexistent', 'XC_RESEARCH_INPUTS_FILE': '/nonexistent',
    'XC_RESEARCH_SOMETHING_NEW': 'x', 'XC_TARGET_SPEC_FILE': '/nonexistent', 'XC_TARGET_SPEC_DIR': '/nonexistent',
    'XC_TARGET_PROVIDER_EXECUTABLE': '/nonexistent', 'XC_TARGET_SOMETHING_NEW': 'x',
}


def value(call, name):
    return call[call.index(name)+1]


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
            result = subprocess.run(['bash', str(ROOT/'scripts'/script), '--capture-output', str(base/'journals'), *arguments], cwd=ROOT, env=env, capture_output=True, text=True, timeout=60)
            calls_path = base/'calls.jsonl'
            calls = [json.loads(line) for line in calls_path.read_text().splitlines()] if calls_path.exists() else []
            env_path = base/'calls.jsonl.env'
            self.environments = [json.loads(line) for line in env_path.read_text().splitlines()] if env_path.exists() else []
            self.created_logs = (base/'logs').exists()
            self.created_journals = (base/'journals').exists()
            return result, calls

    def test_every_claim_runs_exactly_its_configuration_grid(self):
        self.assertEqual(set(GRIDS), {p.name for p in (ROOT/'scripts').glob('claim[1-8]*.sh')})
        for script, grid in GRIDS.items():
            result, calls = self.exercise(script, '--preflight-only')
            actual = [tuple(int(value(call, arg)) for arg in ('--lambda-sq','--n-modes','--precision-digits'))
                      for call in calls]
            self.assertEqual(actual, grid, script)
            self.assertEqual(result.returncode, 1, script)  # First mock invocation fails.
            self.assertFalse(self.created_logs)
            self.assertFalse(self.created_journals)

    def test_each_claim_computes_only_what_its_checks_use(self):
        for script in GRIDS:
            _, calls = self.exercise(script, '--preflight-only')
            for call in calls:
                if script in EVENNESS:
                    self.assertIn('check-evenness', call, script)
                    self.assertNotIn('run', call, script)
                    self.assertEqual(value(call, '--research-capture'), 'gap', script)
                    self.assertNotIn('--root-validation', call)
                else:
                    self.assertIn('run', call, script)
                    self.assertNotIn('check-evenness', call, script)
                    self.assertEqual(value(call, '--research-capture'), 'claim', script)
                    self.assertEqual(value(call, '--root-validation'), 'off', script)
                self.assertEqual(value(call, '--root-acquisition'), 'seeded', script)
                for absent in ('--capture-distance', '--capture-sector-gap-certificate', '--capture-policy',
                               '--capture-prime-power-response', '--capture-u-flow-response',
                               '--research-sector-eigenpairs', '--no-force-even'):
                    self.assertNotIn(absent, call, script)

    def test_research_and_override_controls_are_not_accepted(self):
        for arguments in (('--research-capture', 'ultra'), ('--capture-distance',), ('--no-certification',),
                          ('--root-validation', 'certified'), ('--parity-policy', 'natural'),
                          ('--research-inputs-dir', '/tmp'), ('--capture-sector-gap-certificate',)):
            result, calls = self.exercise('claim1a_lambda13.sh', *arguments)
            self.assertEqual(result.returncode, 2, arguments)
            self.assertEqual(calls, [], arguments)
            self.assertIn('Unknown claim-script argument', result.stderr)

    def test_inherited_research_environment_never_reaches_a_claim(self):
        hostile = dict(HOSTILE_RESEARCH_ENVIRONMENT, RESEARCH_CAPTURE_LEVEL='ultra',
                       ROOT_VALIDATION_LEVEL='certified', PREC='50', N='7')
        for script, budget in BUDGETS.items():
            result, calls = self.exercise(script, '--preflight-only', extra_env=hostile)
            self.assertEqual(len(calls), len(GRIDS[script]), result.stderr)
            for call in calls:
                self.assertNotIn('--precision-digits 50', ' '.join(call))
            for environment in self.environments:
                research = {k: v for k, v in environment.items() if k.startswith(('XC_RESEARCH_', 'XC_TARGET_'))}
                self.assertEqual(research, {'XC_RESEARCH_WORKING_BYTES': str(budget << 30)}, script)

    def test_every_claim_declares_its_working_budget(self):
        for script, budget in BUDGETS.items():
            text = (ROOT / 'scripts' / script).read_text(encoding='utf-8')
            self.assertEqual(text.count('claim_working_budget '), 1, script)
            self.assertIn(f'\nclaim_working_budget {budget}\n', text, script)
            rows = [line for line in (ROOT / 'README.md').read_text(encoding='utf-8').splitlines()
                    if line.startswith(f'| `{script}` |')]
            self.assertEqual(len(rows), 1, script)
            self.assertTrue(rows[0].endswith(f'| {budget} GiB |'), rows[0])
        with tempfile.TemporaryDirectory() as temporary:
            script = Path(temporary) / 'scripts' / 'claim_undeclared.sh'
            script.parent.mkdir()
            (script.parent / 'claim_common.sh').write_text((ROOT / 'scripts' / 'claim_common.sh').read_text(encoding='utf-8'))
            script.write_text('set -euo pipefail\nsource "$(dirname -- "${BASH_SOURCE[0]}")/claim_common.sh"\n'
                              'run_claim run --lambda-sq 13\n')
            result = subprocess.run(['bash', str(script), '--preflight-only'], env=dict(os.environ, BIN='/bin/true'),
                                    capture_output=True, text=True, timeout=30)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('did not declare its working budget', result.stderr)

    @unittest.skipUnless(os.environ.get('CLAIM_ADMISSION_BIN'), 'set CLAIM_ADMISSION_BIN to a built ccm-reproduction')
    def test_real_binary_admits_every_configuration_despite_inherited_settings(self):
        # Source admission only (--preflight-only): no matrix is allocated. Each
        # evenness configuration admits both its natural and even-sector routes.
        for script in GRIDS:
            with self.subTest(script=script):
                env = {k: v for k, v in os.environ.items() if not k.startswith('XC_')}
                env.update(HOSTILE_RESEARCH_ENVIRONMENT, BIN=os.environ['CLAIM_ADMISSION_BIN'], XC_CACHE_REMOTE='none')
                result = subprocess.run(['bash', str(ROOT/'scripts'/script), '--preflight-only'], cwd=ROOT, env=env,
                                        capture_output=True, text=True, timeout=600)
                self.assertEqual(result.returncode, 0, result.stdout[-2000:] + result.stderr[-2000:])
                self.assertEqual(result.stdout.count('SOURCE_ADMISSION'), len(GRIDS[script]))
                if script in EVENNESS:
                    self.assertTrue(all('"parity":"natural_and_even_sector"' in line
                                        for line in result.stdout.splitlines() if 'SOURCE_ADMISSION' in line))

    def test_claim1c_records_its_ground_index_outcome_and_no_other_claim_does(self):
        _, calls = self.exercise('claim1c_lambda1000.sh', '--preflight-only')
        self.assertEqual(len(calls), 1)
        self.assertEqual(value(calls[0], '--ground-index'), 'record-outcome')
        for script in set(GRIDS) - {'claim1c_lambda1000.sh'}:
            _, calls = self.exercise(script, '--preflight-only')
            self.assertTrue(all('--ground-index' not in call for call in calls), script)

    def test_claim8_runs_both_parity_routes_at_identical_configurations(self):
        _, calls = self.exercise('claim8_natural_eigenvector.sh', '--preflight-only')
        self.assertEqual([value(call, '--parity-policy') for call in calls],
                         ['even-sector', 'natural', 'even-sector', 'natural'])
        for even, natural in (calls[0:2], calls[2:4]):
            self.assertEqual([a for a in even if a not in ('even-sector', 'natural')],
                             [a for a in natural if a not in ('even-sector', 'natural')])

    def test_failed_case_does_not_skip_later_cases_and_summary_fails(self):
        result, calls = self.exercise('claim2a_hp200.sh')
        self.assertEqual(len(calls), 5)
        self.assertEqual(result.returncode, 1)
        self.assertIn('Claim claim2a_hp200: FAIL', result.stdout)

    def test_preflight_checks_each_case_without_logs_or_publication(self):
        result, calls = self.exercise('claim2a_hp200.sh', '--preflight-only', '--publish', extra_env={
            'XC_PUBLISH_EXECUTE': 'true', 'XC_PUBLISH_TARGET': 'both'})
        self.assertEqual(len(calls), 5)
        self.assertEqual(result.returncode, 1)  # Deliberately rejected first case.
        self.assertTrue(all('--preflight-only' in call for call in calls))
        self.assertFalse(self.created_logs)
        self.assertFalse(self.created_journals)
        self.assertNotIn('claim summary', result.stdout)
        for environment in self.environments:
            self.assertEqual(environment['XC_PUBLISH_TARGET'], 'none')
            self.assertEqual(environment['XC_PUBLISH_EXECUTE'], 'false')

    def test_publication_routes_claim1a_to_both_and_other_claims_to_private(self):
        for script in GRIDS:
            result, calls = self.exercise(script, '--publish-plan', '--preflight-only',
                                          extra_env={'XC_PUBLISH_TARGET': 'public'})
            self.assertTrue(calls, result.stderr)
        for script in ('claim1a_lambda13.sh', 'claim4a_lambda13.sh'):
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

    def test_sweeps_publish_once_after_their_last_configuration(self):
        email = {'XC_PUBLISH_AUTHOR_EMAIL': 'author@example.invalid'}
        for script, count in (('claim7_convergence_n.sh', 18), ('claim7b_critical_brackets.sh', 40),
                              ('claim8_natural_eigenvector.sh', 4)):
            result, calls = self.exercise(script, '--publish', extra_env=email)
            self.assertEqual(len(calls), count, result.stderr)
            self.assertEqual([e['XC_PUBLISH_EXECUTE'] for e in self.environments],
                             ['false'] * (count - 1) + ['true'], script)

    def test_inherited_publication_cannot_bypass_direct_script_policy(self):
        for script in ('claim1a_lambda13.sh', 'claim1b_lambda100.sh'):
            result, calls = self.exercise(script, extra_env={
                'XC_RUN_PROFILE': 'author', 'XC_PUBLISH_TARGET': 'both',
                'XC_PUBLISH_EXECUTE': 'true', 'PUBLICATION': 'none'})
            self.assertTrue(calls, result.stderr)
            for environment in self.environments:
                self.assertEqual(environment['XC_PUBLISH_TARGET'], 'none')
                self.assertEqual(environment['XC_PUBLISH_EXECUTE'], 'false')

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
