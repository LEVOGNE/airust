import argparse
from datetime import datetime, timezone
import json
import math
import os
from pathlib import Path
import platform
import signal
import subprocess
import sys
import time

from reference_dataset import POLICY, ROOT, digest, validate


def percentile(values, fraction):
    values = sorted(values)
    return values[min(len(values) - 1, math.ceil(len(values) * fraction) - 1)]


def check_metrics(report, judgments):
    for case in report['cases']:
        grades = judgments.get(case['id'], {})
        ranking = case['ranked_ids'][:10]
        if len(ranking) != len(set(ranking)):
            raise ValueError('duplicate ranked ID')
        dcg = sum((2 ** grades.get(identifier, 0) - 1) / math.log2(rank + 2) for rank, identifier in enumerate(ranking))
        ideal = sum((2 ** grade - 1) / math.log2(rank + 2) for rank, grade in enumerate(sorted(grades.values(), reverse=True)[:10]))
        expected = dcg / ideal if ideal else None
        actual = case['metrics']['ndcg_at_k']
        if (expected is None) != (actual is None) or expected is not None and abs(expected - actual) > 1e-12:
            raise ValueError('independent nDCG formula differs from the library metric')
        reciprocal = next((1 / (rank + 1) for rank, identifier in enumerate(ranking) if grades.get(identifier, 0) > 0), 0)
        if abs(reciprocal - case['metrics']['reciprocal_rank_at_k']) > 1e-12:
            raise ValueError('independent reciprocal-rank formula differs')


def summarize(report, peak_rss_bytes, policy):
    positive = [case for case in report['cases'] if case['answerable']]
    negative = [case for case in report['cases'] if not case['answerable']]
    times = [value for case in report['cases'] for value in case['latencies_ms']]
    ndcg = sum(case['metrics']['ndcg_at_k'] for case in positive) / len(positive)
    hits = sum(case['reference_hit_at_6'] for case in positive) / len(positive)
    model = report['profile'] in {'hybrid', 'sqlite_hybrid', 'bm25_rerank', 'hybrid_rerank'}
    rerank = report['profile'].endswith('rerank')
    cap = policy['resources']['reranking_warm_p95_ms'] if rerank else policy['resources']['hybrid_retrieval_warm_p95_ms_excluding_embedding'] if model else policy['resources']['lexical_warm_p95_ms']
    memory_cap = policy['resources']['model_worker_peak_rss_mib'] if model else policy['resources']['lexical_peak_rss_mib']
    resource_passed = percentile(times, .95) <= cap and peak_rss_bytes <= memory_cap * 1024 * 1024
    if not model:
        resource_passed = resource_passed and report['build_ms'] <= policy['resources']['reference_build_ms']
    groups = {}
    for language in ['de', 'en']:
        selected = [case for case in positive if case['language'] == language]
        groups[language] = {'answerable_cases': len(selected), 'reference_ndcg_at_10': sum(case['metrics']['ndcg_at_k'] for case in selected) / len(selected), 'reference_hit_at_6': sum(case['reference_hit_at_6'] for case in selected) / len(selected)}
    return {'profile': report['profile'], 'split': report['split'], 'answerable_cases': len(positive), 'negative_cases': len(negative), 'reference_ndcg_at_10': ndcg, 'reference_hit_at_6': hits, 'negative_candidate_return_rate': sum(case['candidate_returned_on_negative'] for case in negative) / len(negative), 'warm_p50_ms': percentile(times, .5), 'warm_p95_ms': percentile(times, .95), 'peak_rss_bytes': peak_rss_bytes, 'build_ms': report['build_ms'], 'resource_budget_passed': resource_passed, 'quality_objective_passed': ndcg >= policy['quality']['answerable_reference_ndcg_at_10_minimum'] and hits >= policy['quality']['answerable_reference_hit_at_6_minimum'], 'by_language': groups, 'scope_leaks': report['scope_leaks'], 'span_mismatches': report['span_mismatches']}


def qualifies(candidate, baseline, policy):
    return candidate['resource_budget_passed'] and candidate['quality_objective_passed'] and candidate['reference_ndcg_at_10'] - baseline['reference_ndcg_at_10'] + 1e-12 >= policy['quality']['recommended_profile_ndcg_gain_minimum'] and baseline['reference_hit_at_6'] - candidate['reference_hit_at_6'] <= policy['quality']['recommended_profile_hit_at_6_loss_maximum'] + 1e-12


def select_development(summaries, policy):
    if any(summary['split'] != 'development' for summary in summaries.values()):
        raise ValueError('selection accepts development observations only')
    baseline = summaries['bm25']
    allowed = {'prepared', 'coverage', 'german', 'hybrid', 'bm25_rerank', 'hybrid_rerank'}
    candidates = [summary for name, summary in summaries.items() if name in allowed and qualifies(summary, baseline, policy)]
    if not candidates:
        return 'bm25'
    return sorted(candidates, key=lambda item: (-item['reference_ndcg_at_10'], item['warm_p95_ms'], item['profile']))[0]['profile']


