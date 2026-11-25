"""Compare one explicit native reranking policy against a compatible baseline.

Use the frozen authored reference-v1 corpus. No model assets or service are
started/downloaded automatically. Run without compilers/Miri/fuzz/other inference.
"""
import argparse
import copy
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import random
import statistics
import time

from benchmark_pilot import check_sources
from mcp_client import StdioClient
from pilot_dataset import workspace as verify_workspace
from reference_dataset import ROOT, validate
from run_reference_evaluation import host_identity, percentile


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def canonical(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(',', ':'), allow_nan=False).encode()


def put(path, value):
    with path.open('x') as stream:
        stream.write(json.dumps(value, ensure_ascii=False, indent=2, allow_nan=False) + '\n')


def worker(directory, label, binary, workspace, config, cases, repeats, launch_prefix=()):
    target = directory / label
    target.mkdir()
    identity, documents = verify_workspace((workspace / 'workspace.json').read_bytes())
    ordered = list(cases)
    random.Random(1729).shuffle(ordered)
    start = time.perf_counter()
    client = StdioClient([*launch_prefix, binary, 'serve', workspace, '--profile', config], timeout=30, stderr_path=target / 'worker.stderr')
    startup = (time.perf_counter() - start) * 1000
    samples, probes = [], {}
    try:
        for repetition in range(-1, repeats):
            for case in ordered:
                time.sleep(.005)
                start = time.perf_counter()
                reply = client.call('airust_search', {'query': case['query'], 'limit': 6, 'excerpt_bytes': 2000, 'timeout_ms': 10000})
                elapsed = (time.perf_counter() - start) * 1000
                assert not reply.get('isError'), reply
                data = reply['structuredContent']['data']
                check_sources(data, documents, identity['collection'])
                assert data['retrieval']['fallback_reason'] is None
                fingerprint = hashlib.sha256(canonical(data)).hexdigest()
                if repetition < 0:
                    probes[case['id']] = {'data': data, 'sha256': fingerprint, 'first_ms': elapsed}
                else:
                    assert fingerprint == probes[case['id']]['sha256'], 'repeated result changed'
                    samples.append({'id': case['id'], 'repetition': repetition, 'ms': elapsed, 'data_sha256': fingerprint})
                if len(samples) and len(samples) % 100 == 0 and repetition >= 0:
                    print(json.dumps({'worker': label, 'timed_requests': len(samples)}), flush=True)
    finally:
        usage = client.close()
        put(target / 'probes-private.json', probes)
        put(target / 'samples.json', samples)
        put(target / 'usage.json', usage)
    assert usage['exit_code'] == 0 and len(samples) == len(cases) * repeats
    report = {'label': label, 'cases': len(cases), 'timed_requests': len(samples),
              'p50_ms': statistics.median(s['ms'] for s in samples),
              'p95_ms': percentile([s['ms'] for s in samples], .95), 'startup_ms': startup,
              'peak_rss_bytes': usage['peak_rss_bytes'], 'exit_code': usage['exit_code'],
              'source_mismatches': 0, 'scope_leaks': 0, 'binary_sha256': sha(binary), 'config_sha256': sha(config)}
    put(target / 'summary.json', report)
    print(json.dumps(report), flush=True)
    return report, probes, samples


