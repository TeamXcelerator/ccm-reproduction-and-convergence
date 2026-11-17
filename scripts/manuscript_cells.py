"""Compare bound claim measurements with the numbers printed in Paper 1's TeX tables."""
from decimal import Decimal as D, ROUND_HALF_EVEN, localcontext
import json
import re

CELL_COUNTS = {'1a': 4, '1b': 4, '1c': 2, '2a': 5, '2b': 15, '3': 12, '4a': 5, '4b': 5,
               '4c': 10, '4d': 5, '6': 6, '6b': 23, '7': 90, '7b': 8, '8': 16}
HISTORICAL_RESIDUALS = {'4a': {(13, 120)}, '4b': {(100, 500)}, '4c': {(1000, 800), (1000, 890)}}


def tables(path):
    lines = path.read_text(encoding='utf-8').splitlines()
    result = {}
    label = None
    active = False
    for line_number,line in enumerate(lines,1):
        match = re.search(r'\\label\{(tab:[^}]+)\}',line)
        if match:
            label = match[1]
            if label in result:
                raise ValueError(f'duplicate manuscript table {label}')
            result[label] = []
        if label and r'\midrule' in line:
            active = True
        if label and (r'\bottomrule' in line or r'\end{table}' in line):
            active = False
        if active and '&' in line:
            result[label].append(dict(line=line_number,cells=[s.strip() for s in line.split('&')],latex=line))
    return result


def value(cell):
    scientific = re.search(r'([-+]?\d+(?:\.\d+)?)\s*\\times\s*10\^\{([-+]?\d+)\}',cell)
    if scientific:
        return D(scientific[1]+'e'+scientific[2])
    match = re.search(r'[-+]?\d+(?:\.\d+)?',cell)
    if not match:
        raise ValueError(f'No printed number in {cell!r}')
    return D(match[0])


def configuration(cell):
    numbers = re.findall(r'\{=\}(\d+)',cell)
    if len(numbers) != 2:
        raise ValueError('Expected a printed C,N configuration')
    return tuple(map(int,numbers))


def printed_number(cell):
    text = cell.strip()
    if text.endswith(r'\\'):
        text = text[:-2].rstrip()
    if text.startswith('$') and text.endswith('$'):
        text = text[1:-1].strip()
    bold = re.fullmatch(r'\\mathbf\{(.+)\}', text)
    if bold:
        text = bold[1]
    m = re.fullmatch(r'([-+]?\d+(?:\.\d+)?)(?:\s*\\times\s*10\^\{([-+]?\d+)\})?', text)
    if not m:
        raise ValueError(f'expected a printed point value in {cell!r}')
    return D(m[1] + ('e' + m[2] if m[2] else ''))


