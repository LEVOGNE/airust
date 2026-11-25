"""Finite same-host native lifecycle measurement; keeps every failure and sample."""
from pathlib import Path
from datetime import datetime, timezone
import argparse
import hashlib
import json
import os
import selectors
import signal
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]



def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def put(path, value):
    path.write_text(json.dumps(value, indent=2, allow_nan=False) + '\n')


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('binary', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--model-directory', required=True, type=Path)
    parser.add_argument('--runtime', required=True, type=Path)
    parser.add_argument('--cycles', type=int, default=30)
    parser.add_argument('--diagnostics', action='store_true')
    parser.add_argument('--backend', choices=['in_process','process'], default='in_process')
    parser.add_argument('--mode', choices=['mixed','memory','snapshot'], default='mixed')
    parser.add_argument('--prefixes', type=Path, default=ROOT/'tests/fixtures/native-lifecycle-v1.json')
    args = parser.parse_args()
    if not 8 <= args.cycles <= 100 or sys.platform not in {'darwin','linux'}:
        parser.error('cycles must be 8..100; RSS controller supports macOS/Linux')
    if args.backend == 'process' and args.mode != 'snapshot':
        parser.error('process workers require --mode snapshot')
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    binary = args.binary.resolve(strict=True)
    private_tmp = out/'tmp'
    private_tmp.mkdir(mode=0o700)
    command = [str(binary), str(args.prefixes.resolve(strict=True)),
               str(args.model_directory.resolve(strict=True)),
               str(args.runtime.resolve(strict=True)),
               str(args.cycles), args.backend, args.mode, str(private_tmp)]
    policy = {'warmup_cycles':2,'post_warm_postdrop_rss_range_budget_bytes':64*1024*1024,'peak_child_rss_safety_cap_bytes':4*1024*1024*1024,'threads':2,'candidates':20,'outer_batch':1,'resource_origin':'Synthetic resource fixture by default; caller-supplied text may be private. No independent quality or actual 1 GiB-device claim.','rss_scope':'Sampled parent plus direct-child RSS sums can double-count shared pages; not unique physical/system/GPU accounting.'}
    env = dict(os.environ)
    for key in ['MallocSpaceEfficient', 'MallocNanoZone']:
        env.pop(key, None)
    record = {'started_at': datetime.now(timezone.utc).isoformat(), 'command': command,
              'binary_sha256': sha(binary), 'probe_sha256': sha(ROOT / 'examples/native_model_lifecycle.rs'),
              'policy': policy, 'cycles': args.cycles, 'malloc_space_efficient_override': False,
              'source_hashes': {str(p.relative_to(ROOT)): sha(p) for p in [*sorted((ROOT/'src').rglob('*.rs')),ROOT/'Cargo.toml',ROOT/'Cargo.lock',ROOT/'examples/native_model_lifecycle.rs',Path(__file__)]},
              'native_outside_miri': True, 'diagnostics': args.diagnostics, 'backend':args.backend, 'mode':args.mode,
              'prefixes_sha256':sha(args.prefixes), 'raw_queries_published':False}
    put(out / 'policy.json', record)
    rows = []
    sampled_peak = 0
    status = None
    start = time.monotonic()
    with (out / 'stderr.log').open('xb') as errors:
        process = subprocess.Popen(command, cwd=ROOT, env=env, stdin=subprocess.PIPE,
                                   stdout=subprocess.PIPE, stderr=errors, start_new_session=True)
        ready = selectors.DefaultSelector()
        ready.register(process.stdout, selectors.EVENT_READ)
        last_event = time.monotonic()
        try:
            while True:
                if time.monotonic() - last_event > 60 or time.monotonic() - start > 1800:
                    raise TimeoutError('Native lifecycle event/total time bound exceeded')
                rss = subprocess.run(['ps', '-o', 'rss=', '-p', str(process.pid)], capture_output=True, timeout=5)
                if rss.returncode == 0 and rss.stdout.strip():
                    resident = int(rss.stdout.strip()) * 1024
                    children = subprocess.run(['pgrep','-P',str(process.pid)],capture_output=True,text=True,timeout=5)
                    assert children.returncode in [0,1]
                    child_ids=[int(x) for x in children.stdout.splitlines()]
                    assert len(child_ids)<=1, 'Unexpected model-process multiplicity'
                    child_rss=0
                    for child_pid in child_ids:
                        sample=subprocess.run(['ps','-o','rss=','-p',str(child_pid)],capture_output=True,text=True,timeout=5)
                        if sample.returncode==0 and sample.stdout.strip(): child_rss+=int(sample.stdout.strip())*1024
                    sampled_peak = max(sampled_peak, resident+child_rss)
                    if resident+child_rss > policy['peak_child_rss_safety_cap_bytes']:
                        raise RuntimeError('Native child exceeded frozen 4 GiB RSS safety cap')
                if not ready.select(timeout=0.15):
                    continue
                line = process.stdout.readline(65537)
                if not line:
                    break
                assert len(line) <= 65536
                row = json.loads(line)
                assert row['pid'] == process.pid
                row['rss_bytes'] = int(subprocess.check_output(['ps', '-o', 'rss=', '-p', str(process.pid)], text=True, timeout=5).strip()) * 1024
                row['last_sampled_child_rss_bytes']=child_rss
                if row['stage']=='dropped':
                    child_state=subprocess.run(['pgrep','-P',str(process.pid)],capture_output=True,text=True,timeout=5)
                    assert child_state.returncode==1, 'Model worker was not reaped before drop completed'
                row['controller_elapsed_seconds'] = time.monotonic() - start
                if args.diagnostics and row['stage'] in ['initial', 'complete']:
                    for tool in ['/usr/bin/vmmap', '/usr/bin/leaks']:
                        command = [tool, '-summary' if tool.endswith('vmmap') else '-q', str(process.pid)]
                        run = subprocess.run(command, capture_output=True, timeout=45)
                        prefix = out / (Path(tool).name + '-' + row['stage'])
                        prefix.with_suffix('.stdout').write_bytes(run.stdout)
                        prefix.with_suffix('.stderr').write_bytes(run.stderr)
                        row[Path(tool).name + '_exit_code'] = run.returncode
                rows.append(row)
                put(out / 'events.json', rows)
                print(json.dumps({'stage': row['stage'], 'cycle': row['cycle'], 'rss_bytes': row['rss_bytes']}), flush=True)
                process.stdin.write(b'continue\n')
                process.stdin.flush()
                last_event = time.monotonic()
            _, raw_status, usage = os.wait4(process.pid, 0)
            process.returncode = os.waitstatus_to_exitcode(raw_status)
            status = process.returncode
            record['wait4_reported_maxrss_bytes'] = usage.ru_maxrss if sys.platform == 'darwin' else usage.ru_maxrss * 1024
        except Exception as error:
            record['harness_failure'] = repr(error)
            raise
        finally:
            ready.close()
            if process.returncode is None:
                os.killpg(process.pid, signal.SIGKILL)
                process.wait()
            process.stdin.close()
            process.stdout.close()
            record.update(exit_code=status, sampled_sum_peak_rss_bytes=sampled_peak,
                          completed_at=datetime.now(timezone.utc).isoformat(), seconds=time.monotonic() - start)
            put(out / 'partial-result.json', record)
    dropped = [row for row in rows if row['stage'] == 'dropped']
    assert status == 0 and rows[-1]['stage'] == 'complete' and len(dropped) == args.cycles
    steady = [row['rss_bytes'] for row in dropped[policy['warmup_cycles']:]]
    growth = max(steady) - min(steady)
    record.update(functional_controls_passed=True, no_model_workers_after_drop=True, after_drop_rss_bytes=[r['rss_bytes'] for r in dropped],
                  post_warm_range_bytes=growth, rss_growth_budget_passed=growth <= policy['post_warm_postdrop_rss_range_budget_bytes'])
    put(out / 'report.json', record)
    print(json.dumps({'name': args.output.name, 'functional_passed': True, 'growth_budget_passed': record['rss_growth_budget_passed'],
                      'growth_bytes': growth, 'peak_rss_bytes': record['sampled_sum_peak_rss_bytes']}), flush=True)
    if not record['rss_growth_budget_passed']:
        raise SystemExit('Observed memory-growth gate failed; preserve actual results')


if __name__ == '__main__':
    main()
