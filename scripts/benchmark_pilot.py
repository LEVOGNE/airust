"""Measure complete stdio requests with frozen inputs and retain all outcomes."""
import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import random
import statistics
import subprocess
import time
import urllib.request

from mcp_client import StdioClient
from pilot_dataset import workspace as verify_workspace
from reference_dataset import ROOT, validate
from run_reference_evaluation import host_identity, percentile

POLICY = ROOT / 'docs/evaluation/pilot-0.6.0-measurement-policy.json'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def canonical(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(',', ':'), allow_nan=False).encode()


def put(path, value):
    with path.open('xb') as stream:
        stream.write(canonical(value) + b'\n')


def model_memory(pid, url):
    # Only publish the explicitly task-owned service and descendants.
    lines = subprocess.check_output(['ps', '-axo', 'pid=,ppid=,rss='], text=True).splitlines()
    rows = [list(map(int, line.split())) for line in lines if len(line.split()) == 3]
    selected = {pid}
    while True:
        children = {child for child, parent, _ in rows if parent in selected}
        if children <= selected:
            break
        selected |= children
    processes = [{'pid': child, 'parent': parent, 'rss_bytes': rss * 1024} for child, parent, rss in rows if child in selected]
    if not any(process['pid'] == pid for process in processes):
        raise ValueError('task-owned embedding service process is unavailable for memory observation')
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
    with opener.open(url.rstrip('/') + '/api/ps', timeout=5) as response:
        raw = response.read(2 * 1024 * 1024 + 1)
    if len(raw) > 2 * 1024 * 1024:
        raise ValueError('model status byte budget')
    return {'processes': processes, 'sum_rss_bytes_not_unique_physical': sum(p['rss_bytes'] for p in processes),
            'service_reported_models': json.loads(raw), 'gpu_unique_physical_memory_bytes': None,
            'note': 'Sampled process RSS and service-reported model sizes; shared/unified physical allocations are not independently measured.'}


def check_sources(data, documents, collection):
    hits, sources = data['retrieval']['hits'], data['sources']
    if len(hits) != len(sources) or len(hits) > 6:
        raise ValueError('source/hit count')
    for hit, source in zip(hits, sources):
        passage = source.get('passage', {})
        document = documents.get(passage.get('document_id'))
        span = passage.get('span', {})
        if (source.get('status') != 'snapshot_bound' or source.get('entry_id') != hit['id'] or document is None
                or passage['revision'] != document['revision'] or not passage['document_id'].startswith(collection + '/')
                or hit.get('metadata') is not None and hit['metadata'].get('collection') != collection):
            raise ValueError('source revision or collection leak')
        quote = document['raw'][span['start_byte']:span['end_byte']].decode()
        if quote != hit['excerpt']:
            raise ValueError('source span mismatch')


