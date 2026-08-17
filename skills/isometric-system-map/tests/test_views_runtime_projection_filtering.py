import json
import re
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


def extract_script_json(html: str, name: str):
    match = re.search(rf"const {name} = (?P<json>\{{.*?\}});", html, re.DOTALL)
    if match is None:
        raise AssertionError(f"missing public script JSON constant {name}")
    return json.loads(match.group("json"))


def fallback_ids(html: str, kind: str) -> set[str]:
    return set(
        re.findall(
            rf'<button[^>]+data-target-kind="{re.escape(kind)}"[^>]+data-target-id="([^"]+)"',
            html,
        )
    )


class ViewsRuntimeProjectionFilteringTests(unittest.TestCase):
    def test_views_runtime_filters_canvas_public_projection_while_static_sidecars_keep_full_scene_context(self):
        """Public-output contract: runtime filtering is observable in embedded scene JSON and native controls."""

        source_scene = json.loads(SCENE_FIXTURE.read_text())
        source_views = json.loads(VIEWS_FIXTURE.read_text())
        views = json.loads(json.dumps(source_views))
        views["runtime"]["node_ids"].remove("telemetry")
        views["runtime"]["path_ids"].remove("emit-telemetry")
        views["runtime"]["flow_ids"].remove("observe-runtime")

        required_traffic_members = {
            member_id
            for layer in source_scene["traffic"]["layers"]
            for member_id in layer["member_ids"]
        }
        self.assertLessEqual(required_traffic_members, set(views["runtime"]["node_ids"]))
        self.assertEqual({layer["id"] for layer in source_scene["traffic"]["layers"]}, {"ingress", "projects", "data-access", "external-services"})

        with tempfile.TemporaryDirectory() as directory:
            directory_path = Path(directory)
            views_path = directory_path / "directional-views-without-observability-runtime.json"
            output_path = directory_path / "directional-runtime-filtered.html"
            views_path.write_text(json.dumps(views))

            result = subprocess.run(
                [
                    sys.executable,
                    str(RENDERER),
                    str(SCENE_FIXTURE),
                    str(AZURE_THEME),
                    str(output_path),
                    "--views",
                    str(views_path),
                ],
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            html = output_path.read_text()
            rendered_scene = extract_script_json(html, "SCENE")
            rendered_views = extract_script_json(html, "VIEWS_SIDECAR")

            self.assertEqual(rendered_views["runtime"], views["runtime"])

            self.assertEqual(
                [node["id"] for node in rendered_scene["nodes"]],
                views["runtime"]["node_ids"],
                "Canvas Runtime projection node order must be filtered through views.runtime.node_ids",
            )
            self.assertEqual(
                [path["id"] for path in rendered_scene["paths"]],
                views["runtime"]["path_ids"],
                "Canvas Runtime projection paths must be filtered through views.runtime.path_ids",
            )
            self.assertEqual(
                [flow["id"] for flow in rendered_scene["flows"]],
                views["runtime"]["flow_ids"],
                "Flow controls must be sourced from the filtered Runtime projection",
            )
            self.assertEqual(
                {member_id for layer in rendered_scene["traffic"]["layers"] for member_id in layer["member_ids"]},
                required_traffic_members,
                "Every traffic-layer member must remain in the Runtime projection",
            )
            self.assertEqual(fallback_ids(html, "node"), set(views["runtime"]["node_ids"]))
            self.assertEqual(fallback_ids(html, "path"), set(views["runtime"]["path_ids"]))
            self.assertNotIn("telemetry", fallback_ids(html, "node"))
            self.assertNotIn("emit-telemetry", fallback_ids(html, "path"))

            non_canvas_html = re.sub(r"<canvas\b[^>]*>.*?</canvas>", "", html, flags=re.DOTALL)
            self.assertIn("The runtime resources attach to the application VNet.", non_canvas_html)
            self.assertIn("Validate Bicep", non_canvas_html)
            self.assertIn("monitoring workspace is represented", non_canvas_html)
            self.assertNotIn("Observe runtime", non_canvas_html)
