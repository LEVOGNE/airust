"""Run finite synthetic scale/retained-reader controls against built baseline/candidate workers."""
import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import statistics

from reference_dataset import ROOT
from run_reference_evaluation import execute, host_identity


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary-directory', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    policy_path = ROOT / 'docs/evaluation/pilot-0.6.0-measurement-policy.json'
    policy = json.loads(policy_path.read_text())['finite_scale']
    metadata = json.loads((args.binary_directory / 'build-metadata.json').read_text())
    if metadata['baseline_version'] != '0.5.3' or metadata['consumer_sha256'] != sha(ROOT / 'examples/stress_index.rs'):
        raise ValueError('wrong baseline or stress worker source')
    args.output.mkdir(parents=True, exist_ok=False)
    binaries = {build + '-' + storage: args.binary_directory / (build + '-' + storage) for build in ['baseline', 'candidate'] for storage in ['default', 'shared']}
    report = {'format_version': 1, 'started_at': datetime.now(timezone.utc).isoformat(), 'host': host_identity(),
              'policy_sha256': sha(policy_path), 'method_sha256': sha(Path(__file__)), 'worker_sha256': metadata['consumer_sha256'],
              'binary_sha256': {name: sha(path) for name, path in binaries.items()}, 'baseline_version': '0.5.3', 'campaigns': [], 'failures': [],
              'limits': 'Finite synthetic short-record workloads, no model/storage/document capacity or endurance claim. Exact-child RSS is whole-process memory, not isolated index heap. Every slower aggregate remains.'}
    cases = [('scale', size, policy['pairs_per_size']) for size in policy['sizes']] + [('clone', policy['clone_entries'], policy['clone_pairs'])]
    for campaign in range(policy['campaigns']):
        summaries = []
        for mode, size, pairs in cases:
            for storage in ['default', 'shared']:
                observations = []
                for pair in range(pairs):
                    order = ['baseline', 'candidate'] if (campaign + pair) % 2 == 0 else ['candidate', 'baseline']
                    paired = {}
                    for build in order:
                        name = f'campaign-{campaign + 1}-{mode}-{size}-{storage}-pair-{pair + 1}-{build}'
                        log = args.output / (name + '.jsonl')
                        usage = execute([str(binaries[build + '-' + storage].resolve()), mode, str(size)], log, timeout=600)
                        worker = json.loads(log.read_text())
                        observations.append({'build': build, 'pair': pair + 1, 'worker': worker, 'measurement': usage, 'raw': log.name})
                        paired[build] = worker
                    if mode == 'scale' and paired['baseline']['score_signature'] != paired['candidate']['score_signature']:
                        raise ValueError('million/scale score signatures differ; raw outcomes retained')
                    if mode == 'clone' and not all(worker['old_views_unchanged'] for worker in paired.values()):
                        raise ValueError('retained reader changed')
                metric = lambda row: row['worker']['selective']['p95_ms'] if mode == 'scale' else statistics.median(row['worker']['clone_ms'])
                grouped = {build: {'median_build_ms': statistics.median(row['worker']['build_ms'] for row in observations if row['build'] == build),
                                  'median_peak_rss_bytes': statistics.median(row['measurement']['peak_rss_bytes'] for row in observations if row['build'] == build),
                                  'median_query_or_clone_ms': statistics.median(metric(row) for row in observations if row['build'] == build)} for build in ['baseline', 'candidate']}
                ratios = {key: grouped['candidate'][key] / grouped['baseline'][key] for key in grouped['baseline']}
                summary = {'mode': mode, 'entries': size, 'storage': storage, 'observations': grouped, 'candidate_to_baseline_ratios': ratios}
                summaries.append(summary)
                (args.output / f'campaign-{campaign + 1}-{mode}-{size}-{storage}-observations.json').write_text(json.dumps(observations, indent=2) + '\n')
                print(json.dumps({'campaign': campaign + 1, **summary}), flush=True)
        report['campaigns'].append(summaries)
    for index, case in enumerate(report['campaigns'][0]):
        for metric in ['median_build_ms', 'median_peak_rss_bytes', 'median_query_or_clone_ms']:
            if all(campaign[index]['candidate_to_baseline_ratios'][metric] > policy['reproducible_regression_ratio_maximum'] for campaign in report['campaigns']):
                report['failures'].append({'mode': case['mode'], 'entries': case['entries'], 'storage': case['storage'], 'metric': metric})
    report['passed'] = not report['failures']
    report['finished_at'] = datetime.now(timezone.utc).isoformat()
    (args.output / 'summary.json').write_text(json.dumps(report, indent=2) + '\n')
    (args.output / 'artifact-manifest.json').write_text(json.dumps({p.name: {'sha256': sha(p), 'bytes': p.stat().st_size} for p in sorted(args.output.iterdir()) if p.is_file()}, indent=2) + '\n')
    if report['failures']:
        raise SystemExit('Reproduced finite-scale regression; retain and inspect every measurement.')


if __name__ == '__main__':
    main()
