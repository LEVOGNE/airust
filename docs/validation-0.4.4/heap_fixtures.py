import argparse
import json
import pathlib


def main():
    parser = argparse.ArgumentParser(description="Create authored heap-profile fixtures without private source text.")
    parser.add_argument("new_directory", type=pathlib.Path)
    args = parser.parse_args()
    args.new_directory.mkdir()
    cases = {
        "short-two": "alpha beta",
        "long-single": "a" * 65536,
        "long-two": "a" * 8192 + " " + "b" * 8192,
        "long-unicode-two": "資料" * 4096 + " " + "知識" * 4096,
        "repeated-short": " ".join(["alpha"] * 4096),
        "unique-prefix": " ".join("word" + str(n) for n in range(1024)),
        "unique-tail": " ".join("word" + str(n) for n in range(4096)),
        "mixed-long-prefix": "a" * 8192 + " beta " + " ".join("word" + str(n) for n in range(128)),
    }
    for name, text in cases.items():
        records = [{"id": str(n), "text": text, "metadata": {}} for n in range(64)]
        (args.new_directory / (name + ".json")).write_text(json.dumps(records, ensure_ascii=False), encoding="utf-8")
    print(json.dumps({"cases": len(cases), "entries_per_case": 64, "private_source_included": False}))


if __name__ == "__main__":
    main()
