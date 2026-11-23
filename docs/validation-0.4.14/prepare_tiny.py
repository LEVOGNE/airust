"""Create a reproducible one-entry MCP control without private knowledge."""
import hashlib,json,pathlib,sys
out=pathlib.Path(sys.argv[1]);out.mkdir(parents=True,exist_ok=True)
records=[{"id":"tiny-entry","text":"memory ordering vector matrix type inference permission array broadcasting object stream parallel thread function memory","metadata":{"fixture":"one-entry"}}]
path=out/'tiny-mcp.json';path.write_text(json.dumps(records,indent=2)+'\n')
manifest=out/'extended-inputs.json';hashes=json.loads(manifest.read_text())if manifest.exists()else{};hashes[path.name]=hashlib.sha256(path.read_bytes()).hexdigest();manifest.write_text(json.dumps(hashes,indent=2)+'\n')
