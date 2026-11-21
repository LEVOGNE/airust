import pathlib
import subprocess

root = pathlib.Path(__file__).resolve().parents[1]
for feature in [None, "mcp"]:
    args = ["cargo", "tree", "--locked", "--no-default-features", "--edges", "normal", "--prefix", "none", "--format", "{p}"]
    if feature:
        args.extend(["--features", feature])
    lines = subprocess.check_output(args, cwd=root, text=True).splitlines()
    names = {line.split()[0] for line in lines if line.strip()}
    forbidden = {"airust-pdf-backend", "axum", "reqwest", "rusqlite", "fastembed", "ort"}
    if feature is None:
        forbidden.update(["rmcp", "tokio", "tokio-util"])
    assert not names & forbidden, (feature, names & forbidden)
    if feature:
        assert "rmcp" in names and "tokio" in names
print("Default and MCP dependency boundaries verified.")
