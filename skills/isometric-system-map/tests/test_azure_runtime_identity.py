import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
RENDERER = ROOT / "scripts" / "render_canvas.py"
FIXTURE = Path(__file__).parent / "fixtures" / "valid-scene.json"
AZURE_THEME = ROOT / "themes" / "azure-topology.js"
DARK_THEME = ROOT / "themes" / "dark-technical.js"


class AzureRuntimeIdentityTests(unittest.TestCase):
    def render(self, scene: Path, theme: Path, output: Path) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, str(RENDERER), str(scene), str(theme), str(output)],
            capture_output=True,
            text=True,
            check=False,
        )

    def test_azure_runtime_identity_prioritizes_admitted_icon_and_full_visible_text(self):
        scene = json.loads(FIXTURE.read_text())
        scene["nodes"][0].update(
            {
                "label": "Runtime Managed Identity",
                "code": "RMI",
                "resource_type": "Microsoft.ManagedIdentity/userAssignedIdentities",
                "icon": "az-managed-identity",
            }
        )

        with tempfile.TemporaryDirectory() as directory:
            scene_path = Path(directory) / "azure-runtime-identity.json"
            azure_output = Path(directory) / "azure-runtime-identity.html"
            dark_output = Path(directory) / "dark-runtime-identity.html"
            scene_path.write_text(json.dumps(scene))

            azure = self.render(scene_path, AZURE_THEME, azure_output)
            dark = self.render(scene_path, DARK_THEME, dark_output)
            self.assertEqual(azure.returncode, 0, azure.stderr)
            self.assertEqual(dark.returncode, 0, dark.stderr)

            azure_html = azure_output.read_text()
            dark_html = dark_output.read_text()
            failures = []
            if '"az-managed-identity":"\\u003csvg' not in azure_html:
                failures.append("Azure output must embed the admitted managed identity SVG")
            if "const iconWorldSize = Math.min(mass.width, mass.depth) * .78;" not in azure_html:
                failures.append("Azure roof icon footprint must be 0.78 of the cube roof")
            if "ctx.fillText(node.code" in azure_html:
                failures.append("Azure theme must not draw node.code as the visible node label")
            if "ctx.fillText(node.label" not in azure_html:
                failures.append("Azure visible node label must include node.label without hover")
            if "ctx.fillText(node.resource_type" not in azure_html:
                failures.append("Azure visible node label must include node.resource_type without hover")
            if "ctx.fillText(node.code" not in dark_html:
                failures.append("Non-Azure themes must retain code-based labels")

            self.assertEqual([], failures)


if __name__ == "__main__":
    unittest.main()
