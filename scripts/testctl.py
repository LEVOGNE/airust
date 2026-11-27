import argparse
import hashlib
import json
import os
import platform
import random
import re
import shutil
import signal
import socket
import subprocess
import sys
import time
from datetime import datetime, timedelta, timezone
from pathlib import Path

FORMAT_VERSION = 1
PASSED = "passed"
FAILED = "failed"
INCOMPLETE = "incomplete"
UNAVAILABLE = "unavailable"
DOC_START = "<!-- testctl:areas:start -->"
DOC_END = "<!-- testctl:areas:end -->"
FUZZ_RUSTFLAGS = (
    "--cfg fuzzing -Cdebug-assertions -Zsanitizer=address -Cpasses=sancov-module "
    "-Cllvm-args=-sanitizer-coverage-level=4 -Cllvm-args=-sanitizer-coverage-inline-8bit-counters "
    "-Cllvm-args=-sanitizer-coverage-pc-table -Cllvm-args=-sanitizer-coverage-trace-compares"
)
PASSTHROUGH_ENV = ("RUSTFLAGS", "RUSTDOCFLAGS", "CARGO_BUILD_TARGET", "CARGO_ENCODED_RUSTFLAGS", "MIRIFLAGS")
CARGO_ENV = {"CARGO_INCREMENTAL": "0", "CARGO_PROFILE_DEV_DEBUG": "0", "CARGO_PROFILE_TEST_DEBUG": "0",
             "CARGO_PROFILE_RELEASE_DEBUG": "0", "CARGO_TERM_COLOR": "never"}


class ControllerError(Exception):
    pass


class Clock:
    def now(self):
        return datetime.now(timezone.utc)

    def stamp(self, moment=None):
        return (moment or self.now()).isoformat(timespec="seconds")

    def parse(self, value):
        return datetime.fromisoformat(value) if value else None


class Digest:
    @staticmethod
    def text(value):
        return hashlib.sha256(value.encode()).hexdigest()

    @staticmethod
    def data(value):
        return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(",", ":")).encode()).hexdigest()

    @staticmethod
    def file(path):
        digest = hashlib.sha256()
        with open(path, "rb") as stream:
            for block in iter(lambda: stream.read(1 << 20), b""):
                digest.update(block)
        return digest.hexdigest()


class AtomicFile:
    @staticmethod
    def write_json(path, value):
        path.parent.mkdir(parents=True, exist_ok=True)
        temporary = path.with_name(f".{path.name}.{os.getpid()}.tmp")
        try:
            with open(temporary, "w") as stream:
                stream.write(json.dumps(value, indent=2, sort_keys=True) + "\n")
                stream.flush()
                os.fsync(stream.fileno())
            os.replace(temporary, path)
        finally:
            if temporary.exists():
                temporary.unlink()
        directory = os.open(path.parent, os.O_RDONLY)
        try:
            os.fsync(directory)
        finally:
            os.close(directory)


class Pattern:
    def __init__(self, text):
        self.text = text
        parts = []
        index = 0
        while index < len(text):
            if text.startswith("**", index):
                parts.append(".*")
                index += 2
            elif text[index] == "*":
                parts.append("[^/]*")
                index += 1
            elif text[index] == "?":
                parts.append("[^/]")
                index += 1
            else:
                parts.append(re.escape(text[index]))
                index += 1
        self.regex = re.compile("".join(parts) + r"\Z")

    def matches(self, path):
        return self.regex.match(path) is not None


class PathSet:
    def __init__(self, patterns):
        self.include = [Pattern(p) for p in patterns if not p.startswith("!")]
        self.exclude = [Pattern(p[1:]) for p in patterns if p.startswith("!")]

    def matches(self, path):
        return any(p.matches(path) for p in self.include) and not any(p.matches(path) for p in self.exclude)

    def select(self, paths):
        return [path for path in paths if self.matches(path)]


class AreaConfig:
    def __init__(self, area, name, raw):
        self.area = area
        self.name = name
        self.raw = raw
        self.commands = raw.get("commands", [])
        self.deterministic = raw.get("deterministic", True)
        self.repeat = int(raw.get("repeat", 1))
        self.max_age_days = raw.get("max_age_days")
        self.timeout = float(raw["timeout_seconds"])
        self.estimate = float(raw["estimate_seconds"])
        self.fuzz_seconds = raw.get("fuzz_seconds")
        if self.repeat < 1 or self.timeout <= 0 or self.estimate <= 0:
            raise ControllerError(f"invalid budget in {self.key}")
        if area.kind == "fuzz" and not self.fuzz_seconds:
            raise ControllerError(f"fuzz config without fuzz_seconds: {self.key}")
        if area.kind != "fuzz" and not self.commands:
            raise ControllerError(f"config without commands: {self.key}")

    @property
    def key(self):
        return f"{self.area.id}/{self.name}"

    @property
    def deep(self):
        return self.max_age_days is not None

    @property
    def trigger_only(self):
        return self.name == "short" and not self.deep

    def max_age(self):
        return timedelta(days=self.max_age_days) if self.deep else None

    def definition(self):
        return {"area": self.area.raw, "config": self.name}


