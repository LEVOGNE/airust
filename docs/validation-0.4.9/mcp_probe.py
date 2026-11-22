import argparse
import hashlib
import json
import os
import pathlib
import resource
import sys
import selectors
import subprocess
import time


class Connection:
    def __init__(self, command, capture_transcript=False):
        self.process = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                        stderr=subprocess.PIPE, bufsize=0)
        self.selector = selectors.DefaultSelector()
        self.selector.register(self.process.stdout, selectors.EVENT_READ)
        self.buffer = b""
        self.counter = 0
        self.capture_transcript = capture_transcript
        self.transcript = {"tools": [], "results": []}
        self.tool_requests = {}
        self.list_requests = set()

    def send(self, value):
        if self.capture_transcript and "id" in value:
            if value.get("method") == "tools/call":
                self.tool_requests[value["id"]] = value["params"]["name"]
            elif value.get("method") == "tools/list":
                self.list_requests.add(value["id"])
        self.process.stdin.write(json.dumps(value, ensure_ascii=False).encode() + b"\n")
        self.process.stdin.flush()

    def receive(self, timeout=30):
        deadline = time.monotonic() + timeout
        while b"\n" not in self.buffer:
            remaining = deadline - time.monotonic()
            if remaining <= 0 or not self.selector.select(remaining):
                raise TimeoutError("MCP response watchdog")
            chunk = os.read(self.process.stdout.fileno(), 65536)
            if not chunk:
                raise RuntimeError("Unexpected MCP EOF")
            self.buffer += chunk
            if len(self.buffer) > 1024 * 1024:
                raise RuntimeError("Validator response limit")
        line, self.buffer = self.buffer.split(b"\n", 1)
        value = json.loads(line)
        if self.capture_transcript and "id" in value:
            identifier = value["id"]
            if identifier in self.list_requests:
                self.transcript["tools"] = value["result"]["tools"]
                self.list_requests.remove(identifier)
            if identifier in self.tool_requests:
                name = self.tool_requests.pop(identifier)
                if "structuredContent" in value.get("result", {}):
                    self.transcript["results"].append({"request_id": identifier, "tool": name, "result": value["result"]})
        return value

    def request(self, method, params=None, progress=False):
        self.counter += 1
        params = dict(params or {})
        token = "probe-" + str(self.counter)
        if progress:
            params["_meta"] = {"progressToken": token}
        request = {"jsonrpc": "2.0", "id": self.counter, "method": method}
        if params:
            request["params"] = params
        self.send(request)
        events, previous = [], -1
        while True:
            value = self.receive()
            if value.get("method") == "notifications/progress":
                assert progress
                event = value["params"]
                assert event["progressToken"] == token and event["progress"] > previous
                assert "total" not in event and isinstance(event["message"], str)
                previous = event["progress"]
                events.append(event)
            else:
                assert value["id"] == self.counter, value
                return value, events

    def close(self):
        self.process.stdin.close()
        try:
            result = self.process.wait(timeout=5)
            error = self.process.stderr.read()
            assert result == 0, error.decode(errors="replace")
        finally:
            self.selector.close()
            if self.process.poll() is None:
                self.process.kill()
                self.process.wait()


