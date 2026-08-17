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


class NetworkMeasuredAnchorRendererTests(unittest.TestCase):
    def render_network_view(self) -> str:
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "network.html"
            result = subprocess.run(
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
            self.assertEqual(result.returncode, 0, result.stderr)
            return output.read_text(encoding="utf-8")

    def test_network_relationships_measure_source_target_anchors_and_reroute_after_resize(self):
        html = self.render_network_view()

        expected_links = {
            "link-private-endpoint": ("app", "sql-private-endpoint", "direct"),
            "link-private-link-target": ("sql-private-endpoint", "database", "direct"),
            "link-partner-dns": ("app", "partner-api", "inferred"),
            "link-private-data": ("app", "database", "held"),
        }

        missing = []
        for link_id, (source_id, target_id, evidence_level) in expected_links.items():
            relationship = re.search(
                rf'<svg[^>]+data-link-id="{re.escape(link_id)}"[^>]*>.*?</svg>',
                html,
                re.DOTALL,
            )
            if relationship is None:
                missing.append(f"{link_id}: missing SVG relationship")
                continue
            svg = relationship.group(0)
            for attr, expected in (
                ("data-source-node-id", source_id),
                ("data-target-node-id", target_id),
                ("data-evidence-level", evidence_level),
            ):
                if f'{attr}="{expected}"' not in svg:
                    missing.append(f"{link_id}: missing stable {attr}={expected!r}")
            if 'data-connector-shape="orthogonal"' not in svg:
                missing.append(f"{link_id}: missing orthogonal connector semantics")

        for node_id in {node for endpoints in expected_links.values() for node in endpoints[:2]}:
            if f'data-scene-node-id="{node_id}"' not in html:
                missing.append(f"{node_id}: endpoint element is absent from complete text/visual fallback")

        if not re.search(r'<svg[^>]+class="[^"]*(?:connector|overlay)[^"]*"[^>]+data-network-connectors', html, re.DOTALL):
            missing.append("missing dedicated measured connector overlay for resource/container anchors")

        for token in (
            "getBoundingClientRect(",
            "ResizeObserver",
            "data-source-node-id",
            "data-target-node-id",
            "setAttribute(\"d\"",
        ):
            if token not in html:
                missing.append(f"browser reroute script missing {token}")

        if "addEventListener(\"resize\"" not in html and "addEventListener('resize'" not in html:
            missing.append("browser reroute script missing window resize handling")

        static_paths = set(re.findall(r'<path[^>]+data-connector-shape="orthogonal"[^>]+d="([^"]+)"', html))
        if len(static_paths) <= 1:
            missing.append("orthogonal connector geometry is still identical/detached instead of per-link measured")

        summary_pattern = re.compile(
            r'<section[^>]+data-relationship-summary="network"[^>]*>.*?'
            r'<li[^>]+data-link-id="link-private-endpoint"[^>]*>.*?app → sql-private-endpoint.*?direct.*?</li>.*?'
            r'<li[^>]+data-link-id="link-private-link-target"[^>]*>.*?sql-private-endpoint → database.*?direct.*?</li>.*?'
            r'<li[^>]+data-link-id="link-partner-dns"[^>]*>.*?app → partner-api.*?inferred.*?</li>.*?'
            r'<li[^>]+data-link-id="link-private-data"[^>]*>.*?app → database.*?held.*?</li>',
            re.DOTALL,
        )
        self.assertRegex(
            html,
            summary_pattern,
            "complete relationship text fallback with evidence-level semantics must remain present",
        )

        self.assertFalse(
            missing,
            "Network relationship projection should bind each link to measured source/target anchors and reroute orthogonal SVG paths after resize. Missing contract pieces:\n"
            + "\n".join(f"- {item}" for item in missing),
        )


if __name__ == "__main__":
    unittest.main()
