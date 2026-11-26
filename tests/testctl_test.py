import contextlib
import importlib.util
import io
import json
import os
import subprocess
import sys
import tempfile
import unittest
from datetime import datetime, timedelta, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("testctl", ROOT / "scripts" / "testctl.py")
testctl = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(testctl)


class FixedClock(testctl.Clock):
    def __init__(self):
        self.moment = datetime(2026, 11, 26, 12, 0, tzinfo=timezone.utc)

    def now(self):
        return self.moment

    def advance(self, days):
        self.moment += timedelta(days=days)


class Sandbox:
    def __init__(self, directory):
        self.root = Path(directory) / "repo"
        self.counts = Path(directory) / "counts"
        self.state = Path(directory) / "state"
        self.root.mkdir()
        self.counts.mkdir()
        self.clock = FixedClock()
        self.git("init", "-q")
        (self.root / ".gitignore").write_text("ignored/\n")
        self.write("a/input.txt", "a1")
        self.write("b/input.txt", "b1")
        self.write("flag", "0")
        self.write("src/lib.rs", "fn main() {}\n")
        self.areas = []

    def git(self, *args):
        subprocess.run(["git", "-c", "user.name=t", "-c", "user.email=t@example.invalid", *args], cwd=self.root, check=True, capture_output=True)

    def commit(self):
        self.git("add", "-A")
        self.git("commit", "-qm", "fixture")

    def write(self, name, text):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)

    def counter(self, name, extra=""):
        target = self.counts / name
        code = f"import pathlib;p=pathlib.Path({str(target)!r});p.write_text(str(int(p.read_text()) + 1 if p.exists() else 1)){extra}"
        return ["{python}", "-c", code]

    def count(self, name):
        target = self.counts / name
        return int(target.read_text()) if target.exists() else 0

    def area(self, identifier, inputs, configs, kind="check", profiles=("quick", "full"), triggers=None):
        raw = {"id": identifier, "purpose": identifier, "kind": kind, "toolchain": None, "profiles": list(profiles),
               "inputs": inputs, "configs": configs}
        if triggers:
            raw["triggers"] = triggers
        self.areas.append(raw)

    @staticmethod
    def config(commands, **extra):
        return dict({"commands": commands, "timeout_seconds": 30, "estimate_seconds": 1}, **extra)

    def controller(self):
        plan = {"format_version": 1, "host_triple": "test-host",
                "budgets": {"quick_target_seconds": 300, "nightly_minutes": 90, "full_deep_minutes": 120, "starvation_factor": 2},
                "input_groups": {"all": ["**"]}, "areas": self.areas}
        plan_path = self.root.parent / "plan.json"
        plan_path.write_text(json.dumps(plan))
        doc = self.root.parent / "TESTPLAN.md"
        if not doc.exists():
            doc.write_text(f"intro\n{testctl.DOC_START}\n{testctl.DOC_END}\n")
        return testctl.Controller(self.root, self.state, plan_path, doc, self.clock)

    def run(self, profile, **options):
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            code = self.controller().run_profile(profile, **options)
        reports = sorted((self.state / "runs").glob("*.json"), key=lambda p: p.stat().st_mtime_ns)
        return code, json.loads(reports[-1].read_text()), output.getvalue()

    def status(self):
        return json.loads((self.state / "status.json").read_text())


