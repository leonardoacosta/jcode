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

from validate_scene import validate_scene  # noqa: E402


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


def _icon_svgs(scene: dict[str, Any]) -> dict[str, str]:
    used = sorted(
        {
            node["icon"]
            for node in scene.get("nodes", [])
            if isinstance(node, dict) and isinstance(node.get("icon"), str)
        }
    )
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


def render(scene_path: Path, theme_path: Path, output_path: Path) -> None:
    scene = json.loads(scene_path.read_text(encoding="utf-8"))
    errors = validate_scene(scene)
    if errors:
        raise ValueError("\n".join(errors))

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
        "__SCENE_JSON__": _safe_script_json(scene),
        "__ICON_SVGS__": _safe_script_json(_icon_svgs(scene)),
        "__THEME_ADAPTER__": theme_source,
        "__FALLBACK_BUTTONS__": _fallback_buttons(scene),
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
    args = parser.parse_args(argv)
    try:
        render(args.scene, args.theme, args.output)
    except (OSError, json.JSONDecodeError, ValueError) as exc:
        print(exc, file=sys.stderr)
        return 1
    print(f"rendered Canvas isometric map: {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
