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
    return (
        f'<section id="{view_id}" role="tabpanel" aria-labelledby="tab-{view_id}" class="views-panel">'
        f"<h2>{html.escape(title)}</h2>{content}</section>"
    )


def _render_views_shell(views: dict[str, Any]) -> str:
    runtime = views.get("runtime", {}) if isinstance(views.get("runtime"), dict) else {}
    network = views.get("network", {}) if isinstance(views.get("network"), dict) else {}
    pipelines = views.get("pipelines", []) if isinstance(views.get("pipelines"), list) else []

    runtime_parts = ["<p>Runtime view lists the execution resources, flows, and paths represented by the scene.</p>"]
    for label, key in (("Resources", "node_ids"), ("Paths", "path_ids"), ("Flows", "flow_ids")):
        values = runtime.get(key, [])
        if isinstance(values, list):
            runtime_parts.append(f"<h3>{label}</h3><ul>" + "".join(f"<li>{html.escape(str(value))}</li>" for value in values) + "</ul>")

    network_parts = ["<p>Network view preserves the sidecar container hierarchy, memberships, and labeled relationships.</p>"]
    containers = network.get("containers", [])
    if isinstance(containers, list):
        network_parts.append("<h3>Containers</h3><ul>")
        for container in containers:
            if isinstance(container, dict):
                label = html.escape(str(container.get("label", container.get("id", ""))))
                kind = html.escape(str(container.get("kind", "container")))
                network_parts.append(f"<li><strong>{label}</strong> <span>{kind}</span>{_evidence_items(container.get('evidence'))}</li>")
        network_parts.append("</ul>")
    for section, key in (("Memberships", "memberships"), ("Relationships", "links")):
        values = network.get(key, [])
        if isinstance(values, list):
            network_parts.append(f"<h3>{section}</h3><ul>")
            for item in values:
                if isinstance(item, dict):
                    label = item.get("label") or " → ".join(str(item.get(k, "")) for k in ("source_id", "target_id") if item.get(k)) or str(item.get("node_id", item.get("id", "relationship")))
                    network_parts.append(f"<li><strong>{html.escape(str(label))}</strong>{_evidence_items(item.get('evidence'))}</li>")
            network_parts.append("</ul>")

    ado_parts = ["<p>ADO view lists delivery pipelines, stages, citations, and claims from the represented repository.</p>"]
    for pipeline in pipelines:
        if not isinstance(pipeline, dict):
            continue
        ado_parts.append(f"<section><h3>{html.escape(str(pipeline.get('label', pipeline.get('id', 'Pipeline'))))}</h3><ol>")
        for stage in pipeline.get("stages", []):
            if isinstance(stage, dict):
                label = html.escape(str(stage.get("label", stage.get("id", "Stage"))))
                stage_type = html.escape(str(stage.get("stage_type", "stage")))
                ado_parts.append(f"<li><strong>{label}</strong> <span>{stage_type}</span>{_evidence_items(stage.get('evidence'))}</li>")
        ado_parts.append("</ol></section>")

    valid_views = {"runtime", "network", "ado"}
    default_view = views.get("default_view") if views.get("default_view") in valid_views else "runtime"
    panels = "".join([
        _views_panel("Runtime", "runtime", "".join(runtime_parts)),
        _views_panel("Network", "network", "".join(network_parts)),
        _views_panel("ADO", "ado", "".join(ado_parts)),
    ])
    tabs = "".join(
        f'<button id="tab-{view_id}" type="button" role="tab" aria-controls="{view_id}" '
        f'aria-selected="{str(view_id == default_view).lower()}" '
        f'tabindex="{0 if view_id == default_view else -1}">{label}</button>'
        for view_id, label in (("runtime", "Runtime"), ("network", "Network"), ("ado", "ADO"))
    )
    return f'<aside class="views-shell" aria-label="Repository views"><div role="tablist" aria-label="Views">{tabs}</div>{panels}</aside>'


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
                f"data-target-kind=\"{kind}\" "
                f"data-target-id=\"{html.escape(item_id, quote=True)}\" "
                f"aria-label=\"{html.escape(label, quote=True)}\" "
                f"title=\"{html.escape(detail, quote=True)}\">"
                f"{html.escape(label)}</button>"
            )
    return "\n        ".join(rows)


def render(scene_path: Path, theme_path: Path, output_path: Path, views_path: Path | None = None) -> None:
    scene = json.loads(scene_path.read_text(encoding="utf-8"))
    errors = validate_scene(scene)
    if errors:
        raise ValueError("\n".join(errors))
    render_scene = _scene_with_azure_icon_fallbacks(scene)

    views = None
    if views_path is not None:
        views = json.loads(views_path.read_text(encoding="utf-8"))
        view_errors = validate_views(views, scene)
        if view_errors:
            raise ValueError("\n".join(view_errors))
        mismatch_errors = _repository_mismatches(scene, views)
        if mismatch_errors:
            raise ValueError("\n".join(mismatch_errors))

    theme_source = theme_path.read_text(encoding="utf-8").strip()
    if not theme_source:
        raise ValueError(f"{theme_path}: theme adapter is empty")
    theme_name = _theme_name(theme_source, theme_path)

    template_path = SKILL_DIR / "templates" / "canvas-renderer.html"
    template = template_path.read_text(encoding="utf-8")
    repository = scene["repository"]
    title = f"{repository['name']} isometric system map"
    summary = repository["summary"]
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
        "__VIEWS_SCRIPT__": f"const VIEWS_SIDECAR = {_safe_script_json(views)};" if views is not None else "",
        "__ICON_SVGS__": _safe_script_json(_icon_svgs(render_scene, views)),
        "__THEME_ADAPTER__": theme_source,
        "__FALLBACK_BUTTONS__": _fallback_buttons(render_scene),
        "__VIEWS_SHELL__": _render_views_shell(views) if views is not None else "",
    }
    for marker, value in replacements.items():
        template = template.replace(marker, value)
    unresolved = sorted(marker for marker in replacements if marker in template)
    if unresolved:
        raise ValueError(f"unresolved template markers: {', '.join(unresolved)}")

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(template, encoding="utf-8")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("scene", type=Path, help="validated scene JSON")
    parser.add_argument("theme", type=Path, help="Canvas theme adapter JavaScript")
    parser.add_argument("output", type=Path, help="self-contained HTML output")
    parser.add_argument("--views", type=Path, help="optional directional views sidecar JSON")
    args = parser.parse_args(argv)
    try:
        render(args.scene, args.theme, args.output, args.views)
    except (OSError, json.JSONDecodeError, ValueError) as exc:
        print(exc, file=sys.stderr)
        return 1
    print(f"rendered Canvas isometric map: {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