def worker(binary, workspace, cases, policy, output, label, profile=None, service=None):
    directory = output / label
    directory.mkdir()
    command = [binary, 'serve', workspace]
    if profile:
        command += ['--profile', profile]
    documents_identity, documents = verify_workspace((workspace / 'workspace.json').read_bytes())
    ordered = list(cases)
    random.Random(policy['query_order_seed']).shuffle(ordered)
    samples, probes, memory = [], {}, []
    first_ms = None
    previous_started = 0.0
    request_count = 0
    started = time.perf_counter()
    client = StdioClient(command, timeout=30, stderr_path=directory / 'stderr.txt')
    startup_ms = (time.perf_counter() - started) * 1000
    try:
        def query(case, repetition):
            nonlocal previous_started, first_ms, request_count
            # Stay below the normal 256-frame/sec admission boundary. Pacing
            # is outside the timed request and is recorded in the method.
            time.sleep(max(0.0, .005 - (time.perf_counter() - previous_started)))
            previous_started = time.perf_counter()
            result = client.call('airust_search', {'query': case['query'], 'limit': policy['limit'],
                                                   'excerpt_bytes': policy['excerpt_bytes'], 'timeout_ms': policy['timeout_ms']})
            elapsed = (time.perf_counter() - previous_started) * 1000
            request_count += 1
            if first_ms is None:
                first_ms = elapsed
            if result.get('isError'):
                put(directory / f'failed-request-{request_count}.json', result)
                raise ValueError(f'{label} request failed; preserved reply')
            data = result['structuredContent']['data']
            check_sources(data, documents, documents_identity['collection'])
            signature = hashlib.sha256(canonical(data)).hexdigest()
            if repetition < 0:
                probes[case['id']] = {'data': data, 'signature': signature}
            else:
                if probes[case['id']]['signature'] != signature:
                    raise ValueError('ranking/recipe/evidence changed during repeated requests')
                samples.append({'id': case['id'], 'repetition': repetition, 'ms': elapsed, 'data_sha256': signature})
            if service and request_count % 25 == 0:
                memory.append(model_memory(*service))
            if request_count % 100 == 0:
                print(json.dumps({'worker': label, 'completed_requests': request_count, 'phase': 'warmup' if repetition < 0 else 'measured'}), flush=True)

        if service:
            memory.append(model_memory(*service))
        for case in ordered:
            query(case, -1)
        for repetition in range(policy['repetitions_per_query']):
            for case in ordered:
                query(case, repetition)
    except BaseException:
        put(directory / 'partial-probes.json', probes)
        put(directory / 'partial-samples.json', samples)
        put(directory / 'partial-service-memory.json', memory)
        raise
    finally:
        usage = client.close()
    if usage is None or usage['exit_code'] != 0:
        raise ValueError('exact-child worker did not exit successfully')
    observations = []
    for case in cases:
        data = probes[case['id']]['data']
        hits = data['retrieval']['hits']
        ranks = [i + 1 for i, hit in enumerate(hits) if case['answerable'] and case['reference_quote'] in hit['excerpt']]
        observations.append({'id': case['id'], 'language': case['language'], 'split': case['split'], 'answerable': case['answerable'],
                             'known_quote_hit_at_6': bool(ranks), 'known_quote_mrr_at_6': 1 / ranks[0] if ranks else 0,
                             'candidate_returned_on_negative': not case['answerable'] and bool(hits),
                             'data_sha256': probes[case['id']]['signature']})
    groups = {}
    for split in ['development', 'evaluation']:
        positive = [c for c in observations if c['split'] == split and c['answerable']]
        negative = [c for c in observations if c['split'] == split and not c['answerable']]
        groups[split] = {'answerable': len(positive), 'unsupported': len(negative),
                         'known_quote_hit_at_6': statistics.mean(c['known_quote_hit_at_6'] for c in positive),
                         'known_quote_mrr_at_6': statistics.mean(c['known_quote_mrr_at_6'] for c in positive),
                         'negative_candidate_return_rate': statistics.mean(c['candidate_returned_on_negative'] for c in negative)}
    summary = {'label': label, 'command': list(map(str, command)), 'process_startup_to_initialized_ms': startup_ms,
               'first_request_ms_after_preparation': first_ms, 'warm_p50_ms': percentile([s['ms'] for s in samples], .5),
               'warm_p95_ms': percentile([s['ms'] for s in samples], .95), **usage, 'samples': len(samples),
               'span_mismatches': 0, 'scope_leaks': 0, 'by_split': groups,
               'max_sampled_service_sum_rss_bytes_not_unique_physical': max((m['sum_rss_bytes_not_unique_physical'] for m in memory), default=None)}
    put(directory / 'probes.json', probes)
    put(directory / 'samples.json', samples)
    put(directory / 'observations.json', observations)
    put(directory / 'service-memory.json', memory)
    put(directory / 'summary.json', summary)
    return summary, observations


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('mode', choices=['compatibility', 'profiles'])
    parser.add_argument('--binary', required=True, type=Path)
    parser.add_argument('--baseline', type=Path)
    parser.add_argument('--workspace', required=True, type=Path)
    parser.add_argument('--profiles', type=Path)
    parser.add_argument('--service-pid', type=int)
    parser.add_argument('--service-url')
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    identity, _ = validate()
    policy = json.loads(POLICY.read_text())
    if identity['identity_sha256'] != policy['dataset_identity']:
        raise ValueError('frozen dataset differs')
    args.output.mkdir(parents=True, exist_ok=False)
    cases = [json.loads(line) for line in (ROOT / 'tests/fixtures/reference-v1/cases.jsonl').read_text().splitlines()]
    settings = policy['live_profiles']
    report = {'format_version': 1, 'started_at': datetime.now(timezone.utc).isoformat(), 'mode': args.mode,
              'commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
              'policy_sha256': sha(POLICY), 'method_sha256': sha(Path(__file__)), 'client_sha256': sha(ROOT / 'scripts/mcp_client.py'),
              'dataset_identity': identity['identity_sha256'], 'workspace_sha256': sha(args.workspace / 'workspace.json'),
              'candidate_binary_sha256': sha(args.binary), 'host': host_identity(), 'workers': [],
              'timing_scope': 'Full client request write through receipt and JSON decoding of the complete server response; 5ms minimum start spacing outside timing. Startup includes process, workspace and optional profile/model loading; file/service caches are not forcibly cold.',
              'quality_scope': policy['dataset_role'], 'passed': False}
    failures = []
    if args.mode == 'compatibility':
        if args.baseline is None:
            parser.error('--baseline required')
        report['baseline_binary_sha256'] = sha(args.baseline)
        compat = policy['mcp_compatibility']
        expected = None
        campaigns = []
        for campaign in range(compat['campaigns']):
            selected = {'baseline': [], 'candidate': []}
            for pair in range(compat['pairs']):
                order = ['baseline', 'candidate'] if (campaign + pair) % 2 == 0 else ['candidate', 'baseline']
                for build in order:
                    summary, observations = worker(args.baseline if build == 'baseline' else args.binary, args.workspace, cases,
                                                   {**settings, 'repetitions_per_query': compat['repetitions_per_query']}, args.output,
                                                   f'campaign-{campaign + 1}-pair-{pair + 1}-{build}')
                    signatures = {c['id']: c['data_sha256'] for c in observations}
                    if expected is None:
                        expected = signatures
                    if signatures != expected:
                        raise ValueError('baseline/candidate full evidence signatures differ')
                    selected[build].append(summary)
                    report['workers'].append(summary)
                    print(json.dumps(summary), flush=True)
            grouped = {build: {'median_worker_p95_ms': statistics.median(s['warm_p95_ms'] for s in summaries),
                               'median_peak_rss_bytes': statistics.median(s['peak_rss_bytes'] for s in summaries)} for build, summaries in selected.items()}
            campaigns.append({'observations': grouped, 'p95_ratio': grouped['candidate']['median_worker_p95_ms'] / grouped['baseline']['median_worker_p95_ms'],
                              'rss_ratio': grouped['candidate']['median_peak_rss_bytes'] / grouped['baseline']['median_peak_rss_bytes']})
        report['campaigns'] = campaigns
        report['exact_evidence_signatures_match'] = True
        failures = [metric for metric in ['p95_ratio', 'rss_ratio'] if all(c[metric] > compat['reproducible_p95_or_median_rss_ratio_maximum'] for c in campaigns)]
    else:
        if args.profiles is None or args.service_pid is None or args.service_url is None:
            parser.error('--profiles, --service-pid and --service-url required')
        by_profile = {}
        report['profile_configs'] = {}
        for profile in settings['profiles']:
            config = args.profiles / (profile + '.json')
            report['profile_configs'][profile] = {'sha256': sha(config), 'configuration': json.loads(config.read_text())}
            summary, observations = worker(args.binary, args.workspace, cases, settings, args.output, profile, config,
                                           (args.service_pid, args.service_url) if profile != 'bm25' else None)
            cap = settings[{'bm25': 'lexical', 'hybrid': 'hybrid', 'hybrid_rerank': 'rerank'}[profile] + '_complete_warm_p95_ms_maximum']
            summary['latency_budget_ms'] = cap
            summary['resource_budget_passed'] = (summary['warm_p95_ms'] <= cap and summary['peak_rss_bytes'] <= settings['worker_peak_rss_mib_maximum'] * 1024**2
                                                  and (summary['max_sampled_service_sum_rss_bytes_not_unique_physical'] or 0) <= settings['embedding_service_observed_rss_mib_maximum'] * 1024**2)
            if not summary['resource_budget_passed']:
                failures.append(profile + ': resource budget')
            report['workers'].append(summary)
            by_profile[profile] = {c['id']: c for c in observations}
            print(json.dumps(summary), flush=True)
        report['individual_known_quote_losses_against_bm25'] = {profile: [{'id': key, 'hit_delta': int(value['known_quote_hit_at_6']) - int(by_profile['bm25'][key]['known_quote_hit_at_6']),
                                                                         'mrr_delta': value['known_quote_mrr_at_6'] - by_profile['bm25'][key]['known_quote_mrr_at_6']}
                                                                        for key, value in rows.items() if value['known_quote_mrr_at_6'] < by_profile['bm25'][key]['known_quote_mrr_at_6']]
                                                               for profile, rows in by_profile.items() if profile != 'bm25'}
        report['new_production_quality_recommendation'] = None
        report['genuine_independently_reviewed_pilot_gate'] = 'not met by this assistant-authored regression corpus'
    report['failures'] = failures
    report['passed'] = not failures
    report['finished_at'] = datetime.now(timezone.utc).isoformat()
    put(args.output / 'summary.json', report)
    put(args.output / 'artifact-manifest.json', {str(p.relative_to(args.output)): {'sha256': sha(p), 'bytes': p.stat().st_size} for p in sorted(args.output.rglob('*')) if p.is_file()})
    if failures:
        raise SystemExit('Frozen resource gate failed; all observations retained: ' + repr(failures))


if __name__ == '__main__':
    main()
