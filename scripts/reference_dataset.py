import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / 'tests/fixtures/reference-v1'
POLICY = ROOT / 'docs/evaluation/reference-v1-policy.json'


def encoded(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode()


def digest(data):
    return hashlib.sha256(data).hexdigest()


def paragraphs(text, maximum):
    offset = 0
    for part in text.split('\n\n'):
        trimmed = part.strip()
        start = offset + len(part[:len(part) - len(part.lstrip())].encode())
        remaining = trimmed.encode()
        while remaining:
            cut = min(maximum, len(remaining))
            while cut < len(remaining) and remaining[cut] & 0xC0 == 0x80:
                cut -= 1
            if cut < len(remaining):
                boundary = max(remaining.rfind(b' ', 0, cut), remaining.rfind(b'\n', 0, cut))
                if boundary >= maximum // 2:
                    cut = boundary
            fragment = remaining[:cut].decode().rstrip()
            if fragment:
                yield start, start + len(fragment.encode()), fragment
            tail = remaining[cut:].decode()
            start += cut + len(tail.encode()) - len(tail.lstrip().encode())
            remaining = tail.lstrip().encode()
        offset += len(part.encode()) + 2


def prepare(fixture=FIXTURE, policy_path=POLICY):
    manifest = json.loads((fixture / 'source-manifest.json').read_text())
    policy = json.loads(policy_path.read_text())
    cases = [json.loads(line) for line in (fixture / 'cases.jsonl').read_text().splitlines() if line]
    documents = []
    corpus = []
    split_for_source = {}
    for source in manifest['files']:
        name = source['file']
        if Path(name).name != name or not name.endswith('.md'):
            raise ValueError('invalid source filename')
        raw = (fixture / 'sources' / name).read_bytes()
        if digest(raw) != source['sha256'] or len(raw) != source['bytes']:
            raise ValueError('source identity mismatch: ' + name)
        text = raw.decode('utf-8')
        documents.append({'id': name, 'text': text, 'sha256': digest(raw)})
        split_for_source[name] = source['question_split']
        for start, end, passage in paragraphs(text, policy['corpus']['passage_max_bytes']):
            if raw[start:end].decode() != passage:
                raise ValueError('invalid UTF-8 projection')
            corpus.append({'id': f'ref-v1:{name}:{start}:{end}', 'document': name, 'start': start, 'end': end, 'text': passage, 'collection': policy['corpus']['collection']})
    corpus.sort(key=lambda entry: entry['id'])
    guards = [{**entry, 'id': entry['id'].replace('ref-v1:', 'guard-v1:', 1), 'collection': policy['corpus']['guard_collection']} for entry in corpus[:16]]
    corpus.extend(guards)
    ids = set()
    queries = set()
    qrels = []
    source_text = {item['id']: item['text'] for item in documents}
    for case in cases:
        if case['id'] in ids or case['query'].casefold() in queries:
            raise ValueError('duplicate case or question')
        ids.add(case['id'])
        queries.add(case['query'].casefold())
        if case['origin'] != 'assistant_authored' or case['language'] not in {'de', 'en'}:
            raise ValueError('unsubstantiated question origin or language')
        if case['answerable']:
            source = case['source_file']
            quote = case['reference_quote']
            if split_for_source[source] != case['split'] or not quote or quote not in source_text[source]:
                raise ValueError('source split or reference mismatch: ' + case['id'])
            matches = [entry for entry in corpus if entry['collection'] == policy['corpus']['collection'] and quote in entry['text']]
            if not any(entry['document'] == source for entry in matches):
                raise ValueError('reference spans multiple passages: ' + case['id'])
            qrels.extend({'query_id': case['id'], 'document_id': entry['id'], 'grade': 2} for entry in matches)
        elif case['source_file'] is not None or case['reference_quote'] is not None:
            raise ValueError('negative case must not have a positive reference')
    counts = {}
    for split, total in [('development', 90), ('evaluation', 60)]:
        selected = [case for case in cases if case['split'] == split]
        languages = Counter(case['language'] for case in selected)
        negative = sum(not case['answerable'] for case in selected)
        if len(selected) != total or negative != total // 5 or languages != {'de': total // 2, 'en': total // 2}:
            raise ValueError('question balance does not match the frozen policy')
        counts[split] = {'questions': total, 'answerable': total - negative, 'unanswerable': negative, 'languages': dict(languages), 'families': len({case['family'] for case in selected})}
    if len(cases) != 150 or len(ids) != 150:
        raise ValueError('expected 150 questions')
    artifacts = {
        'documents.json': encoded(documents) + b'\n',
        'corpus.jsonl': b''.join(encoded(entry) + b'\n' for entry in corpus),
        'cases.jsonl': b''.join(encoded(case) + b'\n' for case in sorted(cases, key=lambda case: case['id'])),
        'qrels.jsonl': b''.join(encoded(row) + b'\n' for row in sorted(qrels, key=lambda row: (row['query_id'], row['document_id']))),
    }
    inputs = {'source_manifest_sha256': digest((fixture / 'source-manifest.json').read_bytes()), 'cases_sha256': digest((fixture / 'cases.jsonl').read_bytes()), 'policy_sha256': digest(policy_path.read_bytes())}
    identity = {
        'format_version': 1, 'dataset': manifest['dataset'], 'identity_sha256': digest(encoded(inputs)),
        'source_commit': manifest['source_commit'], 'inputs': inputs,
        'documents': len(documents), 'source_bytes': sum(len(document['text'].encode()) for document in documents),
        'corpus_entries': len(corpus), 'scope_guard_entries': len(guards), 'question_origins': {'assistant_authored': 150, 'historical_user_traffic': 0},
        'splits': counts, 'judgment_rows': len(qrels),
        'artifacts': {name: {'sha256': digest(data), 'bytes': len(data)} for name, data in artifacts.items()},
    }
    return identity, artifacts


def validate(fixture=FIXTURE, policy_path=POLICY):
    identity, artifacts = prepare(fixture, policy_path)
    frozen = json.loads((fixture / 'fixture-manifest.json').read_text())
    if frozen != identity:
        raise ValueError('dataset or policy changed; version one is frozen')
    return identity, artifacts


def main():
    parser = argparse.ArgumentParser(description='Validate and export the frozen AIRust reference dataset.')
    parser.add_argument('--freeze', action='store_true')
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    if args.freeze:
        identity, artifacts = prepare()
        with (FIXTURE / 'fixture-manifest.json').open('x') as stream:
            stream.write(json.dumps(identity, indent=2) + '\n')
    else:
        identity, artifacts = validate()
    if args.output:
        args.output.mkdir(parents=True, exist_ok=False)
        for name, data in artifacts.items():
            (args.output / name).write_bytes(data)
        (args.output / 'manifest.json').write_text(json.dumps(identity, indent=2) + '\n')
        (args.output / 'policy.json').write_bytes(POLICY.read_bytes())
    print(json.dumps(identity, indent=2))


if __name__ == '__main__':
    main()
