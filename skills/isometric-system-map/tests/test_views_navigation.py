import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
RENDERER = ROOT / "scripts" / "render_canvas.py"
DIRECTIONAL_SCENE = Path(__file__).parent / "fixtures" / "directional-scene.json"
DIRECTIONAL_VIEWS = Path(__file__).parent / "fixtures" / "directional-views.json"
AZURE_THEME = ROOT / "themes" / "azure-topology.js"


class ViewsNavigationRendererTests(unittest.TestCase):
    def render_directional_views(self, output: Path) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [
                sys.executable,
                str(RENDERER),
                str(DIRECTIONAL_SCENE),
                str(AZURE_THEME),
                str(output),
                "--views",
                str(DIRECTIONAL_VIEWS),
            ],
            capture_output=True,
            text=True,
            check=False,
        )

    def test_views_enabled_native_tabs_are_accessible_navigable_and_static_fallback_ordered(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "directional-views.html"
            result = self.render_directional_views(output)
            self.assertEqual(result.returncode, 0, result.stderr)
            html = output.read_text()

        checks = {
            "runtime tab button exposes inactive roving tabindex state": re.search(
                r'<button[^>]+id="tab-runtime"[^>]+role="tab"[^>]+aria-selected="false"[^>]+tabindex="-1"',
                html,
            ),
            "network tab button exposes default selected roving tabindex state": re.search(
                r'<button[^>]+id="tab-network"[^>]+role="tab"[^>]+aria-selected="true"[^>]+tabindex="0"',
                html,
            ),
            "ado tab button exposes inactive roving tabindex state": re.search(
                r'<button[^>]+id="tab-ado"[^>]+role="tab"[^>]+aria-selected="false"[^>]+tabindex="-1"',
                html,
            ),
            "runtime panel is present without static hidden for no-JS fallback": re.search(
                r'<section(?=[^>]+id="runtime")(?=[^>]+role="tabpanel")(?![^>]+hidden)[^>]*>',
                html,
            ),
            "network panel is present without static hidden for no-JS fallback": re.search(
                r'<section(?=[^>]+id="network")(?=[^>]+role="tabpanel")(?![^>]+hidden)[^>]*>',
                html,
            ),
            "ado panel is present without static hidden for no-JS fallback": re.search(
                r'<section(?=[^>]+id="ado")(?=[^>]+role="tabpanel")(?![^>]+hidden)[^>]*>',
                html,
            ),
            "default_view drives initial activation": "default_view" in html and re.search(
                r'(?:activate|select|setActive)\w*View\([^)]*VIEWS_SIDECAR\.default_view',
                html,
            ),
            "enhancement script sets panel.hidden during activation": re.search(
                r'panel\.hidden\s*=.*(?:active|selected|view)',
                html,
            ),
            "hash fragments #runtime #network #ado are parsed": re.search(
                r'location\.hash|new URL\([^)]*hash',
                html,
            )
            and "runtime" in html
            and "network" in html
            and "ado" in html,
            "tab activation updates browser history fragment": re.search(
                r'history\.(?:pushState|replaceState)\([^)]*#',
                html,
            ),
            "Left and Right arrow keys move tab focus": "ArrowLeft" in html and "ArrowRight" in html,
            "Home and End keys move tab focus": "Home" in html and "End" in html,
            "Enter and Space activate the focused tab": "Enter" in html and re.search(r'event\.key\s*===\s*["\'] ["\']|event\.code\s*===\s*["\']Space["\']|["\']Space["\']', html),
        }
        missing = [description for description, passed in checks.items() if not passed]

        shell_match = re.search(r'<aside class="views-shell".*?</aside>', html, re.DOTALL)
        if shell_match is None:
            missing.append("complete static runtime/network/ado panel text remains present for no-JS fallback")
        else:
            shell_html = shell_match.group(0)
            ordered_static_text = [
                "Runtime view lists the execution resources",
                "apim",
                "Network view preserves the sidecar container hierarchy",
                "runtime-resource-group",
                "ADO view lists delivery pipelines",
                "Foundation release",
                "Deploy telemetry wiring",
            ]
            positions = [shell_html.find(text) for text in ordered_static_text]
            if any(position == -1 for position in positions):
                missing.append("complete static runtime/network/ado panel text remains present for no-JS fallback")
            elif positions != sorted(positions):
                missing.append("complete static runtime/network/ado panel text remains in document order for no-JS fallback")

        self.assertFalse(missing, "missing views-enabled native tab behavior: " + "; ".join(missing))


if __name__ == "__main__":
    unittest.main()
