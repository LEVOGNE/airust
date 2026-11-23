"""Frozen local performance/compatibility comparison; source text stays private."""
import collections,gzip,hashlib,json,os,pathlib,random,resource,statistics,subprocess,sys,time
ROOT=pathlib.Path(__file__).resolve().parents[2]
BASE=pathlib.Path(os.environ.get('AIRUST_BENCH_DIR',ROOT/'local-lab/ranking-0.4.15')).resolve()
WORKERS=BASE/'measured-workers'
def read(n):return json.loads((BASE/n).read_text())
def save(n,x):
 p=BASE/n;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n')
def verify():
 for n,h in read('extended-inputs.json').items():assert hashlib.sha256((BASE/n).read_bytes()).hexdigest()==h,n
 for n,h in read('frozen-inputs.json').items():assert hashlib.sha256((BASE/n).read_bytes()).hexdigest()==h,n
 for n,h in read('method-sha256.json').items():assert hashlib.sha256((ROOT/n).read_bytes()).hexdigest()==h,n
 gate=json.loads(pathlib.Path('/private/tmp/airust-ranking-0415/VALIDATION-COMPLETE.json').read_text());assert gate['version']=='0.4.15'
def stats(values):
 a=sorted(values);assert a
 def p(f):
  at=(len(a)-1)*f;lo=int(at);hi=min(lo+1,len(a)-1);return a[lo]+(a[hi]-a[lo])*(at-lo)
 return {'n':len(a),'mean':statistics.mean(a),'median':statistics.median(a),'p95':p(.95),'min':a[0],'max':a[-1]}
class Worker:
 def __init__(self,label):
  self.p=subprocess.Popen([str(WORKERS/('retrieval-'+label)),str(BASE/'chunks.json')],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=open(BASE/f'retrieval-{label}.stderr','w'),text=True,bufsize=1);line=self.p.stdout.readline();assert line;self.ready=json.loads(line)
 def query(self,p):
  self.p.stdin.write(json.dumps(p,ensure_ascii=False)+'\n');self.p.stdin.flush();line=self.p.stdout.readline();assert line;return json.loads(line)
 def close(self):self.p.stdin.close();assert self.p.wait(timeout=30)==0

def retrieval():
 verify();qs=read('questions.json');ws={v:Worker(v)for v in ['baseline','candidate']};save('retrieval-builds.json',{v:w.ready for v,w in ws.items()});timings=[];outcomes=[];compared=0
 with gzip.open(BASE/'retrieval-responses-private.jsonl.gz','wt')as log:
  for i,q in enumerate(qs):
   for mode in ['baseline','prepared','evidence']:
    results={};order=['baseline','candidate'];random.Random(4800+i+sum(map(ord,mode))).shuffle(order)
    for version in order:
     def run():
      start=time.perf_counter();r=ws[version].query({'mode':mode,'project':q['project'],'query':q['question']});return r,(time.perf_counter()-start)*1000
     run();runs=[run()for _ in range(5)];assert all(r['response_json']==runs[0][0]['response_json']for r,_ in runs);results[version]=runs[0][0]['response_json']
     for j,(r,ms)in enumerate(runs):timings.append({'query_id':q['id'],'project':q['project'],'mode':mode,'version':version,'repeat':j,'roundtrip_ms':ms,'worker_ms':r['search_ms']})
     response=json.loads(results[version]);hits=response['hits'];gold=set(q['gold_ids']);rank=next((j+1 for j,h in enumerate(hits)if h['id']in gold and q.get('gold_quote','')in h['excerpt']),None)
     outcomes.append({'query_id':q['id'],'project':q['project'],'mode':mode,'version':version,'answerable':q['answerable'],'reference_quote_rank':rank,'hits':len(hits),'response_bytes':len(results[version].encode())})
     log.write(json.dumps({'query_id':q['id'],'mode':mode,'version':version,'response_json':results[version]},ensure_ascii=False)+'\n')
    assert results['baseline']==results['candidate'],('response drift',q['id'],mode);compared+=1
   if i%40==0:print(json.dumps({'stage':'retrieval','queries_completed':i+1,'total':len(qs)}),flush=True)
 save('retrieval-timings.json',timings);save('retrieval-outcomes.json',outcomes);save('retrieval-compatibility.json',{'complete_response_comparisons':compared,'byte_identical':True,'warm_samples':len(timings),'questions':len(qs),'configs':3,'no_fresh_holdout_or_semantic_quality_claim':True});save('retrieval-rss.json',{v:int(subprocess.check_output(['ps','-o','rss=','-p',str(w.p.pid)],text=True).strip())*1024 for v,w in ws.items()})
 for w in ws.values():w.close()
 summary={mode:{v:stats([r['roundtrip_ms']for r in timings if r['mode']==mode and r['version']==v])for v in ws}for mode in ['baseline','prepared','evidence']};save('retrieval-summary.json',summary);print(json.dumps({'stage':'retrieval','complete_comparisons':compared,'timing_samples':len(timings),'medians_ms':{m:{v:r['median']for v,r in vs.items()}for m,vs in summary.items()}}),flush=True)

