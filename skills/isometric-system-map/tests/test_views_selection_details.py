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


class ViewsSelectionDetailsRendererTests(unittest.TestCase):
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

    def test_views_enabled_selection_details_are_persistent_selectable_and_semantically_stable(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "directional-views.html"
            result = self.render_directional_views(output)
            self.assertEqual(result.returncode, 0, result.stderr)
            html = output.read_text()

        missing = []
        tab_panel_spans = [match.span() for match in re.finditer(r'<section[^>]+role="tabpanel".*?</section>', html, re.DOTALL)]
        details_regions = list(
            re.finditer(
                r'<(?:aside|section|div)[^>]+(?=[^>]*(?:id|data-testid)="views-selection-details")(?=[^>]*(?:aria-live|role="region"|aria-label="Selection details"))[^>]*>.*?</(?:aside|section|div)>',
                html,
                re.DOTALL,
            )
        )
        outside_tab_panels = [
            region
            for region in details_regions
            if not any(start <= region.start() < end for start, end in tab_panel_spans)
        ]
        if len(outside_tab_panels) != 1:
            missing.append("one persistent selection details region exists outside all tab panels")

        semantic_ids = {
            "runtime fallback control": "runtime:fallback:show-all",
            "network resource card": "network:resource:app",
            "network connector": "network:relationship:link-private-endpoint",
            "network container": "network:container:resource-group-runtime",
            "ADO stage": "ado:stage:foundation-release:deploy-app",
            "ADO transition": "ado:transition:foundation-release:edge-gate-deploy-app",
        }
        for description, semantic_id in semantic_ids.items():
            if not re.search(
                rf'<(?:button|a|li|article|g|path|div|section)[^>]+(?=[^>]*(?:data-selectable="true"|role="button"|tabindex="0"))(?=[^>]*data-semantic-id="{re.escape(semantic_id)}")',
                html,
            ):
                missing.append(f"{description} exposes stable selectable semantic id {semantic_id}")

        if not re.search(r'(?:selectedSemanticId|selected_semantic_id)\s*=\s*(?:localStorage\.getItem|sessionStorage\.getItem|window\.)', html):
            missing.append("activation stores one selected semantic id across tab switches")
        if not re.search(r'(?:activate|select|setActive)\w*View[\s\S]{0,600}(?:selectedSemanticId|selected_semantic_id)', html):
            missing.append("tab activation preserves and reuses the selected semantic id")

        detail_payloads = {
            "runtime node details": ["runtime:resource:app", "app", "resource", "active"],
            "network relationship details": [
                "network:relationship:link-private-endpoint",
                "App to SQL private endpoint",
                "private-endpoint",
                "forward",
                "infra/modules/sql.bicep:36-96",
                "The subnet hosts a private endpoint connection used by the app for SQL access.",
            ],
            "network container details": [
                "network:container:resource-group-runtime",
                "runtime-resource-group",
                "resource-group",
                "active",
                "infra/main.bicep:19-40",
                "The runtime resources are organized under the represented resource group.",
            ],
            "ADO stage details": [
                "ado:stage:foundation-release:deploy-app",
                "Deploy application",
                "deployment",
                "active",
                ".azuredevops/deploy.yml:31-44",
                "The release pipeline deploys the represented application target.",
            ],
            "ADO transition details": [
                "ado:transition:foundation-release:edge-gate-deploy-app",
                "deploy app",
                "manual",
                ".azuredevops/deploy.yml:31-44",
                "Approval opens the application deployment stage.",
            ],
        }
        for description, required_values in detail_payloads.items():
            for value in required_values:
                if value not in html:
                    missing.append(f"{description} can populate selection details with {value}")

        if re.search(r'data-semantic-id="ado:stage:foundation-release:deploy-app"[^>]+data-node-id="app"', html) is None:
            missing.append("ADO deployment target card reuses scene node id app")
        if re.search(r'data-semantic-id="runtime:resource:app"[^>]+data-node-id="app"', html) is None:
            missing.append("Runtime deployment target card reuses scene node id app")
        if re.search(r'data-semantic-id="network:resource:app"[^>]+data-node-id="app"', html) is None:
            missing.append("Network deployment target card reuses scene node id app")

        self.assertFalse(missing, "missing selectable views details behavior: " + "; ".join(missing))


if __name__ == "__main__":
    unittest.main()
