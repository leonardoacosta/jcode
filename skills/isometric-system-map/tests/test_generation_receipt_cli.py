import hashlib
import json
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
RENDERER = ROOT / "scripts" / "render_canvas.py"
SCENE_FIXTURE = Path(__file__).parent / "fixtures" / "directional-scene.json"
VIEWS_FIXTURE = Path(__file__).parent / "fixtures" / "directional-views.json"
AZURE_THEME = ROOT / "themes" / "azure-topology.js"
TEMPLATE = ROOT / "templates" / "canvas-renderer.html"
SPRITE = ROOT / "assets" / "azure-icons.svg"
REPO_ROOT = ROOT.parents[1]


def sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def used_symbol_ids(scene_path: Path, views_path: Path) -> list[str]:
    scene = json.loads(scene_path.read_text(encoding="utf-8"))
    views = json.loads(views_path.read_text(encoding="utf-8"))
    symbols = {
        node["icon"]
        for node in scene.get("nodes", [])
        if isinstance(node, dict) and isinstance(node.get("icon"), str)
    }
    for pipeline in views.get("pipelines", []):
        if not isinstance(pipeline, dict):
            continue
        for stage in pipeline.get("stages", []):
            if isinstance(stage, dict) and isinstance(stage.get("icon"), str):
                symbols.add(stage["icon"])
    return sorted(symbols)


class GenerationReceiptCliTests(unittest.TestCase):
    def test_render_canvas_writes_deterministic_generation_receipt(self):
        with tempfile.TemporaryDirectory() as directory:
            work = Path(directory)
            scene = work / "directional-scene.json"
            views = work / "directional-views.json"
            theme = work / "azure-topology.js"
            notes = work / "run-notes.txt"
            html = work / "map.html"
            receipt = work / "generation-receipt.json"

            shutil.copyfile(SCENE_FIXTURE, scene)
            shutil.copyfile(VIEWS_FIXTURE, views)
            shutil.copyfile(AZURE_THEME, theme)
            notes.write_text(
                "deterministic receipt contract test\n"
                "operator: stdlib unittest\n"
                "purpose: reproduce generation metadata\n",
                encoding="utf-8",
            )

            base_command = [
                sys.executable,
                str(RENDERER),
                str(scene),
                str(theme),
                str(html),
                "--views",
                str(views),
                "--receipt",
                str(receipt),
                "--run-notes",
                str(notes),
            ]
            result = subprocess.run(base_command, capture_output=True, text=True, check=False)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue(html.exists(), "successful render should write HTML")
            self.assertTrue(receipt.exists(), "successful render should write generation-receipt.json")

            receipt_data = json.loads(receipt.read_text(encoding="utf-8"))
            receipt_json = json.dumps(receipt_data, sort_keys=True).lower()
            self.assertNotIn("timestamp", receipt_json)
            self.assertNotIn("generated_at", receipt_json)
            self.assertNotIn("created_at", receipt_json)

            scene_data = json.loads(scene.read_text(encoding="utf-8"))
            expected_commit = subprocess.check_output(
                ["git", "rev-parse", "HEAD"], cwd=REPO_ROOT, text=True
            ).strip()
            self.assertEqual(
                receipt_data["repository"],
                {
                    "name": scene_data["repository"]["name"],
                    "ref": scene_data["repository"]["ref"],
                    "commit": scene_data["repository"]["commit"],
                },
            )
            self.assertEqual(receipt_data["renderer_command"], base_command)
            self.assertEqual(receipt_data["tool_commit"], expected_commit)
            self.assertEqual(
                receipt_data["sha256"],
                {
                    "scene": sha256_file(scene),
                    "views": sha256_file(views),
                    "theme": sha256_file(theme),
                    "template": sha256_file(TEMPLATE),
                    "sprite": sha256_file(SPRITE),
                    "html": sha256_file(html),
                    "run_notes": sha256_file(notes),
                },
            )
            self.assertEqual(receipt_data["used_symbol_ids"], used_symbol_ids(scene, views))

            first_html = html.read_bytes()
            first_receipt = receipt.read_bytes()
            repeat_result = subprocess.run(base_command, capture_output=True, text=True, check=False)
            self.assertEqual(repeat_result.returncode, 0, repeat_result.stderr)
            self.assertEqual(first_html, html.read_bytes())
            self.assertEqual(first_receipt, receipt.read_bytes())

            notes.write_text(notes.read_text(encoding="utf-8") + "tampered: yes\n", encoding="utf-8")
            tampered_result = subprocess.run(base_command, capture_output=True, text=True, check=False)
            self.assertEqual(tampered_result.returncode, 0, tampered_result.stderr)
            tampered_data = json.loads(receipt.read_text(encoding="utf-8"))
            self.assertNotEqual(receipt_data["sha256"]["run_notes"], tampered_data["sha256"]["run_notes"])
            self.assertEqual(tampered_data["sha256"]["run_notes"], sha256_file(notes))


if __name__ == "__main__":
    unittest.main()
