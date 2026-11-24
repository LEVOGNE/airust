import argparse
import json
import pathlib
import subprocess


def main():
    parser = argparse.ArgumentParser(description="Verify the Cargo package contains only intended repository files.")
    parser.add_argument("--report", type=pathlib.Path)
    args = parser.parse_args()
    root = pathlib.Path(__file__).resolve().parents[1]
    command = ["cargo", "package", "--list", "--allow-dirty", "--locked", "--offline"]
    result = subprocess.run(command, cwd=root, text=True, capture_output=True, check=True)
    files = result.stdout.splitlines()
    allowed_files = {
        ".cargo_vcs_info.json", "Cargo.toml", "Cargo.toml.orig", "Cargo.lock", "build.rs",
        "README.md", "STARTUP.md", "CHANGELOG.md", "CONTRIBUTING.md", "AGENTS.md", "ROADMAP.md",
        "LICENSE", "test.sh", "Dockerfile", ".dockerignore", "knowledge/train.json",
        "data/pdf/sample.pdf",
    }
    allowed_roots = {"src", "examples", "scripts", "tests", "docs"}
    required = {"Cargo.toml", "Cargo.lock", "README.md", "LICENSE", "src/lib.rs", "scripts/check_package.py"}
    unexpected = []
    for name in files:
        path = pathlib.PurePosixPath(name)
        if path.is_absolute() or ".." in path.parts or not path.parts:
            unexpected.append(name)
        elif name not in allowed_files and path.parts[0] not in allowed_roots:
            unexpected.append(name)
        elif path.parts[:2] == ("docs", "measurements") and path.suffix == ".json":
            unexpected.append(name)
    if unexpected or not required.issubset(files):
        raise SystemExit(f"Unexpected package content: {unexpected}; missing required files: {sorted(required-set(files))}")
    if args.report:
        args.report.write_text(json.dumps({"files": files, "count": len(files), "unexpected": unexpected,
                                          "scope": "Read-only dirty-tree listing; actual release packaging requires a clean commit."}, indent=2) + "\n")
    print(f"Package boundary verified: {len(files)} intended files.")


if __name__ == "__main__":
    main()
