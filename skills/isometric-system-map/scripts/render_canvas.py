#!/usr/bin/env python3
"""Render a validated isometric scene through a self-contained Canvas theme adapter."""

from __future__ import annotations

import argparse
import copy
import hashlib
import html
import json
import re
import sys
import subprocess
import xml.etree.ElementTree as ET
from pathlib import Path
from typing import Any

SCRIPT_DIR = Path(__file__).resolve().parent
SKILL_DIR = SCRIPT_DIR.parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

from validate_scene import azure_icon_fallback_for_resource_type, validate_scene  # noqa: E402
from validate_views import validate_views  # noqa: E402


THEME_NAME_RE = re.compile(r"\bname\s*:\s*([\"'])(?P<name>.+?)\1")
SVG_NS = "http://www.w3.org/2000/svg"
ET.register_namespace("", SVG_NS)


def _safe_script_json(value: Any) -> str:
    """Serialize JSON without allowing data to terminate the script element."""

    return (
        json.dumps(value, ensure_ascii=False, separators=(",", ":"))
        .replace("&", "\\u0026")
        .replace("<", "\\u003c")
        .replace(">", "\\u003e")
        .replace("\u2028", "\\u2028")
        .replace("\u2029", "\\u2029")
    )


def _semantic_hash(scene: Any) -> str:
    canonical = json.dumps(scene, sort_keys=True, separators=(",", ":"), ensure_ascii=False)
    return hashlib.sha256(canonical.encode("utf-8")).hexdigest()


def _theme_name(source: str, path: Path) -> str:
    match = THEME_NAME_RE.search(source)
    if match is None or not match.group("name").strip():
        raise ValueError(f"{path}: theme adapter must declare name: \"...\"")
    return match.group("name").strip()


def _icon_svgs(scene: dict[str, Any], views: dict[str, Any] | None = None) -> dict[str, str]:
    used_icons = {
        node["icon"]
        for node in scene.get("nodes", [])
        if isinstance(node, dict) and isinstance(node.get("icon"), str)
    }
    if isinstance(views, dict):
        for pipeline in views.get("pipelines", []):
            if not isinstance(pipeline, dict):
                continue
            for stage in pipeline.get("stages", []):
                if isinstance(stage, dict) and isinstance(stage.get("icon"), str):
                    used_icons.add(stage["icon"])
    used = sorted(used_icons)
    if not used:
        return {}

    sprite_path = SKILL_DIR / "assets" / "azure-icons.svg"
    source = sprite_path.read_text(encoding="utf-8")
    source = re.sub(r"<!--.*?-->", "", source, flags=re.DOTALL)
    root = ET.fromstring(source)
    symbols = {
        symbol.get("id"): symbol
        for symbol in root.iter(f"{{{SVG_NS}}}symbol")
        if symbol.get("id")
    }
    standalone: dict[str, str] = {}
    for icon_id in used:
        symbol = symbols.get(icon_id)
        if symbol is None:
            raise ValueError(f"{sprite_path}: missing symbol '{icon_id}'")
        attributes = {key: value for key, value in symbol.attrib.items() if key != "id"}
        icon_root = ET.Element(f"{{{SVG_NS}}}svg", attributes)
        for child in symbol:
            icon_root.append(copy.deepcopy(child))
        standalone[icon_id] = ET.tostring(icon_root, encoding="unicode", short_empty_elements=True)
    return standalone


def _scene_with_azure_icon_fallbacks(scene: dict[str, Any]) -> dict[str, Any]:
    """Return a render-only scene copy with deterministic Azure icon fallbacks filled in."""

    normalized = copy.deepcopy(scene)
    nodes = normalized.get("nodes")
    if not isinstance(nodes, list):
        return normalized
    for node in nodes:
        if not isinstance(node, dict) or "icon" in node:
            continue
        icon = azure_icon_fallback_for_resource_type(node.get("resource_type"))
        if icon is not None:
            node["icon"] = icon
    return normalized


def _scene_with_normalized_path_evidence(scene: dict[str, Any]) -> dict[str, Any]:
    """Return a render-only scene copy with omitted path evidence made explicit."""

    normalized = copy.deepcopy(scene)
    paths = normalized.get("paths")
    if not isinstance(paths, list):
        return normalized
    for path in paths:
        if isinstance(path, dict) and "evidence_level" not in path:
            path["evidence_level"] = "direct"
    path_evidence_by_id = {
        path.get("id"): path.get("evidence_level", "direct")
        for path in paths
        if isinstance(path, dict)
    }
    for flow in normalized.get("flows", []):
        if not isinstance(flow, dict):
            continue
        for step in flow.get("steps", []):
            if isinstance(step, dict) and step.get("path") in path_evidence_by_id:
                step["evidence_level"] = path_evidence_by_id[step["path"]]
    return normalized


def _filter_items_by_ids(items: Any, ids: Any) -> Any:
    if not isinstance(items, list) or not isinstance(ids, list):
        return items
    by_id = {item.get("id"): item for item in items if isinstance(item, dict)}
    return [copy.deepcopy(by_id[item_id]) for item_id in ids if item_id in by_id]


