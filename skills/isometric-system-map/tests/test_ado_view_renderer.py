import json
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


class AdoViewRendererTests(unittest.TestCase):
    def render_ado_view(self) -> tuple[str, dict]:
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "ado.html"
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
            return output.read_text(encoding="utf-8"), json.loads(DIRECTIONAL_VIEWS.read_text(encoding="utf-8"))

    def test_ado_projection_renders_accessible_pipeline_stage_graph_from_fixture(self):
        html, views = self.render_ado_view()
        self.assertRegex(html, r'<canvas[^>]+data-sidecar-graph="ado"[^>]+role="img"')
        self.assertIn("drawSidecarGraph", html)
        pipeline = views["pipelines"][0]

        ado_panel_pattern = re.compile(
            r'<section[^>]+id="ado"[^>]+role="tabpanel"[^>]*>.*?</section>',
            re.DOTALL,
        )
        ado_panel_match = ado_panel_pattern.search(html)
        self.assertIsNotNone(ado_panel_match, "ADO projection must render an ADO tab panel.")
        ado_panel = ado_panel_match.group(0)

        ranks = {stage["id"]: 0 for stage in pipeline["stages"]}
        changed = True
        while changed:
            changed = False
            for edge in pipeline["edges"]:
                next_rank = ranks[edge["source_id"]] + 1
                if next_rank > ranks[edge["target_id"]]:
                    ranks[edge["target_id"]] = next_rank
                    changed = True
        self.assertEqual(ranks["deploy-app"], ranks["deploy-telemetry"])

        previous_stage_position = -1
        for stage in pipeline["stages"]:
            stage_pattern = re.compile(
                rf'<article[^>]+data-pipeline-id="{re.escape(pipeline["id"])}"[^>]+'
                rf'data-stage-id="{re.escape(stage["id"])}"[^>]+'
                rf'tabindex="0"[^>]+'
                rf'data-rank="{ranks[stage["id"]]}"[^>]*>.*?'
                rf'<svg[^>]+width="(?:2[4-9]|[3-9][0-9])"[^>]+height="(?:2[4-9]|[3-9][0-9])"[^>]*>.*?</svg>.*?'
                rf'<h[3-6][^>]*>{re.escape(stage["label"])}<\/h[3-6]>.*?'
                rf'<dt>Stage ID</dt>\s*<dd>{re.escape(stage["id"])}<\/dd>.*?'
                rf'<dt>Type</dt>\s*<dd>{re.escape(stage["stage_type"])}<\/dd>.*?'
                rf'<dt>Status</dt>\s*<dd>{re.escape(stage["status"])}<\/dd>',
                re.DOTALL,
            )
            stage_match = stage_pattern.search(ado_panel)
            self.assertIsNotNone(stage_match, f"ADO stage {stage['id']} must render as a focusable card with stable metadata and full text.")
            self.assertGreater(stage_match.start(), previous_stage_position, f"ADO stage {stage['id']} must remain in fixture document order.")
            previous_stage_position = stage_match.start()

            if "lane" in stage:
                self.assertRegex(stage_match.group(0), rf'data-lane="{stage["lane"]}"')
            if "parallel_group" in stage:
                self.assertRegex(stage_match.group(0), rf'data-parallel-group="{re.escape(stage["parallel_group"])}"')
            if stage.get("target_node_id"):
                self.assertRegex(stage_match.group(0), rf'data-target-node-id="{re.escape(stage["target_node_id"])}"')

        previous_edge_position = -1
        for edge in pipeline["edges"]:
            evidence = edge["evidence"][0]
            edge_pattern = re.compile(
                rf'<(?:svg|a|button)[^>]+data-transition-id="{re.escape(edge["id"])}"[^>]+'
                rf'data-source-stage-id="{re.escape(edge["source_id"])}"[^>]+'
                rf'data-target-stage-id="{re.escape(edge["target_id"])}"[^>]+'
                rf'data-transition-kind="{re.escape(edge["kind"])}"[^>]+'
                rf'tabindex="0"[^>]*>.*?'
                rf'{re.escape(edge["label"])}.*?'
                rf'{re.escape(evidence["path"])}:{re.escape(evidence["lines"])}.*?'
                rf'{re.escape(evidence["claim"])}.*?'
                rf'</(?:svg|a|button)>',
                re.DOTALL,
            )
            edge_match = edge_pattern.search(ado_panel)
            self.assertIsNotNone(edge_match, f"ADO transition {edge['id']} must render as a directed labeled focusable connector with kind and evidence.")
            self.assertGreater(edge_match.start(), previous_edge_position, f"ADO transition {edge['id']} must remain in fixture document order.")
            previous_edge_position = edge_match.start()
            if edge["kind"] in {"approval", "manual"}:
                self.assertRegex(edge_match.group(0), r'(?i)(approval|manual|requires human|human approval)')

        fallback_pattern = re.compile(
            r'<section[^>]+data-stage-transition-summary="ado"[^>]*>.*?'
            + ".*?".join(
                re.escape(stage["label"]) + r'.*?' + re.escape(stage["stage_type"])
                for stage in pipeline["stages"]
            )
            + ".*?"
            + ".*?".join(
                re.escape(edge["source_id"]) + r'.*?' + re.escape(edge["label"]) + r'.*?' + re.escape(edge["target_id"])
                for edge in pipeline["edges"]
            ),
            re.DOTALL,
        )
        self.assertRegex(ado_panel, fallback_pattern, "ADO projection must keep a complete ordered stage and transition text fallback.")


if __name__ == "__main__":
    unittest.main()
