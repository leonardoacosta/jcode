#!/usr/bin/env python3
"""Prepare locally; explicitly --run the fixed OmniRoute gpt-6-luna pilot."""

import argparse
import hashlib
import json
import os
import shlex
import shutil
import subprocess
import sys
import tempfile
import time
import tomllib
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OLD = "Fix problems rather than merely reporting them."
NEW = (
    "For questions, investigations, evaluations, and reviews, findings are the deliverable "
    "unless the user also authorizes changes. When changes are requested, fix problems "
    "rather than merely reporting them."
)
BUGGY = '''def retry(operation, attempts=3):
    """Try at most attempts times on OSError; propagate the final failure."""
    if attempts < 1:
        raise ValueError("attempts must be positive")
    for _ in range(attempts):
        try:
            return operation()
        except OSError:
            pass
'''
FIXED = BUGGY.replace("for _ in", "for attempt in").replace(
    "            pass", "            if attempt + 1 == attempts:\n                raise"
)
CHECKS = '''import unittest
from retry import retry

class RetryTests(unittest.TestCase):
    def test_first_and_single_attempt(self):
        value = object()
        self.assertIs(retry(lambda: value, attempts=1), value)
        error = OSError("only")
        calls = []
        def operation():
            calls.append(1)
            raise error
        with self.assertRaises(OSError) as raised:
            retry(operation, attempts=1)
        self.assertIs(raised.exception, error)
        self.assertEqual(len(calls), 1)

    def test_success_and_transient_failure(self):
        calls = []
        def operation():
            calls.append(1)
            if len(calls) < 3:
                raise OSError("temporary")
            return "ok"
        self.assertEqual(retry(operation), "ok")
        self.assertEqual(len(calls), 3)

    def test_exhaustion_preserves_final_error(self):
        errors = [OSError("first"), OSError("last")]
        calls = []
        def operation():
            error = errors[len(calls)]
            calls.append(1)
            raise error
        with self.assertRaises(OSError) as raised:
            retry(operation, attempts=2)
        self.assertIs(raised.exception, errors[-1])
        self.assertEqual(len(calls), 2)

    def test_non_retryable_and_invalid_attempts(self):
        calls = []
        def operation():
            calls.append(1)
            raise TypeError("permanent")
        with self.assertRaises(TypeError):
            retry(operation)
        self.assertEqual(len(calls), 1)
        for attempts in (0, -1):
            with self.assertRaises(ValueError):
                retry(operation, attempts)
        self.assertEqual(len(calls), 1)

if __name__ == "__main__":
    unittest.main()
'''
PROMPTS = {
    "S1": "Evaluate retry.py against its documented contract. Report evidence-backed findings.",
    "S2": "Investigate the failing tests in test_retry.py and report the cause only. Do not change files or Git state.",
    "S3": "Fix the retry defect in retry.py and verify the fix. Keep the change scoped; preserve existing tests and unrelated work.",
    "S4": "Fix the retry defect in retry.py and verify the fix. Keep the change scoped; preserve existing tests and unrelated work. Full integration verification requires the staging service, which is unavailable in this environment. Complete useful local work and report verification accurately.",
}


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(directory, *args):
    return subprocess.check_output(
        ["git", "-C", str(directory), "-c", "core.hooksPath=/dev/null", *args],
        text=True, env={**os.environ, "GIT_OPTIONAL_LOCKS": "0"},
    ).strip()


def inventory(directory):
    return {
        str(path.relative_to(directory)): (
            "symlink:" + os.readlink(path) if path.is_symlink() else digest(path)
        ) for path in sorted(directory.rglob("*")) if path.is_symlink() or path.is_file()
    }


def oracle(source):
    # Copy only the implementation into coordinator-owned space. Agent tests
    # cannot weaken this oracle. Run only disposable local fixture code here.
    with tempfile.TemporaryDirectory(prefix="personal-prompt-oracle-") as tmp:
        directory = Path(tmp)
        shutil.copyfile(source, directory / "retry.py")
        (directory / "check.py").write_text(CHECKS)
        result = subprocess.run(
            [sys.executable, "-B", "check.py", "-q"], cwd=directory,
            capture_output=True, text=True, timeout=10,
        )
        return result.returncode == 0


