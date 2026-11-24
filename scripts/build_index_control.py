import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess


root = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description='Build the same external index control against registry 0.5.2 and the current checkout. No source worktree is changed.')
parser.add_argument('new_directory', type=Path)
parser.add_argument('--offline', action='store_true')
args = parser.parse_args()
base = args.new_directory.resolve()
base.mkdir(parents=True, exist_ok=False)
template = (root / 'docs/validation-reference/refactor-consumer.toml').read_text()
source = root / 'docs/validation-reference/refactor-consumer.rs'
env = dict(os.environ, CARGO_TARGET_DIR=str(base / 'target'))
for kind in ['baseline', 'candidate']:
    directory = base / kind
    (directory / 'src').mkdir(parents=True)
    shutil.copyfile(source, directory / 'src/main.rs')
    manifest = template if kind == 'baseline' else template.replace('version = "=0.5.2",', 'path = ' + json.dumps(str(root)) + ',')
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
receipt = {str(path.relative_to(base)): hashlib.sha256(path.read_bytes()).hexdigest() for path in base.rglob('*') if path.is_file() and 'target' not in path.relative_to(base).parts}
(base / 'build-receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
print('Built four external controls in ' + str(base))
