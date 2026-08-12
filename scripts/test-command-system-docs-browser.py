#!/usr/bin/env python3
"""Browser acceptance checks for the static Jcode command-system field manual."""
from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import tempfile
from html.parser import HTMLParser
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_SITE = ROOT / "docs/diagrams/jcode-command-system"
ROUTES = [
    "index.html", "command-lifecycle.html", "lane-protocol.html",
    "apply-orchestration.html", "model-routing.html", "evaluation-tournament.html",
    "telemetry-results.html", "agent-stack.html", "stack-surface.html",
    "stack-orchestration.html", "stack-context.html", "stack-model.html",
    "stack-tools.html", "stack-runtime.html", "stack-memory.html",
    "daily-driven-ecosystem.html",
]
VIEWPORTS = [(1440, 1000), (393, 852)]
COMMAND_JOURNEY = [
    "index.html", "command-lifecycle.html", "lane-protocol.html",
    "apply-orchestration.html", "model-routing.html", "evaluation-tournament.html",
    "telemetry-results.html", "daily-driven-ecosystem.html", "index.html",
]
ATLAS_JOURNEY = [
    "index.html", "agent-stack.html", "stack-surface.html", "stack-orchestration.html",
    "stack-context.html", "stack-model.html", "stack-tools.html", "stack-runtime.html",
    "stack-memory.html", "agent-stack.html", "daily-driven-ecosystem.html", "index.html",
]


class RenderedDocument(HTMLParser):
    def __init__(self) -> None:
        super().__init__()
        self.links: list[tuple[str, str]] = []
        self.current_links: list[str] = []
        self.focusable = 0

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        values = {key: value or "" for key, value in attrs}
        if tag == "a" and values.get("href"):
            self.links.append((values["href"], values.get("aria-current", "")))
            self.focusable += 1
            if values.get("aria-current") == "page":
                self.current_links.append(values["href"])
        elif tag in {"button", "summary", "input", "select", "textarea"}:
            self.focusable += 1


def chromium_binary() -> str | None:
    return next((path for name in ("chromium", "chromium-browser", "google-chrome") if (path := shutil.which(name))), None)


def run_probe(chromium: str, url: str, width: int, height: int, javascript: bool) -> tuple[bool, str]:
    expression = "JSON.stringify({title:document.title,h1:document.querySelectorAll('h1').length,main:!!document.querySelector('#main'),nav:!!document.querySelector('nav'),overflow:document.documentElement.scrollWidth>document.documentElement.clientWidth,links:[...document.querySelectorAll('a[href]')].length,skip:!!document.querySelector('.skip'),atlas:[...document.querySelectorAll('a[href^=\"stack-\"]')].length})"
    with tempfile.TemporaryDirectory() as profile:
        cmd = [
            chromium, "--headless", "--no-sandbox", "--disable-gpu",
            f"--user-data-dir={profile}", f"--window-size={width},{height}",
            "--virtual-time-budget=1200", "--enable-logging=stderr", "--dump-dom",
        ]
        if not javascript:
            cmd.append("--disable-javascript")
        cmd.append(url)
        result = subprocess.run(cmd, text=True, capture_output=True, timeout=30)
        if result.returncode:
            return False, result.stderr.strip() or f"chromium exited {result.returncode}"
        severe = [
            line for line in result.stderr.splitlines()
            if "CONSOLE" in line.upper() or "NET::ERR_" in line.upper()
        ]
        if severe:
            return False, "console/network failure: " + severe[0].strip()
        html = result.stdout
        checks = {
            "title": "<title>" in html.lower(),
            "h1": html.lower().count("<h1") == 1,
            "main": 'id="main"' in html,
            "nav": "<nav" in html.lower(),
            "skip": 'class="skip"' in html,
        }
        failed = [name for name, passed in checks.items() if not passed]
        if failed:
            return False, f"rendered DOM missing {', '.join(failed)}"
        # Chromium's dump is the real rendered document. A narrow viewport is also
        # checked for the responsive navigation contract directly in computed DOM
        # shape by requiring the menu to remain present.
        if width == 393 and 'class="chapter-menu"' not in html:
            return False, "mobile chapter menu absent"
        rendered = RenderedDocument()
        rendered.feed(html)
        route = url.rsplit("/", 1)[-1]
        if len(rendered.current_links) != 1:
            return False, f"expected one aria-current link, found {len(rendered.current_links)}"
        if not rendered.current_links[0].split("#", 1)[0].endswith(route):
            return False, f"aria-current points to {rendered.current_links[0]!r}, not {route!r}"
        if rendered.focusable < 4:
            return False, f"insufficient keyboard-focusable controls: {rendered.focusable}"
        return True, json.dumps({"route": url.rsplit("/", 1)[-1], "viewport": f"{width}x{height}", "javascript": javascript})


def validate_journey(site: Path, journey: list[str], label: str) -> list[str]:
    errors: list[str] = []
    for source, target in zip(journey, journey[1:]):
        text = (site / source).read_text()
        if f'href="{target}"' not in text:
            errors.append(f"[DOCS-INDEX] {source}#{label}: cannot follow journey edge to {target}")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--site", type=Path, default=DEFAULT_SITE)
    args = parser.parse_args()
    site = args.site.resolve()
    chromium = chromium_binary()
    if not chromium:
        print("[DOCS-A11Y] <browser>#runtime: Chromium is unavailable")
        return 1
    errors: list[str] = []
    errors.extend(validate_journey(site, COMMAND_JOURNEY, "command-journey"))
    errors.extend(validate_journey(site, ATLAS_JOURNEY, "atlas-journey"))
    for route in ROUTES:
        path = site / route
        if not path.exists():
            errors.append(f"[DOCS-INDEX] {route}#route: missing route")
            continue
        url = path.as_uri()
        for width, height in VIEWPORTS:
            for javascript in (True, False):
                ok, detail = run_probe(chromium, url, width, height, javascript)
                if not ok:
                    errors.append(f"[DOCS-A11Y] {route}#{width}x{height}: {detail}")
    for error in errors:
        print(error)
    print(f"command-system-docs-browser: {'PASS' if not errors else 'FAIL'} ({len(ROUTES)} routes; {len(VIEWPORTS)} viewports; JS on/off)")
    return len(errors)


if __name__ == "__main__":
    raise SystemExit(main())