class Area:
    def __init__(self, raw, groups):
        self.raw = raw
        self.id = raw["id"]
        self.purpose = raw["purpose"]
        self.kind = raw["kind"]
        self.profiles = set(raw["profiles"])
        self.toolchain = raw.get("toolchain", "default")
        self.inputs = PathSet(self.expand(raw["inputs"], groups))
        self.triggers = PathSet(raw.get("triggers", []))
        self.has_triggers = bool(raw.get("triggers"))
        self.fuzz = raw.get("fuzz")
        self.configs = {name: AreaConfig(self, name, value) for name, value in raw["configs"].items()}

    @staticmethod
    def expand(patterns, groups):
        result = []
        for pattern in patterns:
            if pattern.startswith("@"):
                if pattern[1:] not in groups:
                    raise ControllerError(f"unknown input group {pattern}")
                result.extend(groups[pattern[1:]])
            else:
                result.append(pattern)
        return result

    def toolchains(self, config):
        names = set() if self.toolchain is None else {self.toolchain}
        for command in config.commands:
            if command and command[0] == "cargo" and len(command) > 1 and command[1].startswith("+"):
                names.add(command[1][1:])
        return sorted(names)

    def uses_python(self, config):
        return any("{python}" in command for command in config.commands)


class Plan:
    def __init__(self, path):
        self.path = path
        try:
            self.raw = json.loads(path.read_text())
        except (OSError, ValueError) as error:
            raise ControllerError(f"cannot read test plan {path}: {error}") from error
        if self.raw.get("format_version") != FORMAT_VERSION:
            raise ControllerError("unsupported test plan format")
        self.budgets = self.raw["budgets"]
        self.host = self.raw.get("host_triple") or Tools.host_triple()
        self.areas = [Area(raw, self.raw["input_groups"]) for raw in self.raw["areas"]]
        ids = [area.id for area in self.areas]
        if len(ids) != len(set(ids)):
            raise ControllerError("duplicate area identifiers")
        self.all_inputs = PathSet(["**.rs"] + [p for area in self.raw["areas"] for p in Area.expand(area["inputs"], self.raw["input_groups"]) + area.get("triggers", []) if not p.startswith("!")])

    def configs(self):
        return [config for area in self.areas for config in area.configs.values()]

    def render_table(self):
        rows = ["| Area | Config | Kind | Profiles | Max age | Trigger paths | Commands | Purpose |",
                "| --- | --- | --- | --- | --- | --- | --- | --- |"]
        for config in self.configs():
            area = config.area
            if area.kind == "fuzz":
                commands = f"libFuzzer `{area.fuzz['target']}` for {config.fuzz_seconds} s (ASAN, nightly)"
            else:
                rendered = ["`" + " ".join(command).replace("|", "\\|") + "`" for command in config.commands]
                commands = "<br>".join(rendered) + (f" (x{config.repeat})" if config.repeat > 1 else "")
            age = f"{config.max_age_days} d" if config.deep else "input change"
            triggers = ", ".join(f"`{t}`" for t in area.raw.get("triggers", [])) or "-"
            profiles = ", ".join(sorted(area.profiles | ({"quick (trigger)"} if config.trigger_only else set())))
            rows.append(f"| {area.id} | {config.name} | {area.kind} | {profiles} | {age} | {triggers} | {commands} | {area.purpose} |")
        return "\n".join(rows)

    def documentation(self, doc):
        text = doc.read_text()
        if DOC_START not in text or DOC_END not in text:
            raise ControllerError(f"{doc} lacks the generated area markers")
        head, rest = text.split(DOC_START, 1)
        _, tail = rest.split(DOC_END, 1)
        return text, head + DOC_START + "\n" + self.render_table() + "\n" + DOC_END + tail


class Tools:
    def __init__(self, cwd):
        self.cwd = cwd
        self.cache = {}

    @staticmethod
    def host_triple():
        try:
            output = subprocess.run(["rustc", "-vV"], capture_output=True, text=True, check=True).stdout
        except (OSError, subprocess.CalledProcessError):
            return None
        match = re.search(r"^host: (\S+)$", output, re.MULTILINE)
        return match.group(1) if match else None

    def probe(self, command):
        key = tuple(command)
        if key not in self.cache:
            try:
                result = subprocess.run(command, cwd=self.cwd, capture_output=True, text=True, timeout=120)
                self.cache[key] = result.stdout.strip() if result.returncode == 0 else None
            except (OSError, subprocess.TimeoutExpired):
                self.cache[key] = None
        return self.cache[key]

    def toolchain(self, name):
        prefix = [] if name == "default" else ["+" + name]
        rustc = self.probe(["rustc", *prefix, "-vV"])
        cargo = self.probe(["cargo", *prefix, "-V"])
        if rustc is None or cargo is None:
            return None
        return {"rustc": rustc, "cargo": cargo}

    def versions(self, area, config):
        versions = {}
        missing = []
        for name in area.toolchains(config):
            found = self.toolchain(name)
            if found is None:
                missing.append(f"Rust toolchain {name}")
            versions[name] = found
        if area.kind == "miri":
            versions["miri"] = self.probe(["cargo", "+nightly", "miri", "--version"])
            if versions["miri"] is None:
                missing.append("cargo-miri (nightly)")
        if area.uses_python(config):
            versions["python"] = platform.python_version()
        return versions, missing

    def sysroot(self, name):
        prefix = [] if name == "default" else ["+" + name]
        return self.probe(["rustc", *prefix, "--print", "sysroot"])


