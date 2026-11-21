import argparse
import json
import pathlib
import resource
import subprocess
import sys
import time


def main():
    parser = argparse.ArgumentParser(description="One fresh child per wrapper, with lifetime child RSS.")
    parser.add_argument("binary", type=pathlib.Path)
    parser.add_argument("mode", choices=["previous", "current"])
    parser.add_argument("source", type=pathlib.Path)
    parser.add_argument("count", type=int)
    parser.add_argument("queries", type=pathlib.Path)
    parser.add_argument("output", type=pathlib.Path)
    args = parser.parse_args()
    start = time.monotonic()
    subprocess.run([str(args.binary.resolve()), args.mode, str(args.source), str(args.count),
                    str(args.queries), str(args.output)], check=True, timeout=600)
    elapsed = time.monotonic() - start
    peak = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
    data = json.loads(args.output.read_text())
    data["child_lifetime_peak_rss_bytes"] = int(peak if sys.platform == "darwin" else peak * 1024)
    data["child_wall_seconds"] = elapsed
    data["rss_scope"] = "Whole child including full JSON parsing before prefix selection, source buffers, index and queries; excludes this wrapper."
    args.output.write_text(json.dumps(data, indent=2) + "\n")


if __name__ == "__main__":
    main()
