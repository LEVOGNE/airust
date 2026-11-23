"""Exploratory follow-up of the slower wide MCP fixture; original campaigns remain intact."""
import hashlib
import json
import pathlib
import subprocess
import sys
import benchmark as common
from host_monitor import HostMonitor

ROOT, BASE, WORKERS = common.ROOT, common.BASE, common.WORKERS
read, save, stats = common.read, common.save, common.stats

def main():
    common.verify()
    identities = read('diagnostic-method-sha256.json')
    for name, expected in identities.items():
        assert hashlib.sha256((ROOT / name).read_bytes()).hexdigest() == expected, name
    source = BASE / 'wide-mcp.json'
    original = hashlib.sha256(source.read_bytes()).hexdigest()
    reports, compared = [], 0
    with HostMonitor(BASE / 'wide-followup-host-samples.json'):
        for pair in range(10):
            transcripts = {}
            for arm in (['baseline', 'candidate'] if pair % 2 == 0 else ['candidate', 'baseline']):
                folder = BASE / f'wide-followup/{pair}-{arm}'
                folder.mkdir(parents=True)
                with (folder / 'probe.log').open('w') as log:
                    subprocess.run([sys.executable, '-B', str(pathlib.Path(__file__).with_name('mcp_probe.py')), '--binary', str(WORKERS / ('mcp-' + arm)), '--knowledge', str(source), '--queries', str(BASE / 'wide-mcp-queries.json'), '--report', str(folder / 'report.json'), '--transcript', str(folder / 'transcript-private.json')], stdout=log, stderr=subprocess.STDOUT, check=True, timeout=240)
                result = subprocess.run([str(WORKERS / 'schema'), str(folder / 'transcript-private.json'), str(folder / 'schema.json')], capture_output=True, text=True, timeout=120)
                (folder / 'schema.log').write_text(result.stdout + result.stderr)
                assert result.returncode == 0
                report = json.loads((folder / 'report.json').read_text())
                reports.append({'arm': arm, 'pair': pair, **report, 'schema_validation': json.loads((folder / 'schema.json').read_text())})
                transcripts[arm] = json.loads((folder / 'transcript-private.json').read_text())
            for arm, version in [('baseline', '0.4.17'), ('candidate', '0.4.18')]:
                for tool in transcripts[arm]['tools']:
                    assert tool['outputSchema']['$id'] == f"urn:airust:mcp:{version}:{tool['name']}"
                    tool['outputSchema']['$id'] = f"urn:airust:mcp:VERSION:{tool['name']}"
            assert transcripts['baseline']['tools'] == transcripts['candidate']['tools']
            values = {arm: {row['request_id']: row['result']['structuredContent'] for row in data['results']} for arm, data in transcripts.items()}
            for arm in values:
                for result in values[arm].values():
                    if result.get('operation') == 'airust_status':
                        result['data'].pop('version')
            assert values['baseline'] == values['candidate']
            compared += len(values['baseline'])
            print(json.dumps({'stage': 'exploratory-wide-followup', 'pair': pair + 1, 'complete_response_pairs': compared}), flush=True)
    summary = {}
    for arm in ['baseline', 'candidate']:
        rows = [row for row in reports if row['arm'] == arm]
        summary[arm] = {'startup_ms': stats([row['startup_to_initialized_ms'] for row in rows]), 'peak_rss_bytes': stats([row['server_peak_rss_bytes'] for row in rows])}
        for phase, field in [('initial', 'initial_search_samples'), ('repeated', 'search_samples')]:
            summary[arm][phase + '_ms'] = {alg: stats([x['milliseconds'] for row in rows for x in row[field] if x['algorithm'] == alg]) for alg in ['bm25', 'tfidf']}
            summary[arm][phase + '_per_query_ms'] = {str(q): {alg: stats([x['milliseconds'] for row in rows for x in row[field] if x['algorithm'] == alg and x['query_number'] == q]) for alg in ['bm25', 'tfidf']} for q in range(10)}
    assert hashlib.sha256(source.read_bytes()).hexdigest() == original
    validated = sum(row['schema_validation']['actual_results_validated'] for row in reports)
    assert len(reports) == 20 and compared == 680 and validated == 1360
    proof = {'scope': 'Exploratory diagnostic planned after the complete selected-source campaigns because a wide repeated MCP median was slower. Ten alternating pairs; same immutable programs and input. No samples replace or merge into main campaigns, no unique timing cause inferred.', 'processes': 20, 'complete_response_pairs': compared, 'actual_schema_results_validated': validated, 'all_complete_responses_equal': True, 'input_unchanged': True}
    save('wide-followup-reports.json', reports)
    save('wide-followup-summary.json', summary)
    save('wide-followup-compatibility.json', proof)
    print(json.dumps(proof), flush=True)

if __name__ == '__main__':
    main()
