"""Separate allocation profiles; their duration is never native latency evidence."""
import hashlib,json,pathlib,subprocess
import benchmark as common
common.verify();versions=dict(zip(['reference','baseline','candidate'],common.read('plan.json')['large_primary_versions']))
reports=[];cases=[('tiny16',common.BASE/'tiny16-mcp.json',16),('canonical',common.ROOT/'local-lab/pdf-stability/imports/canonical-combined-0315.json',167942)]
for case,source,count in cases:
 with source.open('rb')as f:before=hashlib.file_digest(f,'sha256').hexdigest()
 for arm in versions:
  out=common.BASE/'startup-heap'/f'{case}-{arm}';out.mkdir(parents=True)
  result=subprocess.run([str(common.WORKERS/('load-'+arm)),str(source),str(out/'profile.json'),str(count)],capture_output=True,text=True,check=True,timeout=600)
  (out/'stdout.log').write_text(result.stdout);(out/'stderr.log').write_text(result.stderr)
  value=json.loads(result.stdout);assert value['version']==versions[arm]and value['entries']==count and value['source_bytes']==source.stat().st_size
  reports.append({'case':case,'arm':arm,'input_sha256':before,**value})
  print(json.dumps({'stage':'startup-heap','case':case,'arm':arm}),flush=True)
 with source.open('rb')as f:assert hashlib.file_digest(f,'sha256').hexdigest()==before
assert len(reports)==6
proof={'processes':6,'cases':2,'versions':versions,'input_hashes_unchanged':True,'entry_counts_checked':True,'scope':'Separate DHAT startup-load profiles, not process RSS or native latency. All source reading, owned parsing, index and schema construction included. Raw profiles retained locally; no query or source text exported.'}
common.save('startup-heap-reports.json',reports);common.save('startup-heap-compatibility.json',proof)
