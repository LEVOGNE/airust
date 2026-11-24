import importlib.util
import json
from pathlib import Path
import shutil
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('reference_dataset', ROOT / 'scripts/reference_dataset.py')
dataset = importlib.util.module_from_spec(spec)
spec.loader.exec_module(dataset)


class ReferenceDatasetTests(unittest.TestCase):
    def test_frozen_identity_and_reserved_source_families(self):
        identity, artifacts = dataset.validate()
        self.assertEqual(identity['question_origins']['historical_user_traffic'], 0)
        cases = [json.loads(line) for line in artifacts['cases.jsonl'].splitlines()]
        groups = {split: {case['family'] for case in cases if case['split'] == split} for split in ['development', 'evaluation']}
        self.assertFalse(groups['development'] & groups['evaluation'])
        self.assertEqual((len(groups['development']), len(groups['evaluation'])), (12, 8))
        self.assertEqual(dataset.validate(), (identity, artifacts))

    def test_every_passage_resolves_to_original_utf8_and_guards_are_separate(self):
        _, artifacts = dataset.validate()
        originals = {item['id']: item['text'].encode() for item in json.loads(artifacts['documents.json'])}
        rows = [json.loads(line) for line in artifacts['corpus.jsonl'].splitlines()]
        guards = [row for row in rows if row['collection'] == 'out-of-scope-fixture']
        self.assertEqual(len(guards), 16)
        for row in rows:
            self.assertEqual(originals[row['document']][row['start']:row['end']].decode(), row['text'])
            self.assertLessEqual(len(row['text'].encode()), 1800)

    def test_changed_source_bytes_are_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            fixture = Path(temporary) / 'fixture'
            shutil.copytree(dataset.FIXTURE, fixture)
            path = fixture / 'sources/SEMANTIC.md'
            path.write_bytes(path.read_bytes() + b'\nchanged\n')
            with self.assertRaisesRegex(ValueError, 'source identity mismatch'):
                dataset.validate(fixture)

    def test_changed_acceptance_policy_cannot_reuse_version_one(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / 'policy.json'
            policy = json.loads(dataset.POLICY.read_text())
            policy['resources']['lexical_warm_p95_ms'] = 2500
            path.write_text(json.dumps(policy))
            with self.assertRaisesRegex(ValueError, 'version one is frozen'):
                dataset.validate(policy_path=path)

    def test_authored_questions_cannot_be_relabelled_as_customer_traffic(self):
        with tempfile.TemporaryDirectory() as temporary:
            fixture = Path(temporary) / 'fixture'
            shutil.copytree(dataset.FIXTURE, fixture)
            path = fixture / 'cases.jsonl'
            cases = [json.loads(line) for line in path.read_text().splitlines()]
            cases[0]['origin'] = 'historical_user_traffic'
            path.write_text(''.join(json.dumps(case) + '\n' for case in cases))
            with self.assertRaisesRegex(ValueError, 'unsubstantiated question origin'):
                dataset.validate(fixture)


if __name__ == '__main__':
    unittest.main()
