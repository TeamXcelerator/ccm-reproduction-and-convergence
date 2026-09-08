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
    def exercise(self, script):
        with tempfile.TemporaryDirectory(prefix='paper claim test ') as temporary:
            base = Path(temporary)
            binary = base / 'test binary'
            binary.write_text('''#!/usr/bin/env python3
import json, os, sys
from pathlib import Path
p=Path(os.environ['TEST_CALLS'])
with p.open('a') as f: f.write(json.dumps(sys.argv[1:])+'\\n')
print('intentional test executable: no scientific measurements')
raise SystemExit(1 if len(p.read_text().splitlines())==1 else 0)
''', encoding='utf-8')
            binary.chmod(0o700)
            env = dict(os.environ, BIN=str(binary), CLAIM_LOG_ROOT=str(base/'logs'), TEST_CALLS=str(base/'calls.jsonl'))
            result = subprocess.run(['bash', str(ROOT/'scripts'/script), '--capture-output', str(base/'journals')], cwd=ROOT, env=env, capture_output=True, text=True, timeout=30)
            calls = [json.loads(line) for line in (base/'calls.jsonl').read_text().splitlines()]
            return result, calls

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

    def test_claim1c_selects_the_documented_policy_before_execution(self):
        _, calls = self.exercise('claim1c_lambda1000.sh')
        self.assertEqual(len(calls), 1)
        self.assertEqual(calls[0][calls[0].index('--capture-policy')+1], 'claim1c-hp1000')
        _, calls = self.exercise('claim1b_lambda100.sh')
        self.assertNotIn('--capture-policy', calls[0])


if __name__ == '__main__':
    unittest.main()
