"""Additional fixed controls; private full responses never enter public reports."""
import hashlib
import json
import pathlib
import subprocess
import sys
import benchmark as common

ROOT, BASE, WORKERS = common.ROOT, common.BASE, common.WORKERS
read, save, stats, verify = common.read, common.save, common.stats, common.verify

def controlled():
    verify()
    reports, samples = [], []
    for round_number in range(7):
        ready_pair = {}
        for arm in (['baseline', 'candidate'] if round_number % 2 == 0 else ['candidate', 'baseline']):
            folder = BASE / f'controlled/{round_number}-{arm}'
            folder.mkdir(parents=True)
            with (folder / 'stderr.log').open('w') as errors:
                process = subprocess.Popen([str(WORKERS / ('controlled-' + arm)), str(ROOT / 'local-lab/pdf-stability/imports/canonical-combined-0315.json'), 'matrix', '10'], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=errors, text=True)
                ready = json.loads(process.stdout.readline())
                assert ready['entries'] == 167942 and ready['version'] == {'baseline': '0.4.23', 'candidate': '0.4.24'}[arm]
                (folder / 'ready-private.json').write_text(json.dumps(ready) + '\n')
                ready_pair[arm] = ready
                process.stdin.write('RUN\n')
                process.stdin.flush()
                result = json.loads(process.stdout.read())
                assert process.wait(timeout=120) == 0
            assert len(result['rows']) == 600
            save(f'controlled/{round_number}-{arm}/samples.json', result)
            reports.append({'arm': arm, 'round_pair': round_number, 'version': ready['version'], 'entries': ready['entries'], 'candidate_counts': ready['candidate_counts'], 'calls': len(result['rows'])})
            samples.extend({'arm': arm, 'round_pair': round_number, **row} for row in result['rows'])
        assert ready_pair['baseline']['responses'] == ready_pair['candidate']['responses']
        assert ready_pair['baseline']['candidate_counts'] == ready_pair['candidate']['candidate_counts']
        print(json.dumps({'stage': 'controlled', 'round_pair': round_number + 1, 'complete_original_responses_equal': True}), flush=True)
    summary = {}
    for mode in ['plain', 'deadline', 'progress']:
        summary[mode] = {}
        for algorithm in ['Bm25', 'Tfidf']:
            rs = [r for r in samples if r['control'] == mode and r['algorithm'] == algorithm]
            for query_number in range(10):
                assert len({r['signature'] for r in rs if r['query_number'] == query_number}) == 1
            summary[mode][algorithm] = {arm: stats([r['elapsed_ns'] / 1e6 for r in rs if r['arm'] == arm]) for arm in ['baseline', 'candidate']}
            summary[mode][algorithm]['per_query_ms'] = {str(q): {arm: stats([r['elapsed_ns'] / 1e6 for r in rs if r['arm'] == arm and r['query_number'] == q]) for arm in ['baseline', 'candidate']} for q in range(10)}
    save('controlled-reports.json', reports)
    save('controlled-samples.json', samples)
    save('controlled-summary.json', summary)
    save('controlled-compatibility.json', {'processes': 14, 'timed_calls': len(samples), 'initial_full_response_comparisons': 140, 'all_original_responses_equal': True, 'all_timing_signatures_equal': True, 'scope': 'Warm canonical Retriever calls in one process with plain/deadline/progress controls, rotated within each round; no stdio transport, startup or profiler in call timer. Private readiness retains full JSON.'})

