#!/usr/bin/env python3
"""Run trigger evaluation for a skill description.

Tests whether a skill's description causes Claude to trigger (read the skill)
for a set of queries. Outputs results as JSON.
"""

import argparse
import json
import os
import select
import subprocess
import sys
import time
import uuid
from concurrent.futures import ProcessPoolExecutor, as_completed
from pathlib import Path

from scripts.utils import parse_skill_md


def find_project_root() -> Path:
    """Find the project root by walking up from cwd looking for .claude/.

    Mimics how Claude Code discovers its project root, so the command file
    we create ends up where claude -p will look for it.
    """
    current = Path.cwd()
    for parent in [current, *current.parents]:
        if (parent / ".claude").is_dir():
            return parent
    return current


def _names_match(haystack: str, clean_name: str, skill_name: str) -> bool:
    """True if the tool input names the probe copy OR the real skill.

    When the skill under test is already installed in the project -- the normal
    case when optimizing an existing skill -- Claude invokes the REAL skill, not
    the uniquely-named probe copy. Matching only the probe name scored those
    correct triggers as misses, which is why every description scored identically
    on 2026-07-28: the model kept picking the installed skill regardless of what
    the candidate description said. See _warn_if_shadowed for the attribution
    caveat this creates.
    """
    return clean_name in haystack or f'"{skill_name}"' in haystack


def run_single_query(
    query: str,
    skill_name: str,
    skill_description: str,
    timeout: int,
    project_root: str,
    model: str | None = None,
) -> tuple[bool, bool]:
    """Run a single query; return (triggered, timed_out).

    A timeout is reported separately because recording it as "did not trigger"
    makes an unmeasured run indistinguishable from a real negative -- the defect
    that made every description score identically on 2026-07-28.

    Creates a command file in .claude/commands/ so it appears in Claude's
    available_skills list, then runs `claude -p` with the raw query.
    Uses --include-partial-messages to detect triggering early from
    stream events (content_block_start) rather than waiting for the
    full assistant message, which only arrives after tool execution.
    """
    unique_id = uuid.uuid4().hex[:8]
    clean_name = f"{skill_name}-skill-{unique_id}"
    project_commands_dir = Path(project_root) / ".claude" / "commands"
    command_file = project_commands_dir / f"{clean_name}.md"

    try:
        project_commands_dir.mkdir(parents=True, exist_ok=True)
        # Use YAML block scalar to avoid breaking on quotes in description
        indented_desc = "\n  ".join(skill_description.split("\n"))
        command_content = (
            f"---\n"
            f"description: |\n"
            f"  {indented_desc}\n"
            f"---\n\n"
            f"# {skill_name}\n\n"
            f"This skill handles: {skill_description}\n"
        )
        command_file.write_text(command_content)

        cmd = [
            "claude",
            "-p", query,
            "--output-format", "stream-json",
            "--verbose",
            "--include-partial-messages",
        ]
        if model:
            cmd.extend(["--model", model])

        # Remove CLAUDECODE env var to allow nesting claude -p inside a
        # Claude Code session. The guard is for interactive terminal conflicts;
        # programmatic subprocess usage is safe.
        env = {k: v for k, v in os.environ.items() if k != "CLAUDECODE"}

        process = subprocess.Popen(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            cwd=project_root,
            env=env,
        )

        triggered = False
        start_time = time.time()
        buffer = ""
        # Track state for stream event detection
        pending_tool_name = None
        accumulated_json = ""

        timed_out = True  # cleared on any conclusive stream outcome below
        try:
            while time.time() - start_time < timeout:
                if process.poll() is not None:
                    remaining = process.stdout.read()
                    if remaining:
                        buffer += remaining.decode("utf-8", errors="replace")
                    timed_out = False  # the run finished; absence of a Skill call is a real negative
                    break

                ready, _, _ = select.select([process.stdout], [], [], 1.0)
                if not ready:
                    continue

                chunk = os.read(process.stdout.fileno(), 8192)
                if not chunk:
                    timed_out = False  # EOF on the stream, not a budget expiry
                    break
                buffer += chunk.decode("utf-8", errors="replace")

                while "\n" in buffer:
                    line, buffer = buffer.split("\n", 1)
                    line = line.strip()
                    if not line:
                        continue

                    try:
                        event = json.loads(line)
                    except json.JSONDecodeError:
                        continue

                    # Early detection via stream events
                    if event.get("type") == "stream_event":
                        se = event.get("event", {})
                        se_type = se.get("type", "")

                        if se_type == "content_block_start":
                            cb = se.get("content_block", {})
                            if cb.get("type") == "tool_use":
                                tool_name = cb.get("name", "")
                                if tool_name in ("Skill", "Read"):
                                    pending_tool_name = tool_name
                                    accumulated_json = ""
                                else:
                                    return False, False

                        elif se_type == "content_block_delta" and pending_tool_name:
                            delta = se.get("delta", {})
                            if delta.get("type") == "input_json_delta":
                                accumulated_json += delta.get("partial_json", "")
                                if _names_match(accumulated_json, clean_name, skill_name):
                                    return True, False

                        elif se_type in ("content_block_stop", "message_stop"):
                            if pending_tool_name:
                                return _names_match(accumulated_json, clean_name, skill_name), False
                            if se_type == "message_stop":
                                return False, False

                    # Fallback: full assistant message
                    elif event.get("type") == "assistant":
                        message = event.get("message", {})
                        for content_item in message.get("content", []):
                            if content_item.get("type") != "tool_use":
                                continue
                            tool_name = content_item.get("name", "")
                            tool_input = content_item.get("input", {})
                            if tool_name == "Skill" and _names_match(
                                str(tool_input.get("skill", "")), clean_name, skill_name
                            ):
                                triggered = True
                            elif tool_name == "Read" and _names_match(
                                str(tool_input.get("file_path", "")), clean_name, skill_name
                            ):
                                triggered = True
                            return triggered, False

                    elif event.get("type") == "result":
                        return triggered, False
        finally:
            # Clean up process on any exit path (return, exception, timeout)
            if process.poll() is None:
                process.kill()
                process.wait()

        # Fell out of the while-loop: the budget expired before any conclusive
        # event. That is an ABSENT measurement, not a negative one.
        return triggered, timed_out
    finally:
        if command_file.exists():
            command_file.unlink()


