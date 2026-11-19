import argparse
import json
import pathlib
import subprocess
import time
import tomllib


def main():
    parser = argparse.ArgumentParser(description="Check every declared Cargo feature independently.")
    parser.add_argument("--toolchain")
    parser.add_argument("--report", type=pathlib.Path)
    args = parser.parse_args()
    root = pathlib.Path(__file__).resolve().parents[1]
    manifest = tomllib.loads((root / "Cargo.toml").read_text())
    assert manifest["features"]["default"] == [], "The default library must remain dependency-light"
    cargo = ["cargo"] + (["+" + args.toolchain] if args.toolchain else [])
    results = []
    for feature in sorted(manifest["features"]):
        started = time.monotonic()
        command = cargo + ["check", "--locked", "--all-targets", "--no-default-features", "--features", feature]
        print("Checking isolated feature:", feature, flush=True)
        result = subprocess.run(command, cwd=root, check=False)
        results.append({"feature": feature, "exit_code": result.returncode,
                        "elapsed_seconds": time.monotonic() - started})
        if result.returncode:
            break
    if args.report:
        args.report.write_text(json.dumps({"version": manifest["package"]["version"],
                                          "toolchain": args.toolchain or "active",
                                          "checks": results}, indent=2) + "\n")
    if any(item["exit_code"] for item in results):
        raise SystemExit(1)
    print("All", len(results), "isolated feature checks passed.", flush=True)


if __name__ == "__main__":
    main()
