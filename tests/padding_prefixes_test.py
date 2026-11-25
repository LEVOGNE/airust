"""Real CLI controls for reconstructed frozen tokenizer inputs and original files."""
import csv
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
PAIRS = ROOT / 'docs/measurements/profile-practical-0.6.0/token-padding-pairs.csv'
SCRIPT = ROOT / 'scripts/prepare_padding_prefixes.py'
metadata = json.loads(subprocess.check_output(
    ['cargo', 'metadata', '--no-deps', '--format-version', '1', '--locked', '--offline'], cwd=ROOT, text=True))
BINARY = Path(metadata['target_directory']) / 'debug/examples/reference_workspace'


class PaddingPrefixesTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temporary = tempfile.TemporaryDirectory()
        cls.root = Path(cls.temporary.name)
        cls.workspace = cls.root / 'workspace'
        result = subprocess.run([str(BINARY), 'build', str(ROOT / 'tests/fixtures/reference-v1/sources'),
                                 str(cls.workspace), 'airust-reference'], capture_output=True, timeout=30)
        if result.returncode:
            cls.temporary.cleanup()
            raise RuntimeError(result.stderr.decode(errors='replace'))
        cls.workspace_bytes = (cls.workspace / 'workspace.json').read_bytes()
        cls.pairs_bytes = PAIRS.read_bytes()
        with PAIRS.open(newline='') as stream:
            reader = csv.DictReader(stream)
            cls.fields = reader.fieldnames
            cls.rows = list(reader)

    @classmethod
    def tearDownClass(cls):
        cls.temporary.cleanup()

    def invoke(self, pairs, output):
        return subprocess.run([sys.executable, '-B', str(SCRIPT), str(pairs),
                               str(self.workspace), str(output)], cwd=ROOT, capture_output=True, timeout=30)

    def test_public_coordinates_reconstruct_every_observed_query_and_candidate(self):
        output = self.root / 'valid-prefixes.json'
        result = self.invoke(PAIRS, output)
        self.assertEqual(result.returncode, 0, result.stderr)
        receipt = json.loads(result.stdout)
        self.assertEqual((receipt['queries'], receipt['pairs']), (150, 3000))
        self.assertEqual(receipt['output_sha256'], hashlib.sha256(output.read_bytes()).hexdigest())
        prefixes = json.loads(output.read_bytes())
        self.assertEqual(len(prefixes), 150)
        actual = {(case['id'], rank): entry for case in prefixes
                  for rank, entry in enumerate(case['candidates'])}
        for row in self.rows:
            entry = actual[(row['id'], int(row['position']))]
            self.assertEqual(entry['id'], row['document_id'])
            self.assertEqual(hashlib.sha256(entry['text'].encode()).hexdigest(), row['text_sha256'])
        self.assertEqual((self.workspace / 'workspace.json').read_bytes(), self.workspace_bytes)
        self.assertEqual(PAIRS.read_bytes(), self.pairs_bytes)

    def test_stale_or_changed_source_and_duplicate_rank_publish_no_output(self):
        for number, (field, value) in enumerate([
            ('source_revision', '0' * 64), ('document_id', 'airust-passage-v1:' + '0' * 64),
            ('text_sha256', '0' * 64), ('query_sha256', '0' * 64), ('position', '1')]):
            with self.subTest(field=field):
                rows = [dict(row) for row in self.rows]
                rows[0][field] = value
                source = self.root / f'changed-{number}.csv'
                with source.open('x', newline='') as stream:
                    writer = csv.DictWriter(stream, fieldnames=self.fields)
                    writer.writeheader()
                    writer.writerows(rows)
                original = source.read_bytes()
                output = self.root / f'failed-{number}.json'
                result = self.invoke(source, output)
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(result.stdout, b'')
                self.assertFalse(output.exists())
                self.assertEqual(source.read_bytes(), original)
        self.assertEqual((self.workspace / 'workspace.json').read_bytes(), self.workspace_bytes)

    def test_existing_destination_and_oversized_input_are_preserved(self):
        output = self.root / 'original-output.json'
        output.write_bytes(b'original user output')
        self.assertNotEqual(self.invoke(PAIRS, output).returncode, 0)
        self.assertEqual(output.read_bytes(), b'original user output')
        sparse = self.root / 'oversized.csv'
        with sparse.open('xb') as stream:
            stream.truncate(8 * 1024 * 1024 + 1)
        result = self.invoke(sparse, self.root / 'oversized-output.json')
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(result.stdout, b'')
        self.assertFalse((self.root / 'oversized-output.json').exists())
        self.assertEqual(sparse.stat().st_size, 8 * 1024 * 1024 + 1)

    def test_duplicate_headers_and_missing_candidates_are_rejected(self):
        for label, content in [
            ('duplicate-header', ','.join(self.fields + [self.fields[0]]) + '\n'),
            ('missing-candidates', self.pairs_bytes.decode().splitlines(keepends=True)[:20])]:
            with self.subTest(case=label):
                source = self.root / (label + '.csv')
                source.write_text(content if isinstance(content, str) else ''.join(content))
                output = self.root / (label + '.json')
                result = self.invoke(source, output)
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(result.stdout, b'')
                self.assertFalse(output.exists())


if __name__ == '__main__':
    unittest.main()