def _warn_if_shadowed(skill_name: str, project_root: Path) -> bool:
    """Warn when the real skill is installed alongside the probe.

    The probe injects a uniquely-named copy carrying the CANDIDATE description,
    but an installed skill of the same name carries its OWN description and
    competes for the same queries. When both are present a trigger cannot be
    attributed to the candidate, so a comparison between descriptions may be
    measuring nothing. Callers get the truth rather than a confident number.
    """
    hits = [
        d for d in (
            project_root / "skills" / skill_name,
            project_root / ".claude" / "skills" / skill_name,
            Path.home() / ".claude" / "skills" / skill_name,
        )
        if d.is_dir()
    ]
    if hits:
        print(
            f"WARNING: skill '{skill_name}' is already installed at {hits[0]}. "
            f"Claude may invoke the INSTALLED skill (with its own description) "
            f"instead of the probe copy, so trigger counts cannot be cleanly "
            f"attributed to the candidate description. Compare descriptions in a "
            f"project where this skill is NOT installed for a clean signal.",
            file=sys.stderr,
        )
    return bool(hits)


def run_eval(
    eval_set: list[dict],
    skill_name: str,
    description: str,
    num_workers: int,
    timeout: int,
    project_root: Path,
    runs_per_query: int = 1,
    trigger_threshold: float = 0.5,
    model: str | None = None,
) -> dict:
    """Run the full eval set and return results."""
    results = []
    shadowed = _warn_if_shadowed(skill_name, Path(project_root))

    with ProcessPoolExecutor(max_workers=num_workers) as executor:
        future_to_info = {}
        for item in eval_set:
            for run_idx in range(runs_per_query):
                future = executor.submit(
                    run_single_query,
                    item["query"],
                    skill_name,
                    description,
                    timeout,
                    str(project_root),
                    model,
                )
                future_to_info[future] = (item, run_idx)

        query_triggers: dict[str, list[bool]] = {}
        query_timeouts: dict[str, int] = {}
        query_items: dict[str, dict] = {}
        for future in as_completed(future_to_info):
            item, _ = future_to_info[future]
            query = item["query"]
            query_items[query] = item
            query_triggers.setdefault(query, [])
            query_timeouts.setdefault(query, 0)
            try:
                triggered, timed_out = future.result()
                query_triggers[query].append(triggered)
                if timed_out:
                    query_timeouts[query] += 1
            except Exception as e:
                print(f"Warning: query failed: {e}", file=sys.stderr)
                query_triggers[query].append(False)
                query_timeouts[query] += 1

    for query, triggers in query_triggers.items():
        item = query_items[query]
        trigger_rate = sum(triggers) / len(triggers)
        should_trigger = item["should_trigger"]
        if should_trigger:
            did_pass = trigger_rate >= trigger_threshold
        else:
            did_pass = trigger_rate < trigger_threshold
        results.append({
            "query": query,
            "should_trigger": should_trigger,
            "trigger_rate": trigger_rate,
            "triggers": sum(triggers),
            "runs": len(triggers),
            "timeouts": query_timeouts[query],
            "pass": did_pass,
        })

    passed = sum(1 for r in results if r["pass"])
    total = len(results)
    total_runs = sum(r["runs"] for r in results)
    total_timeouts = sum(r["timeouts"] for r in results)

    # A run that timed out measured nothing. When most runs time out, every
    # description scores the same and the comparison is meaningless -- say so
    # loudly rather than letting a confident-looking pass rate stand in for it.
    if total_timeouts:
        pct = 100 * total_timeouts / total_runs if total_runs else 0
        print(
            f"WARNING: {total_timeouts}/{total_runs} runs ({pct:.0f}%) hit the "
            f"{timeout}s budget before any conclusive event. Those runs measured "
            f"nothing and were counted as non-triggers; scores below are "
            f"unreliable. Raise --timeout.",
            file=sys.stderr,
        )

    return {
        "skill_name": skill_name,
        "description": description,
        "results": results,
        "summary": {
            "total": total,
            "passed": passed,
            "failed": total - passed,
            "runs": total_runs,
            "timeouts": total_timeouts,
            "timeout_rate": (total_timeouts / total_runs) if total_runs else 0.0,
            "shadowed_by_installed_skill": shadowed,
        },
    }


