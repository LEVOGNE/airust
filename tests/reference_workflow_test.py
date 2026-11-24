import asyncio
import json
from pathlib import Path
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
metadata = json.loads(subprocess.check_output(['cargo', 'metadata', '--no-deps', '--format-version', '1', '--locked', '--offline'], cwd=ROOT, text=True))
BINARY = Path(metadata['target_directory']) / 'debug/examples/reference_workspace'


class ReferenceWorkflowTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.sources = self.root / 'sources'
        self.sources.mkdir()
        (self.sources / 'guide.md').write_text('Introduction.\n\nCafe\u0301 — Rust ownership protects memory.\n\nOther information.')
        self.workspace = self.root / 'workspace'
        self.run_cli('build', self.sources, self.workspace, 'manuals')

    def run_cli(self, *args, check=True):
        return subprocess.run([str(BINARY), *map(str, args)], text=True, capture_output=True, timeout=10, check=check)

    def test_cli_delivers_complete_json_and_exact_source_coordinates(self):
        result = self.run_cli('query', self.workspace, 'Café Rust memory')
        value = json.loads(result.stdout)
        self.assertEqual(value['answerability'], 'not_assessed')
        self.assertEqual(value['source_integrity'], 'verified')
        self.assertLessEqual(len(result.stdout.encode()), 65537)
        self.assertTrue(value['citations'])
        stored = json.loads(json.loads((self.workspace / 'workspace.json').read_text())['payload'])
        snapshot = json.loads(stored['documents'][0]['snapshot'])
        text = snapshot['text'].encode()
        for hit, citation in zip(value['retrieval']['hits'], value['citations']):
            self.assertEqual(text[citation['start_byte']:citation['end_byte']].decode(), hit['excerpt'])

    def test_existing_destination_and_modified_workspace_fail_without_success_output(self):
        path = self.workspace / 'workspace.json'
        original = path.read_bytes()
        result = self.run_cli('build', self.sources, self.workspace, 'manuals', check=False)
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(path.read_bytes(), original)
        value = json.loads(original)
        value['payload'] = value['payload'].replace('manuals', 'changed')
        path.write_text(json.dumps(value))
        result = self.run_cli('query', self.workspace, 'Rust', check=False)
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(result.stdout, '')

    def test_real_stdio_search_and_read_use_the_saved_document_revision(self):
        citation_reply = json.loads(self.run_cli('query', self.workspace, 'Rust').stdout)
        citation = citation_reply['citations'][0]
        expected_text = citation_reply['retrieval']['hits'][0]['excerpt']

        async def exercise():
            process = await asyncio.create_subprocess_exec(str(BINARY), 'serve', str(self.workspace), stdin=asyncio.subprocess.PIPE, stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.PIPE)

            async def send(value):
                process.stdin.write((json.dumps(value) + '\n').encode())
                await process.stdin.drain()

            async def receive(identifier):
                for _ in range(20):
                    line = await asyncio.wait_for(process.stdout.readline(), timeout=5)
                    self.assertTrue(line, 'unexpected server EOF')
                    reply = json.loads(line)
                    if reply.get('id') == identifier:
                        self.assertNotIn('error', reply)
                        return reply['result']
                self.fail('too many unrelated notifications')

            try:
                await send({'jsonrpc': '2.0', 'id': 0, 'method': 'initialize', 'params': {'protocolVersion': '2025-11-25', 'capabilities': {}, 'clientInfo': {'name': 'reference-workflow-test', 'version': '1'}}})
                await receive(0)
                await send({'jsonrpc': '2.0', 'method': 'notifications/initialized'})
                await send({'jsonrpc': '2.0', 'id': 1, 'method': 'tools/list'})
                self.assertEqual(len((await receive(1))['tools']), 5)
                await send({'jsonrpc': '2.0', 'id': 2, 'method': 'tools/call', 'params': {'name': 'airust_search', 'arguments': {'query': 'Rust', 'limit': 6}}})
                result = await receive(2)
                self.assertFalse(result.get('isError', False))
                sources = result['structuredContent']['data']['sources']
                self.assertTrue(sources)
                self.assertTrue(all(source['status'] == 'snapshot_bound' for source in sources))
                arguments = {name: citation[name] for name in ['document_id', 'revision', 'start_byte', 'end_byte']}
                await send({'jsonrpc': '2.0', 'id': 3, 'method': 'tools/call', 'params': {'name': 'airust_read_passage', 'arguments': arguments}})
                result = await receive(3)
                self.assertFalse(result.get('isError', False))
                self.assertEqual(result['structuredContent']['data']['text'], expected_text)
                process.stdin.close()
                await asyncio.wait_for(process.wait(), timeout=5)
                self.assertEqual(process.returncode, 0)
            finally:
                if process.returncode is None:
                    process.kill()
                    await process.wait()

        asyncio.run(exercise())


if __name__ == '__main__':
    if not BINARY.is_file():
        raise SystemExit('First build: cargo build --locked --features mcp --example reference_workspace')
    unittest.main()