def digest(path):
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def main():
    parser = argparse.ArgumentParser(description="Read-only AIRust stdio protocol validator; local source text is not printed.")
    parser.add_argument("--binary", type=pathlib.Path, required=True)
    source = parser.add_mutually_exclusive_group(required=True)
    source.add_argument("--knowledge", type=pathlib.Path)
    source.add_argument("--document", type=pathlib.Path)
    parser.add_argument("--queries", type=pathlib.Path, required=True)
    parser.add_argument("--expected", type=pathlib.Path)
    parser.add_argument("--report", type=pathlib.Path, required=True)
    parser.add_argument("--transcript", type=pathlib.Path)
    args = parser.parse_args()
    path = args.knowledge or args.document
    before = digest(path)
    query_bytes = args.queries.read_bytes()
    queries = json.loads(query_bytes)
    assert queries, "Supply at least one query"
    expected = json.loads(args.expected.read_text()) if args.expected else None
    snapshot = json.loads(path.read_text()) if args.document else None
    text_bytes = snapshot["text"].encode() if snapshot else None
    command = [str(args.binary.resolve()), "--knowledge" if args.knowledge else "--document", str(path.resolve())]
    start = time.monotonic()
    client = Connection(command, capture_transcript=args.transcript is not None)
    timings, total_hits, checked_spans, events_count = [], 0, 0, 0
    first_hit_id, extra_tool_calls = None, 0
    try:
        initialized, _ = client.request("initialize", {"protocolVersion": "2025-11-25", "clientInfo": {"name": "airust-public-probe", "version": "1"}, "capabilities": {}})
        assert initialized["result"]["serverInfo"]["name"] == "airust"
        version = initialized["result"]["serverInfo"]["version"]
        ready_ms = (time.monotonic() - start) * 1000
        client.send({"jsonrpc": "2.0", "method": "notifications/initialized"})
        listing, _ = client.request("tools/list")
        assert len(listing["result"]["tools"]) == 5
        for tool in listing["result"]["tools"]:
            assert tool["annotations"]["readOnlyHint"] and not tool["annotations"]["openWorldHint"]
        for number, query in enumerate(queries):
            query = query if isinstance(query, str) else query["query"]
            for algorithm in ["bm25", "tfidf"]:
                started = time.monotonic()
                response, events = client.request("tools/call", {"name": "airust_search", "arguments": {"query": query, "algorithm": algorithm}}, progress=True)
                elapsed = (time.monotonic() - started) * 1000
                assert not response["result"].get("isError", False), response
                data = response["result"]["structuredContent"]["data"]
                assert data["untrusted_source_content"]
                hits = data["retrieval"]["hits"]
                assert len(hits) <= 5 and len(hits) == len(data["sources"])
                if hits and first_hit_id is None:
                    first_hit_id = hits[0]["id"]
                if expected is not None:
                    assert data["retrieval"] == expected[number][algorithm]
                for hit, binding in zip(hits, data["sources"]):
                    assert hit["id"] == binding["entry_id"]
                    if snapshot:
                        assert binding["status"] == "snapshot_bound"
                        passage = binding["passage"]
                        assert passage["revision"] == snapshot["revision"]
                        span = passage["span"]
                        assert text_bytes[span["start_byte"]:span["end_byte"]].decode() == hit["excerpt"]
                        read, _ = client.request("tools/call", {"name": "airust_read_passage", "arguments": {"document_id": passage["document_id"], "revision": passage["revision"], "start_byte": span["start_byte"], "end_byte": span["end_byte"]}})
                        assert read["result"]["structuredContent"]["data"]["text"] == hit["excerpt"]
                        checked_spans += 1
                    else:
                        assert binding["status"] in ["not_attached", "snapshot_unavailable"]
                events_count += len(events)
                total_hits += len(hits)
                timings.append({"query_number": number, "algorithm": algorithm, "milliseconds": elapsed, "hits": len(hits), "progress_events": len(events)})
        pending, last_progress, concurrent_events = {}, {}, 0
        concurrent_started = time.monotonic()
        for offset in range(4):
            number = offset % len(queries)
            item = queries[number]
            query = item if isinstance(item, str) else item["query"]
            client.counter += 1
            identifier = client.counter
            token = "parallel-" + str(identifier)
            pending[identifier] = number
            last_progress[token] = -1
            client.send({"jsonrpc": "2.0", "id": identifier, "method": "tools/call", "params": {"name": "airust_search", "arguments": {"query": query}, "_meta": {"progressToken": token}}})
        while pending:
            value = client.receive()
            if value.get("method") == "notifications/progress":
                event = value["params"]
                token = event["progressToken"]
                assert token in last_progress and event["progress"] > last_progress[token]
                assert "total" not in event
                last_progress[token] = event["progress"]
                concurrent_events += 1
            else:
                number = pending.pop(value["id"])
                assert not value["result"].get("isError", False), value
                if expected is not None:
                    assert value["result"]["structuredContent"]["data"]["retrieval"] == expected[number]["bm25"]
        concurrent_ms = (time.monotonic() - concurrent_started) * 1000
        if args.transcript:
            first_query = queries[0] if isinstance(queries[0], str) else queries[0]["query"]
            matched, _ = client.request("tools/call", {"name": "airust_find_matches", "arguments": {"text": first_query, "limit": 1, "min_similarity": 0}})
            assert not matched["result"].get("isError", False)
            extra_tool_calls += 1
            if first_hit_id is not None:
                related, _ = client.request("tools/call", {"name": "airust_relate", "arguments": {"id": first_hit_id, "limit": 1}})
                assert not related["result"].get("isError", False)
                extra_tool_calls += 1
        rejected, _ = client.request("tools/call", {"name": "airust_search", "arguments": {"query": "Rust", "path": "/etc/passwd"}})
        assert rejected["result"]["isError"]
        status, _ = client.request("tools/call", {"name": "airust_status", "arguments": {}})
        assert status["result"]["structuredContent"]["data"]["read_only"]
    finally:
        client.close()
    if args.transcript:
        with args.transcript.open("x") as output:
            json.dump(client.transcript, output, ensure_ascii=False)
            output.write("\n")
    assert digest(path) == before
    peak_rss = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
    peak_bytes = int(peak_rss if sys.platform == "darwin" else peak_rss * 1024)
    report = {"additional_matching_relation_calls": extra_tool_calls, "server_peak_rss_bytes": peak_bytes, "concurrent_search_calls": 4, "concurrent_complete_results_checked": True, "concurrent_elapsed_ms": concurrent_ms, "concurrent_progress_events": concurrent_events, "version": version, "mode": "document" if snapshot else "knowledge", "input_sha256": before, "query_file_sha256": hashlib.sha256(query_bytes).hexdigest(), "queries": len(queries), "search_calls": len(timings), "hits": total_hits, "snapshot_spans_and_readbacks": checked_spans, "progress_events": events_count, "progress_sequences_checked": True, "input_unchanged": True, "direct_retriever_equality_checked": expected is not None, "startup_to_initialized_ms": ready_ms, "search_samples": timings, "scope": "Warm local stdio round trips; search samples include progress and response transport, exclude exact passage follow-up calls. No native PDF parsing or relevance judgment."}
    with args.report.open("x") as output:
        json.dump(report, output, indent=2)
        output.write("\n")
    print(json.dumps({key: value for key, value in report.items() if key != "search_samples"}))


if __name__ == "__main__":
    main()
