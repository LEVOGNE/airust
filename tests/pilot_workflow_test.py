import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'scripts'))
from mcp_client import StdioClient
import pilot_dataset

metadata = json.loads(subprocess.check_output(['cargo', 'metadata', '--no-deps', '--format-version', '1', '--locked', '--offline'], cwd=ROOT, text=True))
BINARY = Path(metadata['target_directory']) / 'debug/examples/reference_workspace'


class PilotWorkflowTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.sources = self.root / 'sources'
        self.sources.mkdir()
        self.raw = 'Café — Rust ownership protects memory.\n\nIndependent Unicode: 東京.\n'.encode()
        (self.sources / 'guide.md').write_bytes(self.raw)
        self.pilot = self.root / 'pilot'
        self.run_cli('pilot', 'init', self.sources, self.pilot, 'manuals')
        self.workspace = self.pilot / 'workspace'

    def run_cli(self, *args, check=True):
        result = subprocess.run([str(BINARY), *map(str, args)], text=True, capture_output=True, timeout=10)
        if check:
            self.assertEqual(result.returncode, 0, result.stderr)
        else:
            self.assertNotEqual(result.returncode, 0, result.stdout)
            self.assertEqual(result.stdout, '')
        return result

    def capture(self, identifier='synthetic-001', origin='synthetic_test'):
        request = self.root / (identifier + '-request.json')
        request.write_text(json.dumps({'case_id': identifier, 'query': 'Café Rust memory', 'language': 'de', 'origin': origin,
                                       'origin_evidence': 'Synthetic integration fixture; declared origin is used only to test schema behavior.'}))
        output = self.pilot / 'cases' / (identifier + '.json')
        self.run_cli('pilot', 'capture', self.pilot, request, output)
        return output

    def payload(self, path):
        envelope = json.loads(path.read_text())
        self.assertEqual(hashlib.sha256(envelope['payload'].encode()).hexdigest(), envelope['payload_sha256'])
        return json.loads(envelope['payload'])

    def rewrite(self, path, case):
        payload = json.dumps(case, ensure_ascii=False, separators=(',', ':'))
        path.write_text(json.dumps({'payload': payload, 'payload_sha256': hashlib.sha256(payload.encode()).hexdigest()}))

    def export(self, selected=None, name='export'):
        output = self.root / name
        self.run_cli('pilot', 'export', self.pilot, selected or self.pilot / 'cases', output)
        return output

    def test_complete_capture_review_export_and_independent_validation(self):
        path = self.capture()
        original = path.read_bytes()
        case = self.payload(path)
        self.assertIsNone(case['review'])
        self.assertTrue(case['observation']['evidence'])
        review = self.root / 'review.json'
        review.write_text(json.dumps({'reviewer': 'integration-test', 'reviewer_kind': 'test', 'independence': 'same_author',
                                      'answerability': 'supported', 'relevant_sources': case['observation']['evidence'][:1], 'notes': 'Synthetic fixture, not human review.'}))
        reviewed = self.pilot / 'reviewed' / 'reviewed.json'
        self.run_cli('pilot', 'review', self.pilot, path, review, reviewed)
        changed = self.payload(reviewed)
        self.assertEqual(case['request'], changed['request'])
        self.assertEqual(json.loads(path.read_text())['payload_sha256'], changed['previous_capture_sha256'])
        self.assertEqual(path.read_bytes(), original)
        self.run_cli('pilot', 'check', self.pilot, reviewed)
        exported = self.export(self.pilot / 'reviewed')
        report, rows, manifest = pilot_dataset.validate(exported)
        self.assertEqual(rows[0]['request']['origin'], 'synthetic_test')
        self.assertEqual(report['eligible_settled_cases'], 0)
        self.assertFalse(report['evaluation_ready'])
        self.assertFalse(manifest['evaluation_ready'])
        self.assertEqual(manifest['declared_live_and_independently_reviewed'], 0)
        self.run_cli('query', exported / 'workspace', 'Rust')
        self.assertEqual((self.sources / 'guide.md').read_bytes(), self.raw)

    def test_real_stdio_uses_generated_client_config_and_does_not_record(self):
        config = json.loads((self.pilot / 'mcp-client.json').read_text())['mcpServers']['airust-pilot']
        with StdioClient([config['command'], *config['args']]) as client:
            self.assertEqual(len(client.request('tools/list')['tools']), 5)
            result = client.call('airust_search', {'query': 'Rust', 'limit': 6, 'excerpt_bytes': 2000})
            self.assertFalse(result.get('isError'))
            data = result['structuredContent']['data']
            self.assertTrue(data['sources'])
            source = data['sources'][0]
            self.assertEqual(source['status'], 'snapshot_bound')
        self.run_cli('query', self.workspace, 'Rust')
        self.assertEqual(list((self.pilot / 'cases').iterdir()), [])
        self.assertEqual(list((self.pilot / 'reviewed').iterdir()), [])

    def test_rehash_cannot_hide_bad_utf8_stale_revision_or_workspace(self):
        path = self.capture()
        original = self.payload(path)
        for mutate in [lambda c: c['observation']['evidence'][0].update(start_byte=4),
                       lambda c: c['observation']['evidence'][0].update(revision='0' * 64),
                       lambda c: c['workspace'].update(revision_sha256='0' * 64),
                       lambda c: c.update(extra='unrecognized')]:
            case = json.loads(json.dumps(original))
            mutate(case)
            self.rewrite(path, case)
            self.run_cli('pilot', 'check', self.pilot, path, check=False)
            self.run_cli('pilot', 'export', self.pilot, self.pilot / 'cases', self.root / 'invalid-export', check=False)
            self.assertFalse((self.root / 'invalid-export').exists())

    def test_duplicate_symlink_and_existing_destinations_preserve_data(self):
        path = self.capture()
        original = path.read_bytes()
        self.run_cli('pilot', 'capture', self.pilot, self.root / 'synthetic-001-request.json', path, check=False)
        self.assertEqual(path.read_bytes(), original)
        duplicate = path.with_name('duplicate.json')
        shutil.copyfile(path, duplicate)
        self.run_cli('pilot', 'export', self.pilot, self.pilot / 'cases', self.root / 'duplicate-export', check=False)
        self.assertFalse((self.root / 'duplicate-export').exists())
        duplicate.unlink()
        duplicate.symlink_to(path)
        self.run_cli('pilot', 'export', self.pilot, self.pilot / 'cases', self.root / 'linked-export', check=False)
        duplicate.unlink()
        output = self.export()
        manifest = (output / 'manifest.json').read_bytes()
        self.run_cli('pilot', 'export', self.pilot, self.pilot / 'cases', output, check=False)
        self.assertEqual((output / 'manifest.json').read_bytes(), manifest)

    def test_invalid_requests_review_roles_and_export_tampering_fail(self):
        path = self.capture()
        request_path = self.root / 'synthetic-001-request.json'
        request = json.loads(request_path.read_text())
        for value in [{**request, 'query': 'é' * 2049}, {**request, 'case_id': '../escape'}, {**request, 'extra': True}]:
            request_path.write_text(json.dumps(value))
            self.run_cli('pilot', 'capture', self.pilot, request_path, self.root / 'invalid.json', check=False)
            self.assertFalse((self.root / 'invalid.json').exists())
        review = self.root / 'invalid-review.json'
        review.write_text(json.dumps({'reviewer': 'not-a-human', 'reviewer_kind': 'assistant', 'independence': 'independent',
                                      'answerability': 'unsupported', 'relevant_sources': [], 'notes': ''}))
        self.run_cli('pilot', 'review', self.pilot, path, review, self.root / 'invalid.json', check=False)
        exported = self.export()
        with self.assertRaises(ValueError):
            pilot_dataset.freeze(exported, self.root / 'absent-plan.json', self.root / 'not-real-v2')
        self.assertFalse((self.root / 'not-real-v2').exists())
        (exported / 'cases.jsonl').write_bytes((exported / 'cases.jsonl').read_bytes().replace(b'synthetic_test', b'user_supplied'))
        with self.assertRaises(ValueError):
            pilot_dataset.validate(exported)

    def test_freeze_enforces_split_boundaries_and_preserves_declared_assurance(self):
        # Exercise the format's declared-role branch only. These are fabricated
        # temporary test declarations and are never a genuine pilot dataset.
        (self.sources / 'other.md').write_text('Other source family about Rust memory.')
        another = self.root / 'second-pilot'
        self.run_cli('pilot', 'init', self.sources, another, 'manuals')
        self.pilot, self.workspace = another, another / 'workspace'
        path = self.capture()
        base = self.payload(path)
        sources = [c for c in json.loads(self.run_cli('query', self.workspace, 'Rust memory').stdout)['citations']]
        self.assertEqual(len({s['document_id'] for s in sources}), 2)
        snapshots = pilot_dataset.workspace((self.workspace / 'workspace.json').read_bytes())[1]
        positives = []
        for source in sources:
            raw = snapshots[source['document_id']]['raw']
            positives.append({k: source[k] for k in ['document_id', 'revision', 'start_byte', 'end_byte']}
                             | {'quote': raw[source['start_byte']:source['end_byte']].decode()})
        path.unlink()
        assignments = {}
        for n in range(50):
            case = json.loads(json.dumps(base))
            identifier = f'declared-fixture-{n:03}'
            case['request'].update(case_id=identifier, query=f'Synthetic question {n}', origin='user_supplied',
                                   origin_evidence='Fabricated declaration solely for temporary format tests; not actual traffic.')
            negative = n % 5 == 0
            case['review'] = {'reviewer': 'synthetic-human-role-fixture', 'reviewer_kind': 'human', 'independence': 'independent',
                              'answerability': 'unsupported' if negative else 'supported', 'relevant_sources': [] if negative else [positives[n // 25]],
                              'notes': 'Fabricated role declaration used only to exercise validation.'}
            self.rewrite(self.pilot / 'cases' / (identifier + '.json'), case)
            assignments[identifier] = {'family': f'family-{n // 25}', 'split': 'development' if n < 25 else 'evaluation'}
        exported = self.export()
        policy = {'ndcg_at_10_minimum': .75, 'hit_at_6_minimum': .80, 'optional_ndcg_gain_minimum': .03,
                  'optional_hit_loss_maximum': .02, 'lexical_warm_p95_ms': 25, 'hybrid_warm_p95_ms': 250,
                  'rerank_warm_p95_ms': 1500, 'worker_peak_rss_mib': 4096, 'embedding_service_peak_rss_mib': 4096}
        plan = {'schema_version': 1, 'dataset': 'reference-v2', 'provenance_reviewed_by': 'synthetic-format-test',
                'assignments': assignments, 'policy': policy}
        plan_path = self.root / 'plan.json'
        plan_path.write_text(json.dumps(plan))
        receipt = pilot_dataset.freeze(exported, plan_path, self.root / 'synthetic-freeze')
        self.assertFalse(receipt['quality_gate_passed'])
        self.assertIn('Caller declarations', receipt['origin_assurance'])
        plan['assignments']['declared-fixture-026']['family'] = 'family-0'
        plan_path.write_text(json.dumps(plan))
        with self.assertRaises(ValueError):
            pilot_dataset.freeze(exported, plan_path, self.root / 'leaking-freeze')
        self.assertFalse((self.root / 'leaking-freeze').exists())


if __name__ == '__main__':
    unittest.main()