def execute(command, log, timeout=600):
    if not hasattr(os, 'wait4'):
        raise RuntimeError('Exact-child RSS measurement requires macOS or Linux wait4.')
    with log.open('xb') as stream:
        child = subprocess.Popen(command, stdout=stream, stderr=subprocess.STDOUT, start_new_session=True)
        started = time.monotonic()
        while True:
            pid, status, usage = os.wait4(child.pid, os.WNOHANG)
            if pid:
                child.returncode = os.waitstatus_to_exitcode(status)
                break
            if time.monotonic() - started > timeout:
                os.killpg(child.pid, signal.SIGKILL)
                _, status, usage = os.wait4(child.pid, 0)
                child.returncode = os.waitstatus_to_exitcode(status)
                raise RuntimeError('worker timeout; output preserved')
            time.sleep(.05)
    if child.returncode:
        raise RuntimeError('worker failed; read ' + str(log))
    multiplier = 1 if sys.platform == 'darwin' else 1024
    return {'peak_rss_native': usage.ru_maxrss, 'peak_rss_native_unit': 'bytes' if sys.platform == 'darwin' else 'KiB', 'peak_rss_bytes': int(usage.ru_maxrss * multiplier), 'elapsed_seconds': time.monotonic() - started, 'exit_code': child.returncode}


def losses(baseline, candidate):
    original = {case['id']: case for case in baseline['cases'] if case['answerable']}
    result = []
    for case in candidate['cases']:
        if not case['answerable']:
            continue
        before = original[case['id']]
        delta = case['metrics']['ndcg_at_k'] - before['metrics']['ndcg_at_k']
        hit_delta = int(case['reference_hit_at_6']) - int(before['reference_hit_at_6'])
        if delta < -1e-12 or hit_delta < 0:
            result.append({'id': case['id'], 'reference_ndcg_delta': delta, 'reference_hit_at_6_delta': hit_delta})
    return result


def host_identity():
    result = {'os': platform.system(), 'release': platform.release(), 'machine': platform.machine(), 'processor': platform.processor(), 'logical_cpus': os.cpu_count()}
    if sys.platform == 'darwin':
        for name, command in [('cpu_brand', ['sysctl', '-n', 'machdep.cpu.brand_string']), ('physical_memory_bytes', ['sysctl', '-n', 'hw.memsize']), ('power_observation', ['pmset', '-g', 'batt'])]:
            try:
                value = subprocess.check_output(command, text=True, stderr=subprocess.DEVNULL).strip()
                result[name] = int(value) if name == 'physical_memory_bytes' else value
            except (OSError, subprocess.CalledProcessError):
                result[name] = 'unavailable in this process environment'
    return result


