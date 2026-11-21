import argparse,json,os,pathlib,resource,subprocess,sys,threading,time
p=argparse.ArgumentParser(description='Run one bounded document-validation process and record its complete scope.')
p.add_argument('binary',type=pathlib.Path);p.add_argument('mode',choices=['pdf','restore']);p.add_argument('input',type=pathlib.Path);p.add_argument('queries',type=pathlib.Path);p.add_argument('output',type=pathlib.Path);p.add_argument('document_id',nargs='?');a=p.parse_args()
if a.mode=='pdf'and a.document_id is None:p.error('pdf mode requires a stable document ID')
assert not a.output.exists();a.output.parent.mkdir(parents=True,exist_ok=True)
def caps():
 resource.setrlimit(resource.RLIMIT_CPU,(200,200));resource.setrlimit(resource.RLIMIT_FSIZE,(600*1024*1024,600*1024*1024))
args=[str(a.binary.resolve()),a.mode,str(a.input.resolve()),str(a.queries.resolve()),str(a.output.resolve())]+([a.document_id]if a.mode=='pdf'else[])
expired=threading.Event();start=time.monotonic()
with a.output.with_suffix('.stdout').open('xb')as stdout,a.output.with_suffix('.stderr').open('xb')as stderr:
 child=subprocess.Popen(args,stdout=stdout,stderr=stderr,preexec_fn=caps)
 def stop():
  expired.set()
  try:child.kill()
  except ProcessLookupError:pass
 timer=threading.Timer(240,stop);timer.daemon=True;timer.start()
 try:
  _,status,usage=os.wait4(child.pid,0);child.returncode=os.waitstatus_to_exitcode(status)
 finally:timer.cancel()
wall=time.monotonic()-start
assert child.returncode==0 and not expired.is_set(),(child.returncode,expired.is_set())
summary=json.loads((a.output/'summary.json').read_text());measurement={'wall_seconds':wall,'process_peak_rss_bytes':usage.ru_maxrss if sys.platform=='darwin'else usage.ru_maxrss*1024,'scope':'Complete verification process: document/JSON, all page positions/entries, both current and previous lexical engines, query comparisons, clones and result exports. restore mode does no native PDF extraction. Not one index or snapshot memory, an isolated parser time, cold-start or general capacity guarantee.','summary':summary}
(a.output/'measurement.json').write_text(json.dumps(measurement,indent=2)+'\n');print(json.dumps({'mode':a.mode,'wall_seconds':wall,'process_peak_rss_bytes':measurement['process_peak_rss_bytes']}))
