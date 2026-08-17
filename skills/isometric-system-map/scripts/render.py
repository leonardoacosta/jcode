#!/usr/bin/env python3
"""Validate and render evidence-backed isometric system maps."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any

SKILL_DIR = Path(__file__).resolve().parents[1]
TEMPLATE_PATH = SKILL_DIR / "templates" / "renderer.html"
DATA_TOKEN = "__ISO_MAP_DATA__"

PALETTES = {"midnight", "paper"}
NODE_KINDS = {
    "entry",
    "pipeline",
    "governance",
    "module",
    "network",
    "compute",
    "data",
    "identity",
    "messaging",
    "observability",
    "external",
}
NODE_STATUSES = {"active", "held", "external", "deprecated"}
EDGE_TYPES = {
    "control",
    "data",
    "delivery",
    "dependency",
    "identity",
    "network",
    "telemetry",
}
SOURCE_PATH_RE = re.compile(r"^(?!/)(?![A-Za-z]:[\\/])(?!.*(?:^|/)\.\.(?:/|$)).+?(?::\d+(?:-\d+)?)?$")
COMMIT_RE = re.compile(r"^[0-9a-fA-F]{7,40}$")
ID_RE = re.compile(r"^[a-z0-9][a-z0-9-]*$")


def _is_nonempty_string(value: Any) -> bool:
    return isinstance(value, str) and bool(value.strip())


def _unknown_keys(value: Any, allowed: set[str], path: str, errors: list[str]) -> None:
    if not isinstance(value, dict):
        return
    for key in sorted(set(value) - allowed):
        errors.append(f"{path}.{key}: unknown field")


def _required_string(value: Any, key: str, path: str, errors: list[str]) -> None:
    if not isinstance(value, dict) or not _is_nonempty_string(value.get(key)):
        errors.append(f"{path}.{key}: required non-empty string")


def _validate_id(value: Any, path: str, errors: list[str]) -> None:
    if not _is_nonempty_string(value) or not ID_RE.fullmatch(value):
        errors.append(f"{path}: must be a lowercase kebab-case identifier")


def _validate_source_paths(value: Any, path: str, errors: list[str]) -> None:
    if not isinstance(value, list) or not value:
        errors.append(f"{path}: requires at least one repo-relative citation")
        return
    for index, item in enumerate(value):
        if not _is_nonempty_string(item) or not SOURCE_PATH_RE.fullmatch(item):
            errors.append(
                f"{path}[{index}]: must be a repo-relative path with an optional :line or :start-end suffix"
            )


def validate_document(document: Any) -> list[str]:
    """Return structural and integrity errors for a map document."""

    errors: list[str] = []
    if not isinstance(document, dict):
        return ["$: document must be an object"]

    _unknown_keys(
        document,
        {"version", "repository", "palette", "zones", "nodes", "edges", "flows"},
        "$",
        errors,
    )
    if document.get("version") != 1:
        errors.append("$.version: must equal 1")

    repository = document.get("repository")
    if not isinstance(repository, dict):
        errors.append("$.repository: required object")
    else:
        _unknown_keys(repository, {"name", "ref", "commit", "scope", "summary"}, "repository", errors)
        for key in ("name", "ref", "scope", "summary"):
            _required_string(repository, key, "repository", errors)
        commit = repository.get("commit")
        if not _is_nonempty_string(commit) or not COMMIT_RE.fullmatch(commit):
            errors.append("repository.commit: must be a 7-40 character hexadecimal git commit")

    if document.get("palette") not in PALETTES:
        errors.append(f"$.palette: must be one of {sorted(PALETTES)}")

    zones = document.get("zones")
    if not isinstance(zones, list) or not zones:
        errors.append("$.zones: requires 1-8 zones")
        zones = []
    elif len(zones) > 8:
        errors.append("$.zones: maximum 8 zones")

    zone_ids: set[str] = set()
    for index, zone in enumerate(zones):
        path = f"zones[{index}]"
        if not isinstance(zone, dict):
            errors.append(f"{path}: must be an object")
            continue
        _unknown_keys(zone, {"id", "label", "description"}, path, errors)
        _validate_id(zone.get("id"), f"{path}.id", errors)
        for key in ("label", "description"):
            _required_string(zone, key, path, errors)
        zone_id = zone.get("id")
        if isinstance(zone_id, str):
            if zone_id in zone_ids:
                errors.append(f"{path}.id: duplicate zone id '{zone_id}'")
            zone_ids.add(zone_id)

    nodes = document.get("nodes")
    if not isinstance(nodes, list) or not nodes:
        errors.append("$.nodes: requires 1-24 nodes")
        nodes = []
    elif len(nodes) > 24:
        errors.append("$.nodes: maximum 24 nodes")

    node_ids: set[str] = set()
    positions: set[tuple[int, int]] = set()
    for index, node in enumerate(nodes):
        path = f"nodes[{index}]"
        if not isinstance(node, dict):
            errors.append(f"{path}: must be an object")
            continue
        _unknown_keys(
            node,
            {
                "id",
                "code",
                "label",
                "kind",
                "zone",
                "position",
                "status",
                "purpose",
                "behavior",
                "implementation",
                "source_paths",
            },
            path,
            errors,
        )
        _validate_id(node.get("id"), f"{path}.id", errors)
        for key in ("code", "label", "purpose", "behavior", "implementation"):
            _required_string(node, key, path, errors)
        if isinstance(node.get("code"), str) and len(node["code"]) > 5:
            errors.append(f"{path}.code: maximum 5 characters")
        if node.get("kind") not in NODE_KINDS:
            errors.append(f"{path}.kind: must be one of {sorted(NODE_KINDS)}")
        if node.get("status") not in NODE_STATUSES:
            errors.append(f"{path}.status: must be one of {sorted(NODE_STATUSES)}")
        if node.get("zone") not in zone_ids:
            errors.append(f"{path}.zone: references unknown zone '{node.get('zone')}'")
        _validate_source_paths(node.get("source_paths"), f"{path}.source_paths", errors)

        position = node.get("position")
        if not isinstance(position, dict):
            errors.append(f"{path}.position: required object")
        else:
            _unknown_keys(position, {"x", "y"}, f"{path}.position", errors)
            x, y = position.get("x"), position.get("y")
            if not isinstance(x, int) or isinstance(x, bool) or not 0 <= x <= 12:
                errors.append(f"{path}.position.x: integer from 0 to 12 required")
            if not isinstance(y, int) or isinstance(y, bool) or not 0 <= y <= 12:
                errors.append(f"{path}.position.y: integer from 0 to 12 required")
            if isinstance(x, int) and isinstance(y, int) and not isinstance(x, bool) and not isinstance(y, bool):
                point = (x, y)
                if point in positions:
                    errors.append(f"{path}.position: duplicate grid point {point}")
                positions.add(point)

        node_id = node.get("id")
        if isinstance(node_id, str):
            if node_id in node_ids:
                errors.append(f"{path}.id: duplicate node id '{node_id}'")
            node_ids.add(node_id)

    edges = document.get("edges")
    if not isinstance(edges, list) or not edges:
        errors.append("$.edges: requires 1-48 edges")
        edges = []
    elif len(edges) > 48:
        errors.append("$.edges: maximum 48 edges")

    edge_ids: set[str] = set()
    for index, edge in enumerate(edges):
        path = f"edges[{index}]"
        if not isinstance(edge, dict):
            errors.append(f"{path}: must be an object")
            continue
        _unknown_keys(edge, {"id", "from", "to", "type", "label", "source_paths"}, path, errors)
        _validate_id(edge.get("id"), f"{path}.id", errors)
        _required_string(edge, "label", path, errors)
        if edge.get("from") not in node_ids:
            errors.append(f"{path}.from: references unknown node '{edge.get('from')}'")
        if edge.get("to") not in node_ids:
            errors.append(f"{path}.to: references unknown node '{edge.get('to')}'")
        if edge.get("type") not in EDGE_TYPES:
            errors.append(f"{path}.type: must be one of {sorted(EDGE_TYPES)}")
        _validate_source_paths(edge.get("source_paths"), f"{path}.source_paths", errors)
        edge_id = edge.get("id")
        if isinstance(edge_id, str):
            if edge_id in edge_ids:
                errors.append(f"{path}.id: duplicate edge id '{edge_id}'")
            edge_ids.add(edge_id)

    flows = document.get("flows")
    if not isinstance(flows, list) or not flows:
        errors.append("$.flows: requires 1-6 flows")
        flows = []
    elif len(flows) > 6:
        errors.append("$.flows: maximum 6 flows")

    flow_ids: set[str] = set()
    for flow_index, flow in enumerate(flows):
        path = f"flows[{flow_index}]"
        if not isinstance(flow, dict):
            errors.append(f"{path}: must be an object")
            continue
        _unknown_keys(flow, {"id", "label", "summary", "payload", "steps"}, path, errors)
        _validate_id(flow.get("id"), f"{path}.id", errors)
        for key in ("label", "summary"):
            _required_string(flow, key, path, errors)
        flow_id = flow.get("id")
        if isinstance(flow_id, str):
            if flow_id in flow_ids:
                errors.append(f"{path}.id: duplicate flow id '{flow_id}'")
            flow_ids.add(flow_id)

        payload = flow.get("payload")
        if not isinstance(payload, dict):
            errors.append(f"{path}.payload: required object")
        else:
            _unknown_keys(payload, {"label", "description", "schema", "source_paths"}, f"{path}.payload", errors)
            for key in ("label", "description", "schema"):
                _required_string(payload, key, f"{path}.payload", errors)
            _validate_source_paths(payload.get("source_paths"), f"{path}.payload.source_paths", errors)

        steps = flow.get("steps")
        if not isinstance(steps, list) or not steps:
            errors.append(f"{path}.steps: requires 1-12 steps")
            continue
        if len(steps) > 12:
            errors.append(f"{path}.steps: maximum 12 steps")
        for step_index, step in enumerate(steps):
            step_path = f"{path}.steps[{step_index}]"
            if not isinstance(step, dict):
                errors.append(f"{step_path}: must be an object")
                continue
            _unknown_keys(step, {"edge", "label", "detail", "source_paths"}, step_path, errors)
            for key in ("label", "detail"):
                _required_string(step, key, step_path, errors)
            if step.get("edge") not in edge_ids:
                errors.append(f"{step_path}.edge: references unknown edge '{step.get('edge')}'")
            _validate_source_paths(step.get("source_paths"), f"{step_path}.source_paths", errors)

    return errors


def render_document(document: dict[str, Any]) -> str:
    """Render a validated map document into one self-contained HTML file."""

    errors = validate_document(document)
    if errors:
        raise ValueError("Invalid isometric system map:\n" + "\n".join(errors))
    template = TEMPLATE_PATH.read_text(encoding="utf-8")
    if template.count(DATA_TOKEN) != 1:
        raise ValueError(f"Renderer template must contain {DATA_TOKEN} exactly once")
    payload = json.dumps(document, ensure_ascii=False, separators=(",", ":"))
    payload = payload.replace("<", "\\u003c").replace(">", "\\u003e").replace("&", "\\u0026")
    return template.replace(DATA_TOKEN, payload)


def _read_document(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise ValueError(f"Unable to read {path}: {exc}") from exc
    if not isinstance(value, dict):
        raise ValueError(f"{path}: top-level JSON value must be an object")
    return value


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--validate", action="store_true", help="validate only; do not render")
    parser.add_argument("input", type=Path, help="map JSON document")
    parser.add_argument("output", type=Path, nargs="?", help="self-contained HTML output")
    args = parser.parse_args(argv)

    try:
        document = _read_document(args.input)
        errors = validate_document(document)
        if errors:
            for error in errors:
                print(error, file=sys.stderr)
            return 1
        if args.validate:
            print(f"valid: {args.input}")
            return 0
        if args.output is None:
            parser.error("output is required unless --validate is used")
        html = render_document(document)
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(html, encoding="utf-8")
        print(
            f"rendered {len(document['nodes'])} nodes, {len(document['edges'])} edges, "
            f"and {len(document['flows'])} flows to {args.output}"
        )
        return 0
    except ValueError as exc:
        print(exc, file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