def _runtime_projected_scene(scene: dict[str, Any], views: dict[str, Any] | None) -> dict[str, Any]:
    """Return the canvas scene projected to views.runtime when a sidecar is present."""

    if views is None:
        return scene
    runtime = views.get("runtime")
    if not isinstance(runtime, dict):
        return scene

    projected = copy.deepcopy(scene)
    projected["nodes"] = _filter_items_by_ids(scene.get("nodes", []), runtime.get("node_ids"))
    projected["paths"] = _filter_items_by_ids(scene.get("paths", []), runtime.get("path_ids"))
    if "flow_ids" in runtime:
        projected["flows"] = _filter_items_by_ids(scene.get("flows", []), runtime.get("flow_ids"))
    return projected


def _path_evidence(scene: dict[str, Any]) -> list[dict[str, str]]:
    paths = scene.get("paths", [])
    if not isinstance(paths, list):
        return []
    return [
        {"id": str(path.get("id", "")), "evidence_level": str(path.get("evidence_level", "direct"))}
        for path in paths
        if isinstance(path, dict)
    ]


def _repository_mismatches(scene: dict[str, Any], views: dict[str, Any]) -> list[str]:
    scene_repo = scene.get("repository", {})
    views_repo = views.get("repository", {})
    errors: list[str] = []
    for key in ("name", "ref", "commit"):
        if scene_repo.get(key) != views_repo.get(key):
            errors.append(f"repository.{key}: views sidecar must match scene repository")
    return errors


def _evidence_items(entries: Any) -> str:
    if not isinstance(entries, list) or not entries:
        return ""
    items: list[str] = []
    for entry in entries:
        if not isinstance(entry, dict):
            continue
        path = html.escape(str(entry.get("path", "")))
        lines = html.escape(str(entry.get("lines", "")))
        claim = html.escape(str(entry.get("claim", "")))
        citation = f"{path}:{lines}" if path or lines else "Evidence"
        items.append(f"<li><cite>{citation}</cite> — <span>{claim}</span></li>")
    return "<ul>" + "".join(items) + "</ul>" if items else ""


def _views_panel(title: str, view_id: str, content: str) -> str:
    projection_attr = f' data-projection="{html.escape(view_id, quote=True)}"' if view_id == "network" else ""
    return (
        f'<section id="{view_id}" role="tabpanel" aria-labelledby="tab-{view_id}" class="views-panel"{projection_attr}>'
        f"<h2>{html.escape(title)}</h2>{content}</section>"
    )


def _node_card(node: dict[str, Any], membership: dict[str, Any], icons: dict[str, str]) -> str:
    node_id = html.escape(str(node.get("id", "")), quote=True)
    label = html.escape(str(node.get("label", node.get("id", "Resource"))))
    icon = icons.get(str(node.get("icon", "")), "")
    if icon:
        icon = icon.replace("<svg ", '<svg width="32" height="32" ', 1)
    fields = (
        "<dl class=\"resource-fields\">"
        f"<dt>Resource type</dt><dd>{html.escape(str(node.get('resource_type', 'unknown')))}</dd>"
        f"<dt>Status</dt><dd>{html.escape(str(node.get('status', 'unknown')))}</dd>"
        "</dl>"
    )
    return (
        f'<article class="network-resource-card" data-semantic-id="network:resource:{node_id}" data-selectable="true" tabindex="0" data-node-id="{node_id}" data-scene-node-id="{node_id}" data-network-anchor="node">'
        f"{icon}<h4>{label}</h4>{fields}{_evidence_items(membership.get('evidence'))}</article>"
    )


def _network_relationship_svg(link: dict[str, Any], ordinal: int = 0) -> str:
    link_id = html.escape(str(link.get("id", "")), quote=True)
    label = html.escape(str(link.get("label", link.get("id", "Relationship"))))
    evidence_level = html.escape(str(link.get("evidence_level", "direct")))
    source_id = html.escape(str(link.get("source_id", "")), quote=True)
    target_id = html.escape(str(link.get("target_id", "")), quote=True)
    source_kind = html.escape(str(link.get("source_kind", "node")), quote=True)
    target_kind = html.escape(str(link.get("target_kind", "node")), quote=True)
    direction = str(link.get("direction", "forward"))
    start_marker_id = f"network-arrow-start-{link_id}"
    end_marker_id = f"network-arrow-end-{link_id}"
    marker_start = f' marker-start="url(#{start_marker_id})"' if direction in {"reverse", "both"} else ""
    marker_end = f' marker-end="url(#{end_marker_id})"' if direction in {"forward", "both"} else ""
    dash = ' stroke-dasharray="8 6"' if evidence_level in {"inferred", "held"} else ""
    route_lane = (ordinal % 5) - 2
    lane_offset = route_lane * 10
    evidence_text = "held (not deployed)" if evidence_level == "held" else evidence_level
    return (
        f'<svg class="network-relationship evidence-{evidence_level}" data-semantic-id="network:relationship:{link_id}" data-selectable="true" data-link-id="{link_id}" tabindex="0" role="img" '
        f'data-source-node-id="{source_id}" data-target-node-id="{target_id}" '
        f'data-source-kind="{source_kind}" data-target-kind="{target_kind}" data-evidence-level="{evidence_level}" '
        f'data-route-ordinal="{ordinal}" data-route-lane="{route_lane}" data-route-lane-offset="{lane_offset}" '
        'width="360" height="72" viewBox="0 0 360 72">'
        f'<title>{label}</title>'
        f'<defs><marker id="{start_marker_id}" viewBox="0 0 10 10" refX="1" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M10 0 0 5 10 10Z" fill="currentColor"/></marker>'
        f'<marker id="{end_marker_id}" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 10 5 0 10Z" fill="currentColor"/></marker></defs>'
        f'<path data-connector-shape="orthogonal" data-semantic-id="network:relationship:{link_id}" data-selectable="true" tabindex="0" d="M{16 + ordinal * 3} {36 + ordinal} H{128 + ordinal * 2} V{20 + ordinal} H{232 - ordinal} V{36 + ordinal} H{344 - ordinal * 2}" fill="none" stroke="currentColor" stroke-width="2"{dash}{marker_start}{marker_end}/>'
        f'<text x="16" y="14">{label}</text><text x="16" y="64">Evidence: {evidence_text}</text></svg>'
    )


