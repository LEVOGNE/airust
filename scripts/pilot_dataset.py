"""Independent, bounded verification of local pilot exports; no model calls."""
import argparse
import hashlib
import json
import math
from pathlib import Path
import re
import shutil
import stat

from reference_dataset import paragraphs

RECIPE = 'utf8-markdown-blank-paragraph-v1'
ASSURANCE = 'Caller declarations; neither traffic origin nor reviewer identity is authenticated.'


def encoded(value):
    return json.dumps(value, ensure_ascii=False, separators=(',', ':'), allow_nan=False).encode()


def digest(data):
    return hashlib.sha256(data).hexdigest()


def require(condition, message):
    if not condition:
        raise ValueError(message)


def read(path, maximum):
    require(stat.S_ISREG(path.lstat().st_mode), 'expected non-symlink regular file: ' + str(path))
    with path.open('rb') as stream:
        data = stream.read(maximum + 1)
    require(len(data) <= maximum, 'file byte budget: ' + str(path))
    return data


def parse(data):
    def pairs(items):
        result = {}
        for key, value in items:
            require(key not in result, 'duplicate JSON key: ' + key)
            result[key] = value
        return result

    def invalid(value):
        raise ValueError('nonfinite JSON number: ' + value)

    return json.loads(data, object_pairs_hook=pairs, parse_constant=invalid)


def fields(value, names):
    require(isinstance(value, dict) and set(value) == set(names.split()), 'unexpected schema fields')


def label(value, maximum):
    require(isinstance(value, str) and value.strip() and len(value.encode()) <= maximum
            and not any(ord(c) < 32 or 127 <= ord(c) < 160 for c in value), 'invalid label')


def source_range(source, documents):
    fields(source, 'document_id revision start_byte end_byte quote')
    snapshot = documents.get(source['document_id'])
    require(snapshot is not None and source['revision'] == snapshot['revision'], 'stale source revision')
    start, end, quote = source['start_byte'], source['end_byte'], source['quote']
    require(type(start) is int and type(end) is int and 0 <= start < end <= len(snapshot['raw']), 'source range bounds')
    require(isinstance(quote, str) and 0 < len(quote.encode()) <= 2000, 'source quote budget')
    require(snapshot['raw'][start:end].decode('utf-8') == quote, 'source quote mismatch')


def workspace(data):
    envelope = parse(data)
    fields(envelope, 'payload_sha256 payload')
    require(digest(envelope['payload'].encode()) == envelope['payload_sha256'], 'workspace envelope checksum')
    stored = parse(envelope['payload'])
    fields(stored, 'schema_version collection documents')
    require(stored['schema_version'] == 1 and 1 <= len(stored['documents']) <= 256, 'workspace version/count')
    label(stored['collection'], 128)
    documents = {}
    passages = 0
    total = 0
    for item in stored['documents']:
        fields(item, 'name snapshot')
        name = item['name']
        label(name, 768)
        require('\\' not in name and all(p not in {'', '.', '..'} for p in name.split('/')), 'source name')
        snapshot = parse(item['snapshot'])
        fields(snapshot, 'format_version document_id source recipe text_space text_sha256 revision text pages')
        raw = snapshot['text'].encode()
        total += len(raw)
        require(len(raw) <= 4 * 1024 * 1024 and total <= 32 * 1024 * 1024, 'source byte budget')
        require(snapshot['document_id'] == stored['collection'] + '/' + name, 'document binding')
        require(snapshot['format_version'] == 1 and snapshot['recipe'] == RECIPE
                and snapshot['text_space'] == 'supplied' and snapshot['pages'] == [], 'snapshot recipe/space')
        fields(snapshot['source'], 'sha256 byte_len')
        require(snapshot['source'] == {'sha256': digest(raw), 'byte_len': len(raw)}
                and snapshot['text_sha256'] == digest(raw), 'source/text fingerprint')
        framing = [b'airust/document/v1', snapshot['document_id'].encode(), digest(raw).encode(),
                   len(raw).to_bytes(8, 'little'), RECIPE.encode(), b'supplied', digest(raw).encode(), (0).to_bytes(8, 'little')]
        revision = digest(b''.join(len(part).to_bytes(8, 'little') + part for part in framing))
        require(revision == snapshot['revision'], 'snapshot revision framing')
        require(snapshot['document_id'] not in documents, 'duplicate document')
        documents[snapshot['document_id']] = {**snapshot, 'raw': raw}
        for _ in paragraphs(snapshot['text'], 1800):
            passages += 1
            require(passages <= 100000, 'workspace passage count')
    revisions = [[key, documents[key]['revision']] for key in sorted(documents)]
    identity = {'collection': stored['collection'], 'revision_sha256': digest(encoded([1, stored['collection'], RECIPE, revisions])),
                'documents': len(documents), 'passages': passages}
    return identity, documents