def compare(old, new, cases):
    scores, slower, quote_losses = [], [], []
    for case in cases:
        before, after = old[1][case['id']]['data'], new[1][case['id']]['data']
        first, second = copy.deepcopy(before), copy.deepcopy(after)
        assert [h['id'] for h in first['retrieval']['hits']] == [h['id'] for h in second['retrieval']['hits']], 'top-six IDs/order changed'
        for left, right in zip(first['retrieval']['hits'], second['retrieval']['hits']):
            a, b = left.pop('rerank_score', None), right.pop('rerank_score', None)
            assert (a is None) == (b is None)
            if a is not None:
                difference = abs(a - b)
                assert difference <= 1e-4 + 1e-5 * abs(a), 'native numerical tolerance exceeded'
                if difference:
                    scores.append({'id': case['id'], 'document_id': left['id'], 'baseline': a, 'candidate': b, 'absolute_difference': difference})
        assert canonical(first) == canonical(second), 'non-logit response/evidence changed'
        a = statistics.mean(s['ms'] for s in old[2] if s['id'] == case['id'])
        b = statistics.mean(s['ms'] for s in new[2] if s['id'] == case['id'])
        if b > a:
            slower.append({'id': case['id'], 'baseline_mean_ms': a, 'candidate_mean_ms': b, 'ratio': b / a})
        if case.get('answerable'):
            previous = any(case['reference_quote'] in h['excerpt'] for h in before['retrieval']['hits'])
            current = any(case['reference_quote'] in h['excerpt'] for h in after['retrieval']['hits'])
            if previous and not current:
                quote_losses.append(case['id'])
    return {'p95_ratio_to_baseline': new[0]['p95_ms'] / old[0]['p95_ms'],
            'memory_ratio_to_baseline': new[0]['peak_rss_bytes'] / old[0]['peak_rss_bytes'],
            'all_logit_changes': scores, 'all_slower_queries': slower, 'all_known_quote_at_6_losses': quote_losses,
            'all_non_logit_response_data_identical': True, 'top_six_ids_order_changed': False}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for option in ['baseline', 'candidate', 'workspace', 'baseline-profile', 'candidate-profile', 'output']:
        parser.add_argument('--' + option, required=True, type=Path)
    parser.add_argument('--repetitions', type=int, default=2)
    args = parser.parse_args()
    if not 1 <= args.repetitions <= 5:
        parser.error('repetitions must be 1..5')
    for name in ['baseline', 'candidate', 'workspace', 'baseline_profile', 'candidate_profile']:
        setattr(args, name, getattr(args, name).resolve(strict=True))
    for profile in [args.baseline_profile, args.candidate_profile]:
        with profile.open('rb') as stream:
            raw = stream.read(16 * 1024 + 1)
        if len(raw) > 16 * 1024:
            raise ValueError('profile exceeds 16KiB')
        config = json.loads(raw)
        if config.get('profile') != 'hybrid_rerank' or config.get('failure_policy') != 'error':
            raise ValueError('comparison requires strict hybrid_rerank profiles')
        native = config['reranker']
        if native.get('candidates') not in [None, 20]:
            raise ValueError('comparison keeps the actual 20-candidate prefix')
    identity, _ = validate()
    cases = [json.loads(line) for line in (ROOT / 'tests/fixtures/reference-v1/cases.jsonl').read_text().splitlines() if line]
    args.output.mkdir(parents=True, exist_ok=False)
    policy = {'frozen_at': datetime.now(timezone.utc).isoformat(), 'campaigns': 2,
              'cases': len(cases), 'repetitions': args.repetitions, 'warmups_per_case': 1,
              'baseline_binary_sha256': sha(args.baseline), 'candidate_binary_sha256': sha(args.candidate),
              'baseline_profile_sha256': sha(args.baseline_profile), 'candidate_profile_sha256': sha(args.candidate_profile),
              'workspace_sha256': sha(args.workspace / 'workspace.json'),
              'dataset_identity': identity['identity_sha256'], 'method_sha256': sha(Path(__file__)),
              'score_absolute_tolerance': 1e-4, 'score_relative_tolerance': 1e-5,
              'maximum_repeated_p95_ratio': .8, 'strict_other_response_equality': True,
              'host': host_identity(), 'origin': 'Frozen assistant-authored regression inputs, not independent quality judgments.',
              'scope': 'Full MCP request write through decoded receipt; one worker at a time; startup/initial requests separate; no answer generation or OS RAM quota.'}
    put(args.output / 'policy.json', policy)
    records = []
    for campaign in [1, 2]:
        order = [('baseline', args.baseline, args.baseline_profile), ('candidate', args.candidate, args.candidate_profile)]
        if campaign % 2 == 0:
            order.reverse()
        results = {}
        for label, binary, config in order:
            results[label] = worker(args.output, f'campaign-{campaign}-{label}', binary,
                                    args.workspace, config, cases, args.repetitions)
        comparison = compare(results['baseline'], results['candidate'], cases)
        put(args.output / f'campaign-{campaign}-comparison.json', comparison)
        records.append({'campaign': campaign, 'baseline': results['baseline'][0], 'candidate': results['candidate'][0],
                        'p95_ratio': comparison['p95_ratio_to_baseline'], 'memory_ratio': comparison['memory_ratio_to_baseline'],
                        'slower_query_count': len(comparison['all_slower_queries']),
                        'known_quote_losses': comparison['all_known_quote_at_6_losses']})
    result = {'completed_at': datetime.now(timezone.utc).isoformat(), 'campaigns': records,
              'all_complete': True, 'same_top_six_and_other_evidence': True,
              'no_known_quote_losses': all(not r['known_quote_losses'] for r in records),
              'repeated_p95_gain_at_least_20_percent': all(r['p95_ratio'] <= .8 for r in records),
              'independent_quality_claim': False}
    put(args.output / 'report.json', result)
    if not result['no_known_quote_losses'] or not result['repeated_p95_gain_at_least_20_percent']:
        raise SystemExit('comparison did not meet frozen gates; preserve every result')


if __name__ == '__main__':
    main()
