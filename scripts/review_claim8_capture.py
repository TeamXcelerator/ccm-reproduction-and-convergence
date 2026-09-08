#!/usr/bin/env python3
"""Reassess historical Claim 8 journals without computing or rewriting evidence."""
import argparse
import hashlib
import json
from decimal import localcontext
from pathlib import Path

from claim_summary import (CLAIM8_NATURAL_REQUESTS, assess_point, assess_series,
                           claim8_natural_checkpoint, numerical_review)


RESPONSE_REASON = 'requires an isolated even-sector state; primary parity preserved'
CHECKPOINT_REASON = 'checkpoint eigenstate unavailable; partial export retained in prefix ladder'


def review_run(root):
    report = {'schema_version':1, 'assessment':'historical-claim8-applicability-review',
              'original_capture_status':'unchanged', 'original_journals_modified':False,
              'new_capture_receipt_created':False, 'publication_reverified':False,
              'scope':'Local retained journal review; not a rerun, artifact repair, interval certificate, or new publication.',
              'passed':False, 'checks':[], 'journals':[]}
    points = []
    directories = sorted(path for path in root.iterdir() if path.is_dir())
    report['checks'].append({'description':'exactly four retained claim journals', 'passed':len(directories)==4})
    for directory in directories:
        row = {'directory':str(directory), 'source_sha256':{}, 'passed':False, 'exclusions_on_review':{}}
        report['journals'].append(row)
        def read(name):
            data = (directory/name).read_bytes()
            row['source_sha256'][name] = hashlib.sha256(data).hexdigest()
            return json.loads(data)
        try:
            request, state, record, point = [read(name) for name in
                ('request.json', 'status.json', 'capture.json', 'claim-measurements.json')]
            read('primary.json')
            # Do not merge explicit additional requests into this frozen review.
            if (directory/'capture-explicit.json').exists():
                raise ValueError('additional explicit requests need their own assessment')
            c, n, parity = request['lambda_squared'], request['n_modes'], request['parity_policy']
            if ((c,n) not in ((13,120),(100,500)) or request['precision_bits'] != 3386
                    or parity not in ('natural','even-sector')
                    or request['capture']['level'] != 'ultra'
                    or request['capture']['prefix_checkpoint_dimensions'] != [n+1]
                    or request['capture'].get('prefix_working_precision_bits') is not None
                    or request['capture']['sector_eigenpairs'] != 8):
                raise ValueError('not a frozen Claim 8 Ultra journal')
            if ((point['lambda_squared'],point['n_modes'],point['parity_policy'],point['precision_digits'])
                    != (c,n,parity,1000)):
                raise ValueError('measurement configuration differs from the request')
            if (len(point['roots']) != 25 or {r['reference_index'] for r in point['roots']} != set(range(1,26))
                    or any(r['status'] != 'converged' for r in point['roots'])):
                raise ValueError('all 25 requested converged roots must be present exactly once')
            checkpoint = f'prefix_checkpoint_{n+1}'
            expected = CLAIM8_NATURAL_REQUESTS | {checkpoint, 'prime_power_response', 'u_flow_response'}
            outcomes = record['receipt']['outcomes']
            applicability = request['applicability']
            if (applicability != state['applicability'] or applicability['policy'] != 'full'
                    or applicability['excluded_diagnostics'] or set(applicability['requested_diagnostics']) != expected
                    or set(outcomes) != expected):
                raise ValueError('review requires the original full request and all thirteen outcomes')
            exclusions = {}
            if parity == 'natural':
                claim8_natural_checkpoint(request)
                exclusions = {'prime_power_response':('blocked',RESPONSE_REASON),
                              'u_flow_response':('blocked',RESPONSE_REASON),
                              checkpoint:('missing',CHECKPOINT_REASON)}
            for name, outcome in outcomes.items():
                if name in exclusions:
                    if (outcome['status'],outcome.get('reason')) != exclusions[name]:
                        raise ValueError(f'{name}: unexpected failure; cannot classify as unsupported')
                elif outcome['status'] != 'completed' or name not in record['measurements']:
                    raise ValueError(f'{name}: applicable diagnostic not captured')
            if bool(state['capture_complete']) != (parity == 'even-sector'):
                raise ValueError('original capture status contradicts retained outcomes')
            review_policy = {'policy':'claim8-natural', 'excluded_diagnostics':exclusions} if exclusions else None
            reviews = numerical_review(record, review_policy)
            if reviews:
                raise ValueError('; '.join(reviews))
            checks = assess_point('claim8_natural_eigenvector', point)
            if not all(ok for _,ok in checks):
                raise ValueError('headline numerical checks failed')
            points.append(point)
            row.update(passed=True, lambda_squared=c, n_modes=n, parity_policy=parity,
                       original_capture_complete=state['capture_complete'],
                       applicable_diagnostics_completed=len(expected)-len(exclusions),
                       exclusions_on_review={name:reason for name,(_,reason) in exclusions.items()},
                       numerical_checks=[{'description':name,'passed':ok} for name,ok in checks])
        except (OSError, ValueError, KeyError, TypeError, ArithmeticError) as error:
            row['error'] = str(error)
    try:
        report['checks'].extend({'description':name,'passed':ok} for name,ok in
                                assess_series('claim8_natural_eigenvector', points))
    except (ValueError, KeyError, ArithmeticError) as error:
        report['checks'].append({'description':str(error),'passed':False})
    report['passed'] = (bool(report['journals']) and all(row['passed'] for row in report['journals'])
                        and all(row['passed'] for row in report['checks']))
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--run-root', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True, help='New sidecar path; existing files are never overwritten')
    args = parser.parse_args()
    if args.output.resolve().is_relative_to(args.run_root.resolve()):
        parser.error('keep the assessment outside the original run directory')
    with localcontext() as context:
        context.prec = 100
        report = review_run(args.run_root)
    with args.output.open('x', encoding='utf-8') as stream:
        json.dump(report, stream, indent=2)
        stream.write('\n')
    for row in report['journals']:
        print(f'[{"PASS" if row["passed"] else "FAIL"}] {row["directory"]}')
        if row['passed']:
            print(f'  {row["applicable_diagnostics_completed"]} applicable diagnostics complete; {len(row["exclusions_on_review"])} unsupported exports identified on review')
        else:
            print(f'  {row["error"]}')
    print(f'Claim 8 applicable-evidence review: {"PASS" if report["passed"] else "FAIL"}')
    print('Original capture outcomes are unchanged. This is a separate retrospective assessment, not a new capture receipt or publication.')
    print(f'Assessment: {args.output}')
    return 0 if report['passed'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