def query_work():
 verify();samples=[];heap=[]
 for round in range(7):
  order=['baseline','candidate']if round%2==0 else['candidate','baseline']
  for version in order:
   r=subprocess.run([str(WORKERS/f'query-{version}-plain'),'time','1000'],capture_output=True,text=True,check=True,timeout=180);data=json.loads(r.stdout);save(f'query-raw/{round}-{version}.json',data)
   for row in data['rows']:samples.append({'version':version,'round':round,**row})
 for version in ['baseline','candidate']:
  r=subprocess.run([str(WORKERS/f'query-{version}-heap'),'heap','100',str(BASE/f'query-{version}.dhat.json')],capture_output=True,text=True,check=True,timeout=180);data=json.loads(r.stdout);save(f'query-{version}-heap.json',data);(BASE/f'query-{version}-heap.stderr').write_text(r.stderr)
  for row in data['rows']:heap.append({'version':version,**row})
 cold=[]
 for version in ['baseline','candidate']:
  result=subprocess.run([str(WORKERS/f'query-{version}-heap'),'norm_heap','1',str(BASE/f'norm-{version}.dhat.json')],capture_output=True,text=True,check=True,timeout=180)
  data=json.loads(result.stdout);save(f'norm-{version}.json',data);(BASE/f'norm-{version}.stderr').write_text(result.stderr)
  cold.append({'arm':version,**data})
 assert [r['signature']for r in cold[0]['rows']]==[r['signature']for r in cold[1]['rows']]
 save('norm-memory.json',cold)
 cold_times=[];canonical=[]
 for round in range(7):
  for arm in (['baseline','candidate']if round%2==0 else['candidate','baseline']):
   result=subprocess.run([str(WORKERS/f'query-{arm}-plain'),'cold_time','1'],capture_output=True,text=True,check=True,timeout=180)
   data=json.loads(result.stdout);save(f'cold-raw/{round}-{arm}.json',data)
   for row in data['rows']:cold_times.append({'arm':arm,'round':round,**row})
 cold_summary={}
 for fixture in ['index','wide','sparse','tiny','tiny4','tiny16']:
  rs=[r for r in cold_times if r['fixture']==fixture];assert len(rs)==280 and len({r['signature']for r in rs})==1
  cold_summary[fixture]={arm:stats([r['elapsed_ns']for r in rs if r['arm']==arm])for arm in ['baseline','candidate']}
 save('cold-timings.json',cold_times);save('cold-summary.json',cold_summary)
 for arm in ['baseline','candidate']:
  result=subprocess.run([str(WORKERS/f'query-{arm}-heap'),'corpus_norm_heap','1',str(BASE/f'canonical-norm-{arm}.dhat.json'),str(ROOT/'local-lab/pdf-stability/imports/canonical-combined-0315.json')],capture_output=True,text=True,check=True,timeout=600)
  data=json.loads(result.stdout);save(f'canonical-norm-{arm}.json',data);(BASE/f'canonical-norm-{arm}.stderr').write_text(result.stderr);assert data['entries']==167942
  canonical.append({'arm':arm,**data})
 assert [r['signature']for r in canonical[0]['rows']]==[r['signature']for r in canonical[1]['rows']]
 save('canonical-norm-memory.json',canonical)
 labels=sorted({r['fixture']for r in samples});summary=[]
 for label in labels:
  by={v:[r for r in samples if r['fixture']==label and r['version']==v]for v in ['baseline','candidate']};assert len({r['signature']for rs in by.values()for r in rs})==1
  hs={v:next(r for r in heap if r['fixture']==label and r['version']==v)for v in by};assert hs['baseline']['signature']==hs['candidate']['signature'];assert all(r['live_bytes_delta']==0 and r['live_blocks_delta']==0 for r in hs.values())
  med={v:stats([r['ns_per_call']for r in rs])for v,rs in by.items()};summary.append({'fixture':label,'kind':by['baseline'][0]['kind'],'time_ns_per_call':med,'candidate_time_change_percent':100*(med['candidate']['median']/med['baseline']['median']-1),'allocation_blocks_per_call':{v:r['allocation_blocks']/r['iterations']for v,r in hs.items()},'allocated_bytes_per_call':{v:r['allocated_bytes']/r['iterations']for v,r in hs.items()},'retained_bytes_delta':{v:r['live_bytes_delta']for v,r in hs.items()}})
 save('query-timings.json',samples);save('query-heap-deltas.json',heap);save('query-summary.json',summary);small_queries();mixed_queries();print(json.dumps({'stage':'query','fixtures':len(labels),'plain_processes':14,'instrumented_processes':2,'summary':summary}),flush=True)