def reference():
    verify()
    arms = ['reference-0422', 'baseline', 'candidate']
    versions = {'reference-0422': '0.4.22', 'baseline': '0.4.23', 'candidate': '0.4.24'}
    binary = {arm: WORKERS / ('mcp-pre-regression-0422' if arm == 'reference-0422' else 'mcp-' + arm) for arm in arms}
    source = ROOT / 'local-lab/pdf-stability/imports/canonical-combined-0315.json'
    original = hashlib.file_digest(source.open('rb'), 'sha256').hexdigest()
    rows, comparisons = [], 0
    for round_number in range(5):
        order = arms[round_number % 3:] + arms[:round_number % 3]
        if round_number % 2:
            order.reverse()
        transcripts = {}
        for arm in order:
            folder = BASE / f'reference/{round_number}-{arm}'
            folder.mkdir(parents=True)
            report, transcript, schema = folder / 'report.json', folder / 'transcript-private.json', folder / 'schema.json'
            with (folder / 'probe.log').open('w') as log:
                subprocess.run([sys.executable, '-B', str(pathlib.Path(__file__).with_name('mcp_probe.py')), '--binary', str(binary[arm]), '--knowledge', str(source), '--queries', str(BASE / 'mcp-queries.json'), '--report', str(report), '--transcript', str(transcript)], stdout=log, stderr=subprocess.STDOUT, check=True, timeout=240)
            result = subprocess.run([str(WORKERS / 'schema'), str(transcript), str(schema)], capture_output=True, text=True, timeout=120)
            (folder / 'schema.log').write_text(result.stdout + result.stderr)
            assert result.returncode == 0
            data = json.loads(report.read_text())
            assert data['version'] == versions[arm]
            rows.append({'arm': arm, 'round_pair': round_number, **data, 'schema_validation': json.loads(schema.read_text())})
            transcripts[arm] = json.loads(transcript.read_text())
        values = {}
        for arm, transcript in transcripts.items():
            for tool in transcript['tools']:
                assert tool['outputSchema']['$id'] == f"urn:airust:mcp:{versions[arm]}:{tool['name']}"
                tool['outputSchema']['$id'] = f"urn:airust:mcp:VERSION:{tool['name']}"
            values[arm] = {r['request_id']: r['result']['structuredContent'] for r in transcript['results']}
            for value in values[arm].values():
                if value.get('operation') == 'airust_status':
                    assert value['data'].pop('version') == versions[arm]
        assert all(transcripts[arm]['tools'] == transcripts['candidate']['tools'] for arm in arms)
        for arm in arms[:-1]:
            assert values[arm] == values['candidate']
            comparisons += len(values[arm])
        print(json.dumps({'stage': 'reference', 'round_pair': round_number + 1, 'three_versions_equal': True}), flush=True)
    summary = {}
    for arm in arms:
        rs = [r for r in rows if r['arm'] == arm]
        summary[arm] = {'version': versions[arm], 'processes': len(rs), 'startup_ms': stats([r['startup_to_initialized_ms'] for r in rs]), 'peak_child_rss_bytes': stats([r['server_peak_rss_bytes'] for r in rs])}
        for phase, field in [('initial', 'initial_search_samples'), ('repeated', 'search_samples')]:
            summary[arm][phase + '_ms'] = {alg: stats([x['milliseconds'] for r in rs for x in r[field] if x['algorithm'] == alg]) for alg in ['bm25', 'tfidf']}
        summary[arm]['per_query_warm_ms'] = {str(q): {alg: stats([x['milliseconds'] for r in rs for x in r['search_samples'] if x['algorithm'] == alg and x['query_number'] == q]) for alg in ['bm25', 'tfidf']} for q in range(10)}
    assert hashlib.file_digest(source.open('rb'), 'sha256').hexdigest() == original
    validated = sum(r['schema_validation']['actual_results_validated'] for r in rows)
    assert len(rows) == 15 and comparisons == 680 and validated == 1020
    save('reference-reports.json', rows)
    save('reference-summary.json', summary)
    save('reference-compatibility.json', {'processes': 15, 'versions': versions, 'complete_response_comparisons': comparisons, 'schema_validated_results': validated, 'complete_payloads_equal_except_status_version': True, 'tool_schemas_equal_except_asserted_version_urn': True, 'canonical_input_unchanged': True, 'scope': 'Separate rotating three-version canonical control; same programs/data; fresh0.4.22/0.4.23/0.4.24 builds. No samples merged into the two ordinary campaigns.'})

if __name__ == '__main__':
    {'controlled': controlled, 'reference': reference}[sys.argv[1]]()