def main():
    parser = argparse.ArgumentParser(description="Run trigger evaluation for a skill description")
    parser.add_argument("--eval-set", required=True, help="Path to eval set JSON file")
    parser.add_argument("--skill-path", required=True, help="Path to skill directory")
    parser.add_argument("--description", default=None, help="Override description to test")
    parser.add_argument("--num-workers", type=int, default=10, help="Number of parallel workers")
    parser.add_argument("--timeout", type=int, default=180,
                        help="Timeout per query in seconds. A real `claude -p` in a repo with a "
                             "large config stack routinely needs >60s before its first tool call; "
                             "the old 30s default expired first and scored every query as a "
                             "non-trigger.")
    parser.add_argument("--runs-per-query", type=int, default=3, help="Number of runs per query")
    parser.add_argument("--trigger-threshold", type=float, default=0.5, help="Trigger rate threshold")
    parser.add_argument("--model", default=None, help="Model to use for claude -p (default: user's configured model)")
    parser.add_argument("--verbose", action="store_true", help="Print progress to stderr")
    args = parser.parse_args()

    eval_set = json.loads(Path(args.eval_set).read_text())
    skill_path = Path(args.skill_path)

    if not (skill_path / "SKILL.md").exists():
        print(f"Error: No SKILL.md found at {skill_path}", file=sys.stderr)
        sys.exit(1)

    name, original_description, content = parse_skill_md(skill_path)
    description = args.description or original_description
    project_root = find_project_root()

    if args.verbose:
        print(f"Evaluating: {description}", file=sys.stderr)

    output = run_eval(
        eval_set=eval_set,
        skill_name=name,
        description=description,
        num_workers=args.num_workers,
        timeout=args.timeout,
        project_root=project_root,
        runs_per_query=args.runs_per_query,
        trigger_threshold=args.trigger_threshold,
        model=args.model,
    )

    if args.verbose:
        summary = output["summary"]
        print(f"Results: {summary['passed']}/{summary['total']} passed", file=sys.stderr)
        for r in output["results"]:
            status = "PASS" if r["pass"] else "FAIL"
            rate_str = f"{r['triggers']}/{r['runs']}"
            print(f"  [{status}] rate={rate_str} expected={r['should_trigger']}: {r['query'][:70]}", file=sys.stderr)

    print(json.dumps(output, indent=2))


if __name__ == "__main__":
    main()
