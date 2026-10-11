"""Offline, exact lexical reconciliation of all 13 official 2024 daikon XLSX tables.

Expected layout is the same official 48-origin x 12-region x 2-measure format
as the dated 2021-2023 spreadsheets. Any layout deviation fails closed.
"""
import argparse
import hashlib
import json
import re
import unicodedata
from pathlib import Path
from verify_temporal_mapping import spreadsheet_cells, column

ROOT = Path(__file__).resolve().parent


def require(condition, message):
    """Raise a production validation error regardless of Python optimization."""
    if not condition:
        raise ValueError(message)


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def reconcile(book, capture, manifest, directory):
    require(book['table_id'] == capture['table_id'] == '0004044496', 'table ID mismatch')
    require(book['metadata_sha256'] == capture['source_metadata_sha256'], 'metadata checksum mismatch')
    require(len(capture['rows']) == 14976, 'unexpected capture row count')
    axes = {key: {v['code']: v for v in book[key]}
            for key in ('time', 'origin', 'measure', 'consumer_region')}
    origins = [x['code'] for x in book['origin']]
    regions = [x['code'] for x in book['consumer_region']]
    measures = [x['code'] for x in book['measure']]
    require(len(origins) == 48 and len(regions) == 12 and len(measures) == 2,
            'unexpected axis sizes')
    require(len(set(origins)) == 48 and len(set(regions)) == 12 and len(set(measures)) == 2,
            'duplicate axis codes')
    require(len(manifest['periods']) == 13, 'expected 13 workbook periods')
    source = {}
    for row in capture['rows']:
        attrs = row['attributes']
        key = (attrs['@time'], attrs['@cat01'], attrs['@cat02'], attrs['@cat03'])
        require(key not in source, f'duplicate source coordinate {key}')
        source[key] = row['raw_value']
    require(len(source) == 14976, 'unexpected unique source coordinate count')
    periods = {p['period']: p for p in manifest['periods']}
    require(set(periods) == {'total'} | {f'2024-{m:02}' for m in range(1, 13)},
            'missing, duplicate, or unexpected workbook periods')
    time_codes = {x['reporting_month'] or 'total': x['code'] for x in book['time']}
    require(set(time_codes) == set(periods), 'workbook periods do not match codebook time axis')
    comparison_count = 0
    symbols = {'0': 0, '-': 0, 'numeric': 0, '…': 0}
    for period, record in sorted(periods.items()):
        path = directory / record['file']
        require(path.resolve().is_relative_to(directory.resolve()), 'unsafe spreadsheet path')
        require(path.is_file() and path.suffix.lower() == '.xlsx', f'invalid spreadsheet path: {path}')
        require(re.fullmatch(r'[0-9a-f]{64}', record['sha256']) is not None,
                f'invalid spreadsheet checksum: {period}')
        require(digest(path) == record['sha256'], f'checksum mismatch: {period}')
        cells = spreadsheet_cells(path)
        title = unicodedata.normalize('NFKC', str(cells.get('A4', '')))
        month = 0 if period == 'total' else int(period[-2:])
        require('だいこん' in title and title.endswith('計' if month == 0 else f'{month}月'),
                f'workbook title does not match period: {path}: {title!r}')
        row_numbers = {}
        for address, value in cells.items():
            if re.fullmatch(r'B\d+', address) and re.fullmatch(r'\d+', str(value)):
                number = int(value)
                if 1 <= number <= 48:
                    require(number not in row_numbers, f'duplicate origin index {number}')
                    row_numbers[number] = int(re.search(r'\d+', address).group())
        require(set(row_numbers) == set(range(1, 49)), f'incomplete origin rows: {period}')
        for origin_index, origin_code in enumerate(origins, 1):
            sheet_row = row_numbers[origin_index]
            for region_index, region_code in enumerate(regions):
                for measure_index, measure_code in enumerate(measures):
                    cell = f'{column(3 + region_index * 2 + measure_index)}{sheet_row}'
                    observed = cells.get(cell)
                    require(observed is not None, f'missing cell {period}:{cell}')
                    expected = source[(time_codes[period], origin_code, measure_code, region_code)]
                    require(str(observed) == expected, (
                        f'cell mismatch {period}:{cell}: spreadsheet={observed!r} API={expected!r}'
                    ))
                    comparison_count += 1
                    symbols[expected if expected in ('0', '-', '…') else 'numeric'] += 1
    require(comparison_count == 14976, f'unexpected comparison count: {comparison_count}')
    return {'cells_compared': comparison_count, 'periods': 13, 'symbol_counts': symbols,
            'manifest_sha256': digest(directory / 'manifest.json')}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--workbooks', type=Path, required=True,
                        help='directory with manifest.json and all 13 verified official XLSX files')
    parser.add_argument('--capture', type=Path, required=True)
    parser.add_argument('--codebook', type=Path, default=ROOT / '2024_daikon_codebook.json')
    args = parser.parse_args()
    manifest = json.loads((args.workbooks / 'manifest.json').read_text())
    print(json.dumps(reconcile(json.loads(args.codebook.read_text()),
                               json.loads(args.capture.read_text()), manifest, args.workbooks)))


if __name__ == '__main__':
    main()