def small_queries():
 verify();samples=[];heaps=[]
 for round in range(7):
  for arm in (['baseline','candidate']if round%2==0 else['candidate','baseline']):
   result=subprocess.run([str(WORKERS/f'query-{arm}-plain'),'small_time','1000'],capture_output=True,text=True,check=True,timeout=180);data=json.loads(result.stdout);save(f'small-raw/{round}-{arm}.json',data)
   for row in data['rows']:samples.append({'arm':arm,'round':round,**row})
 for arm in ['baseline','candidate']:
  result=subprocess.run([str(WORKERS/f'query-{arm}-heap'),'small_heap','100',str(BASE/f'small-{arm}.dhat.json')],capture_output=True,text=True,check=True,timeout=180);data=json.loads(result.stdout);save(f'small-{arm}-heap.json',data);(BASE/f'small-{arm}.stderr').write_text(result.stderr)
  for row in data['rows']:heaps.append({'arm':arm,**row})
 summary=[]
 for fixture in sorted({r['fixture']for r in samples}):
  rows=[r for r in samples if r['fixture']==fixture];assert len(rows)==14 and len({r['signature']for r in rows})==1
  hs={arm:next(r for r in heaps if r['fixture']==fixture and r['arm']==arm)for arm in ['baseline','candidate']};assert hs['baseline']['signature']==hs['candidate']['signature'];assert all(r['live_bytes_delta']==0 and r['live_blocks_delta']==0 for r in hs.values())
  times={arm:stats([r['ns_per_call']for r in rows if r['arm']==arm])for arm in ['baseline','candidate']}
  summary.append({'fixture':fixture,'entries':rows[0]['entries'],'algorithm':rows[0]['algorithm'],'time_ns_per_call':times,'candidate_time_change_percent':100*(times['candidate']['median']/times['baseline']['median']-1),'allocation_blocks_per_call':{arm:r['allocation_blocks']/r['iterations']for arm,r in hs.items()},'allocated_bytes_per_call':{arm:r['allocated_bytes']/r['iterations']for arm,r in hs.items()},'retained_bytes_delta':{arm:r['live_bytes_delta']for arm,r in hs.items()}})
 save('small-query-timings.json',samples);save('small-query-heap-deltas.json',heaps);save('small-query-summary.json',summary)