class TestControllerSelection(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.box = Sandbox(self.directory.name)

    def tearDown(self):
        self.directory.cleanup()

    def keys(self, items):
        return sorted(item["key"] for item in items)

    def test_identical_inputs_are_reused_and_reported_as_reused(self):
        self.box.area("a", ["a/**"], {"default": Sandbox.config([self.box.counter("a")])})
        code, first, _ = self.box.run("quick")
        self.assertEqual(code, 0)
        self.assertEqual(self.keys(first["executed"]), ["a/default"])
        code, second, text = self.box.run("quick")
        self.assertEqual(code, 0)
        self.assertEqual(second["executed"], [])
        self.assertEqual(self.keys(second["reused"]), ["a/default"])
        self.assertEqual(second["reused"][0]["from_run"], first["run_id"])
        self.assertIn("REUSED", text)
        self.assertEqual(self.box.count("a"), 1)

    def test_only_areas_with_changed_relevant_inputs_execute_again(self):
        self.box.area("a", ["a/**"], {"default": Sandbox.config([self.box.counter("a")])})
        self.box.area("b", ["b/**"], {"default": Sandbox.config([self.box.counter("b")])})
        self.box.run("quick")
        self.box.write("a/input.txt", "a2")
        _, report, _ = self.box.run("quick")
        self.assertEqual(self.keys(report["executed"]), ["a/default"])
        self.assertEqual(self.keys(report["reused"]), ["b/default"])
        self.box.write("unrelated.txt", "x")
        _, report, _ = self.box.run("quick")
        self.assertEqual(report["executed"], [])

    def test_same_content_with_new_timestamp_is_still_reused(self):
        self.box.area("a", ["a/**"], {"default": Sandbox.config([self.box.counter("a")])})
        self.box.run("quick")
        path = self.box.root / "a/input.txt"
        os.utime(path, (1, 1))
        _, report, _ = self.box.run("quick")
        self.assertEqual(self.keys(report["reused"]), ["a/default"])

    def test_failure_is_never_green_and_reruns_until_fixed(self):
        failing = ["{python}", "-c", "import sys;sys.exit(int(open('flag').read()))"]
        self.box.write("flag", "3")
        self.box.area("gate", ["flag"], {"default": Sandbox.config([failing])})
        code, report, _ = self.box.run("quick")
        self.assertEqual(code, 1)
        self.assertEqual(report["executed"][0]["result"], "failed")
        code, report, _ = self.box.run("quick")
        self.assertEqual(code, 1)
        self.assertEqual(report["reused"], [])
        self.assertEqual(len(report["executed"]), 1)
        self.assertNotIn("last_success", self.box.status()["entries"]["gate/default"])
        self.box.write("flag", "0")
        code, report, _ = self.box.run("quick")
        self.assertEqual(code, 0)
        self.assertEqual(report["executed"][0]["result"], "passed")

    def test_corrupt_status_is_preserved_and_nothing_is_reused(self):
        self.box.area("a", ["a/**"], {"default": Sandbox.config([self.box.counter("a")])})
        self.box.run("quick")
        (self.box.state / "status.json").write_text("{broken")
        _, report, _ = self.box.run("quick")
        self.assertEqual(self.keys(report["executed"]), ["a/default"])
        self.assertTrue(any("unreadable" in note for note in report["notes"]))
        self.assertTrue(list(self.box.state.glob("status.corrupt-*.json")))
        self.assertEqual(self.box.count("a"), 2)

    def test_parallel_runs_are_refused_and_stale_locks_recovered(self):
        self.box.area("a", ["a/**"], {"default": Sandbox.config([self.box.counter("a")])})
        controller = self.box.controller()
        lock = testctl.Lock(self.box.state / "lock")
        lock.acquire()
        with self.assertRaises(testctl.ControllerError):
            controller.run_profile("quick")
        lock.release()
        dead = subprocess.Popen([sys.executable, "-c", "pass"])
        dead.wait()
        (self.box.state / "lock").write_text(json.dumps({"pid": dead.pid, "host": testctl.socket.gethostname()}))
        code, _, _ = self.box.run("quick")
        self.assertEqual(code, 0)
        self.assertFalse((self.box.state / "lock").exists())

    def test_timeout_is_incomplete_and_not_success(self):
        sleeper = ["{python}", "-c", "import time;time.sleep(30)"]
        self.box.area("slow", ["a/**"], {"default": dict(Sandbox.config([sleeper]), timeout_seconds=1)})
        code, report, _ = self.box.run("quick")
        self.assertEqual(code, 1)
        self.assertEqual(report["executed"][0]["result"], "incomplete")
        self.assertNotIn("last_success", self.box.status()["entries"]["slow/default"])

    def test_missing_executable_is_unavailable_not_passed(self):
        self.box.area("tool", ["a/**"], {"default": Sandbox.config([["airust-missing-tool-for-test"]])})
        code, report, _ = self.box.run("quick")
        self.assertEqual(code, 1)
        self.assertEqual(report["executed"][0]["result"], "unavailable")

    def test_zero_executed_tests_fail_the_guard(self):
        empty = ["{python}", "-c", "print('test result: ok. 0 passed; 0 failed; 0 ignored')"]
        real = ["{python}", "-c", "print('test result: ok. 3 passed; 0 failed; 1 ignored')"]
        self.box.area("empty", ["a/**"], {"default": Sandbox.config([empty])}, kind="test")
        self.box.area("real", ["b/**"], {"default": Sandbox.config([real])}, kind="test")
        code, report, _ = self.box.run("quick")
        results = {item["key"]: item for item in report["executed"]}
        self.assertEqual(code, 1)
        self.assertEqual(results["empty/default"]["result"], "failed")
        self.assertIn("guard", results["empty/default"])
        self.assertEqual(results["real/default"]["result"], "passed")
        self.assertEqual(results["real/default"]["tests"], {"passed": 3, "failed": 0, "ignored": 1})

    def test_risk_triggers_run_short_checks_only_for_matching_changes(self):
        self.box.area("a", ["a/**"], {"default": Sandbox.config([self.box.counter("a")])})
        self.box.area("parser", ["**"], {"short": Sandbox.config([self.box.counter("short")], deterministic=False),
                                         "deep": Sandbox.config([self.box.counter("deep")], deterministic=False, max_age_days=7)},
                      kind="stress", profiles=("nightly",), triggers=["src/parser.rs"])
        self.box.commit()
        self.box.run("quick")
        self.assertEqual(self.box.count("short"), 0)
        self.box.write("a/input.txt", "a2")
        self.box.run("quick")
        self.assertEqual(self.box.count("short"), 0)
        self.box.write("src/parser.rs", "fn parse() {}\n")
        _, report, _ = self.box.run("quick")
        self.assertEqual(self.box.count("short"), 1)
        self.assertIn("parser/short", self.keys(report["executed"]))
        self.assertEqual(self.box.count("deep"), 0)

    def test_without_baseline_triggers_compare_against_head(self):
        self.box.area("parser", ["**"], {"short": Sandbox.config([self.box.counter("short")], deterministic=False)},
                      kind="stress", profiles=("nightly",), triggers=["src/parser.rs"])
        self.box.write("src/parser.rs", "fn parse() {}\n")
        self.box.commit()
        self.box.run("quick")
        self.assertEqual(self.box.count("short"), 0)
        (self.box.state / "status.json").unlink()
        self.box.write("src/parser.rs", "fn parse() { 1; }\n")
        _, report, _ = self.box.run("quick")
        self.assertEqual(self.box.count("short"), 1)
        self.assertTrue(any("git HEAD" in note for note in report["notes"]))

    def test_unsafe_outside_every_miri_group_is_reported_as_gap(self):
        self.box.area("a", ["a/**"], {"default": Sandbox.config([self.box.counter("a")])})
        self.box.commit()
        self.box.run("quick")
        self.box.write("src/lib.rs", "fn main() { unsafe { core::hint::unreachable_unchecked() } }\n")
        _, report, _ = self.box.run("quick")
        self.assertTrue(any("src/lib.rs" in gap and "Miri" in gap for gap in report["gaps"]))
        self.box.write("src/lib.rs", "#![forbid(unsafe_code)]\n// unsafe { } only in a comment\nfn main() {}\n")
        _, report, _ = self.box.run("quick")
        self.assertFalse(any("Miri" in gap for gap in report["gaps"]))

    def test_nightly_runs_due_work_then_exits_cleanly_when_nothing_is_due(self):
        self.box.area("deep", ["a/**"], {"deep": Sandbox.config([self.box.counter("deep")], max_age_days=7)}, kind="stress", profiles=("nightly",))
        self.box.commit()
        code, report, _ = self.box.run("nightly", in_place=True)
        self.assertEqual(code, 0)
        self.assertEqual(self.keys(report["executed"]), ["deep/deep"])
        code, report, _ = self.box.run("nightly", in_place=True)
        self.assertEqual(code, 0)
        self.assertEqual(report["executed"], [])
        self.assertTrue(any("nothing due" in note for note in report["notes"]))
        self.box.clock.advance(8)
        _, report, _ = self.box.run("nightly", in_place=True)
        self.assertEqual(self.keys(report["executed"]), ["deep/deep"])
        self.assertIn("maximum 7 d", report["executed"][0]["reason"])

    def test_budget_and_focus_defer_work_visibly_and_starvation_raises_priority(self):
        self.box.area("fuzzish", ["a/**"], {"deep": Sandbox.config([self.box.counter("f")], deterministic=False, max_age_days=7, estimate_seconds=10)},
                      kind="fuzz-like", profiles=("nightly",))
        self.box.area("stressy", ["b/**"], {"deep": Sandbox.config([self.box.counter("s")], deterministic=False, max_age_days=7, estimate_seconds=10)},
                      kind="stress", profiles=("nightly",))
        self.box.commit()
        _, report, _ = self.box.run("nightly", in_place=True)
        self.assertEqual(len(report["executed"]), 1)
        self.assertEqual(len(report["deferred_due"]), 1)
        self.assertIn("focus", report["deferred_due"][0]["reason"])
        self.assertTrue(report["gaps"])
        _, report, _ = self.box.run("nightly", in_place=True)
        self.assertEqual(len(report["executed"]), 1)
        self.assertEqual(self.box.count("f") + self.box.count("s"), 2)
        self.box.clock.advance(15)
        self.box.write("a/input.txt", "changed")
        _, report, _ = self.box.run("nightly", in_place=True, budget_minutes=0.1)
        self.assertIn("starvation", report["executed"][0]["reason"])
        _, report, _ = self.box.run("nightly", in_place=True, budget_minutes=0.0001)
        self.assertEqual(report["executed"], [])
        self.assertIn("exceeds remaining budget", report["deferred_due"][0]["reason"])

    def test_known_deep_failure_is_reported_without_rerun_until_inputs_change(self):
        failing = ["{python}", "-c", "import sys;sys.exit(int(open('a/input.txt').read()))"]
        self.box.write("a/input.txt", "1")
        self.box.area("miriish", ["a/**"], {"deep": Sandbox.config([failing], max_age_days=7)}, kind="stress", profiles=("nightly",))
        code, report, _ = self.box.run("nightly", in_place=True)
        self.assertEqual(code, 1)
        code, report, _ = self.box.run("nightly", in_place=True)
        self.assertEqual(code, 1)
        self.assertEqual(report["executed"], [])
        self.assertEqual(self.keys(report["known_failures"]), ["miriish/deep"])
        self.box.write("a/input.txt", "0")
        code, report, _ = self.box.run("nightly", in_place=True)
        self.assertEqual(code, 0)
        self.assertEqual(self.keys(report["executed"]), ["miriish/deep"])

    def test_full_is_an_independent_control_without_reuse(self):
        self.box.area("a", ["a/**"], {"default": Sandbox.config([self.box.counter("a")])})
        self.box.commit()
        self.box.run("quick")
        _, report, _ = self.box.run("full", in_place=True)
        self.assertEqual(self.keys(report["executed"]), ["a/default"])
        self.assertEqual(report["reused"], [])
        self.assertEqual(self.box.count("a"), 2)

    def test_isolated_worktree_uses_the_commit_and_leaves_local_changes_untouched(self):
        reader = ["{python}", "-c", f"import pathlib;pathlib.Path({str(self.box.counts / 'seen')!r}).write_text(open('a/input.txt').read())"]
        self.box.area("deep", ["a/**"], {"deep": Sandbox.config([reader], max_age_days=7)}, kind="stress", profiles=("nightly",))
        self.box.commit()
        self.box.write("a/input.txt", "uncommitted")
        _, report, _ = self.box.run("nightly")
        self.assertEqual((self.box.counts / "seen").read_text(), "a1")
        self.assertEqual((self.box.root / "a/input.txt").read_text(), "uncommitted")
        self.assertTrue(any("uncommitted" in note for note in report["notes"]))
        record = self.box.status()["entries"]["deep/deep"]["last_success"]
        self.assertTrue(record["git"]["isolated_worktree"])
        self.assertFalse((self.box.state / "worktrees" / "isolated").exists())

    def test_status_is_written_atomically_and_records_required_fields(self):
        self.box.area("a", ["a/**"], {"default": Sandbox.config([self.box.counter("a")])})
        self.box.run("quick")
        self.assertEqual(list(self.box.state.glob(".*.tmp")), [])
        record = self.box.status()["entries"]["a/default"]["last_success"]
        for field in ("run_id", "started_at", "finished_at", "duration_seconds", "result", "fingerprint", "manifest",
                      "commands", "tools", "configuration", "git", "log", "tests"):
            self.assertIn(field, record)
        self.assertTrue(Path(record["log"]).is_file())
        self.assertEqual(self.box.status()["entries"]["a/default"]["next_due"], "on relevant input, configuration or toolchain change")

    def test_documentation_drift_fails_the_plan_check(self):
        self.box.area("a", ["a/**"], {"default": Sandbox.config([self.box.counter("a")])})
        controller = self.box.controller()
        with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
            self.assertEqual(controller.plan_command(True, False), 1)
            self.assertEqual(controller.plan_command(False, True), 0)
            self.assertEqual(controller.plan_command(True, False), 0)

    def test_invalid_plan_is_rejected(self):
        self.box.area("a", ["@missing"], {"default": Sandbox.config([["true"]])})
        with self.assertRaises(testctl.ControllerError):
            self.box.controller()


class TestPatterns(unittest.TestCase):
    def test_single_star_stays_in_one_directory_and_double_star_recurses(self):
        paths = testctl.PathSet(["docs/*.md", "src/**", "!src/skip.rs"])
        self.assertTrue(paths.matches("docs/A.md"))
        self.assertFalse(paths.matches("docs/validation/A.md"))
        self.assertTrue(paths.matches("src/a/b.rs"))
        self.assertFalse(paths.matches("src/skip.rs"))


if __name__ == "__main__":
    unittest.main(verbosity=1)
