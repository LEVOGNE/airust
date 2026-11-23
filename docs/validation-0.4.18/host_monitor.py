"""Coarse host-load observations; sampling cannot establish continuous idleness."""
import datetime
import json
import pathlib
import subprocess
import threading

class HostMonitor:
    def __init__(self, path, interval=2):
        self.path = pathlib.Path(path)
        self.interval = interval
        self.stop = threading.Event()
        self.rows = []
        self.thread = threading.Thread(target=self.run, daemon=True)

    def sample(self):
        row = {'at': datetime.datetime.now(datetime.timezone.utc).isoformat()}
        try:
            lines = subprocess.check_output(['ps', '-axo', 'pcpu=,comm='], text=True).splitlines()
            processes = [line.strip().split(maxsplit=1) for line in lines if line.strip()]
            compilers = [x for x in processes if len(x) > 1 and pathlib.Path(x[1]).name == 'rustc']
            row.update(rustc_processes=len(compilers), rustc_cpu_total=sum(float(x[0]) for x in compilers), processes_over_50_cpu=sum(float(x[0]) > 50 for x in processes))
        except Exception as error:
            row['sampling_error'] = type(error).__name__
        self.rows.append(row)
        self.path.write_text(json.dumps({'interval_seconds': self.interval, 'scope': 'Snapshots of all host processes; active benchmark workers are included in CPU counts. Other workloads are not stopped. Gaps between samples remain.', 'samples': self.rows}, indent=2) + '\n')

    def run(self):
        while not self.stop.wait(self.interval):
            self.sample()

    def __enter__(self):
        self.sample()
        self.thread.start()
        return self

    def __exit__(self, *args):
        self.stop.set()
        self.thread.join()
        self.sample()
