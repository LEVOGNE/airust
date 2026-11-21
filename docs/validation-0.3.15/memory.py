import hashlib,json,os,pathlib,resource,shutil,statistics,subprocess,sys,time
here=pathlib.Path(__file__).resolve().parent

def sha(p):return hashlib.file_digest(p.open('rb'),'sha256').hexdigest()

if sys.argv[1]=='prepare':
 previous,current,work=map(pathlib.Path,sys.argv[2:5]);assert not work.exists();(work/'src').mkdir(parents=True)
 source_hashes={}
 for name,root in [('previous',previous),('current',current)]:
  p=root/'src/pdf_page_tree.rs';text=p.read_text().split('#[cfg(test)]',1)[0];source_hashes[name]={'complete_helper_sha256':sha(p),'runtime_helper_sha256':hashlib.sha256(text.encode()).hexdigest()};(work/'src'/f'{name}.rs').write_text(text+(here/'memory-probe.rs').read_text())
 shutil.copy2(here/'memory.toml',work/'Cargo.toml');shutil.copy2(here/'memory.rs',work/'src/main.rs');(work/'source-identity.json').write_text(json.dumps(source_hashes,indent=2)+'\n')
elif sys.argv[1]=='run':
 work,out=map(pathlib.Path,sys.argv[2:4]);assert not out.exists();out.mkdir();binary=pathlib.Path(sys.argv[4]);samples=[]
 def caps():
  resource.setrlimit(resource.RLIMIT_CPU,(10,10));resource.setrlimit(resource.RLIMIT_FSIZE,(1024*1024,1024*1024))
 for repeat in range(6):
  for mode,width in [('prefix',999999),('full',100000)]:
   for version in ['previous','current']if repeat%2==0 else['current','previous']:
    label=f'{mode}-{version}-{repeat}';start=time.monotonic()
    with(out/(label+'.stdout')).open('wb')as stdout,(out/(label+'.stderr')).open('wb')as stderr:
     p=subprocess.Popen([str(binary),version,str(width),mode],stdout=stdout,stderr=stderr,preexec_fn=caps);_,status,usage=os.wait4(p.pid,0);p.returncode=os.waitstatus_to_exitcode(status)
    wall=time.monotonic()-start;assert p.returncode==0,label;assert(out/(label+'.stdout')).read_text().strip()==f'validated:{version}:{width}:{mode}'
    samples.append({'mode':mode,'width':width,'version':version,'repeat':repeat,'wall_seconds':wall,'process_peak_rss_bytes':usage.ru_maxrss});(out/'samples.json').write_text(json.dumps(samples,indent=2)+'\n')
 groups=[]
 for mode,width in [('prefix',999999),('full',100000)]:
  medians={v:{'wall_seconds':statistics.median(s['wall_seconds']for s in samples if s['mode']==mode and s['version']==v),'process_peak_rss_bytes':statistics.median(s['process_peak_rss_bytes']for s in samples if s['mode']==mode and s['version']==v)}for v in ['previous','current']};groups.append({'mode':mode,'child_references':width,'medians':medians})
 summary={'processes':len(samples),'repeats_per_case_version':6,'previous':'0.3.14','current':'0.3.15','source_identity':json.loads((work/'source-identity.json').read_text()),'binary_sha256':sha(binary),'groups':groups,'conditions':'Fresh alternating release processes, same compiled comparison binary and backend0.3.5. Deliberate competing work finished before timing. Other desktop activity uncontrolled. Darwin wait4 RSS is bytes.','scope':'Isolated real metadata-validator helper with fake loader constructing Object arrays/dictionaries. Includes owned input construction, traversal and destruction; no native PDF file parsing/content/glyph/cleanup/indexing. prefix expects same page-limit error at second actual leaf; full requires100000actualleaves. Not a million-page import or whole-job RAM guarantee.'};(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary,indent=2))
else:raise SystemExit('Use prepare PREVIOUS_SOURCE CURRENT_SOURCE NEW_WORK_DIR or run WORK_DIR NEW_OUTPUT_DIR BINARY')
