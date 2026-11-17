import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


class DownloadTests(unittest.TestCase):
    def run_helper(self, mode="valid", existing=None):
        with tempfile.TemporaryDirectory(prefix="airust-download-test-") as directory:
            root = Path(directory)
            (root / "scripts").mkdir()
            (root / "docs/models").mkdir(parents=True)
            (root / "scripts/download_reranker.py").write_bytes((Path(__file__).resolve().parents[1] / "scripts/download_reranker.py").read_bytes())
            manifest = {"repository": "fixture/model", "revision": "fixed", "files": {"onnx/model.onnx": {"bytes": 3, "sha256": hashlib.sha256(b"abc").hexdigest()}}}
            (root / "docs/models/mmarco-minilm-v2.json").write_text(json.dumps(manifest))
            target = root / "models/onnx/model.onnx"
            if existing is not None:
                target.parent.mkdir(parents=True)
                target.write_bytes(existing)
            script = """import sys,io,runpy,urllib.request
from pathlib import Path
root=Path(sys.argv[1]);mode=sys.argv[2]
def fake_open(*args,**kwargs):
 if mode=='forbid':raise AssertionError('Unexpected download')
 if mode=='race':(root/'models/onnx/model.onnx').write_bytes(b'keep')
 return io.BytesIO({'valid':b'abc','race':b'abc','corrupt':b'abd','large':b'abc-extra','short':b'ab'}.get(mode,b'abc'))
urllib.request.urlopen=fake_open
sys.argv=[str(root/'scripts/download_reranker.py'),str(root/'models')]
runpy.run_path(sys.argv[0],run_name='__main__')
"""
            result = subprocess.run([sys.executable, "-c", script, str(root), mode], capture_output=True, text=True)
            return result.returncode, target.read_bytes() if target.exists() else None, list(target.parent.glob(".airust-download-*")), result.stderr

    def test_complete_valid_asset_is_published(self):
        code, data, temporary, _ = self.run_helper()
        self.assertEqual((code, data, temporary), (0, b"abc", []))

    def test_existing_valid_asset_needs_no_network(self):
        self.assertEqual(self.run_helper("forbid", b"abc")[:3], (0, b"abc", []))

    def test_differing_existing_asset_is_preserved(self):
        code, data, temporary, _ = self.run_helper("forbid", b"keep")
        self.assertNotEqual(code, 0)
        self.assertEqual((data, temporary), (b"keep", []))

    def test_bad_checksum_short_and_oversized_downloads_leave_no_partial_asset(self):
        for mode in ["corrupt", "large", "short"]:
            with self.subTest(mode=mode):
                code, data, temporary, _ = self.run_helper(mode)
                self.assertNotEqual(code, 0)
                self.assertEqual((data, temporary), (None, []))

    def test_concurrent_differing_asset_is_never_overwritten(self):
        code, data, temporary, _ = self.run_helper("race")
        self.assertNotEqual(code, 0)
        self.assertEqual((data, temporary), (b"keep", []))


if __name__ == "__main__":
    unittest.main()