def validate_case(case, identity, documents):
    fields(case, 'schema_version captured_unix_ms implementation_version request workspace observation review previous_capture_sha256')
    require(case['schema_version'] == 1 and case['workspace'] == identity, 'case workspace/version')
    require(type(case['captured_unix_ms']) is int and 0 <= case['captured_unix_ms'] < 2**64, 'capture timestamp')
    label(case['implementation_version'], 128)
    parent = case['previous_capture_sha256']
    require(parent is None or isinstance(parent, str) and re.fullmatch('[0-9a-f]{64}', parent), 'parent hash')
    request = case['request']
    fields(request, 'case_id query language origin origin_evidence')
    require(re.fullmatch('[a-zA-Z0-9_-]{1,64}', request['case_id']), 'case ID')
    require(isinstance(request['query'], str) and request['query'].strip() and len(request['query'].encode()) <= 4096, 'query budget')
    require(re.fullmatch('[a-zA-Z-]{2,32}', request['language']), 'language tag')
    require(request['origin'] in {'user_supplied', 'user_question_replay', 'assistant_authored', 'synthetic_test', 'unknown'}, 'origin')
    label(request['origin_evidence'], 2048)
    observation = case['observation']
    fields(observation, 'recipe fallback_reason reranking query_ms truncated evidence')
    label(observation['recipe'], 4096)
    require(type(observation['query_ms']) in {int, float} and math.isfinite(observation['query_ms']) and observation['query_ms'] >= 0, 'query timing')
    require(type(observation['truncated']) is bool and len(observation['evidence']) <= 6, 'observation budget')
    require(sum(len(s['quote'].encode()) for s in observation['evidence']) <= 12000, 'total observed quote budget')
    require(observation['fallback_reason'] is None or len(observation['fallback_reason'].encode()) <= 4096, 'fallback budget')
    require(observation['reranking'] is None or len(encoded(observation['reranking'])) <= 8192, 'reranking budget')
    for source in observation['evidence']:
        source_range(source, documents)
    review = case['review']
    if review is not None:
        fields(review, 'reviewer reviewer_kind independence answerability relevant_sources notes')
        label(review['reviewer'], 128)
        require(review['reviewer_kind'] in {'human', 'assistant', 'test'} and review['independence'] in {'independent', 'same_author', 'unknown'}, 'review declaration')
        require(review['independence'] != 'independent' or review['reviewer_kind'] == 'human', 'independent review requires declared human role')
        require(review['answerability'] in {'supported', 'unsupported', 'uncertain'} and len(review['notes'].encode()) <= 4096, 'review label/notes')
        require(len(review['relevant_sources']) <= 32, 'review source budget')
        require(review['answerability'] != 'supported' or review['relevant_sources'], 'supported review needs sources')
        require(review['answerability'] != 'unsupported' or not review['relevant_sources'], 'unsupported review forbids positive sources')
        for source in review['relevant_sources']:
            source_range(source, documents)


def qualified(case):
    review = case['review']
    return (case['request']['origin'] == 'user_supplied' and review is not None
            and review['reviewer_kind'] == 'human' and review['independence'] == 'independent'
            and review['answerability'] in {'supported', 'unsupported'})


def validate(directory):
    require(stat.S_ISDIR(directory.lstat().st_mode) and stat.S_ISDIR((directory / 'workspace').lstat().st_mode), 'non-symlink export directories required')
    manifest = parse(read(directory / 'manifest.json', 16 * 1024))
    require(manifest['format'] == 'airust-pilot-export-v1' and manifest['schema_version'] == 1, 'export format')
    require(manifest['evaluation_ready'] is False, 'an export cannot certify evaluation readiness')
    expected = {'cases.jsonl': 32 * 1024 * 1024, 'workspace/workspace.json': 64 * 1024 * 1024}
    require(set(manifest['artifacts']) == set(expected), 'unexpected export artifacts')
    artifacts = {name: read(directory / name, limit) for name, limit in expected.items()}
    for name, data in artifacts.items():
        require(manifest['artifacts'][name] == {'sha256': digest(data), 'bytes': len(data)}, 'artifact fingerprint: ' + name)
    identity, documents = workspace(artifacts['workspace/workspace.json'])
    require(identity == manifest['workspace'], 'manifest workspace')
    rows = artifacts['cases.jsonl'].splitlines()
    require(1 <= len(rows) <= 256 and all(0 < len(row) <= 128 * 1024 for row in rows), 'export count/row budget')
    cases = [parse(row) for row in rows]
    for case in cases:
        validate_case(case, identity, documents)
    ids = [case['request']['case_id'] for case in cases]
    require(ids == sorted(set(ids)), 'duplicate/unsorted case IDs')
    human = lambda c: c['review'] is not None and c['review']['reviewer_kind'] == 'human' and c['review']['independence'] == 'independent'
    counts = {'cases': len(cases), 'declared_user_supplied': sum(c['request']['origin'] == 'user_supplied' for c in cases),
              'user_question_replays': sum(c['request']['origin'] == 'user_question_replay' for c in cases),
              'declared_independent_human_reviews': sum(human(c) for c in cases),
              'declared_live_and_independently_reviewed': sum(c['request']['origin'] == 'user_supplied' and human(c) for c in cases)}
    require(all(manifest[name] == count for name, count in counts.items()), 'declared counts differ')
    eligible = [c for c in cases if qualified(c)]
    unique = len({c['request']['query'].casefold().strip() for c in eligible})
    negatives = sum(c['review']['answerability'] == 'unsupported' for c in eligible)
    report = {**counts, 'eligible_settled_cases': len(eligible), 'unique_eligible_questions': unique,
              'eligible_unsupported': negatives, 'input_minimum_met': unique >= 50 and negatives > 0,
              'evaluation_ready': False, 'origin_assurance': ASSURANCE,
              'remaining': 'Independent provenance review, frozen source-family split/policy and measured quality/resource gates are required.'}
    return report, cases, manifest