def mixed_queries():
 verify();rows=[]
 for round in range(5):
  for arm in (['baseline','candidate']if round%2==0 else['candidate','baseline']):
   result=subprocess.run([str(WORKERS/f'query-{arm}-plain'),'mixed','1'],capture_output=True,text=True,check=True,timeout=180);data=json.loads(result.stdout);save(f'mixed-raw/{round}-{arm}.json',data);rows.append({'arm':arm,'round':round,**data})
 assert len({r['signature']for r in rows})==1
 summary={}
 for arm in ['baseline','candidate']:
  rs=[r for r in rows if r['arm']==arm];assert len(rs)==5
  summary[arm]={'processes':5,'new_group_query_ns':stats([x['elapsed_ns']for r in rs for x in r['samples']]),'post_readers_first_ns':stats([r['quiescent_ns'][0]for r in rs]),'post_readers_second_ns':stats([r['quiescent_ns'][1]for r in rs]),'warm_reader_calls':sum(sum(r['warm_reader_calls'])for r in rs)}
 save('mixed-query-reports.json',rows);save('mixed-query-summary.json',summary)

def mcp():
 verify();cases=[('canonical','--knowledge',ROOT/'local-lab/pdf-stability/imports/canonical-combined-0315.json',True),('numpy','--document',ROOT/'local-lab/pdf-stability/document-snapshots/040/numpy-ref/snapshot.json',True),('wide','--knowledge',BASE/'wide-mcp.json',True)]+[(label+(''if progress else'-no-progress'),'--knowledge',BASE/(label+'-mcp.json'),progress)for progress in [True,False]for label in ['tiny','tiny4','tiny16']];rows=[];comparison_count=0
 for case,flag,source,progress_requested in cases:
  original=hashlib.file_digest(source.open('rb'),'sha256').hexdigest()
  for round in range(5):
   versions=['baseline','candidate']if round%2==0 else['candidate','baseline'];transcripts={}
   for version in versions:
    folder=BASE/f'mcp/{case}-{round}-{version}';folder.mkdir(parents=True,exist_ok=False);report=folder/'report.json';transcript=folder/'transcript-private.json'
    cmd=[sys.executable,'-B',str(ROOT/'docs/validation-0.4.15/mcp_probe.py'),'--binary',str(WORKERS/('mcp-'+version)),flag,str(source),'--queries',str(BASE/('wide-mcp-queries.json'if case=='wide'else'mcp-queries.json')),'--report',str(report),'--transcript',str(transcript)]
    if not progress_requested:cmd.append('--no-progress')
    with(folder/'probe.log').open('w')as log:r=subprocess.run(cmd,stdout=log,stderr=subprocess.STDOUT,timeout=240)
    assert r.returncode==0,('MCP probe failed; retained log',str(folder));schema=folder/'schema.json';r=subprocess.run([str(WORKERS/'schema'),str(transcript),str(schema)],capture_output=True,text=True,timeout=120);(folder/'schema.log').write_text(r.stdout+r.stderr);assert r.returncode==0
    data=json.loads(report.read_text());rows.append({'case':case,'round':round,'arm':version,**data,'schema_validation':json.loads(schema.read_text())});transcripts[version]=json.loads(transcript.read_text())
   a=transcripts['baseline'];c=transcripts['candidate']
   for data,version in [(a,'0.4.14'),(c,'0.4.15')]:
    for tool in data['tools']:
     assert tool['outputSchema']['$id']==f"urn:airust:mcp:{version}:{tool['name']}"
     tool['outputSchema']['$id']=f"urn:airust:mcp:VERSION:{tool['name']}"
   assert a['tools']==c['tools'];assert len(a['results'])==len(c['results'])
   old_results={row['request_id']:row for row in a['results']};new_results={row['request_id']:row for row in c['results']};assert old_results.keys()==new_results.keys()
   for request_id in sorted(old_results):
    old=old_results[request_id];new=new_results[request_id]
    assert old['tool']==new['tool'];x=old['result']['structuredContent'];y=new['result']['structuredContent']
    if old['tool']=='airust_status':x['data'].pop('version');y['data'].pop('version')
    assert x==y,('MCP structured result drift',case,round,old['tool']);comparison_count+=1
   print(json.dumps({'stage':'mcp','case':case,'round':round+1,'complete':True}),flush=True)
  assert hashlib.file_digest(source.open('rb'),'sha256').hexdigest()==original
 save('mcp-reports.json',rows);mcp_summary()

