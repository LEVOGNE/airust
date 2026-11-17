#!/usr/bin/env python3
import json
import os
from pathlib import Path
import platform
import resource
import subprocess
import sys
import time

if len(sys.argv) < 3:
    raise SystemExit("Usage: measure.py OUTPUT_JSON EXECUTABLE [ARG ...]")
output = Path(sys.argv[1])
command = sys.argv[2:]
before = resource.getrusage(resource.RUSAGE_CHILDREN)
started = time.perf_counter()
result = subprocess.run(command, capture_output=True, text=True)
elapsed = time.perf_counter() - started
after = resource.getrusage(resource.RUSAGE_CHILDREN)
metadata = {
    "command": command,
    "exit_code": result.returncode,
    "elapsed_seconds": elapsed,
    "user_cpu_seconds": after.ru_utime - before.ru_utime,
    "system_cpu_seconds": after.ru_stime - before.ru_stime,
    "peak_rss_bytes": after.ru_maxrss * (1 if sys.platform == "darwin" else 1024),
    "platform": platform.platform(),
    "logical_cpus": os.cpu_count(),
    "rss_scope": "One child process including all its measured phases and staging",
}
output.parent.mkdir(parents=True, exist_ok=True)
output.with_suffix(".resources.json").write_text(json.dumps(metadata, indent=2) + "\n")
output.with_suffix(".stderr.txt").write_text(result.stderr)
if result.returncode:
    sys.stderr.write(result.stderr)
    raise SystemExit(result.returncode)
parsed = json.loads(result.stdout)
output.write_text(json.dumps(parsed, ensure_ascii=False, indent=2) + "\n")
print(f"{output}: completed in {elapsed:.3f}s; peak RSS {metadata['peak_rss_bytes'] / 1048576:.2f} MiB")