class Workspace:
    def __init__(self, root, isolated):
        self.root = root
        self.isolated = isolated
        self.hashes = {}
        self.listing = None

    def git(self, *args):
        return subprocess.run(["git", *args], cwd=self.root, capture_output=True, check=True).stdout

    def files(self):
        if self.listing is None:
            listed = self.git("ls-files", "-z", "--cached", "--others", "--exclude-standard").split(b"\0")
            names = sorted({name.decode() for name in listed if name})
            self.listing = [name for name in names if (self.root / name).is_file()]
        return self.listing

    def digest(self, name):
        if name not in self.hashes:
            self.hashes[name] = Digest.file(self.root / name)
        return self.hashes[name]

    def manifest(self, pathset):
        return {name: self.digest(name) for name in pathset.select(self.files())}

    def head(self):
        try:
            return self.git("rev-parse", "HEAD").decode().strip()
        except subprocess.CalledProcessError:
            return None

    def dirty(self):
        status = self.git("status", "--porcelain=v1", "-z", "--untracked-files=all").split(b"\0")
        return sorted({entry[3:].decode() for entry in status if len(entry) > 3})

    def changed_since_head(self):
        try:
            changed = self.git("diff", "--name-only", "-z", "HEAD").split(b"\0")
        except subprocess.CalledProcessError:
            return None
        untracked = self.git("ls-files", "-z", "--others", "--exclude-standard").split(b"\0")
        return sorted({name.decode() for name in changed + untracked if name})


class Lock:
    def __init__(self, path):
        self.path = path
        self.held = False

    @staticmethod
    def alive(pid):
        try:
            os.kill(pid, 0)
        except ProcessLookupError:
            return False
        except PermissionError:
            return True
        return True

    def acquire(self):
        self.path.parent.mkdir(parents=True, exist_ok=True)
        for _ in range(2):
            try:
                descriptor = os.open(self.path, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600)
            except FileExistsError:
                try:
                    owner = json.loads(self.path.read_text())
                    pid = int(owner["pid"])
                    same_host = owner.get("host") == socket.gethostname()
                except (OSError, ValueError, KeyError, TypeError):
                    owner, pid, same_host = None, None, True
                if pid is not None and same_host and self.alive(pid):
                    raise ControllerError(f"another testctl run is active (pid {pid}); parallel duplicate runs are refused")
                if pid is not None and not same_host:
                    raise ControllerError(f"lock held by host {owner.get('host')}; remove {self.path} only if that run is gone")
                self.path.unlink(missing_ok=True)
                continue
            with os.fdopen(descriptor, "w") as stream:
                stream.write(json.dumps({"pid": os.getpid(), "host": socket.gethostname(), "started": Clock().stamp()}))
            self.held = True
            return
        raise ControllerError("could not acquire the testctl lock")

    def release(self):
        if self.held:
            self.path.unlink(missing_ok=True)
            self.held = False


class StatusStore:
    def __init__(self, directory):
        self.directory = directory
        self.path = directory / "status.json"
        self.manifests = directory / "manifests"
        self.notes = []
        self.planned = set()
        self.data = self.load()

    def empty(self):
        return {"format_version": FORMAT_VERSION, "entries": {}, "baselines": {}}

    def load(self):
        if not self.path.exists():
            self.notes.append("no previous run status; nothing can be reused")
            return self.empty()
        try:
            data = json.loads(self.path.read_text())
            if data.get("format_version") != FORMAT_VERSION or not isinstance(data.get("entries"), dict):
                raise ValueError("unexpected status format")
            data.setdefault("baselines", {})
            for entry in data["entries"].values():
                if not isinstance(entry, dict):
                    raise ValueError("entry is not an object")
            return data
        except (OSError, ValueError) as error:
            backup = self.path.with_name(f"status.corrupt-{int(time.time())}.json")
            shutil.copy2(self.path, backup)
            self.notes.append(f"status was unreadable ({error}); preserved as {backup.name}; nothing reused")
            return self.empty()

    def entry(self, key):
        return self.data["entries"].setdefault(key, {})

    def peek(self, key):
        return self.data["entries"].get(key, {})

    def store_manifest(self, manifest):
        identifier = Digest.data(manifest)
        path = self.manifests / f"{identifier}.json"
        if not path.exists():
            AtomicFile.write_json(path, manifest)
        self.planned.add(identifier)
        return identifier

    def read_manifest(self, identifier):
        try:
            return json.loads((self.manifests / f"{identifier}.json").read_text())
        except (OSError, ValueError, TypeError):
            return None

    def save(self):
        AtomicFile.write_json(self.path, self.data)
        referenced = {(self.data["baselines"].get("quick") or {}).get("manifest")} | self.planned
        for entry in self.data["entries"].values():
            for record in (entry.get("last_attempt"), entry.get("last_success")):
                if record:
                    referenced.add(record.get("manifest"))
        if self.manifests.exists():
            for path in self.manifests.glob("*.json"):
                if path.stem not in referenced:
                    path.unlink(missing_ok=True)


class Execution:
    def __init__(self, config, fingerprint, manifest_id, versions, missing, workspace, reason):
        self.config = config
        self.fingerprint = fingerprint
        self.manifest_id = manifest_id
        self.versions = versions
        self.missing = missing
        self.workspace = workspace
        self.reason = reason


