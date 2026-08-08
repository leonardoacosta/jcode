#!/usr/bin/env python3
"""Task-decomposition eval fixture utilities.

This script is intentionally stdlib-only so catalog checks work in a fresh Rust
checkout without installing Python packages.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from collections import Counter
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_CATALOG = ROOT / "evals" / "task-decomposition" / "fixtures" / "catalog.json"
DEFAULT_PROMPTS = ROOT / "evals" / "task-decomposition" / "prompts" / "catalog.json"
REQUIRED_ARTIFACTS = ("proposal.md", "design.md", "tasks.md")
COMMIT_RE = re.compile(r"^[0-9a-f]{40}$")
ID_RE = re.compile(r"^[a-z0-9][a-z0-9-]*$")
TOKEN_RE = re.compile(r"[a-zA-Z0-9_/-]+")
BASELINE_MODES = (
    "openspec-gold",
    "jcode-no-openspec",
    "jcode-openspec",
    "jcode-openspec-orchestrated",
)
PROMPT_KINDS = ("original", "reconstructed")
PROMPT_CONFIDENCE = ("high", "medium", "low")
RUBRIC_DIMENSIONS = (
    "requirement_coverage",
    "decomposition_quality",
    "risk_handling",
    "scope_control",
    "executability",
)

EXPECTED_CATEGORIES = {
    "free design/product choices",
    "business/domain logic",
    "infra/platform/config",
    "test strategy/e2e remediation",
    "data/schema/migration",
    "auth/security/permissions",
    "observability/telemetry",
    "developer tooling/agent integration",
    "refactor/dead-code/entropy cleanup",
    "UI/UX polish",
}


class EvalError(RuntimeError):
    pass


def run(cmd: list[str], cwd: Path | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run(cmd, cwd=cwd, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)


def load_catalog(path: Path) -> dict[str, Any]:
    try:
        data = json.loads(path.read_text())
    except json.JSONDecodeError as exc:
        raise EvalError(f"invalid JSON in {path}: {exc}") from exc
    if not isinstance(data, dict):
        raise EvalError("catalog root must be an object")
    return data


def validate_catalog(path: Path = DEFAULT_CATALOG) -> dict[str, Any]:
    data = load_catalog(path)
    failures: list[str] = []
    fixtures = data.get("fixtures")
    if data.get("version") != 1:
        failures.append("version must be 1")
    if not isinstance(fixtures, list) or not fixtures:
        failures.append("fixtures must be a non-empty array")
        fixtures = []

    seen: set[str] = set()
    categories: Counter[str] = Counter()
    required_keys = {
        "id",
        "category",
        "project",
        "remote",
        "base_commit",
        "gold_proposal_commit",
        "change_slug",
        "expected_artifacts",
        "notes",
    }
    for index, fixture in enumerate(fixtures):
        prefix = f"fixtures[{index}]"
        if not isinstance(fixture, dict):
            failures.append(f"{prefix} must be an object")
            continue
        missing = sorted(required_keys - set(fixture))
        extra = sorted(set(fixture) - required_keys)
        if missing:
            failures.append(f"{prefix} missing keys: {', '.join(missing)}")
        if extra:
            failures.append(f"{prefix} has unknown keys: {', '.join(extra)}")
        fixture_id = fixture.get("id")
        if not isinstance(fixture_id, str) or not ID_RE.match(fixture_id):
            failures.append(f"{prefix}.id must be kebab-case")
        elif fixture_id in seen:
            failures.append(f"duplicate fixture id: {fixture_id}")
        else:
            seen.add(fixture_id)
        category = fixture.get("category")
        if not isinstance(category, str) or not category:
            failures.append(f"{prefix}.category must be a non-empty string")
        else:
            categories[category] += 1
        for key in ("project", "remote", "change_slug", "notes"):
            if not isinstance(fixture.get(key), str) or not fixture.get(key):
                failures.append(f"{prefix}.{key} must be a non-empty string")
        for key in ("base_commit", "gold_proposal_commit"):
            value = fixture.get(key)
            if not isinstance(value, str) or not COMMIT_RE.match(value):
                failures.append(f"{prefix}.{key} must be a 40-character lowercase hex commit")
        artifacts = fixture.get("expected_artifacts")
        if not isinstance(artifacts, list) or not all(isinstance(item, str) and item for item in artifacts):
            failures.append(f"{prefix}.expected_artifacts must be a non-empty string array")
        else:
            for required in REQUIRED_ARTIFACTS:
                if required not in artifacts:
                    failures.append(f"{prefix}.expected_artifacts must include {required}")
            if "specs/*/spec.md" not in artifacts:
                failures.append(f"{prefix}.expected_artifacts must include specs/*/spec.md")

    missing_categories = sorted(EXPECTED_CATEGORIES - set(categories))
    if missing_categories:
        failures.append("missing categories: " + ", ".join(missing_categories))

    result = {
        "catalog": str(path),
        "fixture_count": len(fixtures),
        "category_counts": dict(sorted(categories.items())),
        "failures": failures,
    }
    if failures:
        raise EvalError(json.dumps(result, indent=2))
    return result


def parse_repo_roots(values: list[str]) -> dict[str, Path]:
    roots: dict[str, Path] = {}
    for value in values:
        if "=" not in value:
            raise EvalError(f"--repo-root must be project=/path, got {value!r}")
        project, path = value.split("=", 1)
        if not project or not path:
            raise EvalError(f"--repo-root must be project=/path, got {value!r}")
        roots[project] = Path(path).expanduser().resolve()
    return roots


def find_fixture(catalog: dict[str, Any], fixture_id: str) -> dict[str, Any]:
    for fixture in catalog.get("fixtures", []):
        if fixture.get("id") == fixture_id:
            return fixture
    raise EvalError(f"unknown fixture id: {fixture_id}")


def load_prompt_catalog(path: Path = DEFAULT_PROMPTS) -> dict[str, Any]:
    data = load_catalog(path)
    if data.get("version") != 1:
        raise EvalError("prompt catalog version must be 1")
    prompts = data.get("prompts")
    if not isinstance(prompts, list):
        raise EvalError("prompt catalog prompts must be an array")
    return data


def validate_prompt_catalog(path: Path = DEFAULT_PROMPTS, catalog_path: Path = DEFAULT_CATALOG) -> dict[str, Any]:
    fixture_catalog = load_catalog(catalog_path)
    fixture_ids = {fixture.get("id") for fixture in fixture_catalog.get("fixtures", []) if isinstance(fixture, dict)}
    data = load_prompt_catalog(path)
    failures: list[str] = []
    prompts = data["prompts"]
    seen: set[str] = set()

    for index, prompt in enumerate(prompts):
        prefix = f"prompts[{index}]"
        if not isinstance(prompt, dict):
            failures.append(f"{prefix} must be an object")
            continue
        required = {"fixture_id", "kind", "confidence", "source", "prompt", "notes"}
        missing = sorted(required - set(prompt))
        extra = sorted(set(prompt) - required)
        if missing:
            failures.append(f"{prefix} missing keys: {', '.join(missing)}")
        if extra:
            failures.append(f"{prefix} has unknown keys: {', '.join(extra)}")
        fixture_id = prompt.get("fixture_id")
        if not isinstance(fixture_id, str) or not ID_RE.match(fixture_id):
            failures.append(f"{prefix}.fixture_id must be kebab-case")
        elif fixture_id not in fixture_ids:
            failures.append(f"{prefix}.fixture_id is not in fixture catalog: {fixture_id}")
        elif fixture_id in seen:
            failures.append(f"duplicate prompt fixture_id: {fixture_id}")
        else:
            seen.add(fixture_id)
        if prompt.get("kind") not in PROMPT_KINDS:
            failures.append(f"{prefix}.kind must be one of: {', '.join(PROMPT_KINDS)}")
        if prompt.get("confidence") not in PROMPT_CONFIDENCE:
            failures.append(f"{prefix}.confidence must be one of: {', '.join(PROMPT_CONFIDENCE)}")
        for key in ("source", "prompt", "notes"):
            if not isinstance(prompt.get(key), str) or not prompt.get(key):
                failures.append(f"{prefix}.{key} must be a non-empty string")

    result = {
        "catalog": str(path),
        "prompt_count": len(prompts),
        "fixture_ids": sorted(seen),
        "failures": failures,
    }
    if failures:
        raise EvalError(json.dumps(result, indent=2))
    return result


def find_prompt(prompt_catalog: dict[str, Any], fixture_id: str) -> dict[str, Any]:
    for prompt in prompt_catalog.get("prompts", []):
        if prompt.get("fixture_id") == fixture_id:
            return prompt
    raise EvalError(f"missing prompt metadata for fixture: {fixture_id}")


def require_repo_root(fixture: dict[str, Any], roots: dict[str, Path]) -> Path:
    project = fixture["project"]
    root = roots.get(project)
    if root is None:
        raise EvalError(f"missing --repo-root {project}=/path/to/repo")
    if not (root / ".git").exists():
        raise EvalError(f"repo root for {project} is not a git checkout: {root}")
    return root


def verify_commit(repo: Path, commit: str) -> None:
    result = run(["git", "rev-parse", "--verify", f"{commit}^{{commit}}"], cwd=repo)
    if result.returncode != 0:
        raise EvalError(f"commit {commit} is not available in {repo}: {result.stderr.strip()}")


def materialize(args: argparse.Namespace) -> dict[str, Any]:
    catalog = load_catalog(args.catalog)
    fixture = find_fixture(catalog, args.fixture)
    roots = parse_repo_roots(args.repo_root)
    repo = require_repo_root(fixture, roots)
    verify_commit(repo, fixture["base_commit"])
    output = args.output.expanduser().resolve()
    if output.exists():
        raise EvalError(f"output already exists, refusing to overwrite: {output}")
    output.parent.mkdir(parents=True, exist_ok=True)
    clone = run(["git", "clone", "--shared", str(repo), str(output)])
    if clone.returncode != 0:
        raise EvalError(f"git clone failed: {clone.stderr.strip()}")
    checkout = run(["git", "checkout", "--detach", fixture["base_commit"]], cwd=output)
    if checkout.returncode != 0:
        raise EvalError(f"git checkout failed: {checkout.stderr.strip()}")
    metadata = {
        "fixture_id": fixture["id"],
        "category": fixture["category"],
        "project": fixture["project"],
        "base_commit": fixture["base_commit"],
        "gold_proposal_commit": fixture["gold_proposal_commit"],
        "change_slug": fixture["change_slug"],
    }
    (output / ".jcode-eval-fixture.json").write_text(json.dumps(metadata, indent=2) + "\n")
    return {"fixture": fixture["id"], "output": str(output), "metadata": metadata}


def prepare_run(args: argparse.Namespace) -> dict[str, Any]:
    catalog = load_catalog(args.catalog)
    prompt_catalog = load_prompt_catalog(args.prompts)
    fixture = find_fixture(catalog, args.fixture)
    prompt = find_prompt(prompt_catalog, fixture["id"])
    roots = parse_repo_roots(args.repo_root)
    repo = require_repo_root(fixture, roots)
    verify_commit(repo, fixture["base_commit"])
    verify_commit(repo, fixture["gold_proposal_commit"])
    output = args.output.expanduser().resolve()
    if output.exists():
        raise EvalError(f"output already exists, refusing to prepare run: {output}")
    if args.baseline_mode not in BASELINE_MODES:
        raise EvalError(f"baseline mode must be one of: {', '.join(BASELINE_MODES)}")
    return {
        "fixture": fixture["id"],
        "category": fixture["category"],
        "project": fixture["project"],
        "baseline_mode": args.baseline_mode,
        "repo_root": str(repo),
        "output": str(output),
        "base_commit": fixture["base_commit"],
        "gold_proposal_commit": fixture["gold_proposal_commit"],
        "change_slug": fixture["change_slug"],
        "prompt": {
            "kind": prompt["kind"],
            "confidence": prompt["confidence"],
            "source": prompt["source"],
            "text": prompt["prompt"],
        },
        "will_materialize": False,
        "will_run_model": False,
        "next_manual_steps": [
            "materialize the fixture when ready to execute",
            "run the selected baseline mode in the materialized checkout",
            "save generated OpenSpec artifacts under the output path",
            "score artifacts and apply rubric review",
        ],
    }


def git_show(repo: Path, commit: str, rel: str) -> str | None:
    result = run(["git", "show", f"{commit}:{rel}"], cwd=repo)
    if result.returncode != 0:
        return None
    return result.stdout


def list_gold_artifacts(repo: Path, fixture: dict[str, Any]) -> list[str]:
    base = f"openspec/changes/{fixture['change_slug']}"
    result = run(["git", "ls-tree", "-r", "--name-only", fixture["gold_proposal_commit"], base], cwd=repo)
    if result.returncode != 0:
        raise EvalError(f"failed to list gold artifacts: {result.stderr.strip()}")
    rels: list[str] = []
    for path in result.stdout.splitlines():
        if path.endswith(".md"):
            rels.append(path.removeprefix(base + "/"))
    return sorted(rels)


def tokens(text: str) -> Counter[str]:
    return Counter(token.lower() for token in TOKEN_RE.findall(text))


def overlap_score(candidate: str, gold: str) -> float:
    gold_tokens = tokens(gold)
    candidate_tokens = tokens(candidate)
    if not gold_tokens:
        return 1.0 if not candidate_tokens else 0.0
    overlap = sum(min(count, candidate_tokens[token]) for token, count in gold_tokens.items())
    return overlap / sum(gold_tokens.values())


def score_artifacts(args: argparse.Namespace) -> dict[str, Any]:
    catalog = load_catalog(args.catalog)
    fixture = find_fixture(catalog, args.fixture)
    roots = parse_repo_roots(args.repo_root)
    repo = require_repo_root(fixture, roots)
    verify_commit(repo, fixture["gold_proposal_commit"])
    candidate_dir = args.candidate.expanduser().resolve()
    if not candidate_dir.is_dir():
        raise EvalError(f"candidate path is not a directory: {candidate_dir}")

    gold_artifacts = list_gold_artifacts(repo, fixture)
    base = f"openspec/changes/{fixture['change_slug']}"
    artifact_scores = []
    required_present = 0
    for rel in gold_artifacts:
        gold_text = git_show(repo, fixture["gold_proposal_commit"], f"{base}/{rel}")
        candidate_path = candidate_dir / rel
        candidate_text = candidate_path.read_text() if candidate_path.exists() else ""
        present = candidate_path.exists()
        if rel in REQUIRED_ARTIFACTS and present:
            required_present += 1
        artifact_scores.append({
            "artifact": rel,
            "present": present,
            "overlap": round(overlap_score(candidate_text, gold_text or ""), 4),
        })

    required_score = required_present / len(REQUIRED_ARTIFACTS)
    overlap_average = sum(item["overlap"] for item in artifact_scores) / len(artifact_scores) if artifact_scores else 0.0
    total = round((required_score * 0.45) + (overlap_average * 0.55), 4)
    return {
        "fixture": fixture["id"],
        "category": fixture["category"],
        "candidate": str(candidate_dir),
        "score": total,
        "required_artifact_score": round(required_score, 4),
        "overlap_average": round(overlap_average, 4),
        "artifacts": artifact_scores,
    }


def validate_rubric_score(args: argparse.Namespace) -> dict[str, Any]:
    catalog = load_catalog(args.catalog)
    data = load_catalog(args.score)
    failures: list[str] = []
    if data.get("version") != 1:
        failures.append("version must be 1")
    fixture_id = data.get("fixture_id")
    if not isinstance(fixture_id, str) or not ID_RE.match(fixture_id):
        failures.append("fixture_id must be kebab-case")
    else:
        try:
            find_fixture(catalog, fixture_id)
        except EvalError:
            failures.append(f"fixture_id is not in fixture catalog: {fixture_id}")
    baseline_mode = data.get("baseline_mode")
    if baseline_mode not in BASELINE_MODES:
        failures.append(f"baseline_mode must be one of: {', '.join(BASELINE_MODES)}")
    if not isinstance(data.get("reviewer"), str) or not data.get("reviewer"):
        failures.append("reviewer must be a non-empty string")

    scores = data.get("scores")
    notes = data.get("notes")
    if not isinstance(scores, dict):
        failures.append("scores must be an object")
        scores = {}
    if not isinstance(notes, dict):
        failures.append("notes must be an object")
        notes = {}
    extra_scores = sorted(set(scores) - set(RUBRIC_DIMENSIONS))
    extra_notes = sorted(set(notes) - set(RUBRIC_DIMENSIONS))
    if extra_scores:
        failures.append("scores has unknown dimensions: " + ", ".join(extra_scores))
    if extra_notes:
        failures.append("notes has unknown dimensions: " + ", ".join(extra_notes))
    for dimension in RUBRIC_DIMENSIONS:
        score = scores.get(dimension)
        if not isinstance(score, int) or not 1 <= score <= 5:
            failures.append(f"scores.{dimension} must be an integer from 1 to 5")
        note = notes.get(dimension)
        if not isinstance(note, str) or not note:
            failures.append(f"notes.{dimension} must be a non-empty string")

    average = 0.0
    if not failures:
        average = round(sum(scores[dimension] for dimension in RUBRIC_DIMENSIONS) / len(RUBRIC_DIMENSIONS), 2)
    result = {
        "score_file": str(args.score),
        "fixture": fixture_id,
        "baseline_mode": baseline_mode,
        "average": average,
        "dimensions": list(RUBRIC_DIMENSIONS),
        "failures": failures,
    }
    if failures:
        raise EvalError(json.dumps(result, indent=2))
    return result


def emit(result: dict[str, Any]) -> None:
    print(json.dumps(result, indent=2, sort_keys=True))


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--catalog", type=Path, default=DEFAULT_CATALOG)
    sub = parser.add_subparsers(dest="command", required=True)

    sub.add_parser("validate-catalog", help="validate the checked-in fixture catalog")

    prompt_parser = sub.add_parser("validate-prompt-catalog", help="validate the checked-in prompt catalog")
    prompt_parser.add_argument("--prompts", type=Path, default=DEFAULT_PROMPTS)

    prepare_parser = sub.add_parser("prepare-run", help="validate a fixture run plan without materializing or running models")
    prepare_parser.add_argument("--fixture", required=True)
    prepare_parser.add_argument("--output", required=True, type=Path)
    prepare_parser.add_argument("--repo-root", action="append", default=[], help="project=/path/to/local/repo")
    prepare_parser.add_argument("--prompts", type=Path, default=DEFAULT_PROMPTS)
    prepare_parser.add_argument("--baseline-mode", required=True, choices=BASELINE_MODES)

    materialize_parser = sub.add_parser("materialize", help="create a base checkout for one fixture")
    materialize_parser.add_argument("--fixture", required=True)
    materialize_parser.add_argument("--output", required=True, type=Path)
    materialize_parser.add_argument("--repo-root", action="append", default=[], help="project=/path/to/local/repo")

    score_parser = sub.add_parser("score-artifacts", help="score candidate OpenSpec artifacts against gold")
    score_parser.add_argument("--fixture", required=True)
    score_parser.add_argument("--candidate", required=True, type=Path)
    score_parser.add_argument("--repo-root", action="append", default=[], help="project=/path/to/local/repo")

    rubric_parser = sub.add_parser("validate-rubric-score", help="validate a human rubric score JSON file")
    rubric_parser.add_argument("--score", required=True, type=Path)

    args = parser.parse_args(argv)
    try:
        if args.command == "validate-catalog":
            emit(validate_catalog(args.catalog))
        elif args.command == "validate-prompt-catalog":
            validate_catalog(args.catalog)
            emit(validate_prompt_catalog(args.prompts, args.catalog))
        elif args.command == "prepare-run":
            validate_catalog(args.catalog)
            validate_prompt_catalog(args.prompts, args.catalog)
            emit(prepare_run(args))
        elif args.command == "materialize":
            validate_catalog(args.catalog)
            emit(materialize(args))
        elif args.command == "score-artifacts":
            validate_catalog(args.catalog)
            emit(score_artifacts(args))
        elif args.command == "validate-rubric-score":
            validate_catalog(args.catalog)
            emit(validate_rubric_score(args))
        else:  # pragma: no cover
            raise EvalError(f"unsupported command {args.command}")
    except EvalError as exc:
        print(str(exc), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