def _network_summary_item(link: dict[str, Any]) -> str:
    link_id = html.escape(str(link.get("id", "")), quote=True)
    label = html.escape(str(link.get("label", link.get("id", "Relationship"))))
    evidence_level = html.escape(str(link.get("evidence_level", "direct")))
    source = html.escape(str(link.get("source_id", "")))
    target = html.escape(str(link.get("target_id", "")))
    return f'<li data-link-id="{link_id}"><strong>{label}</strong> <span>{source} → {target}</span> <span>{evidence_level}</span>{_evidence_items(link.get("evidence"))}</li>'


def _iso_cube(x: int, y: int, label: str, subtitle: str, color: str, semantic_id: str, node_id: str = "") -> str:
    safe_id = html.escape(semantic_id, quote=True)
    safe_node = html.escape(node_id, quote=True)
    return (
        f'<g class="sidecar-iso-node" data-semantic-id="{safe_id}" data-selectable="true" tabindex="0" data-node-id="{safe_node}" transform="translate({x} {y})">'
        f'<title>{html.escape(label)}</title><polygon points="0,18 34,0 68,18 34,36" fill="{color}" opacity=".95"/>'
        f'<polygon points="0,18 34,36 34,72 0,54" fill="{color}" opacity=".7"/><polygon points="34,36 68,18 68,54 34,72" fill="{color}" opacity=".52"/>'
        f'<text x="34" y="30" text-anchor="middle">{html.escape(label)}</text><text class="sidecar-iso-subtitle" x="34" y="88" text-anchor="middle">{html.escape(subtitle)}</text></g>'
    )


