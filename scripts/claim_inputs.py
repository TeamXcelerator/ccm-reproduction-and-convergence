#!/usr/bin/env python3
"""Select optional, configuration-specific research data before each invocation.

Files contain data, never executable code. The Toolkit validates their contents
and source bindings; absence leaves the corresponding diagnostic unavailable.
"""
import os
import sys
from pathlib import Path


def selected_environment(arguments, environ):
    result = dict(environ)
    def option(name, default):
        return arguments[arguments.index(name) + 1] if name in arguments else default
    c = option('--lambda-sq', '13')
    n = option('--n-modes', '120')
    digits = option('--precision-digits', '1000')
    parity = 'natural' if '--no-force-even' in arguments else option('--parity-policy', 'even-sector')
    if not all(value.isdecimal() for value in (c, n, digits)):
        raise ValueError('research input selection requires integer C, N and decimal precision')
    if parity not in ('natural', 'adaptive-even', 'even-sector'):
        raise ValueError('unrecognized parity for research input selection')
    if result.get('XC_TARGET_SPEC_FILE') and result.get('XC_TARGET_SPEC_DIR'):
        raise ValueError('use either XC_TARGET_SPEC_FILE or XC_TARGET_SPEC_DIR, not both')
    if result.get('XC_TARGET_SPEC_DIR'):
        directory = Path(result['XC_TARGET_SPEC_DIR']).resolve(strict=True)
        candidate = (directory / f'c{int(c)}-d{int(digits)}.json').resolve(strict=True)
        if not candidate.is_file() or not candidate.is_relative_to(directory):
            raise ValueError('target specification must be a file within its configured directory')
        result['XC_TARGET_SPEC_FILE'] = str(candidate)
    filename = f'c{c}-n{n}-d{digits}-{parity}.json'
    for kind in ('REFERENCE', 'INPUTS'):
        file_key, directory_key = f'XC_RESEARCH_{kind}_FILE', f'XC_RESEARCH_{kind}_DIR'
        if result.get(file_key) and result.get(directory_key):
            raise ValueError(f'use either {file_key} or {directory_key}, not both')
        if result.get(directory_key):
            directory = Path(result[directory_key]).resolve(strict=True)
            if not directory.is_dir():
                raise ValueError(f'{directory_key} is not a directory')
            names = [filename] + ([f'c{c}.json'] if kind == 'REFERENCE' else [])
            for name in names:
                candidate = directory / name
                if candidate.is_file():
                    resolved = candidate.resolve(strict=True)
                    if not resolved.is_relative_to(directory):
                        raise ValueError('research input must stay within its configured directory')
                    result[file_key] = str(resolved)
                    break
    return result


def main():
    arguments = sys.argv[1:]
    if not arguments:
        raise SystemExit('usage: claim_inputs.py EXECUTABLE [ARGUMENTS...]')
    try:
        env = selected_environment(arguments[1:], os.environ)
    except (OSError, ValueError, IndexError) as error:
        raise SystemExit(f'Research input selection failed: {error}') from error
    os.execvpe(arguments[0], arguments, env)


if __name__ == '__main__':
    main()
