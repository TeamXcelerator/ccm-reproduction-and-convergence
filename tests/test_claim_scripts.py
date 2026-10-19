"""Exercise claim orchestration with a deliberately failing test executable."""
import json
import os
import shutil
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
    'XC_RESEARCH_PREPARATION_CACHE_ROOT': '/nonexistent/preparation-cache',
    'XC_RESEARCH_PREPARATION_CACHE_BYTES': '1',
    'XC_RESEARCH_PREPARATION_CACHE_ITEM_BYTES': '1',
}


def value(call, name):
    return call[call.index(name)+1]


@unittest.skipIf(os.name == 'nt', 'claim shell integration runs on Linux/WSL')
class ClaimScriptTests(unittest.TestCase):
    def exercise(self, script, *arguments, extra_env=None, target_specs=None,
                 precreated_staging=None, abrupt_exit=None):
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
arguments=sys.argv[1:]
if arguments and arguments[0]=='publish-staged':
    staging=Path(arguments[arguments.index('--staging-root')+1])
    entries=sorted(str(path.relative_to(staging)) for path in (staging/'drafts').glob('*/draft.json'))
    Path(staging/'publication-inventory.json').write_text(json.dumps({'fixture':True,'entries':entries,'target':os.environ['XC_PUBLISH_TARGET']}))
    print('fixture publication finalized: '+str(len(entries))+' synthetic entries',flush=True)
    raise SystemExit(int(os.environ.get('TEST_PUBLISH_STATUS','0')))
if '--defer-publication' in arguments:
    staging=Path(os.environ['XC_PUBLISH_STAGING_ROOT'])
    number=len(p.read_text().splitlines())
    counts=json.loads(os.environ.get('TEST_DRAFT_COUNTS','[]'))
    count=counts[number-1] if counts else 1
    for index in range(count):
        draft=staging/'drafts'/(str(number)+'-'+str(index))/'draft.json'
        draft.parent.mkdir(parents=True,exist_ok=True)
        draft.write_text(json.dumps({'fixture':True,'configuration':number,'index':index}))
