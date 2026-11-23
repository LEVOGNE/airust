"""Finite synthetic scale/clone controls; exact per-child RSS, never cumulative children."""
import json,os,pathlib,signal,subprocess,sys,time
import benchmark as common
from host_monitor import HostMonitor
common.verify()
versions=dict(zip(['reference','baseline','candidate'],common.read('plan.json')['large_primary_versions']))
arms=list(versions);reports=[]
def run(case,mode,count,n,arm):
 folder=common.BASE/'stability-stress'/f'{case}-{n}-{arm}';folder.mkdir(parents=True)
 with (folder/'stdout.log').open('wb')as stdout,(folder/'stderr.log').open('wb')as stderr:
  child=subprocess.Popen([str(common.WORKERS/('stress-'+arm)),mode,str(count)],stdout=stdout,stderr=stderr)
  deadline=time.monotonic()+600;usage=None
  try:
   while True:
    pid,status,observed=os.wait4(child.pid,os.WNOHANG)
    if pid:
     child.returncode=os.waitstatus_to_exitcode(status);usage=observed;break
    if time.monotonic()>deadline:raise TimeoutError('Finite scale control exceeded 600s; not a pass')
    time.sleep(.05)
   assert child.returncode==0,(folder/'stderr.log').read_text()[-2000:]
  finally:
   if child.returncode is None:
    try:os.kill(child.pid,signal.SIGKILL)
    except ProcessLookupError:pass
    pid,status,observed=os.wait4(child.pid,0);child.returncode=os.waitstatus_to_exitcode(status)
 value=json.loads((folder/'stdout.log').read_text());assert value['version']==versions[arm]and value['entries']==count and value['mode']==mode and value['fixture']=='stress-index-v1'
 if mode=='scale':assert value['final_entries']==count-20 and value['concurrent_readers']==4
 else:assert value['old_views_unchanged']and value['retained_clones']==3
 peak=int(usage.ru_maxrss if sys.platform=='darwin'else usage.ru_maxrss*1024)
 return {'case':case,'arm':arm,'triple':n,'exact_child_peak_rss_bytes':peak,'user_cpu_seconds':usage.ru_utime,'system_cpu_seconds':usage.ru_stime,'exit_code':child.returncode,**value}
with HostMonitor(common.BASE/'stability-stress-host-samples.json'):
 for case,mode,count,triples in [('million','scale',1_000_000,7),('retained-clones','clone',500_000,3)]:
  for n in range(triples):
   order=arms[n%3:]+arms[:n%3]
   if n%2:order.reverse()
   group=[run(case,mode,count,n,arm)for arm in order];reports.extend(group)
   if mode=='scale':assert len({x['score_signature']for x in group})==1
   print(json.dumps({'stage':'stability-stress','case':case,'triple':n+1}),flush=True)
assert len(reports)==30
summary={}
for case in ['million','retained-clones']:
 summary[case]={}
 for arm in arms:
  rows=[x for x in reports if x['case']==case and x['arm']==arm]
  metrics={'build_including_entry_generation_ms':[x['build_ms']for x in rows],'exact_child_peak_rss_bytes':[x['exact_child_peak_rss_bytes']for x in rows]}
  if case=='million':
   for name in ['selective','common','absent','concurrent_selective']:metrics[name+'_per_process_p50_ms']=[x[name]['p50_ms']for x in rows]
   for name in ['update_100_ms','delete_20_ms']:metrics[name]=[x[name]for x in rows]
  else:
   metrics['per_process_clone_median_ms']=[sorted(x['clone_ms'])[1]for x in rows];metrics['one_clone_mutation_ms']=[x['one_clone_mutation_ms']for x in rows]
  summary[case][arm]={'version':versions[arm],**{k:common.stats(values)for k,values in metrics.items()}}
proof={'processes':30,'scale_processes':21,'scale_entries_each':1_000_000,'clone_processes':9,'clone_entries_each':500_000,'versions':versions,'rotating_triples':{'million':7,'retained-clones':3},'all_score_signatures_equal_per_triple':True,'all_updates_deletes_reader_and_retained_clone_assertions_passed':True,'all_children_exited_cleanly':True,'source':'Unchanged examples/stress_index.rs across all three versions','scope':'Synthetic short lexical records. Build includes lazy entry generation. Query-category medians are per-process summaries, not pooled independent calls. Child peak via exact wait4, no allocation profiler. Warm OS/process caches. Finite trials, not an uptime/capacity/security guarantee or fresh PDF parsing.'}
for suffix,value in [('reports',reports),('summary',summary),('compatibility',proof)]:common.save('stability-stress-'+suffix+'.json',value)
print(json.dumps(proof),flush=True)