def freeze(directory, plan_path, output):
    report, cases, manifest = validate(directory)
    require(report['input_minimum_met'], 'at least 50 unique declared genuine independently reviewed questions, including unsupported cases, are required')
    plan = parse(read(plan_path, 64 * 1024))
    fields(plan, 'schema_version dataset provenance_reviewed_by assignments policy')
    require(plan['schema_version'] == 1 and re.fullmatch('reference-v[2-9][0-9]*', plan['dataset']), 'new dataset version required')
    label(plan['provenance_reviewed_by'], 128)
    fields(plan['policy'], 'ndcg_at_10_minimum hit_at_6_minimum optional_ndcg_gain_minimum optional_hit_loss_maximum lexical_warm_p95_ms hybrid_warm_p95_ms rerank_warm_p95_ms worker_peak_rss_mib embedding_service_peak_rss_mib')
    require(all(type(v) in {int, float} and math.isfinite(v) and v >= 0 for v in plan['policy'].values()), 'finite predeclared budgets required')
    require(.75 <= plan['policy']['ndcg_at_10_minimum'] <= 1 and .80 <= plan['policy']['hit_at_6_minimum'] <= 1, 'quality floors')
    eligible = {c['request']['case_id']: c for c in cases if qualified(c)}
    require(report['unique_eligible_questions'] == len(eligible), 'duplicate eligible questions must be resolved before freezing')
    require(set(plan['assignments']) == set(eligible), 'assign every eligible case exactly once')
    families = {}
    documents = {}
    for identifier, assignment in plan['assignments'].items():
        fields(assignment, 'family split')
        label(assignment['family'], 128)
        split = assignment['split']
        require(split in {'development', 'evaluation'}, 'unknown split')
        for mapping, keys in [(families, [assignment['family']]), (documents, [s['document_id'] for s in eligible[identifier]['review']['relevant_sources']])]:
            for key in keys:
                require(key not in mapping or mapping[key] == split, 'source/family leakage between splits')
                mapping[key] = split
    for split in ['development', 'evaluation']:
        selected = [eligible[key] for key, a in plan['assignments'].items() if a['split'] == split]
        require(len(selected) >= 10 and any(c['review']['answerability'] == 'supported' for c in selected)
                and any(c['review']['answerability'] == 'unsupported' for c in selected), 'each split needs at least ten cases and both answerability labels')
    # Validate all inputs before reserving a new output. Publish completion last.
    output.mkdir(exist_ok=False)
    (output / 'workspace').mkdir()
    shutil.copyfile(directory / 'workspace/workspace.json', output / 'workspace/workspace.json')
    require(digest((output / 'workspace/workspace.json').read_bytes()) == manifest['artifacts']['workspace/workspace.json']['sha256'], 'workspace changed during freeze')
    (output / 'cases.jsonl').write_bytes(b''.join(encoded({**eligible[key], 'assignment': plan['assignments'][key]}) + b'\n' for key in sorted(eligible)))
    (output / 'plan.json').write_bytes(encoded(plan) + b'\n')
    receipt = {'format': 'airust-pilot-evaluation-freeze-v1', 'dataset': plan['dataset'], 'workspace': manifest['workspace'],
               'origin_assurance': ASSURANCE, 'source_export_sha256': digest(read(directory / 'manifest.json', 16 * 1024)),
               'quality_gate_passed': False, 'status': 'inputs frozen; no candidate execution or quality result asserted',
               'artifacts': {name: {'sha256': digest((output / name).read_bytes()), 'bytes': (output / name).stat().st_size}
                             for name in ['workspace/workspace.json', 'cases.jsonl', 'plan.json']}}
    (output / 'manifest.json').write_bytes(encoded(receipt) + b'\n')
    return receipt


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('export', type=Path)
    parser.add_argument('--freeze-plan', type=Path)
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    if bool(args.freeze_plan) != bool(args.output):
        parser.error('--freeze-plan and --output must be supplied together')
    result = freeze(args.export, args.freeze_plan, args.output) if args.freeze_plan else validate(args.export)[0]
    print(json.dumps(result, indent=2, ensure_ascii=False, allow_nan=False))


if __name__ == '__main__':
    main()
