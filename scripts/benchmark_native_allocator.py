"""Measure a macOS application-launch allocator policy using the same native binary.

No global environment changes, new models, server launches or library allocator
hooks. Run after all compilation, fuzzing and interpreter work has finished.
"""
import argparse
from datetime import datetime, timezone
import json
from pathlib import Path
import sys

from compare_rerank_profiles import compare, put, sha, worker
from reference_dataset import ROOT, validate
from run_reference_evaluation import host_identity


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for option in ['binary', 'workspace', 'profile', 'output']:
        parser.add_argument('--' + option, required=True, type=Path)
    parser.add_argument('--repetitions', type=int, default=1)
    args = parser.parse_args()
    if sys.platform != 'darwin':
        parser.error('this experiment is specific to the macOS process allocator')
    if not 1 <= args.repetitions <= 3:
        parser.error('repetitions must be 1..3')
    for name in ['binary', 'workspace', 'profile']:
        setattr(args, name, getattr(args, name).resolve(strict=True))
    with args.profile.open('rb') as stream:
        raw = stream.read(16 * 1024 + 1)
    if len(raw) > 16 * 1024:
        parser.error('profile exceeds 16KiB')
    config = json.loads(raw)
    if config.get('profile') != 'hybrid_rerank' or config.get('failure_policy') != 'error':
        parser.error('a strict hybrid_rerank profile is required')
    if config['reranker'].get('candidates') not in [None, 20]:
        parser.error('keep the actual 20-candidate prefix')
    identity, _ = validate()
    cases = [json.loads(line) for line in (ROOT / 'tests/fixtures/reference-v1/cases.jsonl').read_text().splitlines() if line]
    args.output.mkdir(parents=True, exist_ok=False)
    launches = {'default': ['/usr/bin/env', '-u', 'MallocSpaceEfficient'],
                'space-efficient': ['/usr/bin/env', 'MallocSpaceEfficient=1']}
    policy = {'frozen_at': datetime.now(timezone.utc).isoformat(),
              'binary_sha256': sha(args.binary), 'profile_sha256': sha(args.profile),
              'workspace_sha256': sha(args.workspace / 'workspace.json'),
              'dataset_identity': identity['identity_sha256'],
              'method_sha256': sha(Path(__file__)),
              'shared_worker_method_sha256': sha(ROOT / 'scripts/compare_rerank_profiles.py'),
              'client_method_sha256': sha(ROOT / 'scripts/mcp_client.py'),
              'campaigns': 2, 'cases': len(cases), 'warmups_per_case': 1,
              'timed_repetitions': args.repetitions, 'child_only_launch_prefixes': launches,
              'score_absolute_tolerance': 1e-4, 'score_relative_tolerance': 1e-5,
              'maximum_repeated_p95_ratio_for_optional_policy': 1.10,
              'host': host_identity(),
              'origin': 'Frozen assistant-authored reference cases; no independent customer/human labels.',
              'scope': 'Complete warm MCP requests; startup/initial requests retained separately; same binary/model/profile. This does not measure repeated model drops, system/GPU memory, energy or 1GiB-device support.',
              'decision': 'An optional process-launch experiment, never an automatic library default.'}
    put(args.output / 'policy.json', policy)
    rows = []
    for campaign in [1, 2]:
        order = ['default', 'space-efficient'] if campaign == 1 else ['space-efficient', 'default']
        results = {}
        for label in order:
            results[label] = worker(args.output, f'campaign-{campaign}-{label}', args.binary,
                                    args.workspace, args.profile, cases, args.repetitions,
                                    launch_prefix=launches[label])
        comparison = compare(results['default'], results['space-efficient'], cases)
        put(args.output / f'campaign-{campaign}-comparison.json', comparison)
        rows.append({'campaign': campaign, 'default': results['default'][0],
                     'space_efficient': results['space-efficient'][0],
                     'p95_ratio': comparison['p95_ratio_to_baseline'],
                     'peak_rss_ratio': comparison['memory_ratio_to_baseline'],
                     'slower_queries': len(comparison['all_slower_queries']),
                     'known_quote_losses': comparison['all_known_quote_at_6_losses']})
    report = {'completed_at': datetime.now(timezone.utc).isoformat(), 'campaigns': rows,
              'same_top_six_and_other_evidence': True,
              'no_known_quote_losses': all(not r['known_quote_losses'] for r in rows),
              'repeated_p95_cost_within_declared_10_percent': all(r['p95_ratio'] <= 1.10 for r in rows),
              'all_complete': True, 'native_outside_miri': True,
              'global_environment_changed': False, 'independent_quality_claim': False}
    put(args.output / 'report.json', report)
    if not report['no_known_quote_losses'] or not report['repeated_p95_cost_within_declared_10_percent']:
        raise SystemExit('optional allocator policy exceeded a declared gate; retain all observations')


if __name__ == '__main__':
    main()
