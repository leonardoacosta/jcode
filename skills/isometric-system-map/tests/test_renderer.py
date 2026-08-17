import importlib.util
import json
import subprocess
import sys
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
RENDERER = ROOT / "scripts" / "render.py"
FIXTURE = Path(__file__).parent / "fixtures" / "valid.json"


def load_renderer():
    spec = importlib.util.spec_from_file_location("isometric_system_map_renderer", RENDERER)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"Unable to load renderer at {RENDERER}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class RendererContractTests(unittest.TestCase):
    def setUp(self):
        self.document = json.loads(FIXTURE.read_text())

    def test_valid_document_has_no_validation_errors(self):
        renderer = load_renderer()
        self.assertEqual(renderer.validate_document(self.document), [])

    def test_validation_rejects_dangling_edges_and_uncited_nodes(self):
        renderer = load_renderer()
        broken = json.loads(json.dumps(self.document))
        broken["nodes"][1]["source_paths"] = []
        broken["edges"][0]["to"] = "missing-node"
        errors = renderer.validate_document(broken)
        self.assertTrue(any("nodes[1].source_paths" in error for error in errors), errors)
        self.assertTrue(any("edges[0].to" in error for error in errors), errors)

    def test_validation_rejects_unknown_flow_edges_and_duplicate_positions(self):
        renderer = load_renderer()
        broken = json.loads(json.dumps(self.document))
        broken["flows"][0]["steps"][0]["edge"] = "missing-edge"
        broken["nodes"][2]["position"] = broken["nodes"][1]["position"]
        errors = renderer.validate_document(broken)
        self.assertTrue(any("flows[0].steps[0].edge" in error for error in errors), errors)
        self.assertTrue(any("nodes[2].position" in error for error in errors), errors)

    def test_validation_rejects_uncited_flow_steps(self):
        renderer = load_renderer()
        broken = json.loads(json.dumps(self.document))
        broken["flows"][0]["steps"][0]["source_paths"] = []
        errors = renderer.validate_document(broken)
        self.assertTrue(
            any(
                error == "flows[0].steps[0].source_paths: requires at least one repo-relative citation"
                for error in errors
            ),
            errors,
        )

    def test_render_is_self_contained_interactive_and_cited(self):
        renderer = load_renderer()
        html = renderer.render_document(self.document)
        self.assertIn("<!doctype html>", html.lower())
        self.assertIn("window.ISO_MAP_DATA", html)
        self.assertIn("Pause flow", html)
        self.assertIn("Trace one step", html)
        self.assertIn("Reset view", html)
        self.assertIn("What it does", html)
        self.assertIn("How it is built", html)
        self.assertIn("prefers-reduced-motion", html)
        self.assertIn('"aria-label": `Path ${edge.label}', html)
        self.assertIn('className = "legend-building"', html)
        self.assertIn("infra/modules/sql.bicep:1-96", html)
        self.assertNotIn("https://", html)
        self.assertNotIn("<script src=", html)
        self.assertNotIn("<link rel=\"stylesheet\"", html)

    def test_render_is_deterministic(self):
        renderer = load_renderer()
        self.assertEqual(
            renderer.render_document(self.document),
            renderer.render_document(self.document),
        )

    def test_cli_validate_and_render(self):
        validate = subprocess.run(
            [sys.executable, str(RENDERER), "--validate", str(FIXTURE)],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(validate.returncode, 0, validate.stderr)
        self.assertIn("valid", validate.stdout.lower())

        out = Path(self.id().replace(".", "-") + ".html")
        try:
            render = subprocess.run(
                [sys.executable, str(RENDERER), str(FIXTURE), str(out)],
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(render.returncode, 0, render.stderr)
            self.assertTrue(out.exists())
            self.assertIn("Deploy pipeline", out.read_text())
        finally:
            out.unlink(missing_ok=True)


if __name__ == "__main__":
    unittest.main()
