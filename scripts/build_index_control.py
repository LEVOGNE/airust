import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import re


root = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description='Build an identical external control against an explicit registry baseline and this checkout.')
parser.add_argument('new_directory', type=Path)
parser.add_argument('--offline', action='store_true')
parser.add_argument('--baseline-version', default='0.5.2')
args = parser.parse_args()
if not re.fullmatch(r'[0-9]+\.[0-9]+\.[0-9]+', args.baseline_version):
    parser.error('baseline version must be an exact numeric release')
base = args.new_directory.resolve()
base.mkdir(parents=True, exist_ok=False)
template = (root / 'docs/validation-reference/refactor-consumer.toml').read_text()
source = root / 'docs/validation-reference/refactor-consumer.rs'
env = dict(os.environ, CARGO_TARGET_DIR=str(base / 'target'))
for kind in ['baseline', 'candidate']:
    directory = base / kind
    (directory / 'src').mkdir(parents=True)
    shutil.copyfile(source, directory / 'src/main.rs')
    manifest = template.replace('version = "=0.5.2",', 'version = "=' + args.baseline_version + '",') if kind == 'baseline' else template.replace('version = "=0.5.2",', 'path = ' + json.dumps(str(root)) + ',')
    (directory / 'Cargo.toml').write_text(manifest)
    cargo = ['cargo', '--offline'] if args.offline else ['cargo']
    subprocess.run(cargo + ['generate-lockfile', '--manifest-path', str(directory / 'Cargo.toml')], env=env, check=True)
    for storage in ['default', 'shared']:
        command = cargo + ['build', '--release', '--locked', '--manifest-path', str(directory / 'Cargo.toml')]
        if storage == 'shared':
            command += ['--features', 'shared']
        with (base / ('build-' + kind + '-' + storage + '.log')).open('xb') as log:
            subprocess.run(command, env=env, check=True, stdout=log, stderr=subprocess.STDOUT)
        shutil.copyfile(base / 'target/release/airust-refactor-consumer', base / (kind + '-' + storage))
        (base / (kind + '-' + storage)).chmod(0o755)
(base / 'build-metadata.json').write_text(json.dumps({'baseline_version': args.baseline_version, 'consumer_sha256': hashlib.sha256(source.read_bytes()).hexdigest(), 'candidate_commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip()}, indent=2) + '\n')
receipt = {str(path.relative_to(base)): hashlib.sha256(path.read_bytes()).hexdigest() for path in base.rglob('*') if path.is_file() and 'target' not in path.relative_to(base).parts}
(base / 'build-receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
print('Built four external controls in ' + str(base))
