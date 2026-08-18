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


class NetworkViewRendererTests(unittest.TestCase):
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

    def test_views_enabled_network_projection_renders_accessible_azure_topology(self):
        html = self.render_network_view()
        self.assertRegex(html, r'<canvas[^>]+data-sidecar-graph="network"[^>]+role="img"')
        self.assertIn("drawSidecarGraph", html)

        nested_hierarchy = re.compile(
            r'<section[^>]+id="network"[^>]+data-projection="network"[^>]*>.*?'
            r'<article[^>]+data-container-id="subscription-main"[^>]+data-container-kind="subscription"[^>]+aria-label="sample-directional-infra subscription"[^>]*>.*?'
            r'<article[^>]+data-container-id="resource-group-runtime"[^>]+data-container-kind="resource-group"[^>]+aria-label="runtime-resource-group"[^>]*>.*?'
            r'<article[^>]+data-container-id="vnet-runtime"[^>]+data-container-kind="vnet"[^>]+aria-label="runtime-vnet"[^>]*>.*?'
            r'<article[^>]+data-container-id="subnet-runtime"[^>]+data-container-kind="subnet"[^>]+aria-label="runtime-subnet"[^>]*>.*?'
            r'<dl[^>]+class="container-fields"[^>]*>.*?<dt>Kind</dt>\s*<dd>subnet</dd>.*?<dt>CIDR</dt>\s*<dd>10\.42\.1\.0/24</dd>',
            re.DOTALL,
        )
        self.assertRegex(
            html,
            nested_hierarchy,
            "Network projection must render an accessible nested Subscription → Resource Group → VNet → Subnet hierarchy with visible kind fields and the subnet CIDR as a container field.",
        )

        for node_id, label, resource_type, status in (
            ("app", "Application", "Microsoft.Web/sites", "active"),
            ("sql-private-endpoint", "SQL private endpoint", "Microsoft.Network/privateEndpoints", "active"),
            ("database", "SQL database", "Microsoft.Sql/servers/databases", "active"),
        ):
            card_pattern = re.compile(
                rf'<article[^>]+data-scene-node-id="{re.escape(node_id)}"[^>]*>.*?'
                rf'<svg[^>]+width="(?:2[4-9]|[3-9][0-9])"[^>]+height="(?:2[4-9]|[3-9][0-9])"[^>]*>.*?</svg>.*?'
                rf'<h[3-6][^>]*>{re.escape(label)}</h[3-6]>.*?'
                rf'<dd[^>]*>{re.escape(resource_type)}</dd>.*?'
                rf'<dd[^>]*>{re.escape(status)}</dd>',
                re.DOTALL,
            )
            self.assertRegex(html, card_pattern)

        for link_id, label, evidence_level in (
            ("link-private-endpoint", "App to SQL private endpoint", "direct"),
            ("link-private-link-target", "Private link target", "direct"),
            ("link-partner-dns", "Partner DNS resolution", "inferred"),
        ):
            connector_pattern = re.compile(
                rf'<svg[^>]+data-link-id="{re.escape(link_id)}"[^>]+tabindex="0"[^>]+role="img"[^>]*>.*?'
                rf'<title>{re.escape(label)}</title>.*?'
                rf'<path[^>]+data-connector-shape="orthogonal"[^>]*>.*?'
                rf'<text[^>]*>Evidence: {re.escape(evidence_level)}</text>',
                re.DOTALL,
            )
            self.assertRegex(html, connector_pattern)

        summary_pattern = re.compile(
            r'<section[^>]+data-relationship-summary="network"[^>]*>.*?'
            r'<li[^>]+data-link-id="link-private-endpoint"[^>]*>.*?App to SQL private endpoint.*?direct.*?</li>.*?'
            r'<li[^>]+data-link-id="link-private-link-target"[^>]*>.*?Private link target.*?direct.*?</li>.*?'
            r'<li[^>]+data-link-id="link-partner-dns"[^>]*>.*?Partner DNS resolution.*?inferred.*?</li>.*?'
            r'<li[^>]+data-link-id="link-private-data"[^>]*>.*?Private SQL data dependency.*?held.*?</li>',
            re.DOTALL,
        )
        self.assertRegex(
            html,
            summary_pattern,
            "Network projection must keep a complete text relationship summary in document order.",
        )


if __name__ == "__main__":
    unittest.main()
