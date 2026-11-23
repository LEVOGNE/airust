"""Preplanned large-corpus controls; all three versions and all campaigns stay separate."""
import hashlib
import json
import pathlib
import subprocess
import sys
import benchmark as common
from host_monitor import HostMonitor

ROOT, BASE, WORKERS = common.ROOT, common.BASE, common.WORKERS
read, save, stats = common.read, common.save, common.stats

def main(campaign):
    assert campaign in ['standard', 'confirmation']
    common.verify()
    source = ROOT / 'local-lab/pdf-stability/imports/canonical-combined-0315.json'
    queries = BASE / 'mcp-queries.json'
    original = {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in [source, queries]}
    arms = ['reference', 'baseline', 'candidate']
    versions = {'reference': '0.4.21', 'baseline': '0.4.22', 'candidate': '0.4.23'}
    binaries = {arm: WORKERS / ('mcp-pre-regression-0421' if arm == 'reference' else 'mcp-' + arm) for arm in arms}
    prefix = 'large-' + campaign
    reports, compared = [], 0
    with HostMonitor(BASE / (prefix + '-host-samples.json')):
        for triple in range(7):
            order = arms[triple % 3:] + arms[:triple % 3]
            if triple % 2:
                order.reverse()
            transcripts = {}
            for arm in order:
                folder = BASE / prefix / f'{triple}-{arm}'
                folder.mkdir(parents=True)
                with (folder / 'probe.log').open('w') as log:
                    subprocess.run([sys.executable, '-B', str(pathlib.Path(__file__).with_name('mcp_probe.py')), '--binary', str(binaries[arm]), '--knowledge', str(source), '--queries', str(queries), '--report', str(folder / 'report.json'), '--transcript', str(folder / 'transcript-private.json')], stdout=log, stderr=subprocess.STDOUT, check=True, timeout=240)
                result = subprocess.run([str(WORKERS / 'schema'), str(folder / 'transcript-private.json'), str(folder / 'schema.json')], capture_output=True, text=True, timeout=120)
                (folder / 'schema.log').write_text(result.stdout + result.stderr)
                assert result.returncode == 0
                report = json.loads((folder / 'report.json').read_text())
                assert report['version'] == versions[arm]
                reports.append({'arm': arm, 'triple': triple, **report, 'schema_validation': json.loads((folder / 'schema.json').read_text())})
                transcripts[arm] = json.loads((folder / 'transcript-private.json').read_text())
            for arm in arms:
                for tool in transcripts[arm]['tools']:
                    assert tool['outputSchema']['$id'] == f"urn:airust:mcp:{versions[arm]}:{tool['name']}"
                    tool['outputSchema']['$id'] = f"urn:airust:mcp:VERSION:{tool['name']}"
            assert all(transcripts[arm]['tools'] == transcripts['candidate']['tools'] for arm in arms)
            values = {arm: {row['request_id']: row['result']['structuredContent'] for row in data['results']} for arm, data in transcripts.items()}
            for arm in values:
                for value in values[arm].values():
                    if value.get('operation') == 'airust_status':
                        assert value['data'].pop('version') == versions[arm]
            for arm in arms[:-1]:
                assert values[arm] == values['candidate']
                compared += len(values[arm])
            print(json.dumps({'stage': prefix, 'triple': triple + 1, 'complete_response_pairs': compared}), flush=True)
    summary = {}
    for arm in arms:
        rows = [row for row in reports if row['arm'] == arm]
        summary[arm] = {'version': versions[arm], 'startup_ms': stats([row['startup_to_initialized_ms'] for row in rows]), 'peak_rss_bytes': stats([row['server_peak_rss_bytes'] for row in rows])}
        for phase, field in [('initial', 'initial_search_samples'), ('repeated', 'search_samples')]:
            summary[arm][phase + '_ms'] = {alg: stats([x['milliseconds'] for row in rows for x in row[field] if x['algorithm'] == alg]) for alg in ['bm25', 'tfidf']}
            summary[arm][phase + '_per_query_ms'] = {str(q): {alg: stats([x['milliseconds'] for row in rows for x in row[field] if x['algorithm'] == alg and x['query_number'] == q]) for alg in ['bm25', 'tfidf']} for q in range(10)}
            count = 10 if phase == 'initial' else 20
            summary[arm][phase + '_process_sum_ms'] = {alg: stats([sum(x['milliseconds'] for x in row[field] if x['algorithm'] == alg) for row in rows]) for alg in ['bm25', 'tfidf']}
            summary[arm][phase + '_calls_per_process_per_ranker'] = count
    assert all(hashlib.sha256(pathlib.Path(name).read_bytes()).hexdigest() == expected for name, expected in original.items())
    validated = sum(row['schema_validation']['actual_results_validated'] for row in reports)
    assert len(reports) == 21 and compared == 952 and validated == 1428
    proof = {'scope': 'Preplanned separate canonical large-query three-version control; seven rotating triples, same programs and inputs, all samples retained without merging across campaigns', 'campaign': campaign, 'versions': versions, 'processes': 21, 'complete_response_pairs': compared, 'actual_schema_results_validated': validated, 'all_complete_responses_equal': True, 'inputs_unchanged': True}
    save(prefix + '-reports.json', reports)
    save(prefix + '-summary.json', summary)
    save(prefix + '-compatibility.json', proof)
    print(json.dumps(proof), flush=True)

if __name__ == '__main__':
    main(sys.argv[1])
