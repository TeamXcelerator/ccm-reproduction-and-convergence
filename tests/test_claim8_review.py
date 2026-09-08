"""Retrospective assessment must preserve evidence and reject unrelated failures."""
import copy
import json
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]/'scripts'))
import review_claim8_capture as review


class Claim8ReviewTests(unittest.TestCase):
    def fixture(self, root):
        for c,n,epsilon,first,twentieth in ((13,120,'3.48399e-59','55.76356','25.33862'),
                                           (100,500,'9.56481e-464','460.09452','422.44198')):
            for parity in ('natural','even-sector'):
                folder = root/f'{c}-{parity}'
                folder.mkdir()
                checkpoint = f'prefix_checkpoint_{n+1}'
                names = review.CLAIM8_NATURAL_REQUESTS | {checkpoint, 'prime_power_response','u_flow_response'}
                policy = {'policy':'full', 'excluded_diagnostics':{}, 'requested_diagnostics':sorted(names)}
                request = {'lambda_squared':c, 'n_modes':n, 'precision_bits':3386, 'parity_policy':parity,
                           'capture':{'level':'ultra','prefix_checkpoint_dimensions':[n+1], 'sector_eigenpairs':8}, 'applicability':policy}
                outcomes = {name:{'status':'completed'} for name in names}
                if parity == 'natural':
                    outcomes.update({name:{'status':'blocked','reason':review.RESPONSE_REASON} for name in ('prime_power_response','u_flow_response')})
                    outcomes[checkpoint] = {'status':'missing','reason':review.CHECKPOINT_REASON}
                measurements = {name:{'value':{}} for name,value in outcomes.items() if value['status']=='completed'}
                measurements['retained_reduction']['value'] = {'checks_passed':True}
                measurements['distance_resolution']['value'] = {'refinements':[]}
                measurements['prefix_ladder']['value'] = {'ladder':{'stopped':None}, 'checkpoints':[
                    {'dimension':n+1,'eigenpair_source':None if parity=='natural' else {},
                     'status':'innovation_export_passed_eigenpair_not_supplied' if parity=='natural' else 'export_checks_passed'}]}
                point = {'lambda_squared':c, 'n_modes':n, 'precision_digits':1000, 'parity_policy':parity,
                         'root_acquisition':'seeded', 'epsilon_N':epsilon,
                         'roots':[{'reference_index':k,'status':'converged','matching_digits':twentieth if k==20 else first} for k in range(1,26)]}
                for name,value in {'request.json':request,'status.json':{'applicability':policy,'capture_complete':parity=='even-sector'},
                                   'capture.json':{'receipt':{'outcomes':outcomes},'measurements':measurements},
                                   'claim-measurements.json':point, 'primary.json':{}}.items():
                    (folder/name).write_text(json.dumps(value))

    def test_preserves_historical_records_and_assesses_both_routes(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.fixture(root)
            before = {str(path):path.read_bytes() for path in root.rglob('*.json')}
            result = review.review_run(root)
            self.assertTrue(result['passed'], result)
            self.assertEqual([r['applicable_diagnostics_completed'] for r in result['journals']], [13,10,13,10])
            self.assertEqual(before,{str(path):path.read_bytes() for path in root.rglob('*.json')})

    def test_unexpected_errors_and_invalid_numerics_cannot_be_excused(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.fixture(root)
            path = root/'13-natural'/'capture.json'
            original = json.loads(path.read_text())
            for kind in ('response_failure','prefix_failure','reduction_failure','missing_measurement'):
                data = copy.deepcopy(original)
                if kind == 'response_failure':
                    data['receipt']['outcomes']['u_flow_response']['reason'] = 'some different failure'
                elif kind == 'prefix_failure':
                    data['measurements']['prefix_ladder']['value']['checkpoints'][0]['status'] = 'export_checks_failed'
                elif kind == 'reduction_failure':
                    data['measurements']['retained_reduction']['value']['checks_passed'] = False
                else:
                    del data['measurements']['distance_profile']
                path.write_text(json.dumps(data))
                self.assertFalse(review.review_run(root)['passed'],kind)
            path.write_text(json.dumps(original))
            (root/'13-natural'/'primary.json').unlink()
            self.assertFalse(review.review_run(root)['passed'])


if __name__ == '__main__':
    unittest.main()
