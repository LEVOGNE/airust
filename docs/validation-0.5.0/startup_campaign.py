"""Separate preplanned startup-only control; exact per-child peaks, all samples retained."""
import hashlib,json,os,pathlib,subprocess,sys
import benchmark as common
from host_monitor import HostMonitor
ROOT,BASE,WORKERS=common.ROOT,common.BASE,common.WORKERS
campaign=sys.argv[1];assert campaign in ['standard','confirmation'];common.verify()
plan=common.read('plan.json');reference,baseline,candidate=plan['large_primary_versions'];versions={'reference':reference,'baseline':baseline,'candidate':candidate};arms=list(versions)
binaries={a:WORKERS/('mcp-pre-regression-'+reference.replace('.','')if a=='reference'else'mcp-'+a)for a in arms}
method=pathlib.Path(__file__).resolve().parent
cases=[('canonical',ROOT/'local-lab/pdf-stability/imports/canonical-combined-0315.json',167942),('tiny16',BASE/'tiny16-mcp.json',16)]
prefix='startup-'+campaign;reports=[];before={str(p):hashlib.file_digest(p.open('rb'),'sha256').hexdigest()for _,p,_ in cases}
with HostMonitor(BASE/(prefix+'-host-samples.json')):
 for case,source,entries in cases:
  for n in range(7):
   order=arms[n%3:]+arms[:n%3]
   if n%2:order.reverse()
   for arm in order:
    folder=BASE/prefix/f'{case}-{n}-{arm}';folder.mkdir(parents=True)
    command=[sys.executable,'-B',str(method/'startup_trial.py'),'--method',str(method),'--binary',str(binaries[arm]),'--knowledge',str(source),'--version',versions[arm],'--entries',str(entries),'--report',str(folder/'report.json')]
    result=subprocess.run(command,capture_output=True,text=True,check=True,timeout=120);(folder/'stdout.log').write_text(result.stdout);(folder/'stderr.log').write_text(result.stderr)
    report=json.loads((folder/'report.json').read_text());assert report['version']==versions[arm]and report['entries']==entries and report['exit_code']==0 and report['input_sha256']==before[str(source)]
    reports.append({'case':case,'arm':arm,'triple':n,**report})
   print(json.dumps({'stage':prefix,'case':case,'triple':n+1}),flush=True)
assert len(reports)==42
for source,sha in before.items():assert hashlib.file_digest(pathlib.Path(source).open('rb'),'sha256').hexdigest()==sha
metrics=['startup_to_initialized_ms','rss_after_initialization_bytes','peak_child_rss_bytes','user_cpu_seconds','system_cpu_seconds']
summary={case:{arm:{'version':versions[arm],**{metric:common.stats([r[metric]for r in reports if r['case']==case and r['arm']==arm])for metric in metrics}}for arm in arms}for case,_,_ in cases}
proof={'campaign':campaign,'versions':versions,'cases':2,'triples_per_case':7,'processes':42,'input_hashes_unchanged':True,'entry_counts_and_read_only_status_checked':True,'all_children_exited_cleanly':True,'peak_source':'Exact server child wait4, not cumulative children; post-ready RSS separate ps sample','scope':'Startup plus one status request, no searches or norm warming; warm filesystem/process caches. Native timing, no allocation profiler. All samples separate from ordinary/other startup campaigns.'}
for suffix,value in [('reports',reports),('summary',summary),('compatibility',proof)]:common.save(prefix+'-'+suffix+'.json',value)
print(json.dumps(proof),flush=True)
