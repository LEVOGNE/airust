"""Export allowlisted numeric lifecycle evidence, including rejected experiments."""
import argparse
import csv
import hashlib
import json
import math
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SERIAL = [
    ('baseline-30', 'private-replay', 'in_process', 'mixed'),
    ('device-initializers-12', 'private-replay', 'in_process', 'mixed'),
    ('cpu-no-arena-12', 'private-replay', 'in_process', 'mixed'),
    ('combined-allocation-12', 'private-replay', 'in_process', 'mixed'),
    ('process-smoke-8', 'synthetic-resource', 'process-prototype', 'mixed'),
    ('baseline-memory', 'synthetic-resource', 'in_process', 'memory'),
    ('isolated-memory', 'synthetic-resource', 'process-rejected', 'memory'),
    ('isolated-snapshot', 'synthetic-resource', 'process', 'snapshot'),
    ('baseline-snapshot', 'synthetic-resource', 'in_process', 'snapshot'),
    ('final-isolated-snapshot', 'synthetic-resource', 'process', 'snapshot'),
    ('final-baseline-memory', 'synthetic-resource', 'in_process', 'memory'),
]
PAIRS = [
    ('primary-memory-prototype', 'baseline-memory', 'isolated-memory'),
    ('primary-file-comparison', 'baseline-snapshot', 'isolated-snapshot'),
    ('primary-selected-vs-default', 'baseline-memory', 'isolated-snapshot'),
    ('serial-confirmation', 'final-baseline-memory', 'final-isolated-snapshot'),
    ('alternating-diagnostic', 'alternating-baseline', 'alternating-candidate'),
]


def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def read(path):
    assert path.stat().st_size <= 16 * 1024 * 1024
    return json.loads(path.read_text())


def write_csv(out, name, rows):
    assert rows
    fields = list(rows[0])
    with (out / name).open('w', newline='') as stream:
        writer = csv.DictWriter(stream, fieldnames=fields, lineterminator='\n')
        writer.writeheader()
        for row in rows:
            writer.writerow({k: str(v).lower() if isinstance(v, bool) else v for k, v in row.items()})


