"""Preplanned fresh small Core triples; every sample and complete comparison is retained."""
import json
import subprocess
import sys
import benchmark as common

ROOT, BASE, WORKERS = common.ROOT, common.BASE, common.WORKERS
read, save, stats, verify = common.read, common.save, common.stats, common.verify

def run(campaign):
    assert campaign in ['small-reference', 'small-confirmation']
    verify()
    arms = ['reference', 'baseline', 'candidate']
    versions = {'reference': '0.4.21', 'baseline': '0.4.25', 'candidate': '0.5.0'}
    samples, compared, total_calls = [], 0, 0
    for round_number in range(7):
        order = arms[round_number % 3:] + arms[:round_number % 3]
        if round_number % 2:
            order.reverse()
        values, signatures = {}, {}
        for arm in order:
            folder = BASE / campaign / f'{round_number}-{arm}'
            folder.mkdir(parents=True)
            for program in ['old-six', 'matrix']:
                binary = WORKERS / (('query-reference-plain' if arm == 'reference' else f'query-{arm}-plain') if program == 'old-six' else f'small-matrix-{arm}')
                command = [str(binary), 'small_time', '1000'] if program == 'old-six' else [str(binary), '1000']
                result = subprocess.run(command, capture_output=True, text=True, check=True, timeout=180)
                (folder / f'{program}.stderr').write_text(result.stderr)
                report = json.loads(result.stdout)
                assert report['version'] == versions[arm]
                assert len(report['rows']) == (6 if program == 'old-six' else 96)
                (folder / f'{program}-private.json').write_text(json.dumps(report, indent=2) + '\n')
                samples.extend({'arm': arm, 'round_triple': round_number, 'program': program, **row} for row in report['rows'])
                total_calls += sum(row['iterations'] for row in report['rows'])
                signatures[(arm, program)] = [(row['fixture'], row['signature']) for row in report['rows']]
                if program == 'matrix':
                    values[arm] = report['complete_results']
        assert all(values[arm] == values['candidate'] for arm in arms)
        assert all(signatures[(arm, program)] == signatures[('candidate', program)] for arm in arms for program in ['old-six', 'matrix'])
        compared += len(values['candidate']) * 2
        print(json.dumps({'stage': campaign, 'round_triple': round_number + 1, 'complete_comparisons': compared}), flush=True)
    summary = []
    for program in ['old-six', 'matrix']:
        for fixture in sorted({row['fixture'] for row in samples if row['program'] == program}):
            rows = [row for row in samples if row['program'] == program and row['fixture'] == fixture]
            assert len(rows) == 21 and len({row['signature'] for row in rows}) == 1
            times = {arm: stats([row['ns_per_call'] for row in rows if row['arm'] == arm]) for arm in arms}
            summary.append({'program': program, 'fixture': fixture, 'entries': rows[0]['entries'], 'algorithm': rows[0]['algorithm'], 'case': rows[0].get('case', 'original_single'), 'time_ns_per_call': times,
                'candidate_vs_025_percent': 100 * (times['candidate']['median'] / times['baseline']['median'] - 1),
                'candidate_vs_021_percent': 100 * (times['candidate']['median'] / times['reference']['median'] - 1)})
    proof = {'versions': versions, 'rotating_triples': 7, 'processes': 42, 'calls': total_calls,
        'old_six_fixtures': 6, 'matrix_fixtures': 96, 'complete_matrix_response_comparisons': compared,
        'complete_results_equal': True, 'all_signatures_equal': True,
        'scope': 'Separate fixed small Core three-version campaign; all warmed setup and original JSON checks outside timing. No samples merged with ordinary or confirmation campaigns.'}
    assert total_calls == 2142000 and compared == 1344
    save(campaign + '-samples.json', samples)
    save(campaign + '-summary.json', summary)
    save(campaign + '-compatibility.json', proof)
    print(json.dumps(proof), flush=True)

if __name__ == '__main__':
    run(sys.argv[1])