def expected_rows(claim,point,table):
    c,n,p = point['lambda_squared'],point['n_modes'],point['precision_digits']
    rows = []
    def add(label,row,column,metric,index=None,scope=None):
        rows.append(dict(table=label,line=row['line'],printed=row['cells'][column],
                         expected=str(printed_number(row['cells'][column])),metric=metric,root_index=index,
                         source_latex=row['latex'],scope=scope))
    if claim in ('1a','1b','1c','8'):
        for row in table['tab:headline']:
            if configuration(row['cells'][0]) == (c,n):
                add('tab:headline',row,1,'root_digits',1)
                add('tab:headline',row,2,'root_digits',20)
    if claim in ('2a','2b'):
        for row in table['tab:lambda_sweep']:
            if int(value(row['cells'][0])) == c:
                add('tab:lambda_sweep',row,3 if p==200 else 4,'root_digits',1)
                if p==1000:
                    add('tab:lambda_sweep',row,2,'epsilon_N')
                    add('tab:lambda_sweep',row,5,'root_accuracy_offset',1)
    if claim=='3':
        for row in table['tab:critical_N']:
            if configuration(row['cells'][0]) == (c,n):
                for column,metric,index in ((1,'epsilon_N',None),(2,'root_digits',1),(3,'root_digits',5)):
                    add('tab:critical_N',row,column,metric,index)
    if claim in ('1a','1b','1c','6','8'):
        for row in table['tab:eps_N']:
            a=row['cells']
            if (int(value(a[0])),int(value(a[1])),int(value(a[3]))) == (c,n,p):
                add('tab:eps_N',row,4,'epsilon_N')
                add('tab:eps_N',row,5,'eigenvalue_depth')
    if claim=='6b':
        for row in table['tab:eps_N_abovefloor']:
            if (int(value(row['cells'][0])),int(value(row['cells'][1]))) == (c,n):
                for column,metric,index in ((3,'epsilon_N',None),(4,'eigenvalue_depth',None),(5,'root_digits',1)):
                    add('tab:eps_N_abovefloor',row,column,metric,index)
                if c!=500:
                    add('tab:eps_N_abovefloor',row,6,'depth_per_prime_doubling')
    if claim=='7':
        for row in table['tab:conv_N']:
            if int(value(row['cells'][0]))==n:
                for index in range(1,6):
                    add('tab:conv_N',row,index,'root_digits',index)
    if claim=='7b':
        for row in table['tab:critical_brackets']:
            for ncol,dcol in ((1,2),(3,4)):
                if int(value(row['cells'][ncol]))==n:
                    add('tab:critical_brackets',row,dcol,'root_digits',1)
    if claim.startswith('4'):
        for row in table['tab:evenness']:
            a=row['cells']
            hp = re.fullmatch(r'HP-(\d+)', a[2])
            if hp is None:
                raise ValueError(f'Unknown precision label in manuscript: {a[2]}')
            if (int(value(a[0])),int(value(a[1])),int(hp[1])) == (c,n,p):
                add('tab:evenness',row,3,'evenness_deviation')
                add('tab:evenness',row,4,'eigenvalue_relative_difference')
        for row in table['tab:sector_separations']:
            if (int(value(row['cells'][0])),int(value(row['cells'][1]))) == (c,n):
                add('tab:sector_separations',row,2,'odd_even_gap')
                add('tab:sector_separations',row,3,'even_gap')
                add('tab:sector_separations',row,4,'residual_gap',scope=(
                    'historical_nearest_point_vs_current_directed_bound' if (c,n) in HISTORICAL_RESIDUALS.get(claim, ())
                    else 'revised_directed_bound'))
    return rows


def round_printed(number, row):
    expected = D(row['expected'])
    if r'\times' in row['printed'] and number:
        places = number.adjusted() - len(expected.as_tuple().digits) + 1
        return number.quantize(D(1).scaleb(places), rounding=ROUND_HALF_EVEN)
    return number.quantize(expected, rounding=ROUND_HALF_EVEN)


def observed(row, point, all_points, journal, table, root_digits, evenness_relative_difference):
    metric=row['metric']
    if metric=='root_digits':
        source=next(r for r in point['roots'] if r['reference_index']==row['root_index'])
        if source.get('matching_digits') is None:
            raise ValueError('Only a lower bound retained; exact printed-value agreement unassessed')
        return root_digits(point,row['root_index'])
    if metric in ('epsilon_N','evenness_deviation'):
        return D(point[metric])
    if metric=='eigenvalue_relative_difference':
        return evenness_relative_difference(D(point['natural_eigenvalue']),D(point['even_eigenvalue']))
    if metric in ('eigenvalue_depth','root_accuracy_offset'):
        depth=-abs(D(point['epsilon_N'])).log10()
        return depth if metric=='eigenvalue_depth' else depth-root_digits(point,1)
    if metric=='depth_per_prime_doubling':
        ladder=table['tab:eps_N_abovefloor']
        i=next(i for i,r in enumerate(ladder) if int(value(r['cells'][0]))==point['lambda_squared'])
        previous=next(x for x in all_points if x['lambda_squared']==int(value(ladder[i-1]['cells'][0])))
        delta=abs(D(previous['epsilon_N'])).log10()-abs(D(point['epsilon_N'])).log10()
        ratio=value(ladder[i]['cells'][2])/value(ladder[i-1]['cells'][2])
        return delta/(ratio.ln()/D(2).ln())
    if metric=='residual_gap':
        return D(json.loads((journal/'spectral-isolation.json').read_bytes())['selected_state_residual_to_gap_upper_bound'])
    gap=json.loads((journal/'capture.json').read_bytes())['measurements']['sector_analysis']['value'][0]
    return D(gap['lambda_difference' if metric=='odd_even_gap' else 'even_simplicity_margin'])


