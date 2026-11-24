import json
from pathlib import Path
import re
import tomllib


root = Path(__file__).resolve().parents[1]
manifest = tomllib.loads((root / 'Cargo.toml').read_text())
support = json.loads((root / 'docs/support-matrix.json').read_text())
if set(manifest['features']) != set(support['features']) or manifest['features']['default'] != support['default_features']:
    raise SystemExit('Declared features and support matrix differ.')
for name, entry in support['features'].items():
    if not (root / 'docs' / entry['guide']).is_file():
        raise SystemExit('Missing feature contract: ' + name)
    expected = '1.92' if name in {'local-embeddings', 'local-reranking'} else manifest['package']['rust-version']
    if entry['minimum_rust'] != expected:
        raise SystemExit('Minimum Rust declaration differs: ' + name)
entry_points = ['README.md', 'STARTUP.md', 'docs/API-COOKBOOK.md', 'docs/REFERENCE-APPLICATION.md', 'docs/SUPPORT.md', 'docs/PDF-MAINTENANCE.md']
for name in entry_points:
    path = root / name
    for target in re.findall(r'\[[^\]]+\]\(([^)]+)\)', path.read_text()):
        if target.startswith(('https://', 'http://', 'mailto:', '#')):
            continue
        target = target.split('#', 1)[0]
        if target and not (path.parent / target).exists():
            raise SystemExit(f'Broken entry-point link in {name}: {target}')
print(f'Support matrix verified: {len(support["features"])} declared entries; entry-point links exist.')
