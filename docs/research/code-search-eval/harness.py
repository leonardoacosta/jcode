#!/usr/bin/env python3
"""Local code-location smoke test, not a matched-model search-agent benchmark."""
import argparse
import datetime
import hashlib
import json
import pathlib
import re
import shutil
import statistics
import subprocess
import sys
import time

HERE = pathlib.Path(__file__).resolve().parent
LOCAL = ("agentgrep", "graft", "ast-grep", "repo-map")
STOP = {"who", "calls", "callers", "of", "the", "struct", "definition", "type", "tool", "with", "and"}
DEF = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?(?:fn|struct|enum|trait)\s+(\w+)", re.M)
SOURCE_GLOB = "{crates,src}/**/*.rs"


def run(cmd, root, timeout=30):
    start = time.monotonic()
    record = {"argv": cmd, "cwd": str(root)}
    try:
        p = subprocess.run(cmd, cwd=root, capture_output=True, text=True, timeout=timeout)
        record.update(returncode=p.returncode, stdout=p.stdout, stderr=p.stderr)
    except (OSError, subprocess.TimeoutExpired) as exc:
        record.update(returncode=None, stdout="", stderr="", error=str(exc))
        if isinstance(exc, subprocess.TimeoutExpired):
            record.update(stdout=(exc.stdout or b"").decode(errors="replace"),
                          stderr=(exc.stderr or b"").decode(errors="replace"))
    record["wall_ms"] = round((time.monotonic() - start) * 1000, 3)
    return record


def source_path(value):
    """Accept only a source path, never a path mentioned in a snippet or title."""
    if not isinstance(value, str):
        raise ValueError("Expected a string source path")
    value = value.removeprefix("./")
    return value if ".." not in pathlib.PurePosixPath(value).parts and re.fullmatch(r"(?:crates|src)/[\w./-]+\.rs", value) else None


def ranked_paths(strategy, output):
    data = json.loads(output)
    if strategy == "agentgrep":
        values = [entry["path"] for entry in data["files"]]
    elif strategy == "graft":
        values = [entry["pointer"] for entry in data["hits"]]
    else:
        values = [entry["file"] for entry in data]
    if any(not isinstance(value, str) for value in values):
        raise ValueError("Expected string paths in result records")
    if strategy == "graft":
        values = [value.split(":", 1)[0] for value in values]
    return list(dict.fromkeys(p for value in values if (p := source_path(value))))


def score(ranked, answers):
    answers = set(answers)
    return {f"recall_at_{k}": len(answers.intersection(ranked[:k])) / len(answers) for k in (1, 5)}


def repomap(q, symbols):
    # Deliberately retain the original regex-map approximation, not Empryo itself.
    toks = [w.lower() for w in re.findall(r"[A-Za-z]+", q["q"]) if w.lower() not in STOP]
    split = lambda s: [w.lower() for w in re.findall(r"[A-Z]?[a-z]+|[A-Z]+(?![a-z])", s)]
    scores = []
    for f, syms in symbols.items():
        pen = 0.5 if "/tests" in f or f.endswith("_tests.rs") else 1.0
        pt = set(split(f.replace("/", " ").replace("_", " ")))
        st = {w for s in syms for w in split(s.replace("_", " "))}
        sc = (sum(2 for w in toks if w in pt) + sum(w in st for w in toks)
              + (10 if q.get("sym") in syms else 0)) * pen
        if sc:
            scores.append((sc, f))
    scores.sort(key=lambda x: (-x[0], x[1]))
    return [f for _, f in scores[:10]]


def evaluate_query(strategy, q, root, binary=None, symbols=None, timeout=30):
    row = {"strategy": strategy, "query": q["id"], "kind": q["kind"], "answers": q["answers"], "commands": []}
    if strategy == "search-subagent":
        return dict(row, status="not_run", reason="No fixed-model agent trials performed by this local harness")
    if strategy == "ast-grep" and not q.get("sg"):
        return dict(row, status="unsupported", reason="No curated structural pattern")
    if strategy != "repo-map" and not binary:
        return dict(row, status="not_run", reason="Executable unavailable")
    start = time.monotonic()
    if strategy == "agentgrep":
        mode = "grep" if q.get("sym") else "find"
        query = q["sym"] + ("(" if q["kind"] == "caller" else "") if q.get("sym") else q["q"]
        commands = [[binary, mode, "--json", "--glob", SOURCE_GLOB, query]]
    elif strategy == "graft":
        commands = [[binary, "ask", "--no-refresh", "--json", "-n", "8", q["q"]]]
    elif strategy == "ast-grep":
        commands = [[binary, "run", "--lang", "rust", "-p", pat, "--json=compact", "crates", "src"] for pat in q["sg"]]
    else:
        commands = []
    ranked = []
    try:
        for cmd in commands:
            record = run(cmd, root, timeout)
            row["commands"].append(record)
            if record["returncode"] != 0:
                raise ValueError("Command failed or timed out; see commands")
            ranked.extend(ranked_paths(strategy, record["stdout"]))
        if strategy == "repo-map":
            ranked = repomap(q, symbols or {})
        ranked = list(dict.fromkeys(ranked))
        if any(not (root / p).is_file() for p in ranked):
            raise ValueError("Returned path absent from current source tree (possibly stale index)")
        row.update(status="ok", ranked_paths=ranked, **score(ranked, q["answers"]))
    except (ValueError, KeyError, TypeError, OSError) as exc:
        row.update(status="error", reason=str(exc))
    row.update(wall_ms=round((time.monotonic() - start) * 1000, 3), cli_calls=len(row["commands"]),
               output_tokens_estimate=sum(len(c["stdout"]) + len(c["stderr"]) for c in row["commands"]) / 4)
    if strategy == "repo-map" and row["status"] == "ok":
        row["output_tokens_estimate"] = len("\n".join(ranked)) / 4
        row["source_files_read"] = 0  # The one-time map build is recorded separately.
    return row


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate_ground_truth(qs, root, sources):
    for q in qs:
        if not q["answers"] or any(a not in sources for a in q["answers"]):
            raise ValueError(f"Invalid ground-truth path: {q['id']}")
        anchors = q.get("answer_symbols", [q["sym"]] if q.get("sym") else [])
        if not anchors:
            raise ValueError(f"Missing ground-truth symbols: {q['id']}")
        for answer in q["answers"]:
            text = (root / answer).read_text()
            calls = "\n".join(line for line in text.splitlines() if not line.lstrip().startswith("//")
                              and not re.search(r"\bfn\s+", line))
            present = (all(re.search(r"\b" + re.escape(s) + r"\s*\(", calls) for s in anchors)
                       if q.get("kind") == "caller" else set(anchors).issubset(DEF.findall(text)))
            if not present:
                raise ValueError(f"Invalid ground-truth symbols: {q['id']} in {answer}")