print('intentional test executable: no scientific measurements')
raise SystemExit(1 if len(p.read_text().splitlines())==1 else 0)
''', encoding='utf-8')
            binary.chmod(0o700)
            env = {k:v for k,v in os.environ.items() if not k.startswith('XC_')}
            env.update(BIN=str(binary), CLAIM_LOG_ROOT=str(base/'logs'), TEST_CALLS=str(base/'calls.jsonl'), **(extra_env or {}))
            if precreated_staging is not None:
                staging=base/'precreated staging'
                staging.mkdir()
                (staging/'vast-job.identity').write_text('fixture identity')
                (staging/'drafts/empty').mkdir(parents=True)
                if precreated_staging == 'retained':
                    (staging/'drafts/empty/draft.json').write_text('{"fixture":"retained"}')
                if precreated_staging == 'busy':
                    (staging/'.claim-publication-active').mkdir()
                env['XC_PUBLISH_STAGING_ROOT']=str(staging)
            selected_script=ROOT/'scripts'/script
            if abrupt_exit is not None:
                selected_script=base/script
                selected_script.write_text('set -euo pipefail\nsource "'+str(ROOT/'scripts/claim_common.sh')+'"\n'
                    'claim_working_budget 16\nrun_claim run --lambda-sq 13 --n-modes 120 --precision-digits 1000 --top 1\n'
                    + ('exit '+str(abrupt_exit)+'\n' if isinstance(abrupt_exit,int) else 'kill -TERM $$\n'))
            if target_specs is not None:
                directory = base / 'target specs'
                directory.mkdir()
                for name in target_specs:
                    (directory / name).write_text('{}', encoding='utf-8')
                env['XC_TARGET_SPEC_DIR'] = str(directory)
            result = subprocess.run(['bash', str(selected_script), '--capture-output', str(base/'journals'), *arguments], cwd=ROOT, env=env, capture_output=True, text=True, timeout=60)
            calls_path = base/'calls.jsonl'
            calls = [json.loads(line) for line in calls_path.read_text().splitlines()] if calls_path.exists() else []
            env_path = base/'calls.jsonl.env'
            self.environments = [json.loads(line) for line in env_path.read_text().splitlines()] if env_path.exists() else []
            self.created_logs = (base/'logs').exists()
            self.created_journals = (base/'journals').exists()
            self.publication_states = [json.loads(path.read_text()) for path in (base/'logs').glob('*.status.json')]
            self.publication_inventory = None
            self.staging_payloads = {}
            if self.environments and self.environments[-1].get('XC_PUBLISH_STAGING_ROOT'):
                staging=Path(self.environments[-1]['XC_PUBLISH_STAGING_ROOT'])
                inventory=staging/'publication-inventory.json'
                if inventory.is_file(): self.publication_inventory=json.loads(inventory.read_text())
                self.staging_payloads={str(path.relative_to(staging)):path.read_bytes() for path in staging.rglob('*') if path.is_file()}
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

    def test_numerical_override_controls_are_not_accepted(self):
        for arguments in (('--capture-distance',), ('--no-certification',),
                          ('--parity-policy', 'natural'), ('--research-inputs-dir', '/tmp')):
            result, calls = self.exercise('claim1a_lambda13.sh', *arguments)
            self.assertEqual(result.returncode, 2, arguments)
            self.assertEqual(calls, [], arguments)
            self.assertIn('Unknown claim-script argument', result.stderr)

    def test_ultra_preserves_inputs_and_requests_all_selected_single_run_artifacts(self):
        inputs = {
            'XC_TARGET_SPEC_FILE': '/prepared/target.json',
            'XC_TARGET_PROVIDER_EXECUTABLE': '/prepared/provider',
            'XC_TARGET_PROVIDER_PURE': '1',
            'XC_RESEARCH_PREPARER': '{"argv":["/prepared/python","/prepared/preparer.py"]}',
            'XC_RESEARCH_PREPARE_TARGET_REFERENCE': '1',
            'XC_RESEARCH_INPUTS_FILE': '/prepared/inputs.json',
            'XC_RESEARCH_REFERENCE_FILE': '/prepared/reference.json',
            'XC_RESEARCH_OUTPUT_BYTES': '104857600',
            'XC_RESEARCH_PREPARATION_CACHE_ROOT': '/prepared/cache',
            'XC_RESEARCH_PREPARATION_CACHE_BYTES': '8589934592',
            'XC_RESEARCH_PREPARATION_CACHE_ITEM_BYTES': '268435456',
            'XC_PUBLICATION_FAMILY_WORKERS': '1',
            'XC_TARGET_PROVIDER_WORKERS': '1',
            'XC_CLAIM_MEMORY_GIB': '96',
        }
        result, calls = self.exercise('claim1a_lambda13.sh', '--preflight-only',
            '--research-sector-eigenpairs', '8', '--root-validation', 'certified',
            '--capture-sector-gap-certificate', '--research-capture', 'ultra', extra_env=inputs)
        self.assertEqual(result.returncode, 1)  # Deliberately failing mock.
        self.assertEqual(len(calls), 1)
        call = calls[0]
        self.assertEqual(value(call, '--research-capture'), 'ultra')
        self.assertEqual(value(call, '--research-sector-eigenpairs'), '8')
        self.assertEqual(value(call, '--root-validation'), 'certified')
        self.assertEqual(value(call, '--root-acquisition'), 'seeded')
        self.assertIn('--capture-sector-gap-certificate', call)
        for name, expected in inputs.items():
            self.assertEqual(self.environments[0][name], expected)
        self.assertEqual(self.environments[0]['XC_RESEARCH_WORKING_BYTES'], str(96 << 30))
        self.assertFalse(self.created_logs)
        self.assertFalse(self.created_journals)

    def test_publication_worker_control_preserves_the_claim_configuration(self):
        for ultra in (False, True):
            arguments = ('--research-capture', 'ultra') if ultra else ()
            result, calls = self.exercise('claim2a_hp200.sh', '--preflight-only', *arguments,
                                          extra_env={'XC_PUBLICATION_FAMILY_WORKERS': '1'})
            self.assertEqual(result.returncode, 1)
            self.assertEqual([(int(value(call, '--lambda-sq')), int(value(call, '--n-modes')),
                               int(value(call, '--precision-digits'))) for call in calls],
                             GRIDS['claim2a_hp200.sh'])
            self.assertTrue(all(e['XC_PUBLICATION_FAMILY_WORKERS'] == '1' for e in self.environments))

    def test_preparation_worker_control_preserves_both_claim_modes_and_grid(self):
        name = 'XC_RESEARCH_PREP_WORKERS'
        for ultra in (False, True):
            arguments = ('--research-capture', 'ultra') if ultra else ()
            for setting, expected in (('1', '1'), ('0016', '16'), ('18446744073709551615', '18446744073709551615')):
                result, calls = self.exercise('claim2a_hp200.sh', '--preflight-only', *arguments,
                                              extra_env={name: setting})
                self.assertEqual(result.returncode, 1)
                self.assertEqual([(int(value(call, '--lambda-sq')), int(value(call, '--n-modes')),
                                   int(value(call, '--precision-digits'))) for call in calls], GRIDS['claim2a_hp200.sh'])
                self.assertTrue(all(environment[name] == expected for environment in self.environments))
            for invalid in ('', '0', '-1', '+1', '1.5', '1 ', '\u0661', '18446744073709551616'):
                result, calls = self.exercise('claim2a_hp200.sh', '--preflight-only', *arguments,
                                              extra_env={name: invalid})
                self.assertEqual(result.returncode, 2, (ultra, invalid, result.stderr))
                self.assertEqual(calls, [])
                self.assertIn(name, result.stderr)

    def test_transform_guard_control_preserves_modes_and_rejects_invalid_before_children(self):
        name = 'XC_TRANSFORM_MAX_GUARD_BITS'
        for ultra in (False, True):
            arguments = ('--research-capture', 'ultra') if ultra else ()
            for setting in ('4096', '8192', '1048576'):
                result, calls = self.exercise('claim2a_hp200.sh', '--preflight-only', *arguments,
                                              extra_env={name: setting})
                self.assertEqual(result.returncode, 1)
                self.assertEqual([(int(value(call, '--lambda-sq')), int(value(call, '--n-modes')),
                                   int(value(call, '--precision-digits'))) for call in calls], GRIDS['claim2a_hp200.sh'])
                self.assertTrue(all(environment[name] == setting for environment in self.environments))
            for invalid in ('', '0', '4095', '4097', '2097152', '04096', '+4096', '-4096',
                            '4096 ', '4096\n', '\u0664\u0660\u0669\u0666', '1<<13'):
                result, calls = self.exercise('claim2a_hp200.sh', '--preflight-only', *arguments,
                                              extra_env={name: invalid})
                self.assertEqual(result.returncode, 2, (ultra, invalid, result.stderr))
                self.assertEqual(calls, [])
                self.assertFalse(self.created_logs)
                self.assertFalse(self.created_journals)
                self.assertIn(name, result.stderr)

    def test_ultra_does_not_request_unselected_proofs(self):
        _, calls = self.exercise('claim1a_lambda13.sh', '--preflight-only', '--research-capture', 'ultra')
        self.assertEqual(value(calls[0], '--research-capture'), 'ultra')
        self.assertEqual(value(calls[0], '--research-sector-eigenpairs'), '8')
        self.assertEqual(value(calls[0], '--root-validation'), 'off')
        self.assertNotIn('--capture-sector-gap-certificate', calls[0])

    def test_ultra_selects_tailored_policy_for_underresolved_and_natural_claims(self):
        _, calls = self.exercise('claim1c_lambda1000.sh', '--research-capture', 'ultra', '--preflight-only',
                                '--root-validation', 'certified')
        self.assertEqual(len(calls), 1)
        self.assertEqual(value(calls[0], '--capture-policy'), 'claim1c-hp1000')
        self.assertEqual(value(calls[0], '--ground-index'), 'record-outcome')
        self.assertEqual(value(calls[0], '--root-validation'), 'certified')
        _, calls = self.exercise('claim8_natural_eigenvector.sh', '--research-capture', 'ultra', '--preflight-only',
                                '--root-validation', 'certified')
        self.assertEqual([value(call, '--capture-policy') for call in calls],
                         ['full', 'claim8-natural', 'full', 'claim8-natural'])
        self.assertEqual([value(call, '--parity-policy') for call in calls],
                         ['even-sector', 'natural', 'even-sector', 'natural'])

    def test_every_claim_ultra_keeps_its_grid_and_certificate_default(self):
        for script, grid in GRIDS.items():
            options = [] if script in EVENNESS else ['--root-validation', 'certified']
            result, calls = self.exercise(script, '--preflight-only', '--research-capture', 'ultra',
                                          '--research-sector-eigenpairs', '8', *options)
            actual = [tuple(int(value(call, arg)) for arg in ('--lambda-sq', '--n-modes', '--precision-digits'))
                      for call in calls]
            self.assertEqual(actual, grid, script)
            self.assertEqual(result.returncode, 1, script)  # Deliberately failing first mock invocation.
            for call in calls:
                self.assertEqual(value(call, '--research-capture'), 'ultra', script)
                self.assertEqual(value(call, '--research-sector-eigenpairs'), '8', script)
                self.assertNotIn('--capture-sector-gap-certificate', call, script)
                if script in EVENNESS:
                    self.assertNotIn('--root-validation', call, script)
                else:
                    self.assertEqual(value(call, '--root-validation'), 'certified', script)
                if script not in ('claim1c_lambda1000.sh', 'claim8_natural_eigenvector.sh'):
                    self.assertEqual(value(call, '--capture-policy'), 'full', script)
            self.assertFalse(self.created_logs)
            self.assertFalse(self.created_journals)

    def test_ultra_eigenvalue_claims_keep_their_existing_one_root_window(self):
        for script in ('claim6_eps_n.sh', 'claim6b_eps_n_abovefloor.sh'):
            _, calls = self.exercise(script, '--preflight-only', '--research-capture', 'ultra',
                                      '--root-validation', 'certified')
            self.assertTrue(calls)
            self.assertTrue(all(value(call, '--top') == '1' for call in calls))
            self.assertTrue(all(value(call, '--root-validation') == 'certified' for call in calls))

    def test_ultra_rejects_a_certificate_excluded_by_the_frozen_policy(self):
        result, calls = self.exercise('claim1c_lambda1000.sh', '--research-capture', 'ultra',
                                      '--capture-sector-gap-certificate', '--preflight-only')
        self.assertEqual(result.returncode, 2)
        self.assertEqual(calls, [])
        self.assertIn('applicability policy excludes the sector-gap certificate', result.stderr)

    def test_ultra_evenness_forwards_supported_flags(self):
        _, calls = self.exercise('claim4a_lambda13.sh', '--preflight-only', '--research-capture', 'ultra',
            '--research-sector-eigenpairs', '12')
        self.assertEqual(value(calls[0], '--research-capture'), 'ultra')
        self.assertEqual(value(calls[0], '--research-sector-eigenpairs'), '12')
        self.assertNotIn('--root-validation', calls[0])
        self.assertNotIn('--capture-sector-gap-certificate', calls[0])

    def test_ultra_evenness_rejects_unsupported_certificate_switches(self):
        for script in EVENNESS:
            for arguments in (('--root-validation', 'off'), ('--root-validation', 'certified'),
                              ('--capture-sector-gap-certificate',)):
                result, calls = self.exercise(script, '--preflight-only', '--research-capture', 'ultra', *arguments)
                self.assertEqual(result.returncode, 2)
                self.assertEqual(calls, [])
                self.assertIn('not supported by check-evenness', result.stderr)

    def test_ultra_self_build_enables_transform_and_parallel_features(self):
        with tempfile.TemporaryDirectory(prefix='paper ultra build test ') as temporary:
            base = Path(temporary)
            scripts = base / 'paper' / 'scripts'
            scripts.mkdir(parents=True)
            for name in ('claim_common.sh', 'claim1a_lambda13.sh'):
                shutil.copyfile(ROOT / 'scripts' / name, scripts / name)
            commands = base / 'commands'
            commands.mkdir()
            cargo = commands / 'cargo'
            cargo.write_text('''#!/usr/bin/env python3
