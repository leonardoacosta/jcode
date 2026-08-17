import json
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
RENDERER = ROOT / "scripts" / "render_canvas.py"
FIXTURE = Path(__file__).parent / "fixtures" / "valid-scene.json"
AZURE_THEME = ROOT / "themes" / "azure-topology.js"


class AzureRuntimeLabelOverlapTests(unittest.TestCase):
    def render(self, scene: Path, output: Path) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, str(RENDERER), str(scene), str(AZURE_THEME), str(output)],
            capture_output=True,
            text=True,
            check=False,
        )

    def test_full_label_plates_get_deterministic_overlap_avoidance_on_resize(self):
        scene = json.loads(FIXTURE.read_text())
        full_labels = [
            (
                "pipeline",
                "Runtime Managed Identity",
                "Microsoft.ManagedIdentity/userAssignedIdentities",
                "active",
                "RMI",
                "az-managed-identity",
                {"x": 5, "y": 3},
            ),
            (
                "app",
                "Runtime Application Service",
                "Microsoft.Web/sites",
                "active",
                "APP",
                "az-app-service",
                {"x": 6, "y": 3},
            ),
            (
                "database",
                "Runtime SQL Database",
                "Microsoft.Sql/servers/databases",
                "held",
                "SQL",
                "az-sql-database",
                {"x": 5, "y": 4},
            ),
        ]
        # These long full-name/service-type/status plates are wider than their projected
        # spacing in desktop and narrow layouts. The renderer must move plates deterministically
        # instead of shrinking them or falling back to abbreviations.
        for node, (node_id, label, resource_type, status, code, icon, _position) in zip(scene["nodes"][:3], full_labels):
            node.update(
                {
                    "id": node_id,
                    "label": label,
                    "resource_type": resource_type,
                    "status": status,
                    "code": code,
                    "icon": icon,
                }
            )

        with tempfile.TemporaryDirectory() as directory:
            scene_path = Path(directory) / "azure-runtime-label-overlap.json"
            output = Path(directory) / "azure-runtime-label-overlap.html"
            scene_path.write_text(json.dumps(scene))

            rendered = self.render(scene_path, output)
            self.assertEqual(rendered.returncode, 0, rendered.stderr)
            html = output.read_text()

        failures = []
        for _, label, resource_type, status, code, _, _ in full_labels:
            if label not in html:
                failures.append(f"missing full visible label text: {label}")
            if resource_type not in html:
                failures.append(f"missing full visible service type text: {resource_type}")
            if status not in html:
                failures.append(f"missing full visible status text: {status}")
            if f"ctx.fillText(node.code" in html or re.search(rf"fillText\([^)]*{re.escape(code)}", html):
                failures.append(f"label renderer must not fall back to abbreviation {code}")

        overlap_contracts = [
            (
                r"label[^\n]{0,80}(overlap|collis|avoid)",
                "render-time label overlap/collision avoidance is absent",
            ),
            (
                r"label[^\n]{0,80}(rect|box|geometry)",
                "label plate geometry is not retained for collision checks/debugging",
            ),
            (
                r"resizeAndRender[\s\S]{0,260}label[^\n]{0,80}(overlap|collis|avoid|geometry|layout)",
                "label layout is not recomputed as part of responsive resize rendering",
            ),
        ]
        for pattern, message in overlap_contracts:
            if not re.search(pattern, html, flags=re.IGNORECASE):
                failures.append(message)

        self.assertEqual([], failures)


if __name__ == "__main__":
    unittest.main()
