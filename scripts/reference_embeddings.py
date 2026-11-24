import argparse
import hashlib
import json
import math
from pathlib import Path
import time
import urllib.parse
import urllib.request


def request(base, path, body=None):
    data = None if body is None else json.dumps(body).encode()
    query = urllib.request.Request(base + path, data=data, headers={'Content-Type': 'application/json'})
    with urllib.request.urlopen(query, timeout=120) as response:
        return json.load(response)


def main():
    parser = argparse.ArgumentParser(description='Explicitly prepare reference vectors with an installed local Ollama model.')
    parser.add_argument('dataset', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--url', default='http://127.0.0.1:11434')
    parser.add_argument('--model', default='bge-m3:latest')
    args = parser.parse_args()
    parsed = urllib.parse.urlsplit(args.url)
    if parsed.scheme != 'http' or parsed.hostname not in {'127.0.0.1', 'localhost', '::1'} or parsed.path or parsed.query or parsed.fragment or parsed.username:
        raise SystemExit('Only an explicit local HTTP Ollama endpoint is supported.')
    if args.output.exists():
        raise SystemExit('The vector output already exists; retain it or choose a new path.')
    version = request(args.url, '/api/version')
    model = next(item for item in request(args.url, '/api/tags')['models'] if item['name'] == args.model)
    rows = [json.loads(line) for line in (args.dataset / 'corpus.jsonl').read_text().splitlines()]
    cases = [json.loads(line) for line in (args.dataset / 'cases.jsonl').read_text().splitlines()]
    manifest = json.loads((args.dataset / 'manifest.json').read_text())
    dimensions = None

    def embed(texts, keep_alive='10m'):
        nonlocal dimensions
        output = request(args.url, '/api/embed', {'model': args.model, 'input': texts, 'truncate': False, 'keep_alive': keep_alive})
        vectors = output['embeddings']
        if len(vectors) != len(texts):
            raise ValueError('embedding count mismatch')
        for vector in vectors:
            if dimensions is None:
                dimensions = len(vector)
            if not dimensions or len(vector) != dimensions or not all(math.isfinite(v) for v in vector) or not any(vector):
                raise ValueError('invalid embedding vector')
        return vectors

    started = time.monotonic()
    cache = {}
    unique = list(dict.fromkeys(row['text'] for row in rows))
    for offset in range(0, len(unique), 16):
        batch = unique[offset:offset + 16]
        cache.update(zip(batch, embed(batch)))
        print('Embedded documents', min(offset + 16, len(unique)), '/', len(unique), flush=True)
    document_seconds = time.monotonic() - started
    queries = {}
    query_timings = {}
    for index, case in enumerate(cases):
        started = time.monotonic()
        queries[case['query']] = embed([case['query']], keep_alive=0 if index == len(cases) - 1 else '10m')[0]
        query_timings[case['id']] = (time.monotonic() - started) * 1000
    after = next(item for item in request(args.url, '/api/tags')['models'] if item['name'] == args.model)
    if model['digest'] != after['digest']:
        raise ValueError('model identity changed during vector preparation')
    output = {
        'format_version': 1, 'dataset_identity': manifest['identity_sha256'],
        'corpus_sha256': hashlib.sha256((args.dataset / 'corpus.jsonl').read_bytes()).hexdigest(),
        'model': args.model, 'model_digest': model['digest'], 'service_version': version,
        'recipe': 'ollama:' + args.model + '@' + model['digest'] + ';raw-text;truncate=false',
        'dimensions': dimensions, 'document_prepare_seconds': document_seconds,
        'single_query_embedding_ms': query_timings,
        'last_query_unloads_model': True,
        'documents': {row['id']: cache[row['text']] for row in rows}, 'queries': queries,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    with args.output.open('x') as stream:
        json.dump(output, stream, separators=(',', ':'))
        stream.write('\n')
    print(json.dumps({'model': args.model, 'digest': model['digest'], 'dimensions': dimensions, 'documents': len(rows), 'queries': len(queries), 'document_prepare_seconds': document_seconds}))


if __name__ == '__main__':
    main()