def mcp_summary():
 verify();rows=read('mcp-reports.json');assert len(rows)==90;summary={};comparison_count=0
 for row in rows:
  arm={'0.4.14':'baseline','0.4.15':'candidate'}[row['version']]
  row['arm']=arm
  folder=BASE/f"mcp/{row['case']}-{row['round']}-{arm}"
  report=json.loads((folder/'report.json').read_text())
  assert all(row[k]==v for k,v in report.items())
 for case in ['canonical', 'numpy', 'wide', 'tiny', 'tiny4', 'tiny16', 'tiny-no-progress', 'tiny4-no-progress', 'tiny16-no-progress']:
  for round in range(5):
   transcripts={v:json.loads((BASE/f'mcp/{case}-{round}-{v}/transcript-private.json').read_text())for v in ['baseline','candidate']}
   for arm,version in [('baseline','0.4.14'),('candidate','0.4.15')]:
    for tool in transcripts[arm]['tools']:
     assert tool['outputSchema']['$id']==f"urn:airust:mcp:{version}:{tool['name']}"
     tool['outputSchema']['$id']=f"urn:airust:mcp:VERSION:{tool['name']}"
   assert transcripts['baseline']['tools']==transcripts['candidate']['tools']
   old={r['request_id']:r for r in transcripts['baseline']['results']};new={r['request_id']:r for r in transcripts['candidate']['results']};assert old.keys()==new.keys()
   for key in old:
    assert old[key]['tool']==new[key]['tool'];x=old[key]['result']['structuredContent'];y=new[key]['result']['structuredContent']
    if old[key]['tool']=='airust_status':x['data'].pop('version');y['data'].pop('version')
    assert x==y;comparison_count+=1
  summary[case]={}
  for v in ['baseline','candidate']:
   rs=[r for r in rows if r['case']==case and r['arm']==v];assert len(rs)==5
   summary[case][v]={'processes':len(rs),'startup_ms':stats([r['startup_to_initialized_ms']for r in rs]),'warm_search_ms':{alg:stats([s['milliseconds']for r in rs for s in r['search_samples']if s['algorithm']==alg])for alg in ['bm25','tfidf']},'initial_search_ms':{alg:stats([s['milliseconds']for r in rs for s in r['initial_search_samples']if s['algorithm']==alg])for alg in ['bm25','tfidf']},'peak_child_rss_bytes':stats([r['server_peak_rss_bytes']for r in rs]),'search_calls':sum(r['search_calls']for r in rs),'progress_events':sum(r['progress_events']for r in rs),'concurrent_calls':sum(r['concurrent_search_calls']for r in rs),'snapshot_readbacks':sum(r['snapshot_spans_and_readbacks']for r in rs),'validated_results':sum(r['schema_validation']['actual_results_validated']for r in rs)}
 save('mcp-reports.json',rows);save('mcp-summary.json',summary);save('mcp-compatibility.json',{'structured_results_compared':comparison_count,'responses_paired_by_request_id':True,'all_equal_except_explicit_status_version':True,'all_advertised_output_schemas_equal_except_expected_version_urn':True,'processes':len(rows),'input_files_unchanged':True});print(json.dumps({'stage':'mcp','structured_results_compared':comparison_count,'processes':len(rows)}),flush=True)

