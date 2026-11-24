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
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    output = args.new_output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    large = ['empty_vocabulary_releases_its_hash_allocation_after_removal',
             'sparse_vocabulary_shrinks_without_changing_retained_readers',
             'token_ties_keep_id_order_and_character_scores_for_long_text',
             'token_top_k_preserves_full_comparison_scores_and_unicode_explanations']
    cases = [
        ('index', ['--no-default-features', '--features', 'shared-index,execution', '--lib', 'index::'], large, False),
        ('core', ['--no-default-features', '--features', 'shared-index', '--test', 'core'], [], False),
        ('stability-state', ['--no-default-features', '--features', 'shared-index,execution', '--test', 'stability_state'], [], False),
        ('mcp-response', ['--no-default-features', '--features', 'mcp', '--lib', 'mcp::response_tests'], [], False),
        ('mcp-backend', ['--no-default-features', '--features', 'mcp', '--lib', 'mcp::search_backend_tests'], [], False),
        ('pilot', ['--no-default-features', '--features', 'mcp', '--example', 'reference_workspace'],
         ['independent_python_corpus_and_rust_snapshot_passages_agree'], True),
    ]
    report = {'format_version': 1, 'started_at': datetime.now(timezone.utc).isoformat(),
              'commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip(),
              'rustc': subprocess.check_output(['rustc', '+nightly', '-vV'], text=True),
              'miri': subprocess.check_output(['cargo', '+nightly', 'miri', '--version'], text=True).strip(),
              'jobs': [], 'exclusions': {'large_index_native_only': large,
                                       'async_mcp_native_only': ['final_reply_work_keeps_admission_and_shutdown_until_cancelled_work_returns',
                                                                 'shutdown_does_not_wait_for_other_server_clones_worker_permits'],
                                       'native_inference_and_http': 'Outside interpreter scope; use actual native runs and loopback integration tests.',
                                       'full_reference_example': 'The large independently generated corpus parity test runs natively; small workspace/pilot tests run here.'},
              'limitations': ['Finite test scope, not proof of absence of undefined behavior.',
                              'Pilot filesystem tests disable Miri isolation explicitly; they use temporary fixtures.',
                              'The stability-state test uses its existing cfg(miri) reduced workload.']}
    for name, options, skips, filesystem in cases:
        flags = '-Zmiri-tree-borrows -Zmiri-no-extra-rounding-error' + (' -Zmiri-disable-isolation' if filesystem else '')
        command = ['cargo', '+nightly', 'miri', 'test', '--locked', *options, '--', '--test-threads=1']
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
        report['passed'] = all(job['exit_code'] == 0 and job['passed_tests'] > 0 for job in report['jobs'])
        report['completed_jobs'] = len(report['jobs'])
        report['finished_at'] = datetime.now(timezone.utc).isoformat()
        (output / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
        print(json.dumps(entry), flush=True)
    if not report['passed']:
        raise SystemExit('Miri scope contains a failed, empty or unavailable job; inspect all retained logs.')


if __name__ == '__main__':
    main()
