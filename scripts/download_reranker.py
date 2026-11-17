#!/usr/bin/env python3
import hashlib
import json
import os
from pathlib import Path
import sys
import tempfile
import urllib.request

if len(sys.argv) != 2:
    raise SystemExit("Usage: download_reranker.py MODEL_DIRECTORY")
manifest_path = Path(__file__).resolve().parents[1] / "docs/models/mmarco-minilm-v2.json"
manifest = json.loads(manifest_path.read_text())
root = Path(sys.argv[1]).resolve()
for name, expected in manifest["files"].items():
    path = root / name
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        with path.open("rb") as source:
            valid = path.stat().st_size == expected["bytes"] and hashlib.file_digest(source, "sha256").hexdigest() == expected["sha256"]
        if not valid:
            raise SystemExit(f"Existing asset differs; choose an empty model directory: {path}")
        print(f"Verified existing {name}", flush=True)
        continue
    url = f"https://huggingface.co/{manifest['repository']}/resolve/{manifest['revision']}/{name}"
    request = urllib.request.Request(url, headers={"User-Agent": "AIRust-explicit-reranker-download/0.2.9"})
    temporary = None
    try:
        with urllib.request.urlopen(request, timeout=60) as response, tempfile.NamedTemporaryFile(dir=path.parent, prefix=".airust-download-", delete=False) as target:
            temporary = Path(target.name)
            digest = hashlib.sha256()
            size = 0
            while chunk := response.read(1024 * 1024):
                size += len(chunk)
                if size > expected["bytes"]:
                    raise ValueError(f"Oversized asset: {name}")
                target.write(chunk)
                digest.update(chunk)
            if size != expected["bytes"] or digest.hexdigest() != expected["sha256"]:
                raise ValueError(f"Asset size/checksum mismatch: {name}")
            target.flush()
            os.fsync(target.fileno())
        try:
            os.link(temporary, path)
        except FileExistsError:
            with path.open("rb") as source:
                valid = path.stat().st_size == expected["bytes"] and hashlib.file_digest(source, "sha256").hexdigest() == expected["sha256"]
            if not valid:
                raise ValueError(f"Concurrent differing asset left unchanged: {path}")
        temporary.unlink()
        temporary = None
        print(f"Downloaded and verified {name}", flush=True)
    finally:
        if temporary is not None:
            temporary.unlink(missing_ok=True)
print(f"Ready: {manifest['repository']} at {manifest['revision']}")
