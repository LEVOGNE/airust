"""Create reproducible 1/4/16-entry MCP controls without private knowledge."""
import hashlib,json,pathlib,sys
out=pathlib.Path(sys.argv[1]);out.mkdir(parents=True,exist_ok=True)
manifest=out/'extended-inputs.json';hashes=json.loads(manifest.read_text())if manifest.exists()else{}
text="memory ordering vector matrix type inference permission array broadcasting object stream parallel thread function memory"
for count,suffix in [(1,""),(4,"4"),(16,"16")]:
 records=[{"id":"tiny-entry"if count==1 else f"tiny-{n}","text":text,"metadata":{"fixture":"one-entry"if count==1 else f"{count}-entry"}}for n in range(count)]
 path=out/f'tiny{suffix}-mcp.json';path.write_text(json.dumps(records,indent=2)+'\n');hashes[path.name]=hashlib.sha256(path.read_bytes()).hexdigest()
manifest.write_text(json.dumps(hashes,indent=2)+'\n')