def main():
    parser = argparse.ArgumentParser(description='Run frozen development selection and then reserved reference evaluation.')
    parser.add_argument('--binary', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--vectors', type=Path)
    parser.add_argument('--reranker', type=Path)
    parser.add_argument('--runtime', type=Path)
    args = parser.parse_args()
    started_at = datetime.now(timezone.utc).isoformat()
    source_commit = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    host = host_identity()
    identity, artifacts = validate()
    policy = json.loads(POLICY.read_text())
    model_metadata = None
    if args.vectors:
        prepared = json.loads(args.vectors.read_text())
        model_metadata = {key: value for key, value in prepared.items() if key not in {'documents', 'queries'}}
        model_metadata['file_sha256'] = digest(args.vectors.read_bytes())
        values = list(model_metadata['single_query_embedding_ms'].values())
        model_metadata['single_query_embedding_p50_ms'] = percentile(values, .5)
        model_metadata['single_query_embedding_p95_ms'] = percentile(values, .95)
        del prepared
    args.output.mkdir(parents=True, exist_ok=False)
    exported = args.output / 'dataset'
    exported.mkdir()
    for name, data in artifacts.items():
        (exported / name).write_bytes(data)
    (exported / 'manifest.json').write_text(json.dumps(identity, indent=2) + '\n')
    (exported / 'policy.json').write_bytes(POLICY.read_bytes())
    judgments = {}
    for line in artifacts['qrels.jsonl'].splitlines():
        row = json.loads(line)
        judgments.setdefault(row['query_id'], {})[row['document_id']] = row['grade']
    methods = [ROOT / 'examples/reference_evaluation.rs', *sorted((ROOT / 'examples/reference_eval').glob('*.rs')), *sorted((ROOT / 'src').rglob('*.rs')), ROOT / 'Cargo.toml', ROOT / 'Cargo.lock', Path(__file__).resolve(), ROOT / 'scripts/reference_dataset.py', ROOT / 'scripts/reference_embeddings.py']
    method_hashes = {str(path.relative_to(ROOT)): digest(path.read_bytes()) for path in methods}
    profiles = ['bm25', 'prepared', 'coverage', 'german', 'sqlite']
    if args.vectors:
        profiles += ['hybrid', 'sqlite_hybrid']
    if args.reranker and args.runtime:
        profiles += ['bm25_rerank']
        if args.vectors:
            profiles += ['hybrid_rerank']
    observations = {}
    reports = {}
    errors = []
    selection = None
    for split in ['development', 'evaluation']:
        if split == 'evaluation':
            selection = select_development(observations['development'], policy)
            (args.output / 'selection-before-evaluation.json').write_text(json.dumps({'dataset_identity': identity['identity_sha256'], 'selected_using': 'development', 'selected_profile': selection, 'development_summaries': observations['development']}, indent=2) + '\n')
        observations[split] = {}
        reports[split] = {}
        for profile in profiles:
            output = args.output / (split + '-' + profile + '.json')
            command = [str(args.binary.resolve()), str(exported.resolve()), split, profile, str(output.resolve())]
            if args.vectors or args.reranker:
                command += [str(args.vectors.resolve()) if args.vectors else '-']
            if args.reranker and args.runtime:
                command += [str(args.reranker.resolve()), str(args.runtime.resolve())]
            print('Running', split, profile, flush=True)
            try:
                process = execute(command, args.output / (split + '-' + profile + '.log'))
                report = json.loads(output.read_text())
                if report['split'] != split or report['profile'] != profile or report['dataset_identity'] != identity['identity_sha256']:
                    raise ValueError('worker output identity mismatch')
                check_metrics(report, judgments)
                reports[split][profile] = report
                observations[split][profile] = summarize(report, process['peak_rss_bytes'], policy)
                observations[split][profile]['process_measurement'] = process
                print(json.dumps(observations[split][profile]), flush=True)
            except Exception as error:
                errors.append({'split': split, 'profile': profile, 'error': str(error)})
                print('Failed', split, profile, str(error), flush=True)
                if profile == 'bm25':
                    raise
    for path in methods:
        if digest(path.read_bytes()) != method_hashes[str(path.relative_to(ROOT))]:
            raise ValueError('evaluation method changed during the campaign')
    promoted = selection != 'bm25' and selection in observations['evaluation'] and qualifies(observations['evaluation'][selection], observations['evaluation']['bm25'], policy)
    summary = {
        'format_version': 1, 'dataset': identity, 'policy': policy,
        'started_at': started_at, 'finished_at': datetime.now(timezone.utc).isoformat(),
        'embedding_preparation': model_metadata,
        'native_reranker': {'asset_manifest': json.loads((ROOT / 'docs/models/mmarco-minilm-v2.json').read_text()), 'runtime_file': args.runtime.name, 'runtime_sha256': digest(args.runtime.read_bytes()), 'threads': 2} if args.reranker and args.runtime else None,
        'worker_binary_sha256': digest(args.binary.read_bytes()), 'method_sha256': method_hashes,
        'host': host,
        'source_commit_at_campaign_start': source_commit,
        'rustc': subprocess.check_output(['rustc', '-vV'], text=True),
        'selection': {'selected_on_development': selection, 'evaluation_qualified': promoted, 'recommended_optional_profile': selection if promoted else None, 'literal_defaults_changed': False},
        'observations': observations,
        'per_query_regressions_against_bm25': {split: {profile: losses(reports[split]['bm25'], report) for profile, report in reports[split].items() if profile != 'bm25'} for split in reports},
        'errors': errors,
        'unavailable_profiles': [profile for profile in ['hybrid', 'sqlite_hybrid', 'bm25_rerank', 'hybrid_rerank'] if profile not in profiles],
        'limits': ['Assistant-authored reference questions and known supporting paragraphs, not independent exhaustive judgments or customer traffic.', 'Evaluation was opened after development selection; the same assistant authored and evaluated the reserved split.', 'Warm six-hit retrieval includes JSON serialization; external query embeddings and construction are separate.', 'Peak RSS is an exact-child process observation including data, index, inference and evaluation bookkeeping; it is not isolated index heap.', 'One shared host and this corpus do not establish general speed, semantic quality, token savings or production capacity.', 'Negative candidate-return rate is not an answer error rate; this workflow generates no answers.'],
    }
    (args.output / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps({'summary': str(args.output / 'summary.json'), 'selection': summary['selection'], 'errors': errors}), flush=True)


if __name__ == '__main__':
    main()
