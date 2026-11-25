"""Run the finite AIRust interpreter scope; retain every result and exclusion."""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import time


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('new_output', type=Path)
    parser.add_argument('--toolchain', default='nightly', help='Installed toolchain, optionally a dated nightly for reproduction.')
    parser.add_argument('--groups', nargs='+', help='Run an explicit subset; the receipt records it as a subset, not the whole scope.')
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    output = args.new_output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    large = ['empty_vocabulary_releases_its_hash_allocation_after_removal',
             'sparse_vocabulary_shrinks_without_changing_retained_readers',
             'token_ties_keep_id_order_and_character_scores_for_long_text',
             'token_top_k_preserves_full_comparison_scores_and_unicode_explanations']
    cases = [
        ('input', ['--no-default-features', '--lib', 'input::'], [], True),
        ('model-assets', ['--no-default-features', '--lib', 'model_assets::'], [], True),
        ('model-worker', ['--no-default-features', '--features', 'local-reranking', '--lib',
                          'reranking::process::'], [], False),
        ('training-knowledge', ['--no-default-features', '--lib', 'knowledge::'], ['test_save_load_roundtrip'], True),
        ('training-properties', ['--no-default-features', '--test', 'knowledge_snapshots'], [], True),
        ('index', ['--no-default-features', '--features', 'shared-index,execution', '--lib', 'index::'], large, False),
        ('core', ['--no-default-features', '--features', 'shared-index', '--test', 'core'], [], False),
        ('stability-state', ['--no-default-features', '--features', 'shared-index,execution', '--test', 'stability_state'], [], False),
        ('mcp-response', ['--no-default-features', '--features', 'mcp', '--lib', 'mcp::response_tests'], [], False),
        ('mcp-backend', ['--no-default-features', '--features', 'mcp', '--lib', 'mcp::search_backend_tests'], [], False),
        ('pilot', ['--no-default-features', '--features', 'mcp', '--example', 'reference_workspace'],
         ['independent_python_corpus_and_rust_snapshot_passages_agree'], True),
        ('profile-config', ['--no-default-features', '--features', 'pilot', '--example', 'reference_workspace',
                            'reference_profiles::configuration_tests::'], [], True),
    ]
    full_scope = [name for name, *_ in cases]
    if args.groups:
        unknown = set(args.groups) - set(full_scope)
        if unknown:
            parser.error('unknown groups: ' + ', '.join(sorted(unknown)))
        cases = [case for case in cases if case[0] in args.groups]
    report = {'format_version': 1, 'full_scope_job_names': full_scope,
              'requested_subset': args.groups, 'subset_is_not_full_scope': bool(args.groups), 'started_at': datetime.now(timezone.utc).isoformat(),
              'commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip(),
              'worktree_status': subprocess.check_output(['git', 'status', '--short'], cwd=root, text=True),
              'source_inputs_sha256': {
                  str(path.relative_to(root)): hashlib.sha256(path.read_bytes()).hexdigest()
                  for path in [*(root / 'src').rglob('*.rs'), *(root / 'tests').rglob('*.rs'),
                               *(root / 'examples').rglob('*.rs'), root / 'Cargo.toml', root / 'Cargo.lock',
                               root / 'build.rs', root / 'knowledge/train.json', root / 'README.md']
              },
              'toolchain': args.toolchain,
              'method_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
              'rustc': subprocess.check_output(['rustc', '+' + args.toolchain, '-vV'], text=True),
              'miri': subprocess.check_output(['cargo', '+' + args.toolchain, 'miri', '--version'], text=True).strip(),
              'expected_jobs': len(cases), 'jobs': [], 'exclusions': {'large_index_native_only': large,
                                       'training_directory_fsync_native_only': ['test_save_load_roundtrip', 'successful save paths in bounded_load and training property fixtures'],
                                       'async_mcp_native_only': ['final_reply_work_keeps_admission_and_shutdown_until_cancelled_work_returns',
                                                                 'shutdown_does_not_wait_for_other_server_clones_worker_permits'],
                                       'native_inference_and_http': 'Outside interpreter scope; use actual native runs and loopback integration tests.',
                                       'model_worker_processes': 'Only pure protocol/input/timeout validation runs under Miri; actual OS subprocesses and ONNX FFI run natively.',
                                       'full_reference_example': 'The large independently generated corpus parity test runs natively; small workspace/pilot tests run here.'},
              'limitations': ['Finite test scope, not proof of absence of undefined behavior.',
                              'Pilot filesystem tests disable Miri isolation explicitly; they use temporary fixtures.',
                              'The stability-state and training property tests use their declared cfg(miri) reduced workloads.',
                              'macOS directory F_FULLFSYNC is unavailable in Miri; production persistence is unchanged and native publication controls remain mandatory.',
                              'Native model-asset tests cross a 64KiB chunk boundary; the Miri checksum fixture is reduced to 71 bytes, without FFI.']}
    for name, options, skips, filesystem in cases:
        flags = '-Zmiri-tree-borrows -Zmiri-no-extra-rounding-error' + (' -Zmiri-disable-isolation' if filesystem else '')
        command = ['cargo', '+' + args.toolchain, 'miri', 'test', '--locked', *options, '--', '--test-threads=1']
        for skip in skips:
            command += ['--skip', skip]
        env = dict(os.environ, MIRIFLAGS=flags)
        started = time.monotonic()
        path = output / (name + '.log')
        with path.open('xb') as stream:
            try:
                result = subprocess.run(command, cwd=root, env=env, stdout=stream, stderr=subprocess.STDOUT, timeout=1800)
                status = result.returncode
            except subprocess.TimeoutExpired:
                status = 124
        raw = path.read_bytes()
        counts = re.findall(r'test result: ok\. (\d+) passed;', raw.decode(errors='replace'))
        entry = {'name': name, 'command': command, 'MIRIFLAGS': flags, 'exit_code': status,
                 'passed_tests': sum(map(int, counts)), 'elapsed_seconds': time.monotonic() - started,
                 'log_sha256': hashlib.sha256(raw).hexdigest()}
        report['jobs'].append(entry)
        report['completed_jobs'] = len(report['jobs'])
        report['scope_completed'] = len(report['jobs']) == len(cases)
        report['passed'] = report['scope_completed'] and all(job['exit_code'] == 0 and job['passed_tests'] > 0 for job in report['jobs'])
        report['finished_at'] = datetime.now(timezone.utc).isoformat()
        (output / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
        print(json.dumps(entry), flush=True)
    if not report['passed']:
        raise SystemExit('Miri scope contains a failed, empty or unavailable job; inspect all retained logs.')


if __name__ == '__main__':
    main()
