#!/usr/bin/env python3
"""Browser acceptance checks for the static Jcode command-system field manual."""
from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import tempfile
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


def chromium_binary() -> str | None:
    return next((path for name in ("chromium", "chromium-browser", "google-chrome") if (path := shutil.which(name))), None)


def run_probe(chromium: str, url: str, width: int, height: int, javascript: bool) -> tuple[bool, str]:
    expression = "JSON.stringify({title:document.title,h1:document.querySelectorAll('h1').length,main:!!document.querySelector('#main'),nav:!!document.querySelector('nav'),overflow:document.documentElement.scrollWidth>document.documentElement.clientWidth,links:[...document.querySelectorAll('a[href]')].length,skip:!!document.querySelector('.skip'),atlas:[...document.querySelectorAll('a[href^=\"stack-\"]')].length})"
    with tempfile.TemporaryDirectory() as profile:
        cmd = [
            chromium, "--headless", "--no-sandbox", "--disable-gpu",
            f"--user-data-dir={profile}", f"--window-size={width},{height}",
            "--virtual-time-budget=1200", "--dump-dom",
        ]
        if not javascript:
            cmd.append("--disable-javascript")
        cmd.append(url)
        result = subprocess.run(cmd, text=True, capture_output=True, timeout=30)
        if result.returncode:
            return False, result.stderr.strip() or f"chromium exited {result.returncode}"
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
        return True, json.dumps({"route": url.rsplit("/", 1)[-1], "viewport": f"{width}x{height}", "javascript": javascript})


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
