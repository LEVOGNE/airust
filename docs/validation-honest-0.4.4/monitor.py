import json, os, pathlib, subprocess, time
base=pathlib.Path(os.environ.get('AIRUST_BENCH_DIR','local-lab/honest-comparison-044')).resolve()
daemon=int(subprocess.check_output(['lsof','-nP','-iTCP:11464','-sTCP:LISTEN','-t'],text=True).strip())
with (base/'model-rss-samples.jsonl').open('a') as f:
    while not (base/'monitor.stop').exists():
        raw=subprocess.check_output(['ps','-axo','pid=,ppid=,rss=,comm='],text=True)
        rows=[]
        for line in raw.splitlines():
            fields=line.strip().split(None,3)
            if len(fields)==4:
                pid,ppid,rss,command=fields
                rows.append({'pid':int(pid),'ppid':int(ppid),'rss_bytes':int(rss)*1024,'command':pathlib.Path(command).name})
        family={daemon};changed=True
        while changed:
            children={r['pid'] for r in rows if r['ppid'] in family};changed=not children.issubset(family);family|=children
        f.write(json.dumps({'unix_time':time.time(),'processes':[r for r in rows if r['pid'] in family]})+'\n');f.flush()
        time.sleep(1)
