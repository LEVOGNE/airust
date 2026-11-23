import hashlib
import json
import pathlib
import sys

folder = pathlib.Path(sys.argv[1])
folder.mkdir(parents=True, exist_ok=True)
records = [{"id": f"sparse{n}", "text": " ".join(f"term{(n*2+j)%512}" for j in range(16))} for n in range(256)]
records.append({"id": "dense", "text": " ".join(f"term{n}" for n in range(512))})
queries = [" ".join(f"term{n}" for n in range(width)) for width in [32,33,64,128,32,33,64,128,33,128]]
for name, value in [("wide-mcp.json", records), ("wide-mcp-queries.json", queries)]:
    (folder/name).write_text(json.dumps(value, ensure_ascii=False, separators=(",", ":")) + "\n")
hashes = {name: hashlib.sha256((folder/name).read_bytes()).hexdigest() for name in ["wide-mcp.json", "wide-mcp-queries.json"]}
(folder/"extended-inputs.json").write_text(json.dumps(hashes, indent=2) + "\n")
print(json.dumps({"records": len(records), "queries": len(queries), "widths": [32,33,64,128]}))
