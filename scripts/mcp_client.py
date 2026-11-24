"""Small bounded stdio client used by integration tests and measurement tools."""
import json
import os
import queue
import subprocess
import sys
import tempfile
import threading
import time


class StdioClient:
    def __init__(self, command, timeout=10):
        self.timeout = timeout
        self.errors = tempfile.TemporaryFile()
        self.process = subprocess.Popen(list(map(str, command)), stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=self.errors)
        self.messages = queue.Queue(maxsize=1024)
        self.next_id = 1
        self.usage = None
        self.reader = threading.Thread(target=self._read, daemon=True)
        self.reader.start()
        try:
            self.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {}, 'clientInfo': {'name': 'airust-validation', 'version': '1'}})
            self.send({'jsonrpc': '2.0', 'method': 'notifications/initialized'})
        except BaseException:
            self.close()
            raise

    def _read(self):
        try:
            while True:
                line = self.process.stdout.readline(1024 * 1024 + 1)
                if not line:
                    self.messages.put_nowait(RuntimeError('unexpected MCP EOF'))
                    return
                if len(line) > 1024 * 1024:
                    raise ValueError('MCP frame exceeds client byte budget')
                self.messages.put_nowait(json.loads(line))
        except (ValueError, OSError, queue.Full) as error:
            try:
                self.messages.put_nowait(error)
            except queue.Full:
                pass

    def send(self, value):
        data = (json.dumps(value, ensure_ascii=False, allow_nan=False, separators=(',', ':')) + '\n').encode()
        if len(data) > 1024 * 1024:
            raise ValueError('client request byte budget')
        self.process.stdin.write(data)
        self.process.stdin.flush()

    def receive(self, identifier):
        deadline = time.monotonic() + self.timeout
        for _ in range(1024):
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise TimeoutError('MCP reply deadline')
            value = self.messages.get(timeout=remaining)
            if isinstance(value, BaseException):
                self.errors.seek(0)
                raise RuntimeError(self.errors.read(8192).decode(errors='replace')) from value
            if value.get('id') == identifier:
                if 'error' in value:
                    raise ValueError(value['error'])
                return value['result']
        raise ValueError('MCP notification budget')

    def request(self, method, params=None):
        identifier = self.next_id
        self.next_id += 1
        value = {'jsonrpc': '2.0', 'id': identifier, 'method': method}
        if params is not None:
            value['params'] = params
        self.send(value)
        return self.receive(identifier)

    def call(self, name, arguments):
        return self.request('tools/call', {'name': name, 'arguments': arguments})

    def close(self):
        if self.process.returncode is None:
            try:
                self.process.stdin.close()
            except OSError:
                pass
            if hasattr(os, 'wait4'):
                deadline = time.monotonic() + self.timeout
                while True:
                    pid, status, usage = os.wait4(self.process.pid, os.WNOHANG)
                    if pid:
                        self.process.returncode = os.waitstatus_to_exitcode(status)
                        self.usage = {'peak_rss_bytes': int(usage.ru_maxrss * (1 if sys.platform == 'darwin' else 1024)), 'exit_code': self.process.returncode}
                        break
                    if time.monotonic() > deadline:
                        self.process.kill()
                    time.sleep(.01)
            else:
                try:
                    self.process.wait(timeout=self.timeout)
                except subprocess.TimeoutExpired:
                    self.process.kill()
                    self.process.wait()
        self.reader.join(timeout=1)
        self.process.stdout.close()
        self.errors.close()
        return self.usage

    def __enter__(self):
        return self

    def __exit__(self, *_):
        self.close()
