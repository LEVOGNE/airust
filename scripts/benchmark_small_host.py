"""Measure the bounded, model-free MCP example on declared synthetic workloads.

Build `cargo build --release --features mcp --example bounded_mcp` first.
This measures one worker on the current host; it does not certify a 1GiB device.
"""
import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import platform
import statistics
import subprocess
import time

from mcp_client import StdioClient


def put(path, value):
    with path.open('x', encoding='utf-8') as stream:
        json.dump(value, stream, ensure_ascii=False, allow_nan=False, indent=2)
        stream.write('\n')


def rss(pid):
    result = subprocess.check_output(['ps', '-o', 'rss=', '-p', str(pid)], text=True)
    return int(result.strip()) * 1024


def entries(kind):
    for i in range(2048):
        if kind == 'high-vocabulary':
            text = 'Rust ' + ' '.join(f'w{i * 96 + j:07x}' for j in range(96))
        elif kind == 'short-unique':
            # More distinct terms per input byte than the eight-character probe.
            alphabet = '0123456789abcdefghijklmnopqrstuvwxyz'
            words = []
            for value in range(i * 192, (i + 1) * 192):
                digits = []
                for _ in range(4):
                    value, remainder = divmod(value, 36)
                    digits.append(alphabet[remainder])
                words.append(''.join(reversed(digits)))
            text = 'Rust ' + ' '.join(words)
        elif kind == 'long-token':
            text = 'Rust ' + 'ä' * 450 + f' topic{i}'
        else:
            text = ('Rust ownership borrowing memory Straße 東京 café. ' * 12) + f' topic{i}'
        yield {'id': f'entry-{i:04}', 'text': text, 'metadata': {'set': i % 2}}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('binary', type=Path)
    parser.add_argument('new_output', type=Path)
    parser.add_argument('--requests', type=int, default=1000)
    parser.add_argument('--campaigns', type=int, default=2)
    parser.add_argument('--workloads', nargs='+', choices=['unicode', 'high-vocabulary', 'long-token', 'short-unique'],
                        default=['unicode', 'high-vocabulary', 'long-token'],
                        help='Explicit workload list; short-unique adds a denser vocabulary stress case.')
    args = parser.parse_args()
    if not 1000 <= args.requests <= 10000 or not 1 <= args.campaigns <= 4:
        parser.error('requests must be 1000..10000 and campaigns 1..4')
    if len(args.workloads) != len(set(args.workloads)):
        parser.error('workloads must not be duplicated')
    binary = args.binary.resolve(strict=True)
    output = args.new_output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    policy = {'started_at': datetime.now(timezone.utc).isoformat(), 'host': platform.platform(),
              'binary_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(),
              'method_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
              'requests_per_worker': args.requests, 'campaigns': args.campaigns, 'workloads': args.workloads,
              'pacing_seconds_outside_timer': .025, 'warmups_per_worker': 32,
              'worker_peak_rss_budget_bytes': 512 * 1024 * 1024,
              'post_warm_rss_growth_budget_bytes': 32 * 1024 * 1024,
              'data_origin': 'Deterministic generated resource probes; no independent traffic or quality judgments.',
              'scope': 'MCP write through decoded reply; startup and sampled RSS separate. No models, service, GPU or OS memory quota.',
              'limits_source': 'examples/bounded_mcp.rs; use an unmodified compatible example binary.'}
    put(output / 'policy.json', policy)
    all_summaries = []
    for kind in args.workloads:
        values = list(entries(kind))
        dataset = output / (kind + '.json')
        put(dataset, values)
        assert dataset.stat().st_size <= 4 * 1024 * 1024
        assert sum(len(v['id'].encode()) + len(v['text'].encode()) + len(json.dumps(v['metadata'], separators=(',', ':')).encode()) for v in values) <= 2 * 1024 * 1024
        by_id = {v['id']: v for v in values}
        for campaign in range(1, args.campaigns + 1):
            run = output / f'{kind}-{campaign}'
            run.mkdir()
            samples, memory = [], []
            start = time.perf_counter()
            client = StdioClient([binary, dataset], timeout=15, stderr_path=run / 'stderr.log')
            startup = (time.perf_counter() - start) * 1000
            try:
                for n in range(32 + args.requests):
                    time.sleep(.025)
                    if n and n % 200 == 0:
                        failure = client.call('airust_search', {'query': 'x' * 1025})
                        assert failure['isError'], 'oversized query accepted'
                        time.sleep(.025)
                    query = f'Rust topic{n % 2048} unknown{n}'
                    algorithm = 'bm25' if n % 2 else 'tfidf'
                    start = time.perf_counter()
                    result = client.call('airust_search', {'query': query, 'limit': 6,
                                                           'algorithm': algorithm, 'excerpt_bytes': 1024})
                    elapsed = (time.perf_counter() - start) * 1000
                    assert not result.get('isError', False), result
                    hits = result['structuredContent']['data']['retrieval']['hits']
                    assert len(hits) <= 6
                    for hit in hits:
                        assert hit['id'] in by_id and hit['excerpt'] in by_id[hit['id']]['text']
                    samples.append({'request': n, 'warmup': n < 32, 'algorithm': algorithm,
                                    'ms': elapsed, 'hits': [v['id'] for v in hits]})
                    if n == 31 or n % 100 == 0:
                        memory.append({'request': n, 'rss_bytes': rss(client.process.pid)})
            finally:
                usage = client.close()
                put(run / 'samples.json', samples)
                put(run / 'memory.json', memory)
                put(run / 'usage.json', usage)
            assert usage and usage['exit_code'] == 0
            times = sorted(v['ms'] for v in samples if not v['warmup'])
            retained = [v['rss_bytes'] for v in memory if v['request'] >= 200]
            growth = max(0, statistics.median(retained[-3:]) - statistics.median(retained[:3])) if len(retained) >= 6 else None
            summary = {'kind': kind, 'campaign': campaign, 'dataset_sha256': hashlib.sha256(dataset.read_bytes()).hexdigest(),
                       'dataset_bytes': dataset.stat().st_size, 'entries': len(values), 'timed_requests': len(times),
                       'startup_ms': startup, 'first_request_ms': samples[0]['ms'],
                       'warmup_total_request_ms': sum(v['ms'] for v in samples if v['warmup']), 'p50_ms': statistics.median(times),
                       'p95_ms': times[max(0, (len(times) * 95 + 99) // 100 - 1)],
                       **usage, 'observed_post_warm_growth_bytes': growth,
                       'worker_peak_within_declared_budget': usage['peak_rss_bytes'] <= policy['worker_peak_rss_budget_bytes'],
                       'post_warm_growth_within_declared_budget': growth <= policy['post_warm_rss_growth_budget_bytes'] if growth is not None else None}
            put(run / 'summary.json', summary)
            all_summaries.append(summary)
            put(output / f'progress-{len(all_summaries)}.json', all_summaries)
            print(json.dumps(summary), flush=True)
    report = {'completed_at': datetime.now(timezone.utc).isoformat(), 'workers': all_summaries,
              'all_workers_complete': len(all_summaries) == len(args.workloads) * args.campaigns,
              'all_peak_rss_budgets_met': all(v['worker_peak_within_declared_budget'] for v in all_summaries),
              'all_observable_growth_budgets_met': all(v['post_warm_growth_within_declared_budget'] is True for v in all_summaries),
              'independent_quality_or_old_device_certification': False}
    put(output / 'summary.json', report)
    if not report['all_peak_rss_budgets_met'] or not report['all_observable_growth_budgets_met']:
        raise SystemExit('worker memory scope failed or incomplete; inspect retained observations')


if __name__ == '__main__':
    main()
