import copy
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
RENDERER = ROOT / "scripts" / "render_canvas.py"
DIRECTIONAL_FIXTURE = Path(__file__).parent / "fixtures" / "directional-scene.json"
AZURE_THEME = ROOT / "themes" / "azure-topology.js"


class AzureIconFallbackRendererTests(unittest.TestCase):
    def render_scene(self, scene: dict, output: Path) -> subprocess.CompletedProcess[str]:
        scene_path = output.with_suffix(".scene.json")
        scene_path.write_text(json.dumps(scene), encoding="utf-8")
        return subprocess.run(
            [sys.executable, str(RENDERER), str(scene_path), str(AZURE_THEME), str(output)],
            capture_output=True,
            text=True,
            check=False,
        )

    def test_node_icon_may_be_omitted_only_when_resource_type_resolves_to_admitted_family_fallback(self):
        base_scene = json.loads(DIRECTIONAL_FIXTURE.read_text(encoding="utf-8"))

        with tempfile.TemporaryDirectory() as directory:
            output_dir = Path(directory)

            with self.subTest("mapped resource_type resolves and renders fallback symbol with visible metadata"):
                scene = copy.deepcopy(base_scene)
                node = scene["nodes"][0]
                node["resource_type"] = "Microsoft.Web/sites"
                node.pop("icon", None)
                node["label"] = "Customer Web App"
                node["description"] = "Microsoft.Web/sites runtime app service"

                output = output_dir / "mapped.html"
                result = self.render_scene(scene, output)

                self.assertEqual(result.returncode, 0, result.stderr)
                html = output.read_text(encoding="utf-8")
                self.assertIn('"az-vm":"\\u003csvg', html)
                self.assertIn('"resource_type":"Microsoft.Web/sites"', html)
                self.assertIn('aria-label="Customer Web App"', html)
                self.assertIn('title="Microsoft.Web/sites runtime app service', html)

            with self.subTest("unmapped resource_type without icon fails with stable icon-field diagnostic"):
                scene = copy.deepcopy(base_scene)
                node = scene["nodes"][0]
                node["resource_type"] = "Example.Unmapped/widgets"
                node.pop("icon", None)

                output = output_dir / "unmapped.html"
                result = self.render_scene(scene, output)

                self.assertNotEqual(result.returncode, 0)
                self.assertIn(
                    "nodes[0].icon: required when resource_type 'Example.Unmapped/widgets' "
                    "does not resolve through azure-tokens.json resource_type_family "
                    "and family_icon_fallbacks",
                    result.stderr,
                )
                self.assertFalse(output.exists())


if __name__ == "__main__":
    unittest.main()