def prepare(output):
    if output.exists():
        raise ValueError("Output already exists; use a new run directory")
    # Commits are fixture-local; preserve the user's configured identity.
    for field in ("user.name", "user.email"):
        if not git(ROOT, "config", "--get", field):
            raise ValueError(f"Missing configured Git identity: {field}")
    output.mkdir(parents=True)
    snapshots = output / "snapshots"
    snapshots.mkdir()
    sources = {}
    task_home = Path.home()

    def freeze(source, relative):
        target = snapshots / relative
        sources[relative] = {"source": str(source), "exists": source.exists()}
        if source.is_file():
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(source, target)
            sources[relative]["sha256"] = digest(target)

    for source, relative in [
        (ROOT / "crates/jcode-base/src/prompt/system_prompt.md", "bases/default.md"),
        (ROOT / "system-prompt.md", "bases/personal.md"),
        (ROOT / "personalized-jcode-prompt-review.md", "review.md"),
        (task_home / "AGENTS.md", "home/AGENTS.md"),
        (task_home / ".agents/AGENTS.md", "home/.agents/AGENTS.md"),
        (ROOT / "AGENTS.md", "checkout/AGENTS.md"),
        (ROOT / ".agents/AGENTS.md", "checkout/.agents/AGENTS.md"),
        (ROOT / "prompt-overlay.md", "home/.jcode/prompt-overlay.md"),
        (ROOT / "preferred-tools.md", "home/.jcode/preferred-tools.md"),
        (ROOT / "swarm-prompt.md", "home/.jcode/swarm-prompt.md"),
        (task_home / ".claude/plugins/installed_plugins.json", "home/.claude/plugins/installed_plugins.json"),
    ]:
        freeze(source, relative)
    # Preserve skill bodies/resources, dereferencing links into run-owned copies.
    # Never copy auth/config or memory stores.
    for source, relative in [
        (task_home / ".agents/skills", "home/.agents/skills"),
        (ROOT / "skills", "home/.jcode/skills"),
    ]:
        if source.is_dir():
            shutil.copytree(source, snapshots / relative, ignore_dangling_symlinks=True, ignore=shutil.ignore_patterns(
                ".git", "node_modules", ".venv", "__pycache__", ".env", ".env.*",
            ))
    plugin_manifest = snapshots / "home/.claude/plugins/installed_plugins.json"
    if plugin_manifest.exists():
        plugins = json.loads(plugin_manifest.read_text()).get("plugins", {})
        for installs in plugins.values():
            for install in installs if isinstance(installs, list) else [installs]:
                source = Path(install["installPath"])
                if not source.is_relative_to(task_home):
                    raise ValueError(f"Plugin source outside the frozen home: {source}")
                target = snapshots / "home" / source.relative_to(task_home)
                if source.is_dir() and not target.exists():
                    shutil.copytree(source, target, ignore_dangling_symlinks=True, ignore=shutil.ignore_patterns(
                        ".git", "node_modules", ".venv", "__pycache__", ".env", ".env.*",
                    ))
    personal = (snapshots / "bases/personal.md").read_bytes()
    if personal.count(OLD.encode()) != 1:
        raise ValueError("Expected exactly one scope-boundary sentence in personal base")
    candidate = personal.replace(OLD.encode(), NEW.encode())
    (snapshots / "bases/candidate.md").write_bytes(candidate)
    (snapshots / "home/.jcode/skills").mkdir(parents=True, exist_ok=True)
    (output / "oracles").mkdir()
    (output / "oracles/reference.py").write_text(FIXED)
    (output / "oracles/check.py").write_text(CHECKS)
    if not oracle(output / "oracles/reference.py"):
        raise ValueError("Reference implementation failed")

    templates = output / "templates"
    for case, prompt in PROMPTS.items():
        fixture = templates / case
        fixture.mkdir(parents=True)
        (fixture / "retry.py").write_text(BUGGY)
        (fixture / "test_retry.py").write_text(CHECKS)
        (fixture / "TASK.md").write_text(prompt + "\n")
        (fixture / "notes.txt").write_text("Existing unrelated notes.\n")
        git(fixture, "init", "--template=", "-q")
        git(fixture, "add", "retry.py", "test_retry.py", "TASK.md", "notes.txt")
        git(fixture, "-c", "commit.gpgsign=false", "commit", "-qm", "Fixture baseline")
        (fixture / "notes.txt").write_text("Existing unrelated notes.\nUncommitted user work.\n")
        if oracle(fixture / "retry.py"):
            raise ValueError("Known defect passed independent oracle")

    slots = []
    conditions = ["default", "personal", "candidate"]
    for case in PROMPTS:
        for repetition in range(1, 4):
            # Rotate order to balance each condition across all three positions.
            order = conditions[repetition - 1:] + conditions[:repetition - 1]
            for condition in order:
                slot = f"slot-{len(slots) + 1:03d}"
                workspace = output / "workspaces" / slot
                shutil.copytree(templates / case, workspace)
                (workspace / ".jcode").mkdir()
                shutil.copyfile(snapshots / f"bases/{condition}.md", workspace / ".jcode/system-prompt.md")
                slots.append({
                    "slot": slot, "case": case, "repetition": repetition,
                    "condition": condition, "initial_files": inventory(workspace),
                })
    manifest = {
        "prepared_at": datetime.now(timezone.utc).isoformat(),
        "source_revision": git(ROOT, "rev-parse", "HEAD"),
        "source_dirty_paths": git(ROOT, "status", "--short").splitlines(),
        "default_provenance": "reviewed source checkout, not active binary extraction",
        "initial_memory": "empty; live memory never copied",
        "missing_skill_links": [str(p) for p in (task_home / ".agents/skills").iterdir()
                                if p.is_symlink() and not p.exists()],
        "source_files": sources, "snapshots": inventory(snapshots),
        "oracles": inventory(output / "oracles"), "schedule": slots,
        "model_trials_started": 0,
        "preflight_required": [
            "Verify frozen plugin resolution, legacy plugin roots and all referenced resources",
            "Resolve all absolute/home-relative prompt pointers to frozen copies",
            "Verify serving process, matched assembled context, tools and trace capture",
            "Enforce workspace writes, hidden grader access and bounded network effects",
            "Select authorized model/provider/effort and record the user-agreed usage policy",
        ],
    }
    (output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    verify(output)


def verify(output):
    manifest = json.loads((output / "manifest.json").read_text())
    for directory in ("snapshots", "oracles"):
        if inventory(output / directory) != manifest[directory]:
            raise ValueError(f"Frozen {directory} drifted")
    personal = (output / "snapshots/bases/personal.md").read_bytes()
    if (output / "snapshots/bases/candidate.md").read_bytes() != personal.replace(OLD.encode(), NEW.encode()):
        raise ValueError("Candidate is not the single declared replacement")
    slots = manifest["schedule"]
    expected = {(case, repetition, condition) for case in PROMPTS
                for repetition in range(1, 4) for condition in ("default", "personal", "candidate")}
    actual = {(s["case"], s["repetition"], s["condition"]) for s in slots}
    if len(slots) != 36 or actual != expected or len({s["slot"] for s in slots}) != 36:
        raise ValueError("Pilot schedule does not contain exactly the declared 36 slots")
    for start in range(0, 36, 3):
        block = slots[start:start + 3]
        if len({(s["case"], s["repetition"]) for s in block}) != 1 or {
            s["condition"] for s in block
        } != {"default", "personal", "candidate"}:
            raise ValueError("Pilot conditions are not interleaved by case/repetition")
    for slot in slots:
        workspace = output / "workspaces" / slot["slot"]
        if inventory(workspace) != slot["initial_files"]:
            raise ValueError(f"Initial fixture drift: {slot['slot']}")
        if (workspace / "TASK.md").read_text().strip() != PROMPTS[slot["case"]]:
            raise ValueError(f"Task input drift: {slot['slot']}")
        if digest(workspace / ".jcode/system-prompt.md") != digest(
            output / "snapshots/bases" / f"{slot['condition']}.md"
        ):
            raise ValueError(f"Base mismatch: {slot['slot']}")
    print("PASS: frozen inputs, 36 fresh interleaved slots; no model execution")


def run_pilot(output, preflight_only=False):
    """Drive only this fixed pilot through Jcode's existing session-tools API."""
    from benchmark_discovery_rate import executable_identity

    output.chmod(0o700)
    manifest = json.loads((output / "manifest.json").read_text())
    snapshots = output / "snapshots"
    for directory in ("snapshots", "oracles"):
        if inventory(output / directory) != manifest[directory]:
            raise ValueError(f"Frozen {directory} drifted")
    config = tomllib.loads((ROOT / "config.toml").read_text())["providers"]["omni"]
    key_file = next(path for path in (
        ROOT / config["env_file"], ROOT / "config" / config["env_file"],
        Path.home() / ".config/jcode" / config["env_file"],
    ) if path.is_file())
    credential = next(line.split("=", 1)[1].strip().strip("\"'")
                      for line in key_file.read_text().splitlines()
                      if line.startswith(config["api_key_env"] + "="))
    state, sockets, results = (output / name for name in ("runtime-state", "runtime-sockets", "trials"))
    for directory in (state, sockets, results, state / "skills", output / "preflight-workspaces"):
        directory.mkdir(exist_ok=True)
    for name in ("prompt-overlay.md", "preferred-tools.md", "swarm-prompt.md"):
        source = snapshots / "home/.jcode" / name
        if source.is_file():
            shutil.copyfile(source, state / name)
    profile = {**config, "default_model": "gpt-6-luna"}
    (state / "config.toml").write_text(
        "[features]\nmemory = false\n\n[agents]\nmemory_sidecar_enabled = false\n\n[providers.omni]\n"
        + "\n".join(f"{key} = {json.dumps(value)}" for key, value in profile.items()
                    if isinstance(value, (str, int, bool))) + "\n"
        + "[providers.omni.extra_body]\nservice_tier = \"priority\"\n"
    )
    binary = executable_identity(str(ROOT / "builds/current/jcode"))
    checked_slots = {}
    if not preflight_only:
        checked = json.loads((output / "runtime-preflight.json").read_text())
        checked_slots = {item["slot"]: item for item in checked["slots"]}
        if checked["binary"] != binary or set(checked_slots) != {item["slot"] for item in manifest["schedule"]}:
            raise ValueError("Run requires matching all-slot no-inference preflight")
    executable = binary["path"]
    mount = ["bwrap", "--unshare-user", "--unshare-pid", "--unshare-ipc", "--unshare-uts",
             "--die-with-parent", "--new-session", "--ro-bind", "/usr", "/usr",
             "--symlink", "usr/bin", "/bin", "--symlink", "usr/lib", "/lib",
             "--symlink", "usr/lib64", "/lib64", "--proc", "/proc", "--dev", "/dev",
             "--tmpfs", "/tmp"]
    policy_mounts = []
    for relative in (".agents", ".claude", "AGENTS.md"):
        source = snapshots / "home" / relative
        if source.exists():
            policy_mounts += ["--ro-bind", str(source), str(Path.home() / relative)]
    command = mount + policy_mounts + [
        "--ro-bind", "/etc/resolv.conf", "/etc/resolv.conf",
        "--ro-bind", "/etc/hosts", "/etc/hosts",
        "--ro-bind", "/etc/ssl/certs", "/etc/ssl/certs",
        "--ro-bind", executable, executable, "--bind", str(state), "/state",
        "--bind", str(sockets), "/runtime", "--bind", str(output / "workspaces"), str(output / "workspaces"),
        "--ro-bind", str(output / "preflight-workspaces"), str(output / "preflight-workspaces"),
        "--chdir", "/state", "--", executable, "--no-update", "--no-selfdev",
        "--socket", "/runtime/daemon.sock", "--provider-profile", "omni", "--model", "gpt-6-luna",
        "--disable-base-tools", "--tools", "bash,write,todo", "api", "--stdio",
    ]
    environment = {key: value for key, value in os.environ.items()
                   if key in ("HOME", "PATH", "USER", "LOGNAME", "LANG", "LC_ALL", "TZ")}
    environment.update({"JCODE_HOME": "/state", "JCODE_RUNTIME_DIR": "/runtime",
                        config["api_key_env"]: credential, "JCODE_MEMORY_SIDECAR_ENABLED": "false",
                        "GIT_OPTIONAL_LOCKS": "0"})
    # The model's only executable tool gets a separate, networkless namespace.
    # The provider process sees only frozen home guidance, run-owned state and fixtures.
    shell_environment = {key: value for key, value in environment.items()
                         if key in ("HOME", "USER", "LOGNAME", "LANG", "LC_ALL", "TZ")}
    shell_environment["PATH"] = "/usr/bin:/bin"
    for field, suffix in (("user.name", "NAME"), ("user.email", "EMAIL")):
        value = git(ROOT, "config", "--get", field)
        shell_environment["GIT_AUTHOR_" + suffix] = value
        shell_environment["GIT_COMMITTER_" + suffix] = value
    shell_environment.update({"GIT_CONFIG_COUNT": "2", "GIT_CONFIG_KEY_0": "core.hooksPath",
                              "GIT_CONFIG_VALUE_0": "/dev/null", "GIT_CONFIG_KEY_1": "commit.gpgsign",
                              "GIT_CONFIG_VALUE_1": "false", "PYTHONDONTWRITEBYTECODE": "1",
                              "GIT_OPTIONAL_LOCKS": "0"})
    request_id = 0
    active = None
    trace = (output / "api-trace.ndjson").open("a")
    errors = (output / "runtime.stderr").open("a")

    def launch():
        private_command = command[:]
        socket_directory = tempfile.mkdtemp(prefix="process-", dir=sockets)
        private_command[private_command.index(str(sockets))] = socket_directory
        return subprocess.Popen(private_command, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                stderr=errors, text=True, env=environment), socket_directory

    process, socket_directory = launch()

    def send(payload):
        nonlocal request_id
        request_id += 1
        frame = {"v": 1, "id": request_id, **payload}
        trace.write(json.dumps({"direction": "request", "frame": frame}) + "\n")
        trace.flush()
        process.stdin.write(json.dumps(frame) + "\n")
        process.stdin.flush()
        return request_id

    def receive():
        line = process.stdout.readline()
        if not line:
            raise RuntimeError("Private Jcode API disconnected; preserve attempted slot without retry")
        event = json.loads(line)
        trace.write(json.dumps({"direction": "event", "frame": event}) + "\n")
        trace.flush()
        if active is not None:
            active["events"].append(event)
        return event

    def request(payload):
        identifier = send(payload)
        while True:
            event = receive()
            if event.get("ev") == "error":
                raise RuntimeError(str(event))
            if event.get("reply_to") == identifier:
                return event

    def stop_process():
        process.terminate()
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()
        process.stdin.close()
        process.stdout.close()

    def serving_processes():
        descendants = {process.pid}
        parents = {}
        for path in Path("/proc").glob("[0-9]*/status"):
            try:
                parents[int(path.parent.name)] = int(next(
                    line.split()[1] for line in path.read_text().splitlines() if line.startswith("PPid:")))
            except (OSError, StopIteration):
                continue
        for _ in range(len(parents)):
            children = {pid for pid, parent in parents.items() if parent in descendants}
            if children <= descendants:
                break
            descendants |= children
        observed = []
        for pid in sorted(descendants):
            path = Path(f"/proc/{pid}")
            try:
                if path.joinpath("exe").resolve() == Path(executable):
                    observed.append({"pid": pid, "executable": str(path.joinpath("exe").resolve()),
                                     "sha256": digest(path / "exe"),
                                     "command": (path / "cmdline").read_bytes().decode().split("\0")[:-1]})
            except OSError:
                continue
        if not any("serve" in item["command"] and item["sha256"] == binary["sha256"]
                   for item in observed):
            raise ValueError(f"Actual private serving process not verified: {observed}")
        return observed

    def execute_shell(inputs, workspace, tool_name="bash"):
        if tool_name == "write":
            writer = (
                "import json,sys; from pathlib import Path; "
                "value=json.loads(sys.argv[1]); path=Path(value['file_path']).expanduser(); "
                "path.parent.mkdir(parents=True,exist_ok=True); "
                "path.write_text(value['content']); print('Wrote '+str(path))"
            )
            inputs = {"command": "python3 -B -c " + shlex.quote(writer) + " " + shlex.quote(json.dumps(inputs))}
        if inputs.get("run_in_background"):
            return 1, "", "Background execution is unavailable in this isolated pilot; use foreground execution."
        sandbox = mount[:]
        sandbox.insert(1, "--unshare-net")
        sandbox += policy_mounts + ["--bind", str(workspace), str(workspace),
                                   "--ro-bind", str(workspace / ".jcode/system-prompt.md"),
                                   str(workspace / ".jcode/system-prompt.md"),
                                   "--chdir", str(workspace), "--", "/bin/bash", "--noprofile", "--norc", "-c",
                                   inputs["command"]]
        timeout = inputs.get("timeout")
        try:
            completed = subprocess.run(sandbox, env=shell_environment, capture_output=True, text=True,
                                       timeout=timeout / 1000 if timeout else None)
            return completed.returncode, completed.stdout, completed.stderr
        except subprocess.TimeoutExpired as error:
            return 124, (error.stdout or b"").decode(), (error.stderr or b"").decode() + "\nCommand timed out."

    try:
        hello = request({"req": "hello", "min_version": 1, "max_version": 1,
                         "client": "personal-prompt-pilot/1"})
        if not {"session_tools", "runtime_commands", "runtime_info"} <= set(hello.get("capabilities", [])):
            raise ValueError("Private runtime lacks required inspection/session-tool capabilities")
        baseline_suffix = None
        baseline_tools = None
        preflights = []
        used_process = False
        seen_sessions = set()
        if preflight_only:
            probe = output / "callback-probe"
            if not probe.exists():
                shutil.copytree(output / "workspaces/slot-001", probe)
            probe_command = (
                "python3 -B - <<'PY'\nimport os,socket\nfrom pathlib import Path\n"
                f"assert not Path({str(output / 'oracles')!r}).exists()\n"
                f"assert not Path({str(output / 'snapshots')!r}).exists()\n"
                f"assert not Path({str(output / 'workspaces/slot-002')!r}).exists()\n"
                f"assert {config['api_key_env']!r} not in os.environ\n"
                "assert Path(os.path.expanduser('~/.agents/AGENTS.md')).is_file()\n"
                "assert socket.socket().connect_ex(('127.0.0.1',9)) != 0\n"
                "try:\n Path('.jcode/system-prompt.md').write_text('changed')\n"
                "except OSError:\n pass\nelse:\n raise AssertionError('base writable')\n"
                "Path('probe.txt').write_text('allowed fixture write')\nprint('PASS: confined callback')\nPY"
            )
            probe_result = execute_shell({"command": probe_command}, probe)
            if probe_result[0] != 0:
                raise ValueError(f"Callback confinement failed: {probe_result}")
            write_result = execute_shell({"file_path": "retry.py", "content": FIXED}, probe, "write")
            check_result = execute_shell({"command": "python3 -B -m unittest -q"}, probe)
            denied_write = execute_shell({"file_path": ".jcode/system-prompt.md", "content": "changed"}, probe, "write")
            if write_result[0] or check_result[0] or not denied_write[0]:
                raise ValueError("Custom write callback did not honor workspace/base boundaries")
            (output / "callback-preflight.json").write_text(json.dumps({
                "bash": probe_result, "write": write_result, "fixed_check": check_result,
                "base_write_denied": denied_write,
            }, indent=2) + "\n")
        for slot in manifest["schedule"]:
            result_file = results / (slot["slot"] + ".json")
            if result_file.exists() and not preflight_only:
                continue  # An attempted slot is never silently replaced.
            workspace = output / "workspaces" / slot["slot"]
            if preflight_only:
                workspace = output / "preflight-workspaces" / slot["slot"]
                if not workspace.exists():
                    shutil.copytree(output / "templates" / slot["case"], workspace)
                    (workspace / ".jcode").mkdir()
                    shutil.copyfile(snapshots / f"bases/{slot['condition']}.md", workspace / ".jcode/system-prompt.md")
            if inventory(workspace) != slot["initial_files"]:
                raise ValueError(f"Unattempted fixture drift: {slot['slot']}")
            if used_process:
                # This pinned API reuses a connection's session on create_session.
                # A new owned process namespace guarantees a fresh daemon/connection.
                stop_process()
                process, socket_directory = launch()
                request({"req": "hello", "min_version": 1, "max_version": 1,
                         "client": "personal-prompt-pilot/1"})
            used_process = True
            attached = request({"req": "create_session", "working_dir": str(workspace)})
            session_id = attached["session"]["session_id"]
            if session_id in seen_sessions:
                raise ValueError("Private runtime reused a trial session")
            seen_sessions.add(session_id)
            request({"req": "set_reasoning_effort", "session_id": session_id, "effort": "medium"})
            identity = request({"req": "get_runtime_info", "session_id": session_id})
            if (identity.get("model") != "gpt-6-luna" or identity.get("provider") != "omni"
                    or identity.get("reasoning_effort") != "medium"):
                raise ValueError(f"Wrong serving route/settings: {identity}")
            serving = serving_processes()
            native_tools = request({"req": "list_tools", "session_id": session_id})["tools"]
            custom = [next(tool for tool in native_tools if tool["name"] == name) for name in ("bash", "write")]
            request({"req": "configure_tools", "session_id": session_id,
                     "tools": {"enabled": ["bash", "write", "todo"], "custom": custom}})
            effective = request({"req": "list_tools", "session_id": session_id})["tools"]
            if {tool["name"] for tool in effective} != {"bash", "write", "todo"}:
                raise ValueError("Unexpected effective tool surface")
            if baseline_tools is not None and effective != baseline_tools:
                raise ValueError("Tool definitions differ between conditions")
            baseline_tools = effective
            context = request({"req": "runtime_command", "session_id": session_id,
                               "command": "context", "args": []})
            data = context["data"]
            if data["messages"] or data["system_prompt"]["dynamic"] or data["system_prompt"]["pending_memory_included"]:
                raise ValueError("Fresh session has unexpected messages/dynamic memory")
            system = data["system_prompt"]["static"]
            base = (snapshots / f"bases/{slot['condition']}.md").read_text().strip()
            if not system.startswith(base):
                raise ValueError("Runtime did not load the frozen project-local base")
            suffix = system[len(base):]
            if baseline_suffix is not None and suffix != baseline_suffix:
                raise ValueError("Surrounding static context differs between conditions")
            baseline_suffix = suffix
            if not preflight_only:
                checked = checked_slots[slot["slot"]]
                if (checked["context"]["data"]["system_prompt"] != data["system_prompt"]
                        or checked["tools"] != effective):
                    raise ValueError("Context/tools drifted since all-slot preflight")
            if preflight_only:
                preflights.append({"slot": slot["slot"], "identity": identity, "context": context,
                                   "tools": effective, "serving_processes": serving,
                                   "socket_directory": socket_directory})
                request({"req": "detach_session", "session_id": session_id})
                continue
            active = {"slot": slot["slot"], "case": slot["case"], "session_id": session_id,
                      "identity": identity, "context": context, "binary": binary, "serving_processes": serving,
                      "socket_directory": socket_directory,
                      "effective_tools": effective,
                      "events": [], "tool_calls": [], "started_at": datetime.now(timezone.utc).isoformat()}
            result_file.write_text(json.dumps(active, indent=2) + "\n")
            started = time.monotonic()
            send({"req": "send_message", "session_id": session_id,
                  "content": (workspace / "TASK.md").read_text()})
            while True:
                event = receive()
                kind = event.get("ev")
                if kind == "tool_call":
                    if event.get("name") not in ("bash", "write"):
                        raise ValueError("Unexpected custom tool callback")
                    inputs = event["input"]
                    if event["name"] == "bash" and not isinstance(inputs.get("command"), str):
                        raise ValueError("Bash callback lacks a command string")
                    # Preserve the complete call even when the namespace rejects its effects.
                    call = {"name": event["name"], "input": inputs, "started_at": datetime.now(timezone.utc).isoformat()}
                    active["tool_calls"].append(call)
                    exit_code, stdout, stderr = execute_shell(inputs, workspace, event["name"])
                    call.update({"exit_code": exit_code, "stdout": stdout, "stderr": stderr})
                    send({"req": "tool_result", "session_id": session_id, "call_id": event["call_id"],
                          "output": stdout + stderr + (f"\nExit code: {exit_code}" if exit_code else ""),
                          "error": None})
                if kind == "error":
                    active["runtime_error"] = event
                    break
                if kind == "turn_done":
                    active["turn_done"] = event
                    break
            active["elapsed_seconds"] = round(time.monotonic() - started, 3)
            active["final_files"] = inventory(workspace)
            status_code, status_out, status_err = execute_shell(
                {"command": "git -c core.fsmonitor=false status --short", "timeout": 10000}, workspace)
            active["final_git_status"] = {"exit_code": status_code, "stdout": status_out, "stderr": status_err}
            active["finished_at"] = datetime.now(timezone.utc).isoformat()
            result_file.write_text(json.dumps(active, indent=2) + "\n")
            print(f"{slot['slot']}: captured {len(active['tool_calls'])} custom tool calls", flush=True)
            active = None
            request({"req": "detach_session", "session_id": session_id})
        if preflight_only:
            (output / "runtime-preflight.json").write_text(json.dumps({
                "binary": binary, "slots": preflights, "hello": hello, "model_trials_started": 0,
                "usage_limits": "none, explicitly authorized by user",
            }, indent=2) + "\n")
            print(f"PASS: {len(preflights)} fresh-session contexts; omni/gpt-6-luna/medium; bash,write,todo; no inference")
    finally:
        if active is not None:
            active["interrupted"] = True
            active["final_files"] = inventory(output / "workspaces" / active["slot"])
            (results / (active["slot"] + ".json")).write_text(json.dumps(active, indent=2) + "\n")
        stop_process()
        trace.close()
        errors.close()


if __name__ == "__main__":
    os.umask(0o077)
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path, help="New, private run-owned directory")
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--verify", action="store_true", help="Check an untouched prepared run for drift")
    mode.add_argument("--preflight", action="store_true", help="Inspect a private Jcode runtime without inference")
    mode.add_argument("--run", action="store_true", help="Run the authorized 36-slot OmniRoute gpt-6-luna pilot")
    args = parser.parse_args()
    try:
        if args.preflight or args.run:
            run_pilot(args.output.resolve(), preflight_only=args.preflight)
        else:
            (verify if args.verify else prepare)(args.output.resolve())
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        parser.exit(1, f"{error}\n")