def corpus():
 verify();source=ROOT/'local-lab/pdf-stability/imports/canonical-combined-0315.json';rows={}
 for v in ['baseline','candidate']:
  start=time.perf_counter();r=subprocess.run([str(WORKERS/('corpus-'+v)),str(source),'167942','1'],capture_output=True,text=True,timeout=1200);(BASE/f'corpus-{v}.stderr').write_text(r.stderr);assert r.returncode==0;data=json.loads(r.stdout);data['child_wall_s']=time.perf_counter()-start;save('corpus-'+v+'.json',data);rows[v]=data
 assert rows['baseline']['source_export_sha256']==rows['candidate']['source_export_sha256']=='13168d87872a137355fd8499c90a6084f52243a2ec7456f29b5fcb2a1c7423c3';assert rows['baseline']['baseline_export_sha256']==rows['candidate']['baseline_export_sha256'];save('corpus-compatibility.json',{'entries':167942,'versions':2,'canonical_input_identical':True,'portable_export_identical':True,'in_memory_updates_rebuilds_roundtrips_concurrent_readers_and_cancellation_checked':True,'fresh_pdf_parse_or_semantic_quality_claim':False});print(json.dumps({'stage':'corpus','entries':167942,'both_versions_passed':True}),flush=True)

def index_build():
 verify(); samples=[]; heaps=[]; source=ROOT/'local-lab/pdf-stability/imports/canonical-combined-0315.json'
 original=hashlib.file_digest(source.open('rb'),'sha256').hexdigest()
 for round in range(5):
  order=['baseline','candidate']if round%2==0 else['candidate','baseline']
  for arm in order:
   for fixture in ['canonical','ascii','mixed','long_tokens']:
    cmd=[str(WORKERS/('build-'+arm+'-plain')),'time',fixture,str(source)]
    result=subprocess.run(cmd,capture_output=True,text=True,check=True,timeout=180)
    data=json.loads(result.stdout);save(f'build-raw/{round}-{arm}-{fixture}.json',data)
    samples.append({'arm':arm,'round':round,**data})
 for arm in ['baseline','candidate']:
  for fixture in ['ascii','mixed','long_tokens']:
   result=subprocess.run([str(WORKERS/('build-'+arm+'-heap')),'heap',fixture,'unused',str(BASE/f'build-{arm}-{fixture}.dhat.json')],capture_output=True,text=True,check=True,timeout=180)
   data=json.loads(result.stdout);save(f'build-{arm}-{fixture}-heap.json',data);(BASE/f'build-{arm}-{fixture}-heap.stderr').write_text(result.stderr)
   heaps.append({'arm':arm,**data})
 summary={}
 for fixture in ['canonical','ascii','mixed','long_tokens']:
  rows=[s for s in samples if s['fixture']==fixture];assert len(rows)==10
  assert len({s['query_signature']for s in rows})==1
  assert len({json.dumps(s['stats'],sort_keys=True)for s in rows})==1
  summary[fixture]={arm:stats([s['build_ms']for s in rows if s['arm']==arm])for arm in ['baseline','candidate']}
  if fixture!='canonical':
   hs=[s for s in heaps if s['fixture']==fixture];assert len(hs)==2
   assert len({s['query_signature']for s in hs}|{rows[0]['query_signature']})==1
   assert hs[0]['live_bytes_after_build']==hs[1]['live_bytes_after_build']
 assert hashlib.file_digest(source.open('rb'),'sha256').hexdigest()==original
 save('build-samples.json',samples);save('build-heap.json',heaps);save('build-summary.json',summary)
 save('build-compatibility.json',{'plain_processes':40,'instrumented_processes':6,'fixtures':4,'canonical_entries':167942,'all_signatures_and_stats_equal':True,'tracked_live_derived_heap_equal':True,'source_sha256':original,'parser_and_input_allocation_excluded_from_build_timer':True})
 print(json.dumps({'stage':'build','summary':summary}),flush=True)

if __name__=='__main__':{'build':index_build,'retrieval':retrieval,'query':query_work,'mcp':mcp,'mcp-summary':mcp_summary,'corpus':corpus}[sys.argv[1]]()
