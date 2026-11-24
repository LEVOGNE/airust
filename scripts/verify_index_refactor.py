import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import platform
import statistics
import subprocess
import sys

from reference_dataset import ROOT, validate
from run_reference_evaluation import execute, host_identity, percentile


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description='Compare already-built identical consumers of registry 0.5.2 and the candidate, using two alternating paired campaigns.')
    parser.add_argument('--binary-directory', required=True, type=Path)
    parser.add_argument('--dataset', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    identity, _ = validate()
    if json.loads((args.dataset / 'manifest.json').read_text())['identity_sha256'] != identity['identity_sha256']:
        raise ValueError('dataset identity differs')
    args.output.mkdir(parents=True, exist_ok=False)
    started = datetime.now(timezone.utc).isoformat()
    binaries = {name: args.binary_directory / name for name in ['baseline-default', 'candidate-default', 'baseline-shared', 'candidate-shared']}
    hashes = {name: digest(path) for name, path in binaries.items()}
    signatures = {}
    for name, binary in binaries.items():
        output = args.output / (name + '-signature.json')
        execute([str(binary.resolve()), 'signature', str(args.dataset.resolve()), str(output.resolve())], args.output / (name + '-signature.log'))
        signatures[name] = digest(output)
    if len(set(signatures.values())) != 1:
        raise ValueError('ordered IDs, score bits, matched terms or original records differ; all signatures preserved')
    campaigns = []
    for campaign in range(2):
        observations = []
        for pair in range(7):
            for storage in ['default', 'shared']:
                order = ['baseline', 'candidate'] if (pair + campaign) % 2 == 0 else ['candidate', 'baseline']
                for build in order:
                    name = build + '-' + storage
                    prefix = f'campaign-{campaign + 1}-pair-{pair + 1}-{name}'
                    output = args.output / (prefix + '.json')
                    process = execute([str(binaries[name].resolve()), 'measure', str(args.dataset.resolve()), str(output.resolve())], args.output / (prefix + '.log'))
                    report = json.loads(output.read_text())
                    observations.append({'pair': pair + 1, 'storage': storage, 'build': build, 'file': output.name, 'measurement': process, 'samples': report['samples']})
        summaries = {}
        for storage in ['default', 'shared']:
            grouped = {}
            for build in ['baseline', 'candidate']:
                selected = [row for row in observations if row['storage'] == storage and row['build'] == build]
                grouped[build] = {'warm_p95_us': percentile([sample['micros'] for row in selected for sample in row['samples']], .95), 'median_peak_rss_bytes': statistics.median(row['measurement']['peak_rss_bytes'] for row in selected), 'max_peak_rss_bytes': max(row['measurement']['peak_rss_bytes'] for row in selected)}
                grouped[build]['by_algorithm'] = {algorithm: percentile([sample['micros'] for row in selected for sample in row['samples'] if sample['algorithm'] == algorithm], .95) for algorithm in ['Bm25', 'Tfidf']}
            baseline, candidate = grouped['baseline'], grouped['candidate']
            slower = []
            for algorithm in ['Bm25', 'Tfidf']:
                for query in sorted({sample['query'] for row in observations for sample in row['samples']}):
                    means = {build: statistics.mean(sample['micros'] for row in observations if row['storage'] == storage and row['build'] == build for sample in row['samples'] if sample['algorithm'] == algorithm and sample['query'] == query) for build in ['baseline','candidate']}
                    if means['candidate'] > means['baseline']:
                        slower.append({'algorithm': algorithm, 'query': query, 'baseline_mean_us': means['baseline'], 'candidate_mean_us': means['candidate'], 'ratio': means['candidate'] / means['baseline']})
            summaries[storage] = {'observations': grouped, 'p95_ratio': candidate['warm_p95_us'] / baseline['warm_p95_us'], 'median_rss_ratio': candidate['median_peak_rss_bytes'] / baseline['median_peak_rss_bytes'], 'slower_queries': slower}
            print(json.dumps({'campaign': campaign + 1, 'storage': storage, **{key:value for key,value in summaries[storage].items() if key != 'slower_queries'}}), flush=True)
        campaigns.append(summaries)
    limit = 1.1
    reproduced = [{'storage':storage, 'metric':metric} for storage in ['default','shared'] for metric in ['p95_ratio','median_rss_ratio'] if all(campaign[storage][metric] > limit for campaign in campaigns)]
    report = {'format_version':1, 'started_at':started,'finished_at':datetime.now(timezone.utc).isoformat(), 'dataset_identity':identity['identity_sha256'], 'host':host_identity(), 'rustc':subprocess.check_output(['rustc','-vV'],text=True), 'consumer_sha256':digest(ROOT/'docs/validation-reference/refactor-consumer.rs'), 'method_sha256':digest(Path(__file__)), 'binary_sha256':hashes, 'signature_sha256':signatures,'exact_signature_match':True,'campaigns':campaigns,'reproducible_regressions_above_10_percent':reproduced,'passed':not reproduced,'limits':['Same source consumer compiled separately for registry 0.5.2 and path candidate, with default and shared storage; no model inference.', 'One host, two alternating campaigns, seven process pairs per storage configuration. Construction is outside warm timing; exact-child RSS includes the full worker.', 'Each worker warms all 158 queries for each algorithm, then records seven repetitions; p95 pools individual query samples over the seven workers.', 'RSS comparison uses median exact-child process peak across seven workers; maxima and every slower query mean are retained.', 'The signature includes all 150 reference questions plus eight boundary/large-query controls under three scopes, both algorithms and four original/edited/retained/restored stages.', 'Identical signatures establish these cases, not equivalence for every input. Tiny-query ratios and a shared host are noisy.']}
    (args.output/'summary.json').write_text(json.dumps(report,indent=2)+'\n')
    (args.output/'artifact-manifest.json').write_text(json.dumps({path.name:{'sha256':digest(path),'bytes':path.stat().st_size} for path in sorted(args.output.glob('*')) if path.is_file()},indent=2)+'\n')
    if reproduced:
        raise SystemExit('Reproducible aggregate regression above frozen budget; inspect preserved observations.')
    print('Exact signatures match; paired aggregate resource gates pass.',flush=True)


if __name__ == '__main__':
    main()
