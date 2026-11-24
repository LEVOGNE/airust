import importlib.util
import json
import math
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'scripts'))
import reference_dataset
import run_reference_evaluation as evaluation


class EvaluationTests(unittest.TestCase):
    def test_selection_rejects_evaluation_results(self):
        with self.assertRaisesRegex(ValueError, 'development observations only'):
            evaluation.select_development({'bm25': {'split': 'evaluation'}}, json.loads(reference_dataset.POLICY.read_text()))

    def test_metric_oracle_handles_a_missed_first_hit_and_unanswerable_query(self):
        score = 1 / math.log2(3)
        report = {'cases': [
            {'id': 'positive', 'ranked_ids': ['wrong', 'right'], 'metrics': {'ndcg_at_k': score, 'reciprocal_rank_at_k': .5}},
            {'id': 'negative', 'ranked_ids': ['wrong'], 'metrics': {'ndcg_at_k': None, 'reciprocal_rank_at_k': 0}},
        ]}
        evaluation.check_metrics(report, {'positive': {'right': 2}})
        report['cases'][0]['metrics']['ndcg_at_k'] = 1
        with self.assertRaisesRegex(ValueError, 'independent nDCG formula differs'):
            evaluation.check_metrics(report, {'positive': {'right': 2}})

    def test_resource_failure_cannot_promote_a_quality_winner(self):
        policy = json.loads(reference_dataset.POLICY.read_text())
        baseline = {'reference_ndcg_at_10': .5, 'reference_hit_at_6': .75}
        candidate = {'reference_ndcg_at_10': .9, 'reference_hit_at_6': .9, 'resource_budget_passed': False}
        self.assertFalse(evaluation.qualifies(candidate, baseline, policy))

    def test_real_lexical_and_sqlite_workers_preserve_scope_spans_and_metrics(self):
        metadata = json.loads(subprocess.check_output(['cargo', 'metadata', '--no-deps', '--format-version', '1', '--locked', '--offline'], cwd=ROOT, text=True))
        binary = Path(metadata['target_directory']) / 'debug/examples/reference_evaluation'
        self.assertTrue(binary.is_file(), 'build the reference_evaluation example before this test')
        identity, artifacts = reference_dataset.validate()
        judgments = {}
        for line in artifacts['qrels.jsonl'].splitlines():
            row = json.loads(line)
            judgments.setdefault(row['query_id'], {})[row['document_id']] = row['grade']
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            for name, data in artifacts.items():
                (directory / name).write_bytes(data)
            (directory / 'manifest.json').write_text(json.dumps(identity))
            (directory / 'policy.json').write_bytes(reference_dataset.POLICY.read_bytes())
            for profile in ['bm25', 'sqlite']:
                output = directory / (profile + '.json')
                subprocess.run([str(binary), str(directory), 'development', profile, str(output)], stdout=subprocess.DEVNULL, check=True, timeout=60)
                result = json.loads(output.read_text())
                self.assertEqual(len(result['cases']), 90)
                self.assertEqual(result['scope_leaks'], 0)
                self.assertEqual(result['span_mismatches'], 0)
                evaluation.check_metrics(result, judgments)
                for case in result['cases']:
                    self.assertTrue(all(not identifier.startswith('guard-v1:') for identifier in case['ranked_ids']))


if __name__ == '__main__':
    unittest.main()
