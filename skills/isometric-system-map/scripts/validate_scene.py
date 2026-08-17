#!/usr/bin/env python3
"""Validate repository-evidenced isometric scene documents."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

ALLOWED_FIELDS = {
    "version", "repository", "art_direction", "canvas", "zones", "nodes",
    "paths", "payloads", "flows",
}


def _rect(node: dict[str, Any]) -> tuple[float, float, float, float]:
    pos = node.get("position", {})
    footprint = node.get("footprint", {})
    x = float(pos.get("x", 0))
    y = float(pos.get("y", 0))
    return (x, y, x + float(footprint.get("width", 1)), y + float(footprint.get("depth", 1)))


def _overlaps(a: tuple[float, float, float, float], b: tuple[float, float, float, float]) -> bool:
    return a[0] < b[2] and a[2] > b[0] and a[1] < b[3] and a[3] > b[1]


def _point_in_rect(x: float, y: float, rect: tuple[float, float, float, float]) -> bool:
    return rect[0] < x < rect[2] and rect[1] < y < rect[3]


def _segment_intersects_rect(a: dict[str, Any], b: dict[str, Any], rect: tuple[float, float, float, float]) -> bool:
    x1, y1 = float(a["x"]), float(a["y"])
    x2, y2 = float(b["x"]), float(b["y"])
    if _point_in_rect(x1, y1, rect) or _point_in_rect(x2, y2, rect):
        return True
    if x1 == x2:
        return rect[0] < x1 < rect[2] and max(min(y1, y2), rect[1]) < min(max(y1, y2), rect[3])
    if y1 == y2:
        return rect[1] < y1 < rect[3] and max(min(x1, x2), rect[0]) < min(max(x1, x2), rect[2])
    # Routes should be orthogonal, but sample the segment defensively.
    for step in range(1, 100):
        t = step / 100
        if _point_in_rect(x1 + (x2 - x1) * t, y1 + (y2 - y1) * t, rect):
            return True
    return False


def _valid_evidence(value: Any) -> bool:
    return isinstance(value, list) and any(
        isinstance(item, dict)
        and all(isinstance(item.get(key), str) and item[key].strip() for key in ("path", "lines", "claim"))
        for item in value
    )


def validate_scene(document: Any) -> list[str]:
    if not isinstance(document, dict):
        return ["$: scene must be an object"]
    errors: list[str] = []
    for field in document:
        if field not in ALLOWED_FIELDS:
            errors.append(f"$.{field}: unknown field")

    nodes = document.get("nodes", [])
    if not isinstance(nodes, list):
        return errors + ["$.nodes: must be an array"]
    node_by_id = {node.get("id"): node for node in nodes if isinstance(node, dict) and node.get("id")}
    for index, node in enumerate(nodes):
        if not isinstance(node, dict):
            errors.append(f"nodes[{index}]: must be an object")
            continue
        if not _valid_evidence(node.get("evidence")):
            errors.append(f"nodes[{index}].evidence: requires at least one path/lines/claim evidence object")
        for previous in range(index):
            if isinstance(nodes[previous], dict) and _overlaps(_rect(node), _rect(nodes[previous])):
                errors.append(f"nodes[{index}]: footprint overlaps nodes[{previous}]")

    forms = {node.get("form") for node in nodes if isinstance(node, dict) and node.get("form")}
    if len(forms) < 3:
        errors.append("$.nodes: requires at least 3 distinct building forms")

    for index, path in enumerate(document.get("paths", [])):
        if not isinstance(path, dict):
            continue
        if not _valid_evidence(path.get("evidence")):
            errors.append(f"paths[{index}].evidence: requires at least one path/lines/claim evidence object")
        route = path.get("route", [])
        excluded = {path.get("from"), path.get("to")}
        for node_id, node in node_by_id.items():
            if node_id in excluded:
                continue
            rect = _rect(node)
            if any(_segment_intersects_rect(a, b, rect) for a, b in zip(route, route[1:])):
                errors.append(f"paths[{index}].route: route intersects node '{node_id}'")

    for flow_index, flow in enumerate(document.get("flows", [])):
        if not isinstance(flow, dict):
            continue
        for step_index, step in enumerate(flow.get("steps", [])):
            if not isinstance(step, dict) or not _valid_evidence(step.get("evidence")):
                errors.append(
                    f"flows[{flow_index}].steps[{step_index}].evidence: requires at least one path/lines/claim evidence object"
                )
    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", type=Path)
    args = parser.parse_args()
    try:
        document = json.loads(args.input.read_text())
    except (OSError, json.JSONDecodeError) as exc:
        parser.error(str(exc))
    errors = validate_scene(document)
    if errors:
        for error in errors:
            print(error)
        return 1
    print(f"Valid isometric scene: {args.input}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
