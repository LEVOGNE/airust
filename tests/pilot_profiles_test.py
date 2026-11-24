"""Synthetic protocol/failure fixtures, never model quality measurements."""
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
import queue
from pathlib import Path
import subprocess
import sys
import tempfile
import threading
import time
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'scripts'))
from mcp_client import StdioClient

metadata = json.loads(subprocess.check_output(['cargo', 'metadata', '--no-deps', '--format-version', '1', '--locked', '--offline'], cwd=ROOT, text=True))
BINARY = Path(os.environ.get('AIRUST_REFERENCE_BINARY', str(Path(metadata['target_directory']) / 'debug/examples/reference_workspace')))
DIGEST = '1' * 64


class FakeModel(BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass

    def reply(self, value, status=200):
        data = json.dumps(value).encode()
        self.send_response(status)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Content-Length', str(len(data)))
        self.end_headers()
        try:
            self.wfile.write(data)
        except (BrokenPipeError, ConnectionResetError):
            pass

    def do_GET(self):
        self.server.paths.append(self.path)
        mode = self.server.mode
        if mode == 'http':
            return self.reply({'error': 'synthetic outage'}, 503)
        if mode == 'redirect':
            self.send_response(302)
            self.send_header('Location', 'http://127.0.0.1:1/must-not-follow')
            self.end_headers()
            return
        digest = '0' * 64 if mode == 'digest_before' or mode == 'digest_after' and self.server.posts else DIGEST
        if mode == 'oversized':
            return self.reply({'padding': 'x' * (2 * 1024 * 1024 + 1)})
        self.reply({'models': [{'name': 'synthetic:fixture', 'digest': digest}]})

    def do_POST(self):
        self.server.paths.append(self.path)
        body = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
        self.server.bodies.append(body)
        self.server.posts += 1
        self.server.embedding_started.set()
        if self.server.mode == 'slow':
            time.sleep(.25)
        vectors = []
        for text in body['input']:
            vectors.append([float(text.lower().count('rust') + 1), float(text.lower().count('memory') + 1), 1.0])
        if self.server.mode == 'dimension':
            vectors = [[1.0, 2.0] for _ in vectors]
        elif self.server.mode == 'zero':
            vectors = [[0.0, 0.0, 0.0] for _ in vectors]
        elif self.server.mode == 'nonfinite':
            vectors = [[float('nan'), 1.0, 1.0] for _ in vectors]
        elif self.server.mode == 'count':
            vectors = []
        self.reply({'embeddings': vectors})


class ProfileWorkflowTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        sources = self.root / 'sources'
        sources.mkdir()
        (sources / 'source.md').write_text('Café — Rust ownership protects memory.\n\nTokyo Unicode 東京.\n')
        self.workspace = self.root / 'workspace'
        self.run_cli('build', sources, self.workspace, 'manuals')
        self.server = ThreadingHTTPServer(('127.0.0.1', 0), FakeModel)
        self.server.mode = 'normal'
        self.server.paths, self.server.bodies, self.server.posts = [], [], 0
        self.server.embedding_started = threading.Event()
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()
        self.addCleanup(self.stop_server)
        self.cache = self.root / 'vectors.json'
        self.config = self.root / 'hybrid.json'
        self.write_config()
        self.run_cli('profile-build', self.workspace, self.config, self.cache)
        self.server.posts = 0

    def stop_server(self):
        self.server.shutdown()
        self.server.server_close()
        self.thread.join(timeout=2)

    def write_config(self, policy='error', **changes):
        value = {'schema_version': 1, 'profile': 'hybrid', 'failure_policy': policy,
                 'embedding': {'url': f'http://127.0.0.1:{self.server.server_port}', 'model': 'synthetic:fixture',
                               'digest': DIGEST, 'dimensions': 3, 'timeout_ms': 1000}, 'vectors': 'vectors.json', 'reranker': None}
        value.update(changes)
        self.config.write_text(json.dumps(value))

    def run_cli(self, *args, check=True):
        result = subprocess.run([str(BINARY), *map(str, args)], text=True, capture_output=True, timeout=10)
        self.assertEqual(result.returncode == 0, check, result.stderr)
        if not check:
            self.assertEqual(result.stdout, '')
        return result

    def test_live_hybrid_cli_and_stdio_use_same_recipe_and_source(self):
        cli = json.loads(self.run_cli('query', self.workspace, 'Rust memory', '--profile', self.config).stdout)
        self.assertIn(DIGEST, cli['retrieval']['recipe'])
        self.assertIsNone(cli['retrieval']['fallback_reason'])
        self.assertTrue(cli['retrieval']['hits'])
        with StdioClient([BINARY, 'serve', self.workspace, '--profile', self.config]) as client:
            result = client.call('airust_search', {'query': 'Rust memory', 'limit': 6, 'excerpt_bytes': 2000})
            self.assertFalse(result.get('isError'))
            data = result['structuredContent']['data']
            self.assertEqual(data['retrieval']['recipe'], cli['retrieval']['recipe'])
            self.assertEqual([h['id'] for h in data['retrieval']['hits']], [h['id'] for h in cli['retrieval']['hits']])
            citation = cli['citations'][0]
            read = client.call('airust_read_passage', {k: citation[k] for k in ['document_id', 'revision', 'start_byte', 'end_byte']})
            self.assertEqual(read['structuredContent']['data']['text'], cli['retrieval']['hits'][0]['excerpt'])
            tfidf = client.call('airust_search', {'query': 'Rust', 'algorithm': 'tfidf'})
            self.assertTrue(tfidf['isError'], 'model fusion must not silently ignore a TF-IDF request')
        self.assertTrue(all(p in {'/api/tags', '/api/embed'} for p in self.server.paths))
        self.assertTrue(all(b['truncate'] is False and b['model'] == 'synthetic:fixture' for b in self.server.bodies))

    def test_provider_errors_are_strict_or_explicit_lexical_fallback(self):
        for mode in ['http', 'redirect', 'digest_before', 'digest_after', 'dimension', 'zero', 'nonfinite', 'count', 'oversized']:
            with self.subTest(mode=mode):
                self.server.mode, self.server.posts = mode, 0
                self.write_config()
                self.run_cli('query', self.workspace, 'Rust memory', '--profile', self.config, check=False)
                self.server.posts = 0
                self.write_config(policy='lexical_fallback')
                value = json.loads(self.run_cli('query', self.workspace, 'Rust memory', '--profile', self.config).stdout)
                self.assertIn('fallback', value['retrieval']['recipe'])
                self.assertTrue(value['retrieval']['fallback_reason'])
                self.assertTrue(value['retrieval']['hits'])

    def test_invalid_cache_or_config_never_becomes_setup_fallback(self):
        original = self.cache.read_bytes()
        self.write_config(policy='lexical_fallback')
        self.cache.write_bytes(original[:-10])
        self.run_cli('query', self.workspace, 'Rust', '--profile', self.config, check=False)
        self.cache.write_bytes(original)
        self.write_config(vectors='missing.json')
        self.run_cli('query', self.workspace, 'Rust', '--profile', self.config, check=False)
        self.write_config(extra=True)
        self.run_cli('query', self.workspace, 'Rust', '--profile', self.config, check=False)
        self.write_config()
        value = json.loads(self.config.read_text())
        value['embedding']['url'] = 'https://example.invalid/'
        self.config.write_text(json.dumps(value))
        self.run_cli('query', self.workspace, 'Rust', '--profile', self.config, check=False)
        self.assertEqual(self.cache.read_bytes(), original)

    def test_changed_workspace_and_failed_preparation_leave_original_cache(self):
        sources = self.root / 'changed-sources'
        sources.mkdir()
        (sources / 'source.md').write_text('Different Rust source.')
        changed = self.root / 'changed-workspace'
        self.run_cli('build', sources, changed, 'manuals')
        self.run_cli('query', changed, 'Rust', '--profile', self.config, check=False)
        original = self.cache.read_bytes()
        self.run_cli('profile-build', self.workspace, self.config, self.cache, check=False)
        self.assertEqual(original, self.cache.read_bytes())
        self.server.mode = 'digest_after'
        self.server.posts = 0
        destination = self.root / 'failed-cache.json'
        self.run_cli('profile-build', self.workspace, self.config, destination, check=False)
        self.assertFalse(destination.exists())

    def test_deadline_is_not_a_successful_fallback_and_server_recovers(self):
        self.write_config(policy='lexical_fallback')
        with StdioClient([BINARY, 'serve', self.workspace, '--profile', self.config]) as client:
            self.server.mode = 'slow'
            result = client.call('airust_search', {'query': 'Rust', 'timeout_ms': 30})
            self.assertTrue(result['isError'])
            time.sleep(.3)
            self.server.mode = 'normal'
            result = client.call('airust_search', {'query': 'Rust', 'timeout_ms': 2000, 'limit': 1, 'excerpt_bytes': 16})
            self.assertFalse(result.get('isError'))
            hits = result['structuredContent']['data']['retrieval']['hits']
            self.assertLessEqual(len(hits), 1)
            self.assertTrue(all(len(h['excerpt'].encode()) <= 16 for h in hits))

    def test_explicit_bm25_profile_requires_no_service_and_no_assets(self):
        self.write_config(profile='bm25', embedding=None, vectors=None)
        requests = len(self.server.paths)
        value = json.loads(self.run_cli('query', self.workspace, 'Rust', '--profile', self.config).stdout)
        self.assertEqual(value['retrieval']['recipe'], 'airust-pilot-bm25-v1')
        with StdioClient([BINARY, 'serve', self.workspace, '--profile', self.config]) as client:
            value = client.call('airust_search', {'query': 'Rust', 'algorithm': 'tfidf'})
            self.assertEqual(value['structuredContent']['data']['retrieval']['recipe'], 'airust-pilot-tfidf-v1')
        self.assertEqual(len(self.server.paths), requests)

    def test_cancellation_does_not_publish_a_late_model_success(self):
        self.server.embedding_started.clear()
        self.server.mode = 'slow'
        with StdioClient([BINARY, 'serve', self.workspace, '--profile', self.config]) as client:
            client.send({'jsonrpc': '2.0', 'id': 99, 'method': 'tools/call', 'params': {
                'name': 'airust_search', 'arguments': {'query': 'Rust', 'timeout_ms': 2000}}})
            self.assertTrue(self.server.embedding_started.wait(timeout=2))
            client.send({'jsonrpc': '2.0', 'method': 'notifications/cancelled', 'params': {'requestId': 99, 'reason': 'synthetic cancellation test'}})
            time.sleep(.35)
            while True:
                try:
                    value = client.messages.get_nowait()
                except queue.Empty:
                    break
                if isinstance(value, dict) and value.get('id') == 99:
                    self.assertTrue('error' in value or value.get('result', {}).get('isError'), 'cancelled inference published success')
            self.server.mode = 'normal'
            result = client.call('airust_search', {'query': 'Rust', 'timeout_ms': 2000})
            self.assertFalse(result.get('isError'))


if __name__ == '__main__':
    unittest.main()
