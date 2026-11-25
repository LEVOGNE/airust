"""Exercise the model-free host's declared limits and recovery over real stdio."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'scripts'))
from mcp_client import StdioClient

metadata = json.loads(subprocess.check_output(['cargo', 'metadata', '--no-deps', '--format-version', '1', '--locked', '--offline'], cwd=ROOT, text=True))
BINARY = Path(metadata['target_directory']) / 'debug/examples/bounded_mcp'


class BoundedMcpTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.path = self.root / 'entries.json'
        self.path.write_text(json.dumps([{'id': 'rust', 'text': 'Rust borrowing ownership memory Straße 東京'}]))

    def run_host(self, *args):
        return subprocess.run([str(BINARY), *map(str, args)], capture_output=True, timeout=10)

    def test_declared_worker_limit_and_recovery_after_invalid_query(self):
        original = self.path.read_bytes()
        result = self.run_host(self.path, '--check')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout)['worker_limit'], 1)
        with StdioClient([BINARY, self.path]) as client:
            invalid = client.call('airust_search', {'query': 'x' * 1025})
            self.assertTrue(invalid['isError'])
            valid = client.call('airust_search', {'query': 'Rust', 'limit': 6})
            self.assertFalse(valid.get('isError', False))
            self.assertEqual(valid['structuredContent']['data']['retrieval']['hits'][0]['id'], 'rust')
        self.assertEqual(client.usage['exit_code'], 0)
        self.assertEqual(self.path.read_bytes(), original)

    def test_oversized_file_and_invalid_encoding_fail_without_output_or_changes(self):
        with self.path.open('wb') as stream:
            stream.truncate(4 * 1024 * 1024 + 1)
        result = self.run_host(self.path, '--check')
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(result.stdout, b'')
        self.assertEqual(self.path.stat().st_size, 4 * 1024 * 1024 + 1)
        self.path.write_bytes(b'\xff')
        result = self.run_host(self.path, '--check')
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(result.stdout, b'')
        self.assertEqual(self.path.read_bytes(), b'\xff')

    @unittest.skipUnless(hasattr(os, 'mkfifo'), 'requires Unix named pipes')
    def test_nonregular_input_is_rejected_without_waiting_for_a_writer(self):
        pipe = self.root / 'pipe.json'
        os.mkfifo(pipe)
        result = self.run_host(pipe, '--check')
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(result.stdout, b'')
        self.assertTrue(pipe.exists())

    def test_entry_and_record_limits_reject_before_serving(self):
        for values in [[{'id': 'large', 'text': 'x' * 4097}],
                       [{'id': str(i), 'text': 'small'} for i in range(2049)]]:
            original = json.dumps(values).encode()
            self.path.write_bytes(original)
            result = self.run_host(self.path, '--check')
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(result.stdout, b'')
            self.assertEqual(self.path.read_bytes(), original)

    def test_malformed_or_oversized_frames_preserve_documented_recovery(self):
        original = self.path.read_bytes()
        with StdioClient([BINARY, self.path]) as client:
            for data in [b'x' * 20000 + b'\n', b'x' * 20000, b'{invalid}\n']:
                client.process.stdin.write(data)
                client.process.stdin.flush()
                if not data.endswith(b'\n'):
                    client.process.stdin.write(b'\n')
                    client.process.stdin.flush()
                reply = client.call('airust_search', {'query': 'Rust', 'limit': 6})
                self.assertFalse(reply.get('isError', False))
                self.assertEqual(reply['structuredContent']['data']['retrieval']['hits'][0]['id'], 'rust')
        self.assertEqual(client.usage['exit_code'], 0)
        self.assertEqual(self.path.read_bytes(), original)

    def test_actual_inbound_quota_closes_while_client_keeps_input_open(self):
        original = self.path.read_bytes()
        for data in [b'x' * 200000, b'{}\n' * 100]:
            with self.subTest(bytes=len(data), newline=data.endswith(b'\n')):
                process = subprocess.Popen([BINARY, self.path], stdin=subprocess.PIPE,
                                           stdout=subprocess.PIPE, stderr=subprocess.PIPE)
                try:
                    try:
                        process.stdin.write(data)
                        process.stdin.flush()
                    except BrokenPipeError:
                        pass  # The server may reject the budget before the write ends.
                    process.wait(timeout=5)
                    self.assertNotEqual(process.returncode, 0)
                    self.assertEqual(process.stdout.read(65537), b'')
                finally:
                    try:
                        process.stdin.close()
                    except BrokenPipeError:
                        pass
                    if process.poll() is None:
                        process.kill()
                        process.wait(timeout=5)
                    process.stdout.close()
                    process.stderr.close()
                self.assertEqual(self.path.read_bytes(), original)


if __name__ == '__main__':
    unittest.main()
