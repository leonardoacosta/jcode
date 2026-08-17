import json
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
RENDERER = ROOT / "scripts" / "render_canvas.py"
AZURE_THEME = ROOT / "themes" / "azure-topology.js"
DIRECTIONAL_SCENE = Path(__file__).parent / "fixtures" / "directional-scene.json"


class AzureRuntimePathEvidenceSemanticsTests(unittest.TestCase):
    maxDiff = None

    def render(self, scene: Path, output: Path) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, str(RENDERER), str(scene), str(AZURE_THEME), str(output)],
            capture_output=True,
            text=True,
            check=False,
        )

    def test_runtime_paths_default_inferred_and_held_semantics_are_public_and_visual(self):
        scene = json.loads(DIRECTIONAL_SCENE.read_text(encoding="utf-8"))
        scene["paths"] = [
            {
                **scene["paths"][0],
                "id": "direct-default-runtime-path",
                "label": "default direct runtime path",
                "payload_ids": ["diagnostic"],
            },
            {
                **scene["paths"][1],
                "id": "explicit-inferred-runtime-path",
                "label": "inferred runtime path",
                "evidence_level": "inferred",
                "payload_ids": ["diagnostic"],
            },
            {
                **scene["paths"][2],
                "id": "explicit-held-runtime-path",
                "label": "held runtime path",
                "evidence_level": "held",
                "payload_ids": ["diagnostic"],
            },
        ]
        step_evidence = [
            {
                "path": "infra/main.bicep",
                "lines": "22-40",
                "claim": "The temporary test flow exercises runtime path evidence rendering semantics.",
            }
        ]
        scene["flows"] = [
            {
                "id": "runtime-path-evidence-flow",
                "label": "Runtime path evidence flow",
                "description": "Exercises path evidence animation semantics.",
                "steps": [
                    {"path": "direct-default-runtime-path", "payload": "diagnostic", "label": "Direct default", "evidence": step_evidence},
                    {"path": "explicit-inferred-runtime-path", "payload": "diagnostic", "label": "Inferred visible", "evidence": step_evidence},
                    {"path": "explicit-held-runtime-path", "payload": "diagnostic", "label": "Held suppressed", "evidence": step_evidence},
                ],
            }
        ]

        with tempfile.TemporaryDirectory() as directory:
            scene_path = Path(directory) / "runtime-path-evidence-scene.json"
            output_path = Path(directory) / "runtime-path-evidence.html"
            scene_path.write_text(json.dumps(scene), encoding="utf-8")

            rendered = self.render(scene_path, output_path)
            self.assertEqual(rendered.returncode, 0, rendered.stderr)
            html = output_path.read_text(encoding="utf-8")

        failures = []
        if not re.search(r'"id":"direct-default-runtime-path"[^}]+"evidence_level":"direct"', html):
            failures.append("omitted runtime path evidence_level must be normalized to direct in the public scene/debug source")
        if not re.search(r'"id":"explicit-inferred-runtime-path"[^}]+"evidence_level":"inferred"', html):
            failures.append("explicit inferred runtime path evidence_level must remain public in source/debug data")
        if not re.search(r'"id":"explicit-held-runtime-path"[^}]+"evidence_level":"held"', html):
            failures.append("explicit held runtime path evidence_level must remain public in source/debug data")
        if "INFERRED" not in html:
            failures.append("inferred paths need a visible INFERRED label, not only a color change")
        if not re.search(r"ctx\.setLineDash\([^)]*evidence_level[^)]*inferred", html):
            failures.append("inferred paths need a visibly non-solid stroke treatment")
        if "HELD · NOT DEPLOYED" not in html:
            failures.append("held paths need explicit HELD · NOT DEPLOYED text, not only a color change")
        if not re.search(r"filter\([^)]*evidence_level[^)]*!== [\"']held[\"']", html):
            failures.append("held paths must be excluded from payload animation steps")
        if "pathEvidence" not in html:
            failures.append("window.__ISO_MAP_DEBUG__ must expose resolved path evidence semantics")

        self.assertEqual([], failures)


if __name__ == "__main__":
    unittest.main()
