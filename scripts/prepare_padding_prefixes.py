"""Reconstruct observed reference candidate texts for static tokenizer analysis.

Uses the published pair-coordinate CSV and a verified frozen reference workspace.
No retrieval/inference/model/service is executed; candidate ranks are observations.
"""
import argparse
import csv
import hashlib
import io
import json
from pathlib import Path

from pilot_dataset import digest, read, require, workspace
from reference_dataset import FIXTURE, validate


def framed(parts):
    return digest(b''.join(len(part).to_bytes(8, 'little') + part for part in parts))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('pairs_csv', type=Path)
    parser.add_argument('reference_workspace', type=Path)
    parser.add_argument('new_prefixes_json', type=Path)
    args = parser.parse_args()
    identity, _ = validate()
    info, documents = workspace(read(args.reference_workspace / 'workspace.json', 64 * 1024 * 1024))
    require(info['collection'] == 'airust-reference' and info['documents'] == identity['documents'],
            'expected complete frozen reference workspace')
    for identifier, document in documents.items():
        name = identifier.removeprefix('airust-reference/')
        require(document['raw'] == read(FIXTURE / 'sources' / name, 4 * 1024 * 1024),
                'workspace differs from frozen reference source')
    cases = {row['id']: row for row in map(json.loads, (FIXTURE / 'cases.jsonl').read_text().splitlines())}
    data = read(args.pairs_csv, 8 * 1024 * 1024)
    reader = csv.DictReader(io.StringIO(data.decode('utf-8')))
    expected = {'id', 'document_id', 'position', 'text_sha256', 'token_ids_sha256',
                'tokens', 'untruncated_tokens', 'truncated', 'query_sha256',
                'source_document_id', 'source_revision', 'start_byte', 'end_byte'}
    require(len(reader.fieldnames or []) == len(expected) and set(reader.fieldnames) == expected,
            'unexpected pair CSV schema')
    groups = {}
    for count, row in enumerate(reader, 1):
        require(count <= 3000 and None not in row and all(value is not None for value in row.values()),
                'pair count or CSV fields')
        identifier = row['id']
        require(identifier in cases and row['query_sha256'] == digest(cases[identifier]['query'].encode()),
                'query identity mismatch')
        position = int(row['position'])
        require(0 <= position < 20, 'candidate rank bounds')
        source = row['source_document_id']
        require(source in documents and row['source_revision'] == documents[source]['revision'],
                'source identity mismatch')
        start, end = int(row['start_byte']), int(row['end_byte'])
        require(0 <= start < end <= len(documents[source]['raw']), 'source span bounds')
        text = documents[source]['raw'][start:end].decode('utf-8')
        require(len(text.encode()) <= 1800 and row['text_sha256'] == digest(text.encode()),
                'passage text mismatch')
        passage = 'airust-passage-v1:' + framed([
            b'airust/passage/v1', source.encode(), row['source_revision'].encode(),
            start.to_bytes(8, 'little'), end.to_bytes(8, 'little')])
        require(row['document_id'] == passage, 'passage ID framing mismatch')
        group = groups.setdefault(identifier, {})
        require(position not in group, 'duplicate candidate rank')
        group[position] = {'id': passage, 'text': text}
    require(set(groups) == set(cases), 'every frozen query must be represented')
    prefixes = []
    for identifier, group in groups.items():
        require(set(group) == set(range(20)) and len({entry['id'] for entry in group.values()}) == 20,
                'every query needs its twenty distinct observed candidates')
        prefixes.append({'id': identifier, 'query': cases[identifier]['query'],
                         'candidates': [group[position] for position in range(20)]})
    encoded = (json.dumps(prefixes, ensure_ascii=False, separators=(',', ':'), allow_nan=False) + '\n').encode()
    require(len(encoded) <= 64 * 1024 * 1024, 'output byte budget')
    with args.new_prefixes_json.open('xb') as stream:
        stream.write(encoded)
    print(json.dumps({'queries': len(prefixes), 'pairs': 3000, 'output_sha256': hashlib.sha256(encoded).hexdigest(),
                      'scope': 'Reconstructed frozen authored inputs; no retrieval/model/service execution.'}))


if __name__ == '__main__':
    main()