def p95(values):
    return sorted(values)[math.ceil(len(values) * .95) - 1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--lab', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    lab, out = args.lab.resolve(strict=True), args.output.resolve()
    out.mkdir(parents=True, exist_ok=True)
    fixture = read(ROOT / 'tests/fixtures/native-lifecycle-v1.json')
    public_cases = {case['id']: {d['id'] for d in case['candidates']} for case in fixture}
    runs = {}
    identities, workers, events, scores, timings, rss_samples = [], [], [], [], [], []
    for name, origin, backend, mode in SERIAL:
        report = read(lab / name / 'report.json')
        rows = read(lab / name / 'events.json')
        runs[name] = (rows, origin, backend, mode)
        identities.append({'name': name, 'binary_sha256': report['binary_sha256'],
                           'probe_sha256': report['probe_sha256'],
                           'report_sha256': sha(lab / name / 'report.json'),
                           'events_sha256': sha(lab / name / 'events.json'),
                           'policy_sha256': sha(lab / name / 'policy.json')})
        drops = [r['rss_bytes'] for r in rows if r['stage'] == 'dropped']
        assert len(drops) == report['cycles']
        workers.append({'name': name, 'origin': origin, 'backend': backend, 'mode': mode,
                        'cycles': report['cycles'], 'warmup_cycles': 2,
                        'post_warm_range_bytes': report['post_warm_range_bytes'],
                        'growth_budget_passed': report['rss_growth_budget_passed'],
                        'after_drop_final_rss_bytes': drops[-1],
                        'sampled_tree_peak_rss_bytes': report.get('sampled_sum_peak_rss_bytes', report.get('sampled_peak_rss_bytes')),
                        'wait4_reported_maxrss_bytes': report.get('wait4_reported_maxrss_bytes', report.get('peak_rss_bytes', '')),
                        'native_exit_code': report['exit_code'],
                        'functional_controls_passed': report['functional_controls_passed'],
                        'no_children_after_drop': report.get('no_model_workers_after_drop', 'not-applicable-in-process'),
                        'controller_seconds': report['seconds']})
    paired = read(lab / 'interleaved-final/report.json')
    raw_rss = read(lab / 'interleaved-final/rss-samples.json')
    for index, short in enumerate(['baseline', 'candidate']):
        name = 'alternating-' + short
        report = paired['controls'][index]
        path = lab / 'interleaved-final' / (short + '-events.json')
        rows = read(path)
        backend, mode = ('in_process', 'memory') if index == 0 else ('process', 'snapshot')
        runs[name] = (rows, 'synthetic-resource', backend, mode)
        policy = lab / 'interleaved-final/policy.json'
        identities.append({'name': name, 'binary_sha256': read(policy)['binary_sha256'],
                           'probe_sha256': read(policy)['source_inputs_sha256']['examples/native_model_lifecycle.rs'],
                           'report_sha256': sha(lab / 'interleaved-final/report.json'),
                           'events_sha256': sha(path), 'policy_sha256': sha(policy)})
        peak = 0
        for sample in raw_rss:
            tree = sample['trees'][index]
            children = sum(c['rss_bytes'] for c in tree['children'])
            peak = max(peak, tree['parent_rss_bytes'] + children)
            rss_samples.append({'name': name, 'elapsed_seconds': sample['elapsed_seconds'],
                                'parent_rss_bytes': tree['parent_rss_bytes'],
                                'direct_child_rss_bytes': children, 'direct_children': len(tree['children'])})
        workers.append({'name': name, 'origin': 'synthetic-resource', 'backend': backend, 'mode': mode,
                        'cycles': 30, 'warmup_cycles': 2,
                        'post_warm_range_bytes': report['post_warm_range_bytes'],
                        'growth_budget_passed': report['growth_budget_passed'],
                        'after_drop_final_rss_bytes': report['after_drop_rss_bytes'][-1],
                        'sampled_tree_peak_rss_bytes': peak, 'wait4_reported_maxrss_bytes': 'not-measured',
                        'native_exit_code': report['exit_code'], 'functional_controls_passed': True,
                        'no_children_after_drop': report['all_children_reaped_and_private_files_removed'],
                        'controller_seconds': paired['seconds']})
    for name, (rows, origin, backend, mode) in runs.items():
        loads = {r['cycle']: r['extra']['load_ms'] for r in rows if r['stage'] == 'loaded' and 'load_ms' in r['extra']}
        for r in rows:
            x = r['extra']
            case = x.get('case_id', '') if origin == 'synthetic-resource' else ''
            if case:
                assert case in public_cases
            events.append({'name': name, 'origin': origin, 'cycle': r['cycle'], 'stage': r['stage'],
                           'mode': r['mode'], 'case_id': case, 'parent_rss_bytes': r['rss_bytes'],
                           'last_sampled_child_rss_bytes': r.get('last_sampled_child_rss_bytes',
                                sum(c['rss_bytes'] for c in r.get('sampled_children', [])) if 'sampled_children' in r else 'not-measured'),
                           'controller_elapsed_seconds': r['controller_elapsed_seconds'],
                           'snapshot_items': x.get('snapshot_files', ''),
                           'cancelled_after_native_batches': x.get('cancelled_after_native_batches', ''),
                           'expired_deadline_extra_calls': x.get('expired_deadline_extra_calls', ''),
                           'recovery_scores_bits_equal': x.get('recovery_scores_bits_equal', '')})
            if r['stage'] != 'recovered':
                continue
            if origin == 'synthetic-resource':
                for rank, (entry, bits) in enumerate(x['signature'], 1):
                    assert entry in public_cases[case] and 0 <= bits < 2**64
                    scores.append({'name': name, 'cycle': r['cycle'], 'case_id': case,
                                   'rank': rank, 'entry_id': entry, 'score_f64_bits': bits})
            if 'first_ms' in x:
                timings.append({'name': name, 'cycle': r['cycle'], 'case_id': case,
                                'load_ms': loads[r['cycle']], 'first_ms': x['first_ms'],
                                'recovery_ms': x['recovery_ms']})
    comparisons, slower = [], []
    for name, baseline, candidate in PAIRS:
        b = [r for r in runs[baseline][0] if r['stage'] == 'recovered']
        c = [r for r in runs[candidate][0] if r['stage'] == 'recovered']
        assert len(b) == len(c) == 30
        equal = all(x['cycle'] == y['cycle'] and x['extra']['case_id'] == y['extra']['case_id']
                    and x['extra']['signature'] == y['extra']['signature'] for x, y in zip(b, c))
        assert equal
        row = {'name': name, 'baseline': baseline, 'candidate': candidate,
               'cycles_per_backend': 30, 'compared_result_positions': sum(len(r['extra']['signature']) for r in b),
               'all_id_order_score_bits_equal': equal}
        for metric in ['load_ms', 'first_ms', 'recovery_ms']:
            br = [r for r in runs[baseline][0] if r['stage'] == ('loaded' if metric == 'load_ms' else 'recovered')]
            cr = [r for r in runs[candidate][0] if r['stage'] == ('loaded' if metric == 'load_ms' else 'recovered')]
            bp, cp = p95([r['extra'][metric] for r in br]), p95([r['extra'][metric] for r in cr])
            row[metric + '_baseline_p95'] = bp
            row[metric + '_candidate_p95'] = cp
            row[metric + '_p95_ratio'] = cp / bp
            count = 0
            for x, y in zip(br, cr):
                if y['extra'][metric] > x['extra'][metric]:
                    count += 1
                    slower.append({'comparison': name, 'metric': metric, 'cycle': x['cycle'],
                                   'case_id': x['extra'].get('case_id', ''),
                                   'baseline_ms': x['extra'][metric], 'candidate_ms': y['extra'][metric],
                                   'ratio': y['extra'][metric] / x['extra'][metric]})
            row[metric + '_slower_cycles'] = count
        row['recovery_p95_max_ratio'] = 1.10
        row['recovery_cost_gate_passed'] = row['recovery_ms_p95_ratio'] <= 1.10
        comparisons.append(row)
    for name, rows in [('workers.csv', workers), ('events.csv', events), ('score-signatures.csv', scores),
                       ('latency-samples.csv', timings), ('comparisons.csv', comparisons),
                       ('slower-samples.csv', slower), ('alternating-rss-samples.csv', rss_samples),
                       ('identities.csv', identities)]:
        write_csv(out, name, rows)
    evidence = {'format_version': 1, 'baseline_commit': 'c353bde26c69a72854fdc4783d1e2f130181b09d',
                'native_legacy_adapter_sha256': sha(ROOT / 'src/reranking/local.rs'),
                'fixture_sha256': sha(ROOT / 'tests/fixtures/native-lifecycle-v1.json'),
                'model_manifest_sha256': sha(ROOT / 'docs/models/mmarco-minilm-v2.json'),
                'model_weight_sha256': '3e9a03ed1e966f7c5288dd4230e3d6a9bf5e3a170a06f1f4241c5bca12c6487c',
                'model_weight_bytes': 470883696,
                'runtime': {'name': 'ONNX Runtime 1.32.0 macOS ARM64 dynamic CPU library',
                            'sha256': 'fa585fabbea65b823b9a8f7e27d46aee3989d1be7e36b528203d3a1fd613aae7'},
                'host': 'Apple M5 Max, 64 GiB RAM, macOS 27.0.1 ARM64; not a tested 1 GiB device',
                'private_replay_ablation_controls': read(lab / 'private-ablation-comparison.json'),
                'archival_source_export': read(lab / 'public-archival-source.json'),
                'public_patch_reproduction': read(lab / 'public-patch-reproduction.json'),
                'threads_candidates_outer_batch': [2, 20, 1],
                'source_policies': {p: read(lab / p) for p in ['PAIRED-POLICY.json', 'CONFIRMATION-POLICY.json']},
                'alternating_policy': read(lab / 'interleaved-final/policy.json'),
                'exact_paired_source_preservation': read(lab / 'paired-source-preservation.json'),
                'export_method_sha256': sha(Path(__file__)),
                'csv_rows': {p.name: sum(1 for _ in csv.DictReader(p.open())) for p in out.glob('*.csv')},
                'no_private_text_or_service_inventory_exported': True,
                'latency_interpretation': 'Every cohort remains separate. Failed serial confirmation and rejected memory-worker cost gate stay failed. Alternating diagnostic has two resident parents and does not erase earlier failures.',
                'rss_interpretation': 'Parent after-drop RSS is distinct from sampled tree RSS; sums can double-count shared pages. Samples are not unique physical/system/GPU accounting.',
                'independently_reviewed_cases': 0}
    # Policies contain only hashes/settings/relative project paths, never commands or raw text.
    (out / 'evidence.json').write_text(json.dumps(evidence, indent=2, allow_nan=False) + '\n')
    write_checksums(out)
    print(json.dumps({'workers': len(workers), 'total_cycles': sum(w['cycles'] for w in workers),
                      'comparisons': len(comparisons), 'slower_samples': len(slower),
                      'serial_confirmation_failed': not comparisons[3]['recovery_cost_gate_passed']}))


def write_checksums(out):
    paths = sorted([*out.glob('*.csv'), *out.glob('*.svg'), *out.glob('*.png'), *out.glob('*.patch'), *out.glob('*.py.txt'), out / 'plot.py'])
    (out / 'SHA256SUMS').write_text(''.join(f'{sha(p)}  {p.name}\n' for p in paths))


if __name__ == '__main__':
    main()