import json, os, sys
from pathlib import Path
Path(os.environ['TEST_CARGO_ARGS']).write_text(json.dumps(sys.argv[1:]))
binary = Path(sys.argv[sys.argv.index('--target-dir') + 1]) / 'release' / 'ccm-reproduction'
binary.parent.mkdir(parents=True)
binary.write_text('#!/usr/bin/env bash\\nexit 0\\n')
binary.chmod(0o700)
''', encoding='utf-8')
            cargo.chmod(0o700)
            env = {k: v for k, v in os.environ.items() if not k.startswith('XC_') and k != 'BIN'}
            env.update(PATH=str(commands) + os.pathsep + env['PATH'],
                       TEST_CARGO_ARGS=str(base / 'cargo.json'))
            result = subprocess.run(['bash', str(scripts / 'claim1a_lambda13.sh'), '--preflight-only',
                '--research-capture', 'ultra', '--parallel-gl-roots'],
                env=env, capture_output=True, text=True, timeout=30)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            arguments = json.loads((base / 'cargo.json').read_text())
            self.assertEqual(value(arguments, '--features'), 'hp,root-certification,experimental-gl-root-parallel')
            self.assertIn('--locked', arguments)

    def test_research_controls_require_explicit_ultra_and_valid_values(self):
        for arguments in (('--research-sector-eigenpairs', '8'), ('--root-validation', 'off'),
                          ('--root-validation', 'certified'), ('--capture-sector-gap-certificate',)):
            result, calls = self.exercise('claim1a_lambda13.sh', *arguments)
            self.assertEqual(result.returncode, 2)
            self.assertEqual(calls, [])
            self.assertIn('require --research-capture ultra', result.stderr)
        for arguments in (('--research-capture',), ('--research-capture', 'maximum'),
                          ('--research-sector-eigenpairs',), ('--research-sector-eigenpairs', '0'),
                          ('--research-sector-eigenpairs', '1+1'), ('--root-validation',),
                          ('--root-validation', 'numerical')):
            result, calls = self.exercise('claim1a_lambda13.sh', '--research-capture', 'ultra', *arguments)
            self.assertEqual(result.returncode, 2, arguments)
            self.assertEqual(calls, [], arguments)

    def test_ultra_sweep_selects_each_configurations_prepared_target(self):
        names = [f'c{c}-d200.json' for c in (13, 20, 30, 50, 100)]
        result, calls = self.exercise('claim2a_hp200.sh', '--preflight-only', '--research-capture', 'ultra',
                                      target_specs=names)
        self.assertEqual(len(calls), 5, result.stderr)
        self.assertEqual([Path(e['XC_TARGET_SPEC_FILE']).name for e in self.environments], names)
        self.assertTrue(all(value(call, '--research-capture') == 'ultra' for call in calls))

    def test_missing_ultra_target_refuses_that_configuration_and_continues(self):
        names = [f'c{c}-d200.json' for c in (13, 30, 50, 100)]
        result, calls = self.exercise('claim2a_hp200.sh', '--preflight-only', '--research-capture', 'ultra',
                                      target_specs=names)
        self.assertEqual(result.returncode, 1)
        self.assertEqual([int(value(call, '--lambda-sq')) for call in calls], [13, 30, 50, 100])
        self.assertIn('Prepared target specification is missing:', result.stderr)
        self.assertIn('c20-d200.json', result.stderr)

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

    def test_explicit_claim_memory_override_survives_sanitization_and_preflight(self):
        hostile = dict(HOSTILE_RESEARCH_ENVIRONMENT, XC_CLAIM_MEMORY_GIB='96')
        result, calls = self.exercise('claim1b_lambda100.sh', '--preflight-only', extra_env=hostile)
        self.assertEqual(result.returncode, 1)  # Deliberately failing mock.
        self.assertEqual(len(calls), 1)
        self.assertIn('--preflight-only', calls[0])
        self.assertIn('Claim working budget: 96 GiB (103079215104 bytes)', result.stderr)
        for environment in self.environments:
            research = {k: v for k, v in environment.items() if k.startswith(('XC_RESEARCH_', 'XC_TARGET_'))}
            self.assertEqual(research, {'XC_RESEARCH_WORKING_BYTES': str(96 << 30)})
        self.assertFalse(self.created_logs)
        self.assertFalse(self.created_journals)

    def test_claim_memory_override_rejects_malformed_or_overflow_before_invocation(self):
        for invalid in ('', '0', '-1', '01', '1.5', '96 GiB', '1+1', '8589934592', '18446744073709551615'):
            with self.subTest(value=invalid):
                result, calls = self.exercise('claim1b_lambda100.sh', '--preflight-only',
                                              extra_env={'XC_CLAIM_MEMORY_GIB': invalid})
                self.assertEqual(result.returncode, 2)
                self.assertEqual(calls, [])
                self.assertIn('XC_CLAIM_MEMORY_GIB must be an integer', result.stderr)

    def test_claim_memory_override_signed_arithmetic_boundary(self):
        result, calls = self.exercise('claim1b_lambda100.sh', '--preflight-only',
                                      extra_env={'XC_CLAIM_MEMORY_GIB': '8589934591'})
        self.assertEqual(result.returncode, 1)
        self.assertEqual(len(calls), 1)
        self.assertEqual(self.environments[0]['XC_RESEARCH_WORKING_BYTES'], str(8589934591 << 30))

    @unittest.skipUnless(os.environ.get('CLAIM_ADMISSION_BIN'), 'set CLAIM_ADMISSION_BIN to a built ccm-reproduction')
    def test_real_binary_rejects_invalid_explicit_resource_policy_before_work(self):
        # Reach each managed-session constructor in a real child process. Empty
        # local caches plus require_reuse prevent numerical fallback even if a
        # policy-loading regression were to let a command advance too far.
        policy = {
            'profile': 'normal_workstation',
            'maximum_memory_bytes': 1,
            'maximum_temporary_disk_bytes': 1,
            'maximum_permanent_disk_bytes': 1,
            'maximum_transfer_bytes': 1,
            'maximum_cpu_seconds': 1,
            'maximum_wall_seconds': 1,
            'maximum_threads': 1,
            'allow_out_of_core': False,
            'allow_distributed': False,
            'checkpoint_interval_seconds': 1,
        }
        invalid_policies = (
            (dict(policy, fixture_unknown_policy_field=True), 'unknown field'),
            (dict(policy, maximum_memory_bytes=0), 'resource policy memory limit must be positive'),
        )
        for route in ('publish', 'deferred_capture', 'deferred_evenness', 'finite_replay', 'recovery'):
            for document, expected in invalid_policies:
                with self.subTest(route=route, expected=expected), tempfile.TemporaryDirectory() as temporary:
                    base = Path(temporary)
                    staging = base / 'staging'
                    staging.mkdir()
                    sentinel = staging / 'retained-owner-note'
                    sentinel.write_bytes(b'unchanged fixture evidence')
                    journal = base / 'original'
                    journal.mkdir()
                    policy_path = base / 'policy.json'
                    policy_path.write_text(json.dumps(document), encoding='utf-8')
                    env = {k: v for k, v in os.environ.items() if not k.startswith('XC_')}
                    env.update(XC_CACHE_ROOT=str(base / 'cache'), XC_CACHE_REMOTE='none',
                               XC_CACHE_MODE='require_reuse', XC_RUN_PROFILE='author',
                               XC_PUBLISH_STAGING_ROOT=str(staging), XC_PUBLISH_TARGET='private',
                               XC_PUBLISH_EXECUTE='true', XC_PUBLISH_AUTHOR_EMAIL='fixture@example.invalid',
                               XC_RESOURCE_POLICY_FILE=str(policy_path),
                               XC_RESEARCH_PREPARER='{"argv":["/unreachable-resource-policy-fixture"]}')
                    if route == 'publish':
                        arguments = ['publish-staged', '--staging-root', str(staging)]
                    elif route == 'deferred_capture':
                        arguments = ['--defer-publication', '--capture-output', str(base / 'captures'),
                                     'run', '--lambda-sq', '13', '--n-modes', '2', '--top', '1',
                                     '--precision-digits', '20', '--root-validation', 'off',
                                     '--research-capture', 'claim']
                    elif route == 'deferred_evenness':
                        arguments = ['--defer-publication', '--capture-output', str(base / 'captures'),
                                     'check-evenness', '--lambda-sq', '13', '--n-modes', '10',
                                     '--precision-digits', '64', '--research-capture', 'claim']
                    else:
                        env.update(XC_PUBLISH_TARGET='none', XC_PUBLISH_EXECUTE='false')
                        command = 'replay-finite' if route == 'finite_replay' else 'recover-capture'
                        arguments = [command, '--journal', str(journal), '--output', str(base / 'output')]
                    result = subprocess.run([os.environ['CLAIM_ADMISSION_BIN'], *arguments], cwd=ROOT,
                                            env=env, capture_output=True, text=True, timeout=30)
                    self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
                    self.assertIn(expected, result.stderr)
                    self.assertEqual(sentinel.read_bytes(), b'unchanged fixture evidence')
                    self.assertEqual(list(journal.iterdir()), [])
                    self.assertFalse((base / 'output').exists())
                    self.assertFalse(list(staging.rglob('publication-inventory.json')))
                    self.assertFalse(list(base.rglob('claim-measurements.json')))
                    for events in base.rglob('events.jsonl'):
                        self.assertEqual(events.read_bytes(), b'')

    @unittest.skipUnless(os.environ.get('CLAIM_ADMISSION_BIN'), 'set CLAIM_ADMISSION_BIN to a built ccm-reproduction')
    def test_real_evenness_defers_publication_and_preserves_cold_warm_measurements(self):
        # A real small numerical run must save its result with the remote
        # publisher unavailable. The wrapper prevents all Git network access;
        # an ordinary (nondeferred) warm run must still try to finalize and fail.
        with tempfile.TemporaryDirectory(prefix='deferred evenness ') as temporary:
            base = Path(temporary)
            commands = base / 'commands'
            commands.mkdir()
            git = commands / 'git'
            git.write_text('#!/bin/sh\nprintf "called\\n" >> "$TEST_GIT_CALLS"\n'
                           'echo "fixture publisher unavailable" >&2\nexit 97\n', encoding='utf-8')
            git.chmod(0o700)
            calls = base / 'git-calls'
            staging = base / 'staging'
            env = {k: v for k, v in os.environ.items()
                   if not k.startswith('XC_') and k not in ('GH_TOKEN', 'GITHUB_TOKEN')}
            env.update(PATH=str(commands) + os.pathsep + env.get('PATH', ''),
                       TEST_GIT_CALLS=str(calls), RAYON_NUM_THREADS='1',
                       XC_CACHE_ROOT=str(base / 'cache'), XC_CACHE_REMOTE='none',
                       XC_RUN_PROFILE='author', XC_PUBLISH_STAGING_ROOT=str(staging),
                       XC_PUBLISH_TARGET='private', XC_PUBLISH_EXECUTE='true',
                       XC_PUBLISH_AUTHOR_EMAIL='fixture@example.invalid',
                       XC_DENSE_BACKEND='cpu', XC_RESEARCH_WORKING_BYTES=str(1 << 30))
            common = ['check-evenness', '--lambda-sq', '13', '--n-modes', '10',
                      '--precision-digits', '64', '--research-capture', 'claim']
            measurements = []
            for name, workers in (('cold', '1'), ('warm', '2')):
                output = base / name
                env['RAYON_NUM_THREADS'] = workers
                result = subprocess.run(
                    [os.environ['CLAIM_ADMISSION_BIN'], '--defer-publication',
                     '--capture-output', str(output), *common],
                    cwd=ROOT, env=env, capture_output=True, text=True, timeout=120)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                retained = list(output.rglob('claim-measurements.json'))
                self.assertEqual(len(retained), 1)
                measurements.append(retained[0].read_bytes())
                self.assertFalse(calls.exists(), 'deferred measurement attempted publication')
                self.assertTrue(list((staging / 'drafts').rglob('draft.json')))
                self.assertFalse((staging / 'publication-execution-report.json').exists())
            self.assertEqual(measurements[0], measurements[1])
            result = subprocess.run(
                [os.environ['CLAIM_ADMISSION_BIN'], '--capture-output', str(base / 'ordinary'), *common],
                cwd=ROOT, env=env, capture_output=True, text=True, timeout=120)
            self.assertNotEqual(result.returncode, 0)
            self.assertTrue(calls.exists(), result.stdout + result.stderr)

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

    def test_ultra_keeps_every_claims_publication_destination(self):
        for script in GRIDS:
            _, calls = self.exercise(script, '--publish-plan', '--research-capture', 'ultra',
                                      extra_env={'XC_PUBLISH_TARGET': 'public'})
            self.assertEqual(len(calls), len(GRIDS[script]), script)
            target = 'both' if script == 'claim1a_lambda13.sh' else 'private'
            self.assertTrue(all(e['XC_PUBLISH_TARGET'] == target for e in self.environments), script)
            self.assertTrue(all(e['XC_PUBLISH_EXECUTE'] == 'false' for e in self.environments), script)

    def test_sweeps_publish_once_after_their_last_configuration(self):
        email = {'XC_PUBLISH_AUTHOR_EMAIL': 'author@example.invalid'}
        for script, count in (('claim7_convergence_n.sh', 18), ('claim7b_critical_brackets.sh', 40),
                              ('claim8_natural_eigenvector.sh', 4)):
            result, calls = self.exercise(script, '--publish', extra_env=email)
            self.assertEqual(len(calls), count + 1, result.stderr)
            self.assertTrue(all('--defer-publication' in call for call in calls[:-1]))
            self.assertEqual(calls[-1][0], 'publish-staged')
            self.assertTrue(all(e['XC_PUBLISH_EXECUTE']=='true' for e in self.environments))

    def test_ultra_sweeps_publish_once_after_their_last_configuration(self):
        email = {'XC_PUBLISH_AUTHOR_EMAIL': 'author@example.invalid'}
        for script, count in (('claim7_convergence_n.sh', 18), ('claim7b_critical_brackets.sh', 40),
                              ('claim8_natural_eigenvector.sh', 4)):
            result, calls = self.exercise(script, '--publish', '--research-capture', 'ultra', extra_env=email)
            self.assertEqual(len(calls), count + 1, result.stderr)
            self.assertTrue(all('--defer-publication' in call for call in calls[:-1]))
            self.assertEqual(calls[-1][0], 'publish-staged')
            self.assertTrue(all(e['XC_PUBLISH_EXECUTE']=='true' for e in self.environments))

    def test_deferred_finalization_covers_all_staging_and_preserves_numerical_failure(self):
        result, calls = self.exercise('claim2b_hp1000.sh', '--publish', extra_env={
            'XC_PUBLISH_AUTHOR_EMAIL':'author@example.invalid','TEST_DRAFT_COUNTS':'[77,76,76,76,76]'})
        self.assertEqual(len(calls), 6)
        self.assertEqual(calls[-1][0], 'publish-staged')
        self.assertTrue(all('--defer-publication' in call for call in calls[:-1]))
        self.assertEqual(len(self.publication_inventory['entries']), 381)
        self.assertEqual(self.publication_inventory['target'], 'private')
        self.assertEqual(self.publication_states[0]['status'], 'completed')
        self.assertEqual(self.publication_states[0]['invocations'], 5)
        self.assertEqual(result.returncode, 1)  # Original first numerical failure survives.
        self.assertLess(result.stdout.index('fixture publication finalized'), result.stdout.index('Claim claim2b_hp1000: FAIL'))

    def test_deferred_failure_retains_queue_and_never_prints_completed_claim(self):
        result, calls = self.exercise('claim2b_hp1000.sh', '--publish', extra_env={
            'XC_PUBLISH_AUTHOR_EMAIL':'author@example.invalid','TEST_PUBLISH_STATUS':'23'})
        self.assertEqual(len(calls), 6)
        self.assertEqual(result.returncode, 1)
        self.assertEqual(self.publication_states[0]['status'], 'failed')
        self.assertEqual(len([name for name in self.staging_payloads if name.endswith('draft.json')]), 5)
        self.assertNotIn('Claim claim2b_hp1000: PASS', result.stdout)
        self.assertIn('Publication remains pending or failed', result.stderr)

    def test_interruption_and_unexpected_nonzero_exit_never_start_publication(self):
        for ending in (7, 'signal'):
            with self.subTest(ending=ending):
                result, calls = self.exercise('claim2b_hp1000.sh', '--publish', abrupt_exit=ending,
                    extra_env={'XC_PUBLISH_AUTHOR_EMAIL':'author@example.invalid'})
                self.assertEqual(len(calls), 1)
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(self.publication_states[0]['status'], 'pending')
                self.assertNotIn('Claim claim2b_hp1000: PASS', result.stdout)
                if ending == 7: self.assertEqual(result.returncode, 7)

    def test_deferred_staging_accepts_launcher_scaffolding_but_refuses_existing_drafts_or_owner(self):
        email={'XC_PUBLISH_AUTHOR_EMAIL':'author@example.invalid'}
        result, calls = self.exercise('claim1a_lambda13.sh', '--publish', extra_env=email, precreated_staging='empty')
        self.assertEqual(len(calls), 2, result.stderr)
        self.assertEqual(self.publication_inventory['target'], 'both')
        self.assertEqual(self.staging_payloads['vast-job.identity'], b'fixture identity')
        for kind in ('retained', 'busy'):
            with self.subTest(kind=kind):
                result, calls = self.exercise('claim1a_lambda13.sh', '--publish', extra_env=email, precreated_staging=kind)
                self.assertEqual(calls, [])
                self.assertNotEqual(result.returncode, 0)

    def test_per_configuration_compatibility_and_plan_modes_do_not_use_deferred_finalizer(self):
        for mode in (('--publish','--publication-per-configuration'), ('--publish-plan',)):
            with self.subTest(mode=mode):
                _, calls = self.exercise('claim2b_hp1000.sh', *mode,
                    extra_env={'XC_PUBLISH_AUTHOR_EMAIL':'author@example.invalid'})
                self.assertEqual(len(calls), 5)
                self.assertTrue(all('publish-staged' not in call and '--defer-publication' not in call for call in calls))
                self.assertEqual(self.publication_states, [])

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