def compare_point(claim, point, points, journal, table, root_digits, evenness_relative_difference):
    """Return one row per applicable printed number, retaining full precision for the comparison."""
    rows = expected_rows(claim, point, table)
    identities = {(row['table'], row['metric'], row['root_index']) for row in rows}
    if len(identities) != len(rows):
        raise ValueError('duplicate manuscript measurement for the same invocation')
    if len(rows) != invocation_cell_count(claim, point):
        raise ValueError(f"unexpected manuscript coverage for Claim {claim} at "
                         f"C={point['lambda_squared']}, N={point['n_modes']}")
    with localcontext() as context:
        context.prec = 512
        context.Emax = 999999
        context.Emin = -999999
        for row in rows:
            row.update(cutoff=point['lambda_squared'], modes=point['n_modes'],
                       precision_digits=point['precision_digits'], parity=point.get('parity_policy'))
            try:
                number = observed(row, point, points, journal, table, root_digits, evenness_relative_difference)
                if number is None or not number.is_finite():
                    raise ValueError('measurement is undefined or non-finite')
                expected = D(row['expected'])
                rounded = round_printed(number, row)
                row.update(observed=str(number), rounded=str(rounded), matches_printed_value=rounded == expected)
                if row.get('scope') == 'historical_nearest_point_vs_current_directed_bound':
                    row['verdict'] = 'DIFFERENT_QUANTITY'
                else:
                    row['verdict'] = 'MATCH' if rounded == expected else 'DIFFERS'
            except (ArithmeticError, ValueError, KeyError, TypeError, StopIteration, OSError, IndexError) as error:
                row.update(verdict='UNASSESSED', reason=str(error) or 'missing measurement')
    return rows


def invocation_cell_count(claim, point):
    if claim == '7b':
        return int(point['n_modes'] in (39, 40, 55, 56))
    if claim == '6b':
        return 3 if point['lambda_squared'] == 500 else 4
    return {'1a': 4, '1b': 4, '1c': 2, '2a': 1, '2b': 3, '3': 3, '4a': 5,
            '4b': 5, '4c': 5, '4d': 5, '6': 2, '7': 5, '8': 4}[claim]


def printed_verdict(claim, rows, *, complete):
    if not complete or len(rows) != CELL_COUNTS[claim] or any(r['verdict'] == 'UNASSESSED' for r in rows):
        return 'INCOMPLETE'
    if any(r['verdict'] not in ('MATCH', 'DIFFERS', 'DIFFERENT_QUANTITY') for r in rows):
        return 'INCOMPLETE'
    if any(r['verdict'] == 'DIFFERS' for r in rows):
        return 'DIFFERS'
    for row in rows:
        is_historical = (row['metric'] == 'residual_gap' and row['table'] == 'tab:sector_separations'
                         and (row['cutoff'], row['modes']) in HISTORICAL_RESIDUALS.get(claim, set())
                         and row.get('scope') == 'historical_nearest_point_vs_current_directed_bound')
        if (row['verdict'] == 'DIFFERENT_QUANTITY') != is_historical:
            return 'INCOMPLETE'
        if not is_historical and row.get('matches_printed_value') is not True:
            return 'DIFFERS'
    exceptions = {(r['cutoff'], r['modes']) for r in rows if r['verdict'] == 'DIFFERENT_QUANTITY'}
    if exceptions != HISTORICAL_RESIDUALS.get(claim, set()):
        return 'INCOMPLETE'
    return 'QUALIFIED_MATCH' if exceptions else 'MATCH'
