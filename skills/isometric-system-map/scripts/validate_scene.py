#!/usr/bin/env python3
"""Validate the semantic and geometric contract for an isometric system-map scene."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any

SCRIPT_DIR = Path(__file__).resolve().parent
SKILL_DIR = SCRIPT_DIR.parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

from scene_math import footprints_overlap  # noqa: E402


ID_RE = re.compile(r"^[a-z0-9][a-z0-9-]*$")
COMMIT_RE = re.compile(r"^[0-9a-fA-F]{7,40}$")
LINES_RE = re.compile(r"^\d+(?:-\d+)?$")
SOURCE_PATH_RE = re.compile(
    r"^(?!/)(?![A-Za-z]:[\\/])(?!.*(?:^|/)\.\.(?:/|$))[^\n\r]+$"
)
AZURE_ICON_RE = re.compile(r'<symbol\s+id="(?P<id>az-[a-z0-9-]+)"')


def _load_azure_icon_ids() -> frozenset[str]:
    sprite = SKILL_DIR / "assets" / "azure-icons.svg"
    if not sprite.exists():
        return frozenset()
    return frozenset(match.group("id") for match in AZURE_ICON_RE.finditer(sprite.read_text()))


AZURE_ICON_IDS = _load_azure_icon_ids()

NODE_ROLES = {
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
NODE_FORMS = {"cube"}
NODE_STATUSES = {"active", "conditional", "held", "external", "deprecated"}
AREA_KINDS = {"vnet"}
PATH_KINDS = {
    "control",
    "data",
    "delivery",
    "dependency",
    "identity",
    "network",
    "telemetry",
}
PAYLOAD_KINDS = {
    "command",
    "deployment",
    "event",
    "record",
    "resource-id",
    "secret-reference",
    "telemetry",
    "network-session",
}
PALETTE_ROLES = {
    "background",
    "grid",
    "structure",
    "control_path",
    "data_path",
    "payload",
    "text",
}
PATH_TREATMENT_FIELDS = {
    "stroke_pattern",
    "weight",
    "marker",
    "texture",
    "motion_cadence",
    "reduced_motion",
}


def _is_string(value: Any) -> bool:
    return isinstance(value, str) and bool(value.strip())


def _is_number(value: Any) -> bool:
    return isinstance(value, (int, float)) and not isinstance(value, bool)


def _is_half_step(value: Any) -> bool:
    return _is_number(value) and abs(float(value) * 2 - round(float(value) * 2)) < 1e-9


def _unknown_keys(value: Any, allowed: set[str], path: str, errors: list[str]) -> None:
    if not isinstance(value, dict):
        return
    for key in sorted(set(value) - allowed):
        errors.append(f"{path}.{key}: unknown field")


def _required_string(value: Any, key: str, path: str, errors: list[str]) -> None:
    if not isinstance(value, dict) or not _is_string(value.get(key)):
        errors.append(f"{path}.{key}: required non-empty string")


def _validate_id(value: Any, path: str, errors: list[str]) -> None:
    if not _is_string(value) or ID_RE.fullmatch(value) is None:
        errors.append(f"{path}: must be a lowercase kebab-case identifier")


def _validate_evidence(value: Any, path: str, errors: list[str]) -> None:
    if not isinstance(value, list) or not value:
        errors.append(f"{path}: requires at least one path/lines/claim evidence object")
        return
    for index, item in enumerate(value):
        item_path = f"{path}[{index}]"
        if not isinstance(item, dict):
            errors.append(f"{item_path}: must be an object")
            continue
        _unknown_keys(item, {"path", "lines", "claim"}, item_path, errors)
        for key in ("path", "lines", "claim"):
            _required_string(item, key, item_path, errors)
        source_path = item.get("path")
        if _is_string(source_path) and SOURCE_PATH_RE.fullmatch(source_path) is None:
            errors.append(f"{item_path}.path: must be a safe repo-relative path")
        lines = item.get("lines")
        if _is_string(lines) and LINES_RE.fullmatch(lines) is None:
            errors.append(f"{item_path}.lines: must be line or start-end digits")


def _rect_for_node(node: dict[str, Any]) -> dict[str, float] | None:
    position = node.get("position")
    footprint = node.get("footprint")
    if not isinstance(position, dict) or not isinstance(footprint, dict):
        return None
    values = (
        position.get("x"),
        position.get("y"),
        footprint.get("width"),
        footprint.get("depth"),
    )
    if not all(_is_number(value) for value in values):
        return None
    return {
        "x": float(values[0]),
        "y": float(values[1]),
        "width": float(values[2]),
        "depth": float(values[3]),
    }


def _point_on_or_just_outside_rect(
    point: dict[str, float], rect: dict[str, float], margin: float = 0.5
) -> bool:
    """Return whether a route endpoint touches one footprint edge or its outward half-cell."""

    epsilon = 1e-9
    x, y = point["x"], point["y"]
    left, right = rect["x"], rect["x"] + rect["width"]
    back, front = rect["y"], rect["y"] + rect["depth"]
    within_x = left - epsilon <= x <= right + epsilon
    within_y = back - epsilon <= y <= front + epsilon
    on_vertical_edge = within_y and (
        abs(x - left) <= epsilon or abs(x - right) <= epsilon
    )
    on_horizontal_edge = within_x and (
        abs(y - back) <= epsilon or abs(y - front) <= epsilon
    )
    outside_vertical_edge = back <= y <= front and (
        left - margin <= x < left or right < x <= right + margin
    )
    outside_horizontal_edge = left <= x <= right and (
        back - margin <= y < back or front < y <= front + margin
    )
    return on_vertical_edge or on_horizontal_edge or outside_vertical_edge or outside_horizontal_edge


def _segment_intersects_rect_interior(
    start: dict[str, float], end: dict[str, float], rect: dict[str, float]
) -> bool:
    epsilon = 1e-9
    if abs(start["x"] - end["x"]) < epsilon:
        x = start["x"]
        if not rect["x"] + epsilon < x < rect["x"] + rect["width"] - epsilon:
            return False
        low, high = sorted((start["y"], end["y"]))
        return max(low, rect["y"] + epsilon) < min(high, rect["y"] + rect["depth"] - epsilon)
    if abs(start["y"] - end["y"]) < epsilon:
        y = start["y"]
        if not rect["y"] + epsilon < y < rect["y"] + rect["depth"] - epsilon:
            return False
        low, high = sorted((start["x"], end["x"]))
        return max(low, rect["x"] + epsilon) < min(high, rect["x"] + rect["width"] - epsilon)
    return False


def validate_scene(document: Any) -> list[str]:
    """Return strict semantic, evidence, and geometry errors for one scene document."""

    errors: list[str] = []
    if not isinstance(document, dict):
        return ["$: document must be an object"]

    _unknown_keys(
        document,
        {
            "version",
            "repository",
            "art_direction",
            "canvas",
            "zones",
            "areas",
            "nodes",
            "paths",
            "payloads",
            "flows",
        },
        "$",
        errors,
    )
    if document.get("version") != 1:
        errors.append("$.version: must equal 1")

    repository = document.get("repository")
    if not isinstance(repository, dict):
        errors.append("$.repository: required object")
    else:
        _unknown_keys(
            repository,
            {"name", "ref", "commit", "scope", "summary"},
            "repository",
            errors,
        )
        for key in ("name", "ref", "scope", "summary"):
            _required_string(repository, key, "repository", errors)
        commit = repository.get("commit")
        if not _is_string(commit) or COMMIT_RE.fullmatch(commit) is None:
            errors.append("repository.commit: must be a 7-40 character hexadecimal git commit")

    art_direction = document.get("art_direction")
    path_treatments: dict[str, Any] = {}
    if not isinstance(art_direction, dict):
        errors.append("$.art_direction: required object")
    else:
        _unknown_keys(
            art_direction,
            {
                "name",
                "principles",
                "palette_roles",
                "medium",
                "linework",
                "materials",
                "typography",
                "motion",
                "path_treatments",
            },
            "art_direction",
            errors,
        )
        for key in ("name", "medium", "linework", "materials", "typography", "motion"):
            _required_string(art_direction, key, "art_direction", errors)
        principles = art_direction.get("principles")
        if not isinstance(principles, list) or len(principles) < 2 or not all(
            _is_string(item) for item in principles
        ):
            errors.append("art_direction.principles: requires at least two non-empty strings")
        palette = art_direction.get("palette_roles")
        if not isinstance(palette, dict):
            errors.append("art_direction.palette_roles: required object")
        else:
            for role in sorted(PALETTE_ROLES):
                if not _is_string(palette.get(role)):
                    errors.append(f"art_direction.palette_roles.{role}: required color or paint token")
        treatments = art_direction.get("path_treatments")
        if not isinstance(treatments, dict):
            errors.append("art_direction.path_treatments: required object")
        else:
            path_treatments = treatments
            _unknown_keys(treatments, PATH_KINDS, "art_direction.path_treatments", errors)
            for kind, treatment in treatments.items():
                treatment_path = f"art_direction.path_treatments.{kind}"
                if kind not in PATH_KINDS:
                    continue
                if not isinstance(treatment, dict):
                    errors.append(f"{treatment_path}: must be an object")
                    continue
                _unknown_keys(treatment, PATH_TREATMENT_FIELDS, treatment_path, errors)
                for field in sorted(PATH_TREATMENT_FIELDS):
                    _required_string(treatment, field, treatment_path, errors)

    canvas = document.get("canvas")
    grid_width = grid_depth = 0
    cube_size = 0.0
    if not isinstance(canvas, dict):
        errors.append("$.canvas: required object")
    else:
        _unknown_keys(
            canvas,
            {"grid_width", "grid_depth", "tile_width", "tile_height", "cube_size"},
            "canvas",
            errors,
        )
        for key, low, high in (
            ("grid_width", 4, 32),
            ("grid_depth", 4, 32),
            ("tile_width", 16, 192),
            ("tile_height", 8, 96),
        ):
            value = canvas.get(key)
            if not isinstance(value, int) or isinstance(value, bool) or not low <= value <= high:
                errors.append(f"canvas.{key}: integer from {low} to {high} required")
        if isinstance(canvas.get("grid_width"), int):
            grid_width = canvas["grid_width"]
        if isinstance(canvas.get("grid_depth"), int):
            grid_depth = canvas["grid_depth"]
        if not _is_half_step(canvas.get("cube_size")) or not 0.5 <= float(canvas["cube_size"]) <= 2:
            errors.append("canvas.cube_size: half-grid number from 0.5 to 2 required")
        else:
            cube_size = float(canvas["cube_size"])
        if (
            isinstance(canvas.get("tile_width"), int)
            and isinstance(canvas.get("tile_height"), int)
            and canvas["tile_width"] != canvas["tile_height"] * 2
        ):
            errors.append("canvas: tile_width must equal 2 × tile_height for a true 2:1 isometric grid")

    zones = document.get("zones")
    if not isinstance(zones, list) or not zones:
        errors.append("$.zones: requires 1-8 visual or sourced regions")
        zones = []
    elif len(zones) > 8:
        errors.append("$.zones: maximum 8 regions")
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
    if not isinstance(nodes, list) or len(nodes) < 3:
        errors.append("$.nodes: requires 3-28 resource cubes")
        nodes = []
    elif len(nodes) > 28:
        errors.append("$.nodes: maximum 28 resource cubes")

    node_ids: set[str] = set()
    node_rects: dict[str, dict[str, float]] = {}
    valid_nodes: list[tuple[int, dict[str, Any], dict[str, float]]] = []
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
                "role",
                "form",
                "zone",
                "position",
                "footprint",
                "status",
                "description",
                "evidence",
                "resource_type",
                "icon",
            },
            path,
            errors,
        )
        _validate_id(node.get("id"), f"{path}.id", errors)
        for key in ("code", "label", "description"):
            _required_string(node, key, path, errors)
        if isinstance(node.get("code"), str) and len(node["code"]) > 6:
            errors.append(f"{path}.code: maximum 6 characters")
        if node.get("role") not in NODE_ROLES:
            errors.append(f"{path}.role: must be one of {sorted(NODE_ROLES)}")
        if node.get("form") not in NODE_FORMS:
            errors.append(f"{path}.form: must be one of {sorted(NODE_FORMS)}")
        if node.get("status") not in NODE_STATUSES:
            errors.append(f"{path}.status: must be one of {sorted(NODE_STATUSES)}")
        if node.get("zone") not in zone_ids:
            errors.append(f"{path}.zone: references unknown zone '{node.get('zone')}'")
        if "resource_type" in node and not _is_string(node.get("resource_type")):
            errors.append(f"{path}.resource_type: must be a non-empty string when provided")
        if "icon" in node:
            icon = node.get("icon")
            if not _is_string(icon):
                errors.append(f"{path}.icon: must be a non-empty string when provided")
            elif icon not in AZURE_ICON_IDS:
                errors.append(f"{path}.icon: unsupported Azure topology icon '{icon}'")
        _validate_evidence(node.get("evidence"), f"{path}.evidence", errors)

        position = node.get("position")
        footprint = node.get("footprint")
        if not isinstance(position, dict):
            errors.append(f"{path}.position: required object")
        else:
            _unknown_keys(position, {"x", "y"}, f"{path}.position", errors)
            for key, maximum in (("x", grid_width), ("y", grid_depth)):
                value = position.get(key)
                if not isinstance(value, int) or isinstance(value, bool) or not 0 <= value < maximum:
                    errors.append(f"{path}.position.{key}: integer inside the canvas required")
        if not isinstance(footprint, dict):
            errors.append(f"{path}.footprint: required object")
        else:
            _unknown_keys(footprint, {"width", "depth"}, f"{path}.footprint", errors)
            for key in ("width", "depth"):
                value = footprint.get(key)
                if not isinstance(value, int) or isinstance(value, bool) or not 1 <= value <= 4:
                    errors.append(f"{path}.footprint.{key}: integer from 1 to 4 required")
            if cube_size and all(isinstance(footprint.get(key), int) for key in ("width", "depth")):
                if min(footprint["width"], footprint["depth"]) < cube_size:
                    errors.append(f"{path}.footprint: must contain canvas.cube_size {cube_size:g}")

        node_id = node.get("id")
        if isinstance(node_id, str):
            if node_id in node_ids:
                errors.append(f"{path}.id: duplicate node id '{node_id}'")
            node_ids.add(node_id)
        rect = _rect_for_node(node)
        if rect is not None and grid_width and grid_depth:
            if rect["x"] + rect["width"] > grid_width or rect["y"] + rect["depth"] > grid_depth:
                errors.append(f"{path}.footprint: extends beyond the canvas")
            if isinstance(node_id, str):
                for prior_index, prior_node, prior_rect in valid_nodes:
                    if footprints_overlap(rect, prior_rect):
                        errors.append(
                            f"{path}.footprint: overlaps nodes[{prior_index}] '{prior_node.get('id')}'"
                        )
                valid_nodes.append((index, node, rect))
                node_rects[node_id] = rect

    areas = document.get("areas")
    if not isinstance(areas, list):
        errors.append("$.areas: required list with at most 8 sourced containment areas")
        areas = []
    elif len(areas) > 8:
        errors.append("$.areas: maximum 8 sourced containment areas")
    area_ids: set[str] = set()
    for index, area in enumerate(areas):
        path = f"areas[{index}]"
        if not isinstance(area, dict):
            errors.append(f"{path}: must be an object")
            continue
        _unknown_keys(
            area,
            {"id", "label", "kind", "status", "member_ids", "padding", "description", "evidence"},
            path,
            errors,
        )
        _validate_id(area.get("id"), f"{path}.id", errors)
        for key in ("label", "description"):
            _required_string(area, key, path, errors)
        if area.get("kind") not in AREA_KINDS:
            errors.append(f"{path}.kind: must be one of {sorted(AREA_KINDS)}")
        if area.get("status") not in NODE_STATUSES:
            errors.append(f"{path}.status: must be one of {sorted(NODE_STATUSES)}")
        _validate_evidence(area.get("evidence"), f"{path}.evidence", errors)

        area_id = area.get("id")
        if isinstance(area_id, str):
            if area_id in area_ids:
                errors.append(f"{path}.id: duplicate area id '{area_id}'")
            area_ids.add(area_id)

        padding = area.get("padding")
        valid_padding = _is_half_step(padding) and 0 <= float(padding) <= 2
        if not valid_padding:
            errors.append(f"{path}.padding: half-grid number from 0 to 2 required")

        member_ids = area.get("member_ids")
        member_rects: list[dict[str, float]] = []
        if not isinstance(member_ids, list) or not member_ids:
            errors.append(f"{path}.member_ids: requires 1-20 node ids")
        elif len(member_ids) > 20:
            errors.append(f"{path}.member_ids: maximum 20 node ids")
        else:
            seen_members: set[str] = set()
            for member_index, member_id in enumerate(member_ids):
                member_path = f"{path}.member_ids[{member_index}]"
                if member_id in seen_members:
                    errors.append(f"{member_path}: duplicate node id '{member_id}'")
                seen_members.add(member_id)
                rect = node_rects.get(member_id)
                if rect is None:
                    errors.append(f"{member_path}: references unknown node '{member_id}'")
                else:
                    member_rects.append(rect)

        if member_rects and valid_padding and grid_width and grid_depth:
            pad = float(padding)
            left = min(rect["x"] for rect in member_rects) - pad
            back = min(rect["y"] for rect in member_rects) - pad
            right = max(rect["x"] + rect["width"] for rect in member_rects) + pad
            front = max(rect["y"] + rect["depth"] for rect in member_rects) + pad
            if left < 0 or back < 0 or right > grid_width or front > grid_depth:
                errors.append(f"{path}: padded member bounds extend beyond the canvas")

    payloads = document.get("payloads")
    if not isinstance(payloads, list):
        errors.append("$.payloads: required list with at most 16 concrete payload definitions")
        payloads = []
    elif len(payloads) > 16:
        errors.append("$.payloads: maximum 16 payloads")
    payload_ids: set[str] = set()
    for index, payload in enumerate(payloads):
        path = f"payloads[{index}]"
        if not isinstance(payload, dict):
            errors.append(f"{path}: must be an object")
            continue
        _unknown_keys(payload, {"id", "label", "kind", "description", "evidence"}, path, errors)
        _validate_id(payload.get("id"), f"{path}.id", errors)
        for key in ("label", "description"):
            _required_string(payload, key, path, errors)
        if payload.get("kind") not in PAYLOAD_KINDS:
            errors.append(f"{path}.kind: must be one of {sorted(PAYLOAD_KINDS)}")
        _validate_evidence(payload.get("evidence"), f"{path}.evidence", errors)
        payload_id = payload.get("id")
        if isinstance(payload_id, str):
            if payload_id in payload_ids:
                errors.append(f"{path}.id: duplicate payload id '{payload_id}'")
            payload_ids.add(payload_id)

    paths = document.get("paths")
    if not isinstance(paths, list) or not paths:
        errors.append("$.paths: requires 1-64 directed architecture paths")
        paths = []
    elif len(paths) > 64:
        errors.append("$.paths: maximum 64 directed paths")
    path_ids: set[str] = set()
    path_payload_ids: dict[str, set[str]] = {}
    used_path_kinds: set[str] = set()
    for index, item in enumerate(paths):
        path = f"paths[{index}]"
        if not isinstance(item, dict):
            errors.append(f"{path}: must be an object")
            continue
        _unknown_keys(
            item,
            {"id", "from", "to", "kind", "label", "route", "payload_ids", "evidence"},
            path,
            errors,
        )
        _validate_id(item.get("id"), f"{path}.id", errors)
        _required_string(item, "label", path, errors)
        source_id = item.get("from")
        target_id = item.get("to")
        if source_id not in node_ids:
            errors.append(f"{path}.from: references unknown node '{source_id}'")
        if target_id not in node_ids:
            errors.append(f"{path}.to: references unknown node '{target_id}'")
        path_kind = item.get("kind")
        if path_kind not in PATH_KINDS:
            errors.append(f"{path}.kind: must be one of {sorted(PATH_KINDS)}")
        else:
            used_path_kinds.add(path_kind)
        _validate_evidence(item.get("evidence"), f"{path}.evidence", errors)

        listed_payloads = item.get("payload_ids")
        if not isinstance(listed_payloads, list):
            errors.append(f"{path}.payload_ids: required list, empty only for non-payload dependencies")
        else:
            if not listed_payloads and path_kind != "dependency":
                errors.append(f"{path}.payload_ids: may be empty only when kind is 'dependency'")
            for payload_index, payload_id in enumerate(listed_payloads):
                if payload_id not in payload_ids:
                    errors.append(
                        f"{path}.payload_ids[{payload_index}]: references unknown payload '{payload_id}'"
                    )

        route = item.get("route")
        route_points: list[dict[str, float]] = []
        if not isinstance(route, list) or not 2 <= len(route) <= 16:
            errors.append(f"{path}.route: requires 2-16 explicit grid points")
        else:
            for point_index, point in enumerate(route):
                point_path = f"{path}.route[{point_index}]"
                if not isinstance(point, dict):
                    errors.append(f"{point_path}: must be an object")
                    continue
                _unknown_keys(point, {"x", "y"}, point_path, errors)
                x, y = point.get("x"), point.get("y")
                if not _is_half_step(x) or not 0 <= float(x) <= grid_width:
                    errors.append(f"{point_path}.x: half-grid value inside the canvas required")
                if not _is_half_step(y) or not 0 <= float(y) <= grid_depth:
                    errors.append(f"{point_path}.y: half-grid value inside the canvas required")
                if _is_number(x) and _is_number(y):
                    route_points.append({"x": float(x), "y": float(y)})
            if len(route_points) == len(route):
                for segment_index, (start, end) in enumerate(zip(route_points, route_points[1:])):
                    if start == end:
                        errors.append(f"{path}.route[{segment_index + 1}]: duplicates the previous point")
                    elif start["x"] != end["x"] and start["y"] != end["y"]:
                        errors.append(
                            f"{path}.route[{segment_index}:{segment_index + 2}]: grid route segments must follow one isometric axis"
                        )
                source_rect = node_rects.get(source_id)
                target_rect = node_rects.get(target_id)
                if source_rect is not None and not _point_on_or_just_outside_rect(
                    route_points[0], source_rect
                ):
                    errors.append(
                        f"{path}.route[0]: must start on or just outside source node '{source_id}' boundary"
                    )
                if target_rect is not None and not _point_on_or_just_outside_rect(
                    route_points[-1], target_rect
                ):
                    errors.append(
                        f"{path}.route[-1]: must end on or just outside target node '{target_id}' boundary"
                    )
                for node_id, rect in node_rects.items():
                    if node_id in {source_id, target_id}:
                        continue
                    if any(
                        _segment_intersects_rect_interior(start, end, rect)
                        for start, end in zip(route_points, route_points[1:])
                    ):
                        errors.append(f"{path}.route: route intersects node '{node_id}'")

        path_id = item.get("id")
        if isinstance(path_id, str):
            if path_id in path_ids:
                errors.append(f"{path}.id: duplicate path id '{path_id}'")
            path_ids.add(path_id)
            if isinstance(listed_payloads, list):
                path_payload_ids[path_id] = {
                    payload_id for payload_id in listed_payloads if isinstance(payload_id, str)
                }

    for kind in sorted(used_path_kinds):
        if kind not in path_treatments:
            errors.append(
                f"art_direction.path_treatments.{kind}: required for used path kind"
            )

    flows = document.get("flows")
    if not isinstance(flows, list):
        errors.append("$.flows: required list with at most 6 named payload journeys")
        flows = []
    elif len(flows) > 6:
        errors.append("$.flows: maximum 6 flows")
    flow_ids: set[str] = set()
    for flow_index, flow in enumerate(flows):
        path = f"flows[{flow_index}]"
        if not isinstance(flow, dict):
            errors.append(f"{path}: must be an object")
            continue
        _unknown_keys(flow, {"id", "label", "description", "steps"}, path, errors)
        _validate_id(flow.get("id"), f"{path}.id", errors)
        for key in ("label", "description"):
            _required_string(flow, key, path, errors)
        flow_id = flow.get("id")
        if isinstance(flow_id, str):
            if flow_id in flow_ids:
                errors.append(f"{path}.id: duplicate flow id '{flow_id}'")
            flow_ids.add(flow_id)
        steps = flow.get("steps")
        if not isinstance(steps, list) or not 1 <= len(steps) <= 12:
            errors.append(f"{path}.steps: requires 1-12 ordered transitions")
            continue
        for step_index, step in enumerate(steps):
            step_path = f"{path}.steps[{step_index}]"
            if not isinstance(step, dict):
                errors.append(f"{step_path}: must be an object")
                continue
            _unknown_keys(step, {"path", "payload", "label", "evidence"}, step_path, errors)
            _required_string(step, "label", step_path, errors)
            step_path_id = step.get("path")
            step_payload_id = step.get("payload")
            if step_path_id not in path_ids:
                errors.append(f"{step_path}.path: references unknown path '{step_path_id}'")
            if step_payload_id not in payload_ids:
                errors.append(
                    f"{step_path}.payload: references unknown payload '{step_payload_id}'"
                )
            elif step_path_id in path_payload_ids and step_payload_id not in path_payload_ids[step_path_id]:
                errors.append(
                    f"{step_path}.payload: payload '{step_payload_id}' is not carried by path '{step_path_id}'"
                )
            _validate_evidence(step.get("evidence"), f"{step_path}.evidence", errors)

    return errors


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("scene", type=Path, help="scene contract JSON")
    args = parser.parse_args(argv)
    try:
        document = json.loads(args.scene.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        print(f"Unable to read {args.scene}: {exc}", file=sys.stderr)
        return 1
    errors = validate_scene(document)
    if errors:
        for error in errors:
            print(error, file=sys.stderr)
        return 1
    print(
        f"valid isometric scene: {len(document['nodes'])} cubes, "
        f"{len(document['paths'])} paths, {len(document['flows'])} flows"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
