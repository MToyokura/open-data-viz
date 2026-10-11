"""Synthetic offline tests for the official spreadsheet reconciliation contract."""
import json
import subprocess
import sys
import tempfile
import unittest
import zipfile
from pathlib import Path
from reconcile_2024 import reconcile, digest
from verify_temporal_mapping import column


class ReconciliationTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.book = {
            'table_id': '0004044496', 'metadata_sha256': 'abc',
            'origin': [{'code': str(i)} for i in range(1001, 1049)],
            'measure': [{'code': str(i)} for i in range(1001, 1003)],
            'consumer_region': [{'code': str(i)} for i in range(1001, 1013)],
            'time': [{'code': '10', 'reporting_month': None}] + [
                {'code': f'M{i}', 'reporting_month': f'2024-{i:02}'} for i in range(1, 13)]}
        self.capture = {'table_id': '0004044496', 'source_metadata_sha256': 'abc', 'rows': []}
        self.manifest = {'periods': []}
        pos = 0
        for month in range(13):
            period = 'total' if month == 0 else f'2024-{month:02}'
            timecode = '10' if month == 0 else f'M{month}'
            cells = ['<c r="A4" t="inlineStr"><is><t>だいこん '+ ('計' if month == 0 else f'{month}月') + '</t></is></c>']
            for origin in range(1, 49):
                row = origin + 6
                cells.append(f'<c r="B{row}"><v>{origin}</v></c>')
                for region in range(12):
                    for measure in range(2):
                        pos += 1
                        value = ('-' if region == 0 else '0' if region == 1 else str(origin + region + measure))
                        addr = f'{column(3 + region * 2 + measure)}{row}'
                        if value == '-':
                            cells.append(f'<c r="{addr}" t="inlineStr"><is><t>-</t></is></c>')
                        else:
                            cells.append(f'<c r="{addr}"><v>{value}</v></c>')
                        self.capture['rows'].append({'position': pos, 'raw_value': value,
                            'attributes': {'@time': timecode, '@cat01': str(1000 + origin),
                                           '@cat02': str(1001 + measure), '@cat03': str(1001 + region)}})
            xml = '<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData><row r="4">' + cells[0] + '</row><row r="7">' + ''.join(cells[1:]) + '</row></sheetData></worksheet>'
            name = f'{month:02}.xlsx'
            with zipfile.ZipFile(self.root / name, 'w') as archive:
                archive.writestr('xl/worksheets/sheet1.xml', xml)
            self.manifest['periods'].append({'period': period, 'file': name, 'sha256': digest(self.root / name)})
        (self.root / 'manifest.json').write_text(json.dumps(self.manifest))

    def test_all_cells(self):
        result = reconcile(self.book, self.capture, self.manifest, self.root)
        self.assertEqual(result['cells_compared'], 14976)
        self.assertGreater(result['symbol_counts']['0'], 0)
        self.assertGreater(result['symbol_counts']['-'], 0)

    def test_wrong_checksum(self):
        self.manifest['periods'][0]['sha256'] = 'a' * 64
        with self.assertRaisesRegex(ValueError, 'checksum'):
            reconcile(self.book, self.capture, self.manifest, self.root)

    def test_changed_source_value(self):
        self.capture['rows'][0]['raw_value'] = '999'
        with self.assertRaisesRegex(ValueError, 'cell mismatch'):
            reconcile(self.book, self.capture, self.manifest, self.root)

    def test_missing_period(self):
        self.manifest['periods'].pop()
        with self.assertRaises(ValueError):
            reconcile(self.book, self.capture, self.manifest, self.root)

    def test_validation_failure_under_optimized_python(self):
        self.manifest['periods'][0]['sha256'] = 'a' * 64
        book_path = self.root / 'book.json'
        capture_path = self.root / 'capture.json'
        manifest_path = self.root / 'manifest.json'
        book_path.write_text(json.dumps(self.book))
        capture_path.write_text(json.dumps(self.capture))
        manifest_path.write_text(json.dumps(self.manifest))
        module_dir = Path(__file__).resolve().parent
        script = (
            'import json, sys; from pathlib import Path; '
            f'sys.path.insert(0, {str(module_dir)!r}); '
            'from reconcile_2024 import reconcile; '
            f'reconcile(json.load(open({str(book_path)!r})), '
            f'json.load(open({str(capture_path)!r})), '
            f'json.load(open({str(manifest_path)!r})), '
            f'Path({str(self.root)!r}))'
        )
        result = subprocess.run([sys.executable, '-O', '-c', script],
                                capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('checksum mismatch', result.stderr)


if __name__ == '__main__':
    unittest.main()
