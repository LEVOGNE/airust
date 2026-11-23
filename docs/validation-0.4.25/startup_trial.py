"""One POSIX startup-only MCP child; per-child wait4 peak and post-ready ps RSS."""
import argparse,hashlib,json,os,pathlib,signal,subprocess,sys,time
p=argparse.ArgumentParser();p.add_argument('--method',type=pathlib.Path,required=True);p.add_argument('--binary',type=pathlib.Path,required=True);p.add_argument('--knowledge',type=pathlib.Path,required=True);p.add_argument('--version',required=True);p.add_argument('--entries',type=int,required=True);p.add_argument('--report',type=pathlib.Path,required=True);a=p.parse_args()
sys.path.insert(0,str(a.method));from mcp_probe import Connection
with a.knowledge.open('rb')as stream:before=hashlib.file_digest(stream,'sha256').hexdigest()
start=time.perf_counter();client=Connection([str(a.binary),'--knowledge',str(a.knowledge)]);child=client.process;usage=None
try:
 init,_=client.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'airust-startup-memory-probe','version':'1'}})
 assert init['result']['serverInfo']['name']=='airust' and init['result']['serverInfo']['version']==a.version
 client.send({'jsonrpc':'2.0','method':'notifications/initialized'})
 ready_ms=(time.perf_counter()-start)*1000
 status,events=client.request('tools/call',{'name':'airust_status','arguments':{}})
 assert not events and not status['result'].get('isError',False)
 data=status['result']['structuredContent']['data'];assert data['version']==a.version and data['stats']['entries']==a.entries and data['stats']['in_flight']==0 and data['read_only'] is True
 rss=int(subprocess.check_output(['ps','-o','rss=','-p',str(child.pid)],text=True).strip())*1024
 child.stdin.close();deadline=time.monotonic()+10
 while True:
  pid,wait_status,observed=os.wait4(child.pid,os.WNOHANG)
  if pid:
   child.returncode=os.waitstatus_to_exitcode(wait_status);usage=observed;break
  if time.monotonic()>deadline:raise TimeoutError('MCP EOF shutdown exceeded 10 seconds')
  time.sleep(.01)
 assert child.returncode==0,child.stderr.read().decode(errors='replace')
finally:
 client.selector.close()
 if child.returncode is None:
  try:os.kill(child.pid,signal.SIGKILL)
  except ProcessLookupError:pass
  pid,wait_status,observed=os.wait4(child.pid,0);child.returncode=os.waitstatus_to_exitcode(wait_status)
with a.knowledge.open('rb')as stream:assert hashlib.file_digest(stream,'sha256').hexdigest()==before
peak=int(usage.ru_maxrss if sys.platform=='darwin'else usage.ru_maxrss*1024)
report={'version':a.version,'entries':a.entries,'input_sha256':before,'startup_to_initialized_ms':ready_ms,'rss_after_initialization_bytes':rss,'peak_child_rss_bytes':peak,'user_cpu_seconds':usage.ru_utime,'system_cpu_seconds':usage.ru_stime,'exit_code':child.returncode,'read_only':True,'worker_in_flight':0,'input_unchanged':True,'scope':'Fresh child, warm filesystem/process caches; initialization and one status call only, no search/norm warming. Peak is this exact server child via wait4, not cumulative children; post-ready RSS is a separate ps sample. Native timing, no allocation profiler. No source text or paths in output.'}
with a.report.open('x')as f:json.dump(report,f,indent=2);f.write('\n')
print(json.dumps(report))