def provenance(root):
    git = lambda *args: subprocess.check_output(["git", *args], cwd=root, text=True).strip()
    files = sorted(set(git("ls-files", "--cached", "--others", "--exclude-standard", "*.rs").splitlines()))
    files = [f for f in files if source_path(f) and (root / f).is_file()]
    graph = {str(p.relative_to(root)): digest(p) for p in sorted((root / "graft").rglob("*")) if p.is_file()}
    status = git("status", "--porcelain=v1", "--untracked-files=all")
    return {"root": str(root), "commit": git("rev-parse", "HEAD"), "dirty": bool(status), "git_status": status,
            "source_sha256": {f: digest(root / f) for f in files},
            "graft_graph_sha256": hashlib.sha256(json.dumps(graph, sort_keys=True).encode()).hexdigest(),
            "graft_graph_files": len(graph)}


def summarize(rows):
    successful = {s: {r["query"] for r in rows if r["strategy"] == s and r["status"] == "ok"} for s in LOCAL}
    matched = sorted(set.intersection(*successful.values()))
    result = {"matched_local_query_ids": matched, "per_strategy": {}}
    for s in LOCAL:
        own = [r for r in rows if r["strategy"] == s and r["status"] == "ok"]
        shared = [r for r in own if r["query"] in matched]
        metrics = lambda rs: {"n": len(rs), **{f"recall_at_{k}": statistics.mean(r[f"recall_at_{k}"] for r in rs) if rs else None for k in (1, 5)}}
        result["per_strategy"][s] = {"available_queries": metrics(own), "matched_local_queries": metrics(shared)}
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=pathlib.Path, default=HERE.parents[2])
    for name in LOCAL[:-1]:
        parser.add_argument("--" + name, default=shutil.which(name), help="Executable path (otherwise PATH)")
    parser.add_argument("--timeout", type=float, default=30, help="Seconds per CLI invocation")
    parser.add_argument("--output", type=pathlib.Path, help="New JSON file, never overwrite an existing result")
    args = parser.parse_args()
    if args.timeout <= 0:
        parser.error("--timeout must be positive")
    root = args.root.resolve()
    # Reserve the output before running any trials. Exclusive creation protects measurements.
    output = args.output.open("x") if args.output else sys.stdout
    try:
        before = provenance(root)
        harness_hash = digest(HERE / "harness.py")
        queries_bytes = (HERE / "queries.json").read_bytes()
        queries_hash = hashlib.sha256(queries_bytes).hexdigest()
        qs = json.loads(queries_bytes)
        if len({q["id"] for q in qs}) != len(qs):
            raise ValueError("Duplicate query ID")
        validate_ground_truth(qs, root, before["source_sha256"])
        tools = {}
        for s in LOCAL[:-1]:
            binary = getattr(args, s.replace("-", "_"))
            if binary:
                binary = str(pathlib.Path(shutil.which(binary) or binary).resolve())
            tools[s] = {"binary": binary, "version": run([binary, "--version"], root, args.timeout) if binary else None,
                        "entrypoint_sha256": digest(pathlib.Path(binary)) if binary and pathlib.Path(binary).is_file() else None}
        start = time.monotonic()
        symbols = {f: DEF.findall((root / f).read_text(errors="replace")) for f in before["source_sha256"]}
        map_build = {"wall_ms": round((time.monotonic() - start) * 1000, 3), "source_files_read": len(symbols)}
        rows = [evaluate_query(s, q, root, tools[s]["binary"] if s in tools else None,
                               symbols, args.timeout) for s in (*LOCAL, "search-subagent") for q in qs]
        after = provenance(root)
        stable = (all(before[k] == after[k] for k in ("commit", "source_sha256", "graft_graph_sha256"))
                  and harness_hash == digest(HERE / "harness.py") and queries_hash == digest(HERE / "queries.json"))
        failed = any(r["status"] == "error" for r in rows)
        data = {"schema_version": 2, "status": ("error" if failed else "partial") if stable else "invalid_source_changed", "source_stable": stable,
                "repo_map_build": map_build,
                "recorded_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
                "before": before, "after": after, "tools": tools, "harness_sha256": harness_hash,
                "queries_sha256": queries_hash, "rows": rows, "summary": summarize(rows)}
        json.dump(data, output, indent=2)
        output.write("\n")
        return int(not stable or failed)
    finally:
        if args.output:
            output.close()


if __name__ == "__main__":
    sys.exit(main())