class CommandRunner:
    def __init__(self, controller):
        self.controller = controller

    def environment(self, workspace, toolchain):
        env = dict(os.environ)
        env.update(CARGO_ENV)
        lane = ("worktree-" if workspace.isolated else "") + (toolchain or "default")
        env["CARGO_TARGET_DIR"] = str(self.controller.state / "target" / lane)
        return env

    def substitute(self, command, log_dir):
        return [part.replace("{python}", sys.executable).replace("{log_dir}", str(log_dir)) for part in command]

    def spawn(self, command, cwd, env, log, deadline):
        log.write(f"\n$ {' '.join(command)}\n".encode())
        log.flush()
        try:
            process = subprocess.Popen(command, cwd=cwd, env=env, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
        except FileNotFoundError as error:
            log.write(f"missing executable: {error}\n".encode())
            return UNAVAILABLE, None
        try:
            code = process.wait(timeout=max(1.0, deadline - time.monotonic()))
        except subprocess.TimeoutExpired:
            self.stop(process)
            log.write(b"\ntimeout: process group stopped\n")
            return INCOMPLETE, None
        except KeyboardInterrupt:
            self.stop(process)
            raise
        return (PASSED if code == 0 else FAILED), code

    @staticmethod
    def stop(process):
        for sent in (signal.SIGTERM, signal.SIGKILL):
            try:
                os.killpg(process.pid, sent)
            except ProcessLookupError:
                return
            try:
                process.wait(timeout=10)
                return
            except subprocess.TimeoutExpired:
                continue

    def run(self, execution, log_path, log_dir):
        config = execution.config
        env = self.environment(execution.workspace, config.area.toolchain)
        deadline = time.monotonic() + config.timeout
        commands = []
        result, code = PASSED, 0
        with open(log_path, "ab") as log:
            for iteration in range(config.repeat):
                for command in config.commands:
                    resolved = self.substitute(command, log_dir)
                    commands.append(resolved)
                    if config.repeat > 1:
                        log.write(f"\n== iteration {iteration + 1}/{config.repeat}\n".encode())
                    result, code = self.spawn(resolved, execution.workspace.root, env, log, deadline)
                    if result != PASSED:
                        return {"result": result, "exit_code": code, "commands": commands, "completed_iterations": iteration}
        return {"result": result, "exit_code": code, "commands": commands, "completed_iterations": config.repeat}


class FuzzRunner(CommandRunner):
    def run(self, execution, log_path, log_dir):
        config = execution.config
        spec = config.area.fuzz
        workspace = execution.workspace
        plan = self.controller.plan
        env = dict(os.environ)
        env.update(CARGO_ENV)
        env["RUSTFLAGS"] = FUZZ_RUSTFLAGS
        env["CARGO_TARGET_DIR"] = str(self.controller.state / "target" / ("worktree-fuzz" if workspace.isolated else "fuzz"))
        deadline = time.monotonic() + config.timeout
        build = ["cargo", "+nightly", "build", "--manifest-path", "fuzz/Cargo.toml", "--release", "--locked",
                 "--target", plan.host, "--no-default-features", "--features", ",".join(spec["features"]), "--bin", spec["target"]]
        corpus = self.controller.state / "fuzz" / "corpus" / spec["target"]
        artifacts = self.controller.state / "fuzz" / "artifacts" / spec["target"] / self.controller.run_id
        corpus.mkdir(parents=True, exist_ok=True)
        artifacts.mkdir(parents=True, exist_ok=True)
        seed = random.SystemRandom().randrange(1, 2**31 - 1)
        binary = Path(env["CARGO_TARGET_DIR"]) / plan.host / "release" / spec["target"]
        run = [str(binary), f"-max_total_time={int(config.fuzz_seconds)}", f"-seed={seed}", "-timeout=10",
               "-rss_limit_mb=2048", f"-max_len={spec['max_len']}", "-detect_leaks=0", "-print_final_stats=1",
               f"-artifact_prefix={artifacts}/", str(corpus)]
        if spec.get("seeds") and (workspace.root / spec["seeds"]).is_dir():
            run.append(str(workspace.root / spec["seeds"]))
        before = len(list(corpus.iterdir()))
        with open(log_path, "ab") as log:
            result, code = self.spawn(build, workspace.root, env, log, deadline)
            if result != PASSED:
                return {"result": result, "exit_code": code, "commands": [build], "fuzz": {"stage": "build", "seed": seed}}
            runtime_env = dict(os.environ)
            runtime_env["ASAN_OPTIONS"] = "detect_leaks=0:abort_on_error=1:symbolize=1"
            sysroot = self.controller.tools.sysroot("nightly")
            if sys.platform == "darwin" and sysroot:
                runtime_env["DYLD_LIBRARY_PATH"] = str(Path(sysroot) / "lib" / "rustlib" / plan.host / "lib")
            started = time.monotonic()
            result, code = self.spawn(run, workspace.root, runtime_env, log, deadline)
            elapsed = time.monotonic() - started
        text = log_path.read_text(errors="replace")
        stats = {key: int(value) for key, value in re.findall(r"stat::(\w+):\s+(\d+)", text)}
        found = sorted(str(path) for path in artifacts.iterdir() if path.is_file())
        if not found:
            artifacts.rmdir()
        if result == PASSED and (found or stats.get("number_of_executed_units", 0) == 0):
            result = FAILED
        verdict = ("no crash found within the search budget (not a correctness proof)" if result == PASSED
                   else "crash or failure; artifacts retained" if found else "fuzz run did not complete")
        return {"result": result, "exit_code": code, "commands": [build, run],
                "fuzz": {"seed": seed, "seconds": round(elapsed, 1), "executed_units": stats.get("number_of_executed_units"),
                         "peak_rss_mb": stats.get("peak_rss_mb"), "corpus_files_before": before,
                         "corpus_files_after": len(list(corpus.iterdir())), "artifacts": found, "verdict": verdict}}


class Report:
    def __init__(self, profile, run_id, clock):
        self.profile = profile
        self.run_id = run_id
        self.clock = clock
        self.started = clock.stamp()
        self.executed = []
        self.reused = []
        self.skipped = []
        self.known_failures = []
        self.deferred = []
        self.gaps = []
        self.notes = []

    def failed(self):
        return any(item["result"] != PASSED for item in self.executed) or bool(self.known_failures)

    def document(self):
        return {"run_id": self.run_id, "profile": self.profile, "started_at": self.started, "finished_at": self.clock.stamp(),
                "executed": self.executed, "reused": self.reused, "skipped": self.skipped,
                "known_failures": self.known_failures, "deferred_due": self.deferred, "gaps": self.gaps, "notes": self.notes,
                "overall": "failed" if self.failed() else "passed"}

    def render(self):
        lines = [f"testctl {self.profile} run {self.run_id}"]
        for note in self.notes:
            lines.append(f"  note: {note}")
        for item in self.executed:
            extra = f" [{item['fuzz']['verdict']}]" if item.get("fuzz") and "verdict" in item["fuzz"] else ""
            counts = f" tests={item['tests']}" if item.get("tests") else ""
            lines.append(f"  EXECUTED  {item['key']:<32} {item['result']:<11} {item['duration_seconds']:>8.1f}s{counts}{extra} ({item['reason']})")
        for item in self.reused:
            lines.append(f"  REUSED    {item['key']:<32} passed in run {item['from_run']} at {item['at']} (identical inputs and configuration)")
        for item in self.known_failures:
            lines.append(f"  FAILING   {item['key']:<32} {item['result']} in run {item['from_run']}; inputs unchanged, fix required ({item['log']})")
        for item in self.skipped:
            lines.append(f"  SKIPPED   {item['key']:<32} {item['reason']}")
        for item in self.deferred:
            lines.append(f"  DUE       {item['key']:<32} {item['reason']}")
        for gap in self.gaps:
            lines.append(f"  GAP       {gap}")
        lines.append("  RESULT    " + ("FAILED" if self.failed() else "passed"))
        return "\n".join(lines)


class Controller:
    def __init__(self, root, state, plan_path=None, doc_path=None, clock=None):
        self.root = root.resolve()
        self.state = state.resolve()
        self.plan_path = plan_path or self.root / "docs" / "testplan.json"
        self.doc_path = doc_path or self.root / "docs" / "TESTPLAN.md"
        self.plan = Plan(self.plan_path)
        self.clock = clock or Clock()
        self.tools = Tools(self.root)
        self.run_id = self.clock.now().strftime("%Y%m%dT%H%M%SZ") + "-" + os.urandom(4).hex()
        self.controller_digest = Digest.file(Path(__file__).resolve())

    def fingerprint(self, config, manifest, versions):
        env = {name: os.environ.get(name) for name in PASSTHROUGH_ENV}
        if config.area.kind == "fuzz":
            env = {"fuzz_rustflags": FUZZ_RUSTFLAGS}
        return Digest.data({"definition": config.definition(), "controller": self.controller_digest, "inputs": manifest,
                            "tools": versions, "env": env, "cargo_env": CARGO_ENV, "host": self.plan.host})

    def prepare(self, store, workspace, config, reason):
        manifest = workspace.manifest(config.area.inputs)
        versions, missing = self.tools.versions(config.area, config)
        fingerprint = self.fingerprint(config, manifest, versions)
        return Execution(config, fingerprint, store.store_manifest(manifest), versions, missing, workspace, reason)

    def reusable(self, store, execution):
        entry = store.peek(execution.config.key)
        attempt, success = entry.get("last_attempt"), entry.get("last_success")
        if not execution.config.deterministic or not attempt or not success:
            return None
        if attempt.get("run_id") != success.get("run_id") or success.get("fingerprint") != execution.fingerprint:
            return None
        max_age = execution.config.max_age()
        finished = self.clock.parse(success.get("finished_at"))
        if finished is None or (max_age and self.clock.now() - finished >= max_age):
            return None
        return success

    def known_failure(self, store, execution):
        attempt = store.peek(execution.config.key).get("last_attempt")
        if not attempt or attempt.get("result") != FAILED or attempt.get("fingerprint") != execution.fingerprint:
            return None
        return attempt

    def next_due(self, config, success):
        if not success:
            return "now"
        if config.deep:
            return self.clock.stamp(self.clock.parse(success["finished_at"]) + config.max_age())
        return "on relevant input, configuration or toolchain change"

    def execute(self, store, report, execution):
        config = execution.config
        log_dir = self.state / "logs" / self.run_id
        log_dir.mkdir(parents=True, exist_ok=True)
        area_dir = log_dir / config.key.replace("/", "-")
        area_dir.mkdir(exist_ok=True)
        log_path = log_dir / (config.key.replace("/", "-") + ".log")
        started_wall = self.clock.stamp()
        started = time.monotonic()
        print(f"-> {config.key} ({execution.reason})", flush=True)
        if execution.missing:
            log_path.write_text("unavailable: " + ", ".join(execution.missing) + "\n")
            outcome = {"result": UNAVAILABLE, "exit_code": None, "commands": config.commands}
            report.gaps.append(f"{config.key}: missing tool(s) {', '.join(execution.missing)}; not executed, not passed")
        else:
            runner = FuzzRunner(self) if config.area.kind == "fuzz" else CommandRunner(self)
            try:
                outcome = runner.run(execution, log_path, area_dir)
            except KeyboardInterrupt:
                outcome = {"result": INCOMPLETE, "exit_code": None, "commands": config.commands, "interrupted": True}
        tests = self.test_counts(log_path)
        if outcome["result"] == PASSED and config.area.kind in ("test", "concurrency") and tests["passed"] == 0:
            outcome["result"] = FAILED
            outcome["guard"] = "no tests were executed; a filter or target mismatch is a failure, not a pass"
        if not any(area_dir.iterdir()):
            area_dir.rmdir()
        record = {"run_id": self.run_id, "profile": report.profile, "started_at": started_wall, "finished_at": self.clock.stamp(),
                  "duration_seconds": round(time.monotonic() - started, 1), "result": outcome["result"],
                  "exit_code": outcome.get("exit_code"), "fingerprint": execution.fingerprint, "manifest": execution.manifest_id,
                  "commands": outcome.get("commands"), "tools": execution.versions,
                  "configuration": {"kind": config.area.kind, "config": config.name, "repeat": config.repeat,
                                    "deterministic": config.deterministic, "env": CARGO_ENV,
                                    "passthrough_env": {n: os.environ.get(n) for n in PASSTHROUGH_ENV if os.environ.get(n)}},
                  "git": {"head": execution.workspace.head(), "isolated_worktree": execution.workspace.isolated,
                          "uncommitted_inputs": sorted(set(execution.workspace.dirty()) & set(store.read_manifest(execution.manifest_id) or {}))},
                  "log": str(log_path), "tests": tests, "reason": execution.reason}
        for key in ("fuzz", "guard", "interrupted", "completed_iterations"):
            if key in outcome:
                record[key] = outcome[key]
        entry = store.entry(config.key)
        entry["last_attempt"] = record
        entry["attempts"] = entry.get("attempts", 0) + 1
        if outcome["result"] == PASSED:
            entry["last_success"] = record
        if config.area.kind == "fuzz" and record.get("fuzz", {}).get("seconds"):
            totals = entry.setdefault("fuzz_seconds", {"all": 0.0, "by_fingerprint": {}})
            totals["all"] = round(totals["all"] + record["fuzz"]["seconds"], 1)
            by = totals["by_fingerprint"]
            by[execution.fingerprint] = round(by.get(execution.fingerprint, 0.0) + record["fuzz"]["seconds"], 1)
            for stale in list(by)[:-20]:
                del by[stale]
        entry["next_due"] = self.next_due(config, entry.get("last_success")) if outcome["result"] == PASSED else "now"
        store.save()
        report.executed.append({"key": config.key, "result": outcome["result"], "duration_seconds": record["duration_seconds"],
                                "reason": execution.reason, "log": str(log_path), "tests": tests if tests["passed"] or tests["failed"] else None,
                                **({"fuzz": record["fuzz"]} if "fuzz" in record else {}), **({"guard": record["guard"]} if "guard" in record else {})})
        return outcome["result"]

    @staticmethod
    def test_counts(log_path):
        try:
            text = log_path.read_text(errors="replace")
        except OSError:
            return {"passed": 0, "failed": 0, "ignored": 0}
        counts = {"passed": 0, "failed": 0, "ignored": 0}
        for passed, failed, ignored in re.findall(r"^test result: \w+\. (\d+) passed; (\d+) failed; (\d+) ignored", text, re.MULTILINE):
            counts["passed"] += int(passed)
            counts["failed"] += int(failed)
            counts["ignored"] += int(ignored)
        miri = re.findall(r'"passed_tests": (\d+)', text)
        if miri and not counts["passed"]:
            counts["passed"] = sum(map(int, miri))
        return counts

    def reuse(self, report, config, success):
        report.reused.append({"key": config.key, "from_run": success["run_id"], "at": success["finished_at"], "log": success["log"]})

    def run_or_reuse(self, store, report, workspace, config, reason, allow_reuse=True, retry_known_failure=True):
        execution = self.prepare(store, workspace, config, reason)
        success = self.reusable(store, execution) if allow_reuse else None
        if success:
            self.reuse(report, config, success)
            return PASSED
        failure = None if retry_known_failure else self.known_failure(store, execution)
        if failure:
            report.known_failures.append({"key": config.key, "result": failure["result"], "from_run": failure["run_id"], "log": failure["log"]})
            return FAILED
        return self.execute(store, report, execution)

    def changed_paths(self, store, workspace, report):
        baseline = store.data["baselines"].get("quick")
        previous = store.read_manifest(baseline["manifest"]) if baseline else None
        current = workspace.manifest(self.plan.all_inputs)
        if previous is None:
            changed = workspace.changed_since_head()
            if changed is None:
                report.gaps.append("no quick baseline and no git HEAD; risk triggers could not be evaluated")
                return None, current
            report.notes.append("risk triggers evaluated against git HEAD (no successful quick baseline yet)")
            return changed, current
        changed = sorted({p for p in set(previous) | set(current) if previous.get(p) != current.get(p)})
        return changed, current

    @staticmethod
    def unsafe_paths(workspace, paths):
        pattern = re.compile(r"\bunsafe\s*(\{|fn\b|impl\b|trait\b|extern\b)|#!?\[\s*(allow|warn)\s*\(\s*unsafe_code")
        found = []
        for path in paths:
            target = workspace.root / path
            if path.endswith(".rs") and target.is_file():
                code = re.sub(r"//[^\n]*", "", target.read_text(errors="replace"))
                if pattern.search(code):
                    found.append(path)
        return found

    def quick(self, store, report, workspace, candidate):
        if candidate:
            dirty = workspace.dirty()
            if dirty:
                raise ControllerError("--candidate requires a clean worktree; commit or stash first (uncommitted: " + ", ".join(dirty[:5]) + ")")
            report.notes.append(f"merge candidate {workspace.head()}")
        suite_ok = True
        for area in self.plan.areas:
            if "quick" in area.profiles and "default" in area.configs:
                result = self.run_or_reuse(store, report, workspace, area.configs["default"], "quick suite")
                suite_ok = suite_ok and result == PASSED
        changed, current = self.changed_paths(store, workspace, report)
        if changed is not None:
            unsafe = self.unsafe_paths(workspace, changed)
            covered = set()
            for area in self.plan.areas:
                hits = area.triggers.select(changed) if area.has_triggers else []
                if not hits:
                    continue
                if area.kind == "miri":
                    risky = [p for p in unsafe if area.triggers.matches(p)]
                    if risky:
                        covered.update(risky)
                        self.run_or_reuse(store, report, workspace, area.configs["deep"], "unsafe change in " + ", ".join(risky[:3]))
                    else:
                        report.deferred.append({"key": area.configs["deep"].key, "reason": "relevant change (" + ", ".join(hits[:3]) + "); due in nightly with new-risk priority"})
                elif "short" in area.configs:
                    self.run_or_reuse(store, report, workspace, area.configs["short"], "risk trigger: " + ", ".join(hits[:3]))
            for path in sorted(set(unsafe) - covered):
                report.gaps.append(f"unsafe code in {path} is not covered by any Miri group; add a group to scripts/run_miri.py and docs/testplan.json")
        if suite_ok and not report.failed():
            store.data["baselines"]["quick"] = {"manifest": store.store_manifest(current), "run_id": self.run_id, "at": self.clock.stamp()}
            store.save()

    def priority(self, store, execution, now):
        config = execution.config
        entry = store.peek(config.key)
        attempt, success = entry.get("last_attempt"), entry.get("last_success")
        factor = self.plan.budgets["starvation_factor"]
        if attempt and attempt.get("result") != PASSED:
            if attempt.get("result") == FAILED and attempt.get("fingerprint") == execution.fingerprint:
                return None, "known failure"
            return 0.0, f"last attempt {attempt.get('result')}"
        if not success:
            return 1.0, "never run"
        if success.get("fingerprint") != execution.fingerprint:
            return 1.0, "relevant inputs, configuration or toolchain changed since last success"
        finished = self.clock.parse(success.get("finished_at"))
        if finished is None:
            return 0.0, "last success lacks a timestamp"
        age = now - finished
        if age >= config.max_age() * factor:
            return 0.5, f"coverage {age.days} d old, more than {factor}x the {config.max_age_days} d maximum (starvation guard)"
        if age >= config.max_age():
            return 2.0, f"coverage {age.days} d old, maximum {config.max_age_days} d"
        return None, "covered"

    def deep_queue(self, store, report, workspace, profile):
        now = self.clock.now()
        queue = []
        for area in self.plan.areas:
            if profile not in area.profiles:
                continue
            for config in area.configs.values():
                if not config.deep:
                    continue
                execution = self.prepare(store, workspace, config, "")
                rank, reason = self.priority(store, execution, now)
                if reason == "known failure":
                    failure = store.peek(config.key)["last_attempt"]
                    report.known_failures.append({"key": config.key, "result": failure["result"], "from_run": failure["run_id"], "log": failure["log"]})
                    continue
                if rank is None:
                    success = store.peek(config.key).get("last_success")
                    if config.deterministic and success:
                        self.reuse(report, config, success)
                    continue
                last = store.peek(config.key).get("last_attempt") or {}
                measured = last.get("duration_seconds") if last.get("result") == PASSED else None
                estimate = max(1.0, float(measured if measured is not None else config.estimate))
                execution.reason = reason
                queue.append((rank, last.get("finished_at") or "", config.key, execution, estimate))
        queue.sort(key=lambda item: (item[0], item[1], item[2]))
        return queue

    def deep(self, store, report, workspace, profile, budget_seconds, single_focus, requested_focus=None):
        queue = self.deep_queue(store, report, workspace, profile)
        if not queue:
            report.notes.append("nothing due; no expensive run started")
            return
        started = time.monotonic()
        focus = None
        if requested_focus:
            focus = requested_focus
            report.notes.append(f"focus {focus} requested explicitly")
        elif single_focus:
            fitting = [item for item in queue if item[4] <= budget_seconds]
            focus = (fitting or queue)[0][3].config.area.kind
            report.notes.append(f"focus tonight: {focus} (one compute-intensive focus at a time)")
        for rank, _, key, execution, estimate in queue:
            remaining = budget_seconds - (time.monotonic() - started)
            if focus and execution.config.area.kind != focus:
                report.deferred.append({"key": key, "reason": f"{execution.reason}; waits for its own focus night (priority {rank})"})
            elif estimate > remaining:
                report.deferred.append({"key": key, "reason": f"{execution.reason}; estimate {estimate:.0f}s exceeds remaining budget {max(0, remaining):.0f}s"})
            else:
                self.execute(store, report, execution)
        if report.deferred:
            report.gaps.append(f"{len(report.deferred)} due deep check(s) did not fit this run; they stay due and gain priority with age")

    def full(self, store, report, workspace, budget_seconds):
        report.notes.append("independent control: every quick and full check executes fresh; nothing is reused")
        for area in self.plan.areas:
            if area.profiles & {"quick", "full"} and "default" in area.configs:
                self.run_or_reuse(store, report, workspace, area.configs["default"], "full independent control", allow_reuse=False)
        self.deep(store, report, workspace, "full", budget_seconds, single_focus=False)

    def worktree(self, revision):
        source = Workspace(self.root, False)
        path = self.state / "worktrees" / "isolated"
        if path.exists():
            subprocess.run(["git", "worktree", "remove", "--force", str(path)], cwd=self.root, capture_output=True)
            if path.exists():
                shutil.rmtree(path)
        subprocess.run(["git", "worktree", "prune"], cwd=self.root, capture_output=True)
        commit = source.git("rev-parse", "--verify", f"{revision}^{{commit}}").decode().strip()
        path.parent.mkdir(parents=True, exist_ok=True)
        subprocess.run(["git", "worktree", "add", "--detach", str(path), commit], cwd=self.root, capture_output=True, check=True)
        return Workspace(path, True), commit

    def release_worktree(self, workspace):
        if workspace.isolated:
            subprocess.run(["git", "worktree", "remove", "--force", str(workspace.root)], cwd=self.root, capture_output=True)

    def run_profile(self, profile, candidate=False, revision="HEAD", in_place=False, budget_minutes=None, focus=None):
        lock = Lock(self.state / "lock")
        lock.acquire()
        report = Report(profile, self.run_id, self.clock)
        workspace = Workspace(self.root, False)
        try:
            store = StatusStore(self.state)
            report.notes.extend(store.notes)
            if profile != "quick" and not in_place:
                local = Workspace(self.root, False).dirty()
                workspace, commit = self.worktree(revision)
                report.notes.append(f"isolated detached worktree at {commit}")
                if local:
                    report.notes.append(f"{len(local)} uncommitted local change(s) are not part of this run and remain untouched")
            if profile == "quick":
                self.quick(store, report, workspace, candidate)
            elif profile == "nightly":
                minutes = budget_minutes if budget_minutes is not None else self.plan.budgets["nightly_minutes"]
                report.notes.append(f"budget {minutes} min including compilation")
                self.deep(store, report, workspace, "nightly", minutes * 60, single_focus=True, requested_focus=focus)
            else:
                minutes = budget_minutes if budget_minutes is not None else self.plan.budgets["full_deep_minutes"]
                report.notes.append(f"deep budget {minutes} min after the fresh suite")
                self.full(store, report, workspace, minutes * 60)
        except KeyboardInterrupt:
            report.gaps.append("run interrupted; executed results up to this point are recorded, the rest did not run")
        finally:
            self.release_worktree(workspace)
            document = report.document()
            AtomicFile.write_json(self.state / "runs" / f"{self.run_id}.json", document)
            lock.release()
        print(report.render(), flush=True)
        return 1 if report.failed() or any("interrupted" in gap for gap in report.gaps) else 0

    def status(self, as_json):
        store = StatusStore(self.state)
        rows = []
        for config in self.plan.configs():
            entry = store.peek(config.key)
            attempt, success = entry.get("last_attempt") or {}, entry.get("last_success") or {}
            rows.append({"key": config.key, "last_result": attempt.get("result", "never"), "last_attempt": attempt.get("finished_at"),
                         "last_success": success.get("finished_at"),
                         "next_due": entry.get("next_due", "on matching risk trigger in quick" if config.trigger_only else "now"),
                         "fuzz_seconds_total": entry.get("fuzz_seconds", {}).get("all")})
        if as_json:
            print(json.dumps({"notes": store.notes, "areas": rows}, indent=2))
            return 0
        for note in store.notes:
            print("note:", note)
        for row in rows:
            fuzz = f" fuzz={row['fuzz_seconds_total']}s" if row["fuzz_seconds_total"] else ""
            print(f"{row['key']:<32} {row['last_result']:<11} success={row['last_success'] or '-':<26} due={row['next_due']}{fuzz}")
        return 0

    def plan_command(self, check, write):
        current, expected = self.plan.documentation(self.doc_path)
        if write:
            self.doc_path.write_text(expected)
            print(f"updated {self.doc_path}")
            return 0
        if check and current != expected:
            print(f"{self.doc_path} is out of date with {self.plan_path}; run: scripts/testctl.py plan --write-doc", file=sys.stderr)
            return 1
        if check:
            print(f"test plan valid: {len(self.plan.configs())} area configurations; documentation in sync")
            return 0
        print(self.plan.render_table())
        return 0


class Cli:
    def __init__(self):
        self.parser = argparse.ArgumentParser(prog="testctl", description="Resource-aware AIRust test control (see docs/TESTPLAN.md).")
        self.parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
        self.parser.add_argument("--state", type=Path)
        self.parser.add_argument("--plan", type=Path)
        self.parser.add_argument("--doc", type=Path)
        commands = self.parser.add_subparsers(dest="command", required=True)
        quick = commands.add_parser("quick")
        quick.add_argument("--candidate", action="store_true")
        for name in ("nightly", "full"):
            deep = commands.add_parser(name)
            deep.add_argument("--rev", default="HEAD")
            deep.add_argument("--in-place", action="store_true")
            deep.add_argument("--budget-minutes", type=float)
            if name == "nightly":
                deep.add_argument("--focus", choices=["miri", "fuzz", "concurrency", "stress"])
        status = commands.add_parser("status")
        status.add_argument("--json", action="store_true")
        plan = commands.add_parser("plan")
        plan.add_argument("--check", action="store_true")
        plan.add_argument("--write-doc", action="store_true")

    def main(self, argv=None):
        args = self.parser.parse_args(argv)
        root = args.root.resolve()
        try:
            controller = Controller(root, args.state or root / ".testctl", args.plan, args.doc)
            if args.command == "quick":
                return controller.run_profile("quick", candidate=args.candidate)
            if args.command in ("nightly", "full"):
                return controller.run_profile(args.command, revision=args.rev, in_place=args.in_place,
                                              budget_minutes=args.budget_minutes, focus=getattr(args, "focus", None))
            if args.command == "status":
                return controller.status(args.json)
            return controller.plan_command(args.check, args.write_doc)
        except ControllerError as error:
            print(f"testctl: {error}", file=sys.stderr)
            return 2


if __name__ == "__main__":
    sys.exit(Cli().main())
