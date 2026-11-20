import difflib
import hashlib
import json
from pathlib import Path
import subprocess


def main():
    root = Path(__file__).resolve().parents[1]
    backend = root / "crates/airust-pdf-backend"
    metadata = json.loads(subprocess.check_output([
        "cargo", "metadata", "--manifest-path", str(backend / "Cargo.toml"),
        "--format-version", "1", "--locked", "--offline"
    ], text=True))
    package = next(p for p in metadata["packages"] if p["name"] == "pdf_oxide" and p["version"] == "0.3.78")
    original = Path(package["manifest_path"]).parent
    manifest = json.loads((backend / "UPSTREAM-SHA256.json").read_text())
    changed = {"src/document.rs", "src/lib.rs"}
    for name, expected in manifest["files"].items():
        assert hashlib.file_digest((original / name).open("rb"), "sha256").hexdigest() == expected, name
        if name not in changed:
            assert hashlib.file_digest((backend / name).open("rb"), "sha256").hexdigest() == expected, name
    actual = {p.relative_to(backend).as_posix() for p in (backend / "src").rglob("*") if p.is_file()}
    assert actual == set(manifest["files"]) | {"src/airust_small_object.rs"}
    patch = ""
    for name in ["src/lib.rs", "src/document.rs", "src/airust_small_object.rs"]:
        before = (original / name).read_text().splitlines(keepends=True) if (original / name).exists() else []
        after = (backend / name).read_text().splitlines(keepends=True)
        patch += "".join(difflib.unified_diff(before, after, fromfile="a/" + name if before else "/dev/null", tofile="b/" + name))
    assert patch == (backend / "upstream.patch").read_text(), "Patch/provenance drift"
    for name in ["LICENSE-MIT", "LICENSE-APACHE", "NOTICE", "TRADEMARKS.md"]:
        assert (backend / name).read_bytes() == (original / name).read_bytes(), name
    subprocess.run(["rustfmt", "--edition", "2021", "--check", str(backend / "src/airust_small_object.rs"), str(backend / "tests/regressions.rs")], check=True)
    print(f"PDF backend provenance verified: {len(manifest['files'])} upstream source files; two maintained files and one helper; original notices preserved.")


if __name__ == "__main__":
    main()