def _network_isometric_svg(containers: list[Any], memberships: list[Any], links: list[Any], nodes_by_id: dict[str, dict[str, Any]]) -> str:
    container_items = [c for c in containers if isinstance(c, dict)]
    memberships_by_container: dict[str, list[dict[str, Any]]] = {}
    for membership in memberships:
        if isinstance(membership, dict):
            memberships_by_container.setdefault(str(membership.get("container_id", "")), []).append(membership)
    positions: dict[str, tuple[int, int]] = {}
    terrain: list[str] = []
    cubes: list[str] = []
    for index, container in enumerate(container_items):
        col, row = index % 4, index // 4
        cx, cy = 150 + col * 250, 110 + row * 170
        cid = str(container.get("id", ""))
        positions[cid] = (cx, cy)
        terrain.append(f'<polygon class="sidecar-iso-terrain" points="{cx-105},{cy+45} {cx},{cy-8} {cx+105},{cy+45} {cx},{cy+98}" fill="none" stroke="currentColor" stroke-width="2"/><text class="sidecar-iso-terrain-label" x="{cx}" y="{cy+122}" text-anchor="middle">{html.escape(str(container.get("label", cid)))}</text>')
        for member_index, membership in enumerate(memberships_by_container.get(cid, [])):
            node_id = str(membership.get("node_id", ""))
            node = nodes_by_id.get(node_id)
            if not node:
                continue
            nx, ny = cx - 76 + (member_index % 3) * 76, cy - 34 + (member_index // 3) * 92
            positions[node_id] = (nx + 34, ny + 36)
            cubes.append(_iso_cube(nx, ny, str(node.get("code", node_id))[:12], str(node.get("role", "resource")), "#1683b8", f"network:resource:{node_id}", node_id))
    connector_parts = []
    for link in links:
        if not isinstance(link, dict):
            continue
        source, target = positions.get(str(link.get("source_id"))), positions.get(str(link.get("target_id")))
        if source and target:
            connector_parts.append(f'<path class="sidecar-iso-link evidence-{html.escape(str(link.get("evidence_level", "direct")), quote=True)}" d="M{source[0]},{source[1]} L{target[0]},{target[1]}" data-link-id="{html.escape(str(link.get("id", "")), quote=True)}" data-selectable="true" tabindex="0"><title>{html.escape(str(link.get("label", link.get("id", "Relationship"))))}</title></path>')
    defs = '<defs><marker id="sidecar-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 10 5 0 10Z" fill="currentColor"/></marker></defs>'
    return f'<svg class="sidecar-isometric-graph network-isometric-graph" viewBox="0 0 1120 620" role="img" aria-label="Isometric Network topology graph">{defs}<g class="sidecar-iso-links">{"".join(connector_parts)}</g>{"".join(terrain)}{"".join(cubes)}</svg>'


def _ado_isometric_svg(pipeline: dict[str, Any], ranks: dict[str, int], lanes: dict[str, int]) -> str:
    stages = [stage for stage in pipeline.get("stages", []) if isinstance(stage, dict)]
    positions: dict[str, tuple[int, int]] = {}
    cubes: list[str] = []
    for stage in stages:
        stage_id = str(stage.get("id", ""))
        x, y = 105 + ranks.get(stage_id, 0) * 180, 105 + lanes.get(stage_id, 0) * 120
        positions[stage_id] = (x, y)
        kind = str(stage.get("stage_type", "stage"))
        color = "#9866d8" if kind in {"gate", "approval", "manual"} else "#287fb8"
        cubes.append(_iso_cube(x - 34, y - 36, stage_id[:14], kind, color, f"ado:stage:{pipeline.get('id', '')}:{stage_id}"))
    links = []
    for edge in pipeline.get("edges", []):
        if isinstance(edge, dict) and str(edge.get("source_id")) in positions and str(edge.get("target_id")) in positions:
            a, b = positions[str(edge["source_id"])], positions[str(edge["target_id"])]
            links.append(f'<path class="sidecar-iso-link ado-link" d="M{a[0]},{a[1]} L{b[0]},{b[1]}" data-transition-id="{html.escape(str(edge.get("id", "")), quote=True)}" data-selectable="true" tabindex="0"><title>{html.escape(str(edge.get("label", edge.get("id", "Transition"))))}</title></path>')
    defs = '<defs><marker id="sidecar-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 10 5 0 10Z" fill="currentColor"/></marker></defs>'
    return f'<svg class="sidecar-isometric-graph ado-isometric-graph" viewBox="0 0 1120 520" role="img" aria-label="Isometric Azure DevOps pipeline graph">{defs}<g class="sidecar-iso-links">{"".join(links)}</g>{"".join(cubes)}</svg>'


def _pipeline_stage_ranks(pipeline: dict[str, Any]) -> dict[str, int]:
    stages = pipeline.get("stages", [])
    edges = pipeline.get("edges", [])
    stage_ids = [str(stage.get("id", "")) for stage in stages if isinstance(stage, dict)]
    ranks = dict.fromkeys(stage_ids, 0)
    stage_order = {stage_id: index for index, stage_id in enumerate(stage_ids)}
    outgoing = {stage_id: [] for stage_id in stage_ids}
    indegree = dict.fromkeys(stage_ids, 0)
    for edge in edges:
        if not isinstance(edge, dict):
            continue
        source = str(edge.get("source_id", ""))
        target = str(edge.get("target_id", ""))
        if source not in ranks or target not in ranks:
            continue
        outgoing[source].append(target)
        indegree[target] += 1

    ready = [stage_id for stage_id in stage_ids if indegree[stage_id] == 0]
    visited = 0
    while ready:
        source = ready.pop(0)
        visited += 1
        for target in outgoing[source]:
            ranks[target] = max(ranks[target], ranks[source] + 1)
            indegree[target] -= 1
            if indegree[target] == 0:
                ready.append(target)
        ready.sort(key=stage_order.__getitem__)
    if visited != len(stage_ids):
        raise ValueError(f"pipelines.{pipeline.get('id', '')}: stage graph must be acyclic")
    return ranks


def _pipeline_stage_lanes(stages: list[Any], ranks: dict[str, int]) -> dict[str, int]:
    """Resolve rank-local lanes by declared lane, then sidecar order."""

    used_by_rank: dict[int, set[int]] = {}
    lanes: dict[str, int] = {}
    for stage in stages:
        if not isinstance(stage, dict):
            continue
        stage_id = str(stage.get("id", ""))
        rank = ranks.get(stage_id, 0)
        used = used_by_rank.setdefault(rank, set())
        lane = stage["lane"] if isinstance(stage.get("lane"), int) else 0
        while lane in used:
            lane += 1
        lanes[stage_id] = lane
        used.add(lane)
    return lanes


def _pipeline_stage_card(
    pipeline_id: str,
    stage: dict[str, Any],
    rank: int,
    layout_lane: int,
    icons: dict[str, str],
    nodes_by_id: dict[str, dict[str, Any]],
) -> str:
    stage_id = html.escape(str(stage.get("id", "")), quote=True)
    label = html.escape(str(stage.get("label", stage.get("id", "Stage"))))
    stage_type_raw = str(stage.get("stage_type", "stage"))
    stage_type = html.escape(stage_type_raw)
    status_raw = str(stage.get("status", "unknown"))
    status = html.escape(status_raw)
    declared_lane = stage.get("lane") if isinstance(stage.get("lane"), int) else None
    extra_attrs = ""
    if declared_lane is not None:
        extra_attrs += f' data-lane="{declared_lane}"'
    if "parallel_group" in stage:
        extra_attrs += f' data-parallel-group="{html.escape(str(stage.get("parallel_group")), quote=True)}"'
    target_node_id = str(stage.get("target_node_id", ""))
    if target_node_id:
        extra_attrs += f' data-target-node-id="{html.escape(target_node_id, quote=True)}"'
    icon = icons.get(str(stage.get("icon", "")), "")
    if icon:
        icon = icon.replace("<svg ", '<svg width="32" height="32" ', 1)
    manual_note = ""
    if stage_type_raw.lower() in {"gate", "approval", "manual"} or "approval" in str(stage.get("label", "")).lower():
        manual_note = '<p class="manual-state">Manual approval state requires human approval.</p>'
    elif stage_type_raw.lower() == "held" or status_raw.lower() == "held":
        manual_note = '<p class="held-state">Held stage. This delivery action is not deployed.</p>'
    target_field = ""
    if target_node_id:
        target = nodes_by_id.get(target_node_id, {})
        target_label = html.escape(str(target.get("label", target_node_id)))
        target_type = html.escape(str(target.get("resource_type", "scene resource")))
        target_field = (
            f'<dt>Deployment target</dt><dd data-semantic-id="network:resource:{html.escape(target_node_id, quote=True)}" data-selectable="true" data-node-id="{html.escape(target_node_id, quote=True)}" data-scene-node-id="{html.escape(target_node_id, quote=True)}">'
            f'{target_label} <span class="ado-target-type">({target_type})</span></dd>'
        )
    parallel_fields = ""
    if "parallel_group" in stage:
        parallel_fields += f'<dt>Parallel group</dt><dd>{html.escape(str(stage.get("parallel_group")))}</dd>'
    if declared_lane is not None:
        parallel_fields += f'<dt>Lane</dt><dd>{declared_lane}</dd>'
    fields = (
        '<dl class="ado-stage-fields">'
        f'<dt>Stage ID</dt><dd>{stage_id}</dd>'
        f'<dt>Type</dt><dd>{stage_type}</dd>'
        f'<dt>Status</dt><dd>{status}</dd>'
        f'{parallel_fields}{target_field}'
        '</dl>'
    )
    badges = (
        '<p class="ado-stage-badges" aria-label="Stage type and status">'
        f'<span>Type: {stage_type}</span><span>Status: {status}</span></p>'
    )
    evidence = _evidence_items(stage.get("evidence"))
    evidence_disclosure = (
        f'<details class="ado-stage-evidence"><summary>Evidence</summary>{evidence}</details>'
        if evidence
        else ""
    )
    return (
        f'<article class="ado-stage-card stage-{html.escape(stage_type_raw, quote=True)} status-{html.escape(status_raw, quote=True)}" data-semantic-id="ado:stage:{html.escape(pipeline_id, quote=True)}:{stage_id}" data-selectable="true" data-node-id="{html.escape(target_node_id, quote=True)}" tabindex="0" '
        f'data-pipeline-id="{html.escape(pipeline_id, quote=True)}" data-stage-id="{stage_id}" tabindex="0" '
        f'data-rank="{rank}" data-layout-lane="{layout_lane}" '
        f'style="--ado-column:{rank + 1};--ado-lane:{layout_lane + 1}"{extra_attrs}>'
        f'{icon}<h4>{label}</h4>{badges}{fields}{manual_note}{evidence_disclosure}</article>'
    )


def _pipeline_transition_svg(edge: dict[str, Any]) -> str:
    edge_id = html.escape(str(edge.get("id", "")), quote=True)
    source = html.escape(str(edge.get("source_id", "")), quote=True)
    target = html.escape(str(edge.get("target_id", "")), quote=True)
    kind_raw = str(edge.get("kind", "transition"))
    kind = html.escape(kind_raw, quote=True)
    label = html.escape(str(edge.get("label", edge.get("id", "Transition"))))
    human_note = " Requires human approval." if kind_raw in {"approval", "manual"} else ""
    if kind_raw == "held":
        human_note = " Held transition. This delivery action is not deployed."
    evidence = _evidence_items(edge.get("evidence"))
    return (
        f'<svg class="ado-transition transition-{kind}" data-semantic-id="ado:transition:foundation-release:{edge_id}" data-selectable="true" data-transition-id="{edge_id}" '
        f'data-source-stage-id="{source}" data-target-stage-id="{target}" '
        f'data-transition-kind="{kind}" tabindex="0" role="img" width="360" height="220" viewBox="0 0 360 220">'
        f'<title>{label}</title><desc>{label}. Kind: {kind}.{human_note}</desc>'
        f'<defs><marker id="ado-arrow-{edge_id}" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 10 5 0 10Z" fill="currentColor"/></marker></defs>'
        f'<path data-transition-shape="directed" data-semantic-id="ado:transition:foundation-release:{edge_id}" data-selectable="true" tabindex="0" d="M16 42 H344" fill="none" stroke="currentColor" stroke-width="2" marker-end="url(#ado-arrow-{edge_id})"/>'
        f'<text x="16" y="20">{label}</text><text x="16" y="72">Kind: {kind}.{html.escape(human_note)}</text>'
        f'<foreignObject x="12" y="84" width="336" height="124"><div class="ado-transition-evidence" xmlns="http://www.w3.org/1999/xhtml"><strong>Evidence</strong>{evidence}</div></foreignObject></svg>'
    )


def _render_ado_pipeline(
    pipeline: dict[str, Any],
    icons: dict[str, str],
    nodes_by_id: dict[str, dict[str, Any]],
) -> str:
    pipeline_id = str(pipeline.get("id", ""))
    label = html.escape(str(pipeline.get("label", pipeline.get("id", "Pipeline"))))
    ranks = _pipeline_stage_ranks(pipeline)
    stages = pipeline.get("stages", []) if isinstance(pipeline.get("stages"), list) else []
    edges = pipeline.get("edges", []) if isinstance(pipeline.get("edges"), list) else []
    lanes = _pipeline_stage_lanes(stages, ranks)
    stage_cards = "".join(
        _pipeline_stage_card(
            pipeline_id,
            stage,
            ranks.get(str(stage.get("id", "")), 0),
            lanes.get(str(stage.get("id", "")), 0),
            icons,
            nodes_by_id,
        )
        for stage in stages
        if isinstance(stage, dict)
    )
    transitions = "".join(_pipeline_transition_svg(edge) for edge in edges if isinstance(edge, dict))
    summary_stages = "".join(
        f'<li>{html.escape(str(stage.get("label", stage.get("id", "Stage"))))} · {html.escape(str(stage.get("stage_type", "stage")))}</li>'
        for stage in stages
        if isinstance(stage, dict)
    )
    summary_edges = "".join(
        f'<li>{html.escape(str(edge.get("source_id", "")))} → {html.escape(str(edge.get("label", edge.get("id", "Transition"))))} → {html.escape(str(edge.get("target_id", "")))}</li>'
        for edge in edges
        if isinstance(edge, dict)
    )
    return (
        f'<section class="ado-pipeline" data-pipeline-id="{html.escape(pipeline_id, quote=True)}"><h3>{label}</h3>'
        f'{_ado_isometric_svg(pipeline, ranks, lanes)}'
        f'<div class="ado-stage-graph" role="list" aria-label="{label} DAG stages">{stage_cards}</div>'
        f'<div class="ado-transitions" aria-label="{label} directed transitions">{transitions}</div>'
        f'<section data-stage-transition-summary="ado"><h4>Ordered stage and transition summary</h4><ol>{summary_stages}</ol><ol>{summary_edges}</ol></section></section>'
    )


def _render_network_container(
    container: dict[str, Any],
    children_by_parent: dict[str | None, list[dict[str, Any]]],
    memberships_by_container: dict[str, list[dict[str, Any]]],
    nodes_by_id: dict[str, dict[str, Any]],
    icons: dict[str, str],
) -> str:
    container_id_raw = str(container.get("id", ""))
    container_id = html.escape(container_id_raw, quote=True)
    kind = html.escape(str(container.get("kind", "container")), quote=True)
    label = html.escape(str(container.get("label", container_id_raw)), quote=True)
    fields = [
        "<dl class=\"container-fields\">",
        f"<dt>Kind</dt><dd>{html.escape(str(container.get('kind', 'container')))}</dd>",
        f"<dt>Status</dt><dd>{html.escape(str(container.get('status', 'unknown')))}</dd>",
    ]
    if container.get("cidr") is not None:
        fields.append(f"<dt>CIDR</dt><dd>{html.escape(str(container.get('cidr')))}</dd>")
    fields.append("</dl>")
    cards = []
    for membership in memberships_by_container.get(container_id_raw, []):
        node = nodes_by_id.get(str(membership.get("node_id", "")))
        if node is not None:
            cards.append(_node_card(node, membership, icons))
    children = [
        _render_network_container(child, children_by_parent, memberships_by_container, nodes_by_id, icons)
        for child in children_by_parent.get(container_id_raw, [])
    ]
    return (
        f'<article class="network-container" data-semantic-id="network:container:{container_id}" data-selectable="true" tabindex="0" data-container-id="{container_id}" data-scene-node-id="{container_id}" data-network-anchor="container" data-container-kind="{kind}" aria-label="{label}">'
        f'<h3>{html.escape(str(container.get("label", container_id_raw)))}</h3>{"".join(fields)}'
        f'{_evidence_items(container.get("evidence"))}{"".join(cards)}{"".join(children)}</article>'
    )


def _render_network_projection(network: dict[str, Any], scene: dict[str, Any], icons: dict[str, str]) -> str:
    containers = network.get("containers", []) if isinstance(network.get("containers"), list) else []
    memberships = network.get("memberships", []) if isinstance(network.get("memberships"), list) else []
    links = network.get("links", []) if isinstance(network.get("links"), list) else []
    children_by_parent: dict[str | None, list[dict[str, Any]]] = {}
    for container in containers:
        if isinstance(container, dict):
            parent = str(container["parent_id"]) if container.get("parent_id") is not None else None
            children_by_parent.setdefault(parent, []).append(container)
    memberships_by_container: dict[str, list[dict[str, Any]]] = {}
    for membership in memberships:
        if isinstance(membership, dict):
            memberships_by_container.setdefault(str(membership.get("container_id", "")), []).append(membership)
    nodes_by_id = {str(node.get("id")): node for node in scene.get("nodes", []) if isinstance(node, dict)}
    container_ids = {str(container.get("id")) for container in containers if isinstance(container, dict)}
    hierarchy = "".join(
        _render_network_container(container, children_by_parent, memberships_by_container, nodes_by_id, icons)
        for container in children_by_parent.get(None, [])
    )
    contained_node_ids = {
        str(membership.get("node_id"))
        for membership in memberships
        if isinstance(membership, dict)
    }
    uncontained = "".join(
        _node_card(node, {}, icons)
        for node in scene.get("nodes", [])
        if isinstance(node, dict) and str(node.get("id")) not in contained_node_ids
    )
    endpoint_links: list[dict[str, Any]] = []
    for link in links:
        if not isinstance(link, dict):
            continue
        enriched = copy.deepcopy(link)
        enriched["source_kind"] = "container" if str(link.get("source_id")) in container_ids else "node"
        enriched["target_kind"] = "container" if str(link.get("target_id")) in container_ids else "node"
        endpoint_links.append(enriched)
    relationships = "".join(_network_relationship_svg(link, index) for index, link in enumerate(endpoint_links))
    summary = "".join(_network_summary_item(link) for link in links if isinstance(link, dict))
    return (
        "<p>Network view preserves the sidecar container hierarchy, memberships, and labeled relationships.</p>"
        f'{_network_isometric_svg(containers, memberships, links, nodes_by_id)}'
        f'<div class="network-diagram"><svg class="network-connectors-overlay" data-network-connectors aria-label="Measured network connectors" role="group">{relationships}</svg>'
        f'<div class="network-hierarchy">{hierarchy}<section class="network-uncontained" aria-label="Uncontained and external endpoints">{uncontained}</section></div></div>'
        f'<section data-relationship-summary="network"><h3>Relationship summary</h3><ol>{summary}</ol></section>'
    )


def _render_views_shell(views: dict[str, Any], scene: dict[str, Any], icons: dict[str, str]) -> str:
    runtime = views.get("runtime", {}) if isinstance(views.get("runtime"), dict) else {}
    network = views.get("network", {}) if isinstance(views.get("network"), dict) else {}
    pipelines = views.get("pipelines", []) if isinstance(views.get("pipelines"), list) else []

    runtime_parts = ["<p>Runtime view lists the execution resources, flows, and paths represented by the scene.</p>"]
    for label, key in (("Resources", "node_ids"), ("Paths", "path_ids"), ("Flows", "flow_ids")):
        values = runtime.get(key, [])
        if isinstance(values, list):
            runtime_parts.append(f'<h3>{label}</h3><ul>' + "".join(f'<li data-semantic-id="runtime:resource:{html.escape(str(value), quote=True)}" data-selectable="true" data-node-id="{html.escape(str(value), quote=True)}">{html.escape(str(value))}</li>' if key == "node_ids" else f'<li>{html.escape(str(value))}</li>' for value in values) + "</ul>")

    network_parts = [_render_network_projection(network, scene, icons)]

    ado_parts = ["<p>ADO view lists delivery pipelines, stages, citations, and claims from the represented repository.</p>"]
    nodes_by_id = {str(node.get("id", "")): node for node in scene.get("nodes", []) if isinstance(node, dict)}
    for pipeline in pipelines:
        if isinstance(pipeline, dict):
            ado_parts.append(_render_ado_pipeline(pipeline, icons, nodes_by_id))

    valid_views = {"runtime", "network", "ado"}
    default_view = views.get("default_view") if views.get("default_view") in valid_views else "runtime"
    panels = "".join([
        _views_panel("Runtime", "runtime", "".join(runtime_parts)),
        _views_panel("Network", "network", "".join(network_parts)),
        _views_panel("ADO", "ado", "".join(ado_parts)),
    ])
    tabs = "".join(
        f'<button id="tab-{view_id}" class="views-tab" type="button" role="tab" aria-controls="{view_id}" '
        f'aria-selected="{str(view_id == default_view).lower()}" '
        f'tabindex="{0 if view_id == default_view else -1}">{label}</button>'
        for view_id, label in (("runtime", "Runtime"), ("network", "Network"), ("ado", "ADO"))
    )
    return f'<aside class="views-shell" aria-label="Repository views"><div class="views-tablist" role="tablist" aria-label="Views">{tabs}</div>{panels}<section id="views-selection-details" data-testid="views-selection-details" role="region" aria-live="polite" aria-label="Selection details"><h3>Selection details</h3><p data-selection-empty>Select a runtime resource, network relationship, container, or ADO stage to inspect it.</p><div data-selection-content hidden></div></section></aside>'


def _evidence_text(item: dict[str, Any]) -> str:
    entries = item.get("evidence")
    if not isinstance(entries, list):
        return ""
    return "; ".join(
        f"{entry.get('path', '')}:{entry.get('lines', '')} — {entry.get('claim', '')}"
        for entry in entries
        if isinstance(entry, dict)
    )


def _fallback_buttons(scene: dict[str, Any]) -> str:
    rows: list[str] = []
    traffic = scene.get("traffic")
    traffic_layers = traffic.get("layers", []) if isinstance(traffic, dict) else []
    for kind, items in (
        ("traffic-layer", traffic_layers),
        ("area", scene.get("areas", [])),
        ("node", scene.get("nodes", [])),
        ("path", scene.get("paths", [])),
    ):
        for item in items:
            item_id = str(item["id"])
            label = str(item.get("label", item_id))
            description = str(item.get("description", ""))
            evidence = _evidence_text(item)
            detail = " · ".join(value for value in (description, evidence) if value)
            rows.append(
                "<button type=\"button\" "
                f"id=\"fallback-{kind}-{html.escape(item_id, quote=True)}\" "
                f"data-target-kind=\"{kind}\" data-semantic-id=\"runtime:fallback:show-all\" data-selectable=\"true\" "
                f"data-target-id=\"{html.escape(item_id, quote=True)}\" "
                f"aria-label=\"{html.escape(label, quote=True)}\" "
                f"title=\"{html.escape(detail, quote=True)}\">"
                f"{html.escape(label)}</button>"
            )
    return "\n        ".join(rows)


def render(scene_path: Path, theme_path: Path, output_path: Path, views_path: Path | None = None, receipt_path: Path | None = None, run_notes_path: Path | None = None, renderer_command: list[str] | None = None) -> None:
    scene = json.loads(scene_path.read_text(encoding="utf-8"))
    errors = validate_scene(scene)
    if errors:
        raise ValueError("\n".join(errors))
    views = None
    if views_path is not None:
        views = json.loads(views_path.read_text(encoding="utf-8"))
        view_errors = validate_views(views, scene)
        if view_errors:
            raise ValueError("\n".join(view_errors))
        mismatch_errors = _repository_mismatches(scene, views)
        if mismatch_errors:
            raise ValueError("\n".join(mismatch_errors))

    full_render_scene = _scene_with_azure_icon_fallbacks(_scene_with_normalized_path_evidence(scene))
    render_scene = _runtime_projected_scene(full_render_scene, views)

    theme_source = theme_path.read_text(encoding="utf-8").strip()
    if not theme_source:
        raise ValueError(f"{theme_path}: theme adapter is empty")
    theme_name = _theme_name(theme_source, theme_path)

    template_path = SKILL_DIR / "templates" / "canvas-renderer.html"
    template = template_path.read_text(encoding="utf-8")
    repository = scene["repository"]
    title = f"{repository['name']} isometric system map"
    summary = repository["summary"]
    icon_svgs = _icon_svgs(full_render_scene, views)
    replacements = {
        "__DOCUMENT_TITLE__": html.escape(title),
        "__SCENE_TITLE__": html.escape(repository["name"]),
        "__SCENE_SUBTITLE__": html.escape(
            f"{repository['ref']} · {repository['commit'][:12]} · {theme_name}"
        ),
        "__SCENE_SUMMARY__": html.escape(summary),
        "__SCENE_HASH__": _semantic_hash(scene),
        "__VIEWS_ATTR__": f' data-views-sha256="{_semantic_hash(views)}"' if views is not None else "",
        "__SCENE_JSON__": _safe_script_json(render_scene),
        "__PATH_EVIDENCE__": _safe_script_json(_path_evidence(render_scene)),
        "__VIEWS_SCRIPT__": f"const VIEWS_SIDECAR = {_safe_script_json(views)};" if views is not None else "",
        "__ICON_SVGS__": _safe_script_json(icon_svgs),
        "__THEME_ADAPTER__": theme_source,
        "__FALLBACK_BUTTONS__": _fallback_buttons(render_scene),
        "__VIEWS_SHELL__": _render_views_shell(views, full_render_scene, icon_svgs) if views is not None else "",
    }
    for marker, value in replacements.items():
        template = template.replace(marker, value)
    unresolved = sorted(marker for marker in replacements if marker in template)
    if unresolved:
        raise ValueError(f"unresolved template markers: {', '.join(unresolved)}")

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(template, encoding="utf-8")
    if receipt_path is not None:
        def digest(path: Path) -> str:
            return hashlib.sha256(path.read_bytes()).hexdigest()
        sprite = SKILL_DIR / "assets" / "azure-icons.svg"
        used = set()
        for node in scene.get("nodes", []):
            if isinstance(node, dict) and isinstance(node.get("icon"), str):
                used.add(node["icon"])
        for pipeline in (views or {}).get("pipelines", []):
            for stage in pipeline.get("stages", []) if isinstance(pipeline, dict) else []:
                if isinstance(stage, dict) and isinstance(stage.get("icon"), str):
                    used.add(stage["icon"])
        commit = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=SKILL_DIR.parents[1], text=True).strip()
        receipt = {
            "repository": {key: scene["repository"][key] for key in ("name", "ref", "commit")},
            "renderer_command": renderer_command or [],
            "tool_commit": commit,
            "sha256": {"scene": digest(scene_path), "views": digest(views_path) if views_path else None, "theme": digest(theme_path), "template": digest(template_path), "sprite": digest(sprite), "html": digest(output_path), "run_notes": digest(run_notes_path) if run_notes_path else None},
            "used_symbol_ids": sorted(used),
        }
        receipt_path.parent.mkdir(parents=True, exist_ok=True)
        receipt_path.write_text(json.dumps(receipt, sort_keys=True, indent=2) + "\n", encoding="utf-8")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("scene", type=Path, help="validated scene JSON")
    parser.add_argument("theme", type=Path, help="Canvas theme adapter JavaScript")
    parser.add_argument("output", type=Path, help="self-contained HTML output")
    parser.add_argument("--views", type=Path, help="optional directional views sidecar JSON")
    parser.add_argument("--receipt", type=Path)
    parser.add_argument("--run-notes", type=Path)
    args = parser.parse_args(argv)
    try:
        render(args.scene, args.theme, args.output, args.views, args.receipt, args.run_notes, [sys.executable, *sys.argv])
    except (OSError, json.JSONDecodeError, ValueError) as exc:
        print(exc, file=sys.stderr)
        return 1
    print(f"rendered Canvas isometric map: {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
