import copy
import hashlib
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
DIRECTIONAL_FIXTURE = Path(__file__).parent / "fixtures" / "directional-scene.json"
BICEP_FIXTURE = Path(__file__).parent / "fixtures" / "bicep-scene.json"
DARK_THEME = ROOT / "themes" / "dark-technical.js"
PAPER_THEME = ROOT / "themes" / "warm-paper.js"
AZURE_THEME = ROOT / "themes" / "azure-topology.js"
CANVAS_RECIPES = ROOT / "references" / "canvas-recipes.md"
EXAMPLE_OUTPUTS = (
    ROOT.parents[1] / "docs" / "diagrams" / "isometric-canvas-azure.html",
    ROOT.parents[1] / "docs" / "diagrams" / "isometric-canvas-dark.html",
    ROOT.parents[1] / "docs" / "diagrams" / "isometric-canvas-paper.html",
)


def traffic_scene():
    return json.loads(DIRECTIONAL_FIXTURE.read_text())


class CanvasRendererTests(unittest.TestCase):
    def render(self, scene: Path, theme: Path, output: Path) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, str(RENDERER), str(scene), str(theme), str(output)],
            capture_output=True,
            text=True,
            check=False,
        )

    def test_renderer_creates_a_scene_first_three_layer_canvas_artifact(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "map.html"
            result = self.render(FIXTURE, DARK_THEME, output)
            self.assertEqual(result.returncode, 0, result.stderr)
            html = output.read_text()
            self.assertIn('<canvas data-layer="terrain"', html)
            self.assertIn('<canvas data-layer="architecture"', html)
            self.assertIn('<canvas data-layer="motion"', html)
            self.assertNotIn("metric-card", html)
            self.assertNotIn("navigation-rail", html)
            self.assertNotIn("explainer-panel", html)

    def test_renderer_uses_researched_canvas_interaction_and_lifecycle_apis(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "map.html"
            result = self.render(FIXTURE, DARK_THEME, output)
            self.assertEqual(result.returncode, 0, result.stderr)
            html = output.read_text()
            for capability in (
                "Path2D",
                "isPointInPath",
                "isPointInStroke",
                "ResizeObserver",
                "devicePixelRatio",
                "prefers-reduced-motion",
                "cancelAnimationFrame",
                "drawFocusIfNeeded",
                "toBlob",
            ):
                self.assertIn(capability, html)

    def test_canvas_recipe_documents_directional_theme_adapter_methods(self):
        recipe = CANVAS_RECIPES.read_text()
        for method in ("drawTrafficLayer", "drawTrafficDirection", "drawArea"):
            self.assertRegex(recipe, rf"(?m)^\s*{method}\(")

    def test_tooltips_include_the_structured_evidence_claim(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "map.html"
            result = self.render(DIRECTIONAL_FIXTURE, AZURE_THEME, output)
            self.assertEqual(result.returncode, 0, result.stderr)
            html = output.read_text()
            self.assertIn(
                "const evidenceItems = Array.isArray(item.evidence) ? item.evidence : [];",
                html,
            )
            self.assertIn(
                'evidenceItems.map(entry => `${entry.path}:${entry.lines}`).join("; ")',
                html,
            )
            self.assertIn(
                'evidenceItems.map(entry => entry.claim).filter(Boolean).join("; ")',
                html,
            )
            self.assertIn(
                "const [relationship, description, evidence, claim] = targetDescription(target);",
                html,
            )
            self.assertIn("claimLine.textContent = claim || \"\";", html)
            self.assertIn(
                "tooltip.replaceChildren(title, relationshipLine, descriptionLine, evidenceLine, claimLine);",
                html,
            )

    def test_renderer_supports_named_flow_selection_and_static_scenes(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "map.html"
            result = self.render(FIXTURE, DARK_THEME, output)
            self.assertEqual(result.returncode, 0, result.stderr)
            html = output.read_text()
            self.assertIn('<select id="flow" aria-label="Flow">', html)
            self.assertIn("SCENE.flows.forEach", html)
            self.assertIn("state.activeFlowId", html)
            self.assertIn("pauseButton.hidden = SCENE.flows.length === 0", html)
            self.assertIn("SCENE.flows.length > 0", html)

    def test_renderer_mirrors_nodes_and_paths_as_native_fallback_buttons(self):
        scene = json.loads(FIXTURE.read_text())
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "map.html"
            result = self.render(FIXTURE, DARK_THEME, output)
            self.assertEqual(result.returncode, 0, result.stderr)
            html = output.read_text()
            node_buttons = re.findall(r'<button[^>]+data-target-kind="node"', html)
            path_buttons = re.findall(r'<button[^>]+data-target-kind="path"', html)
            self.assertEqual(len(node_buttons), len(scene["nodes"]))
            self.assertEqual(len(path_buttons), len(scene["paths"]))
            self.assertNotIn(".fallback-copy { display: none; }", html)
            motion_canvas = re.search(
                r'<canvas data-layer="motion"[^>]*>(?P<fallback>.*?)</canvas>',
                html,
                re.DOTALL,
            )
            self.assertIsNotNone(motion_canvas)
            self.assertNotIn("fallback-copy", motion_canvas.group("fallback"))
            self.assertRegex(
                html,
                r'</canvas>\s*<div class="fallback-copy sr-only" role="group"',
            )

    def test_same_scene_renders_through_distinct_theme_adapters(self):
        with tempfile.TemporaryDirectory() as directory:
            dark_output = Path(directory) / "dark.html"
            paper_output = Path(directory) / "paper.html"
            dark = self.render(FIXTURE, DARK_THEME, dark_output)
            paper = self.render(FIXTURE, PAPER_THEME, paper_output)
            self.assertEqual(dark.returncode, 0, dark.stderr)
            self.assertEqual(paper.returncode, 0, paper.stderr)
            dark_html = dark_output.read_text()
            paper_html = paper_output.read_text()
            canonical = json.dumps(json.loads(FIXTURE.read_text()), sort_keys=True, separators=(",", ":"))
            scene_hash = hashlib.sha256(canonical.encode()).hexdigest()
            self.assertIn(f'data-scene-sha256="{scene_hash}"', dark_html)
            self.assertIn(f'data-scene-sha256="{scene_hash}"', paper_html)
            self.assertIn('name: "Dark technical linework"', dark_html)
            self.assertIn('name: "Warm archival paper"', paper_html)
            self.assertNotEqual(dark_html, paper_html)

    def test_checked_in_theme_examples_embed_the_directional_fixture(self):
        canonical = json.dumps(
            json.loads(DIRECTIONAL_FIXTURE.read_text()),
            sort_keys=True,
            separators=(",", ":"),
        )
        scene_hash = hashlib.sha256(canonical.encode()).hexdigest()
        for output in EXAMPLE_OUTPUTS:
            html = output.read_text()
            match = re.search(
                r"const SCENE = (?P<scene>\{.*?\});\s*const ICON_SVGS =",
                html,
                re.DOTALL,
            )
            self.assertIsNotNone(match, output)
            embedded_scene = json.loads(match.group("scene"))
            expected_fixture = BICEP_FIXTURE if output.name == "isometric-canvas-dark.html" else DIRECTIONAL_FIXTURE
            expected_canonical = json.dumps(json.loads(expected_fixture.read_text()), sort_keys=True, separators=(",", ":"))
            expected_hash = hashlib.sha256(expected_canonical.encode()).hexdigest()
            self.assertIn(f'data-scene-sha256="{expected_hash}"', html)
            self.assertEqual(len(re.findall(r'data-target-kind="traffic-layer"', html)), 4)

    def test_renderer_rejects_a_scene_that_fails_geometry_validation(self):
        with tempfile.TemporaryDirectory() as directory:
            broken_path = Path(directory) / "broken.json"
            output = Path(directory) / "map.html"
            broken = copy.deepcopy(json.loads(FIXTURE.read_text()))
            broken["nodes"][1]["position"] = {"x": 2, "y": 1}
            broken_path.write_text(json.dumps(broken))
            result = self.render(broken_path, DARK_THEME, output)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("overlaps", result.stderr)
            self.assertFalse(output.exists())

    def test_azure_theme_embeds_only_used_resource_icons_and_roof_projection_code(self):
        scene = json.loads(FIXTURE.read_text())
        icons = [
            ("Azure DevOps pipeline", "az-release-pipeline"),
            ("Microsoft.Web/sites", "az-app-service"),
            ("Microsoft.Sql/servers/databases", "az-sql-database"),
            ("Microsoft.OperationalInsights/workspaces", "az-log-analytics"),
        ]
        for node, (resource_type, icon) in zip(scene["nodes"], icons, strict=True):
            node["resource_type"] = resource_type
            node["icon"] = icon

        with tempfile.TemporaryDirectory() as directory:
            scene_path = Path(directory) / "azure-scene.json"
            output = Path(directory) / "azure.html"
            scene_path.write_text(json.dumps(scene))
            result = self.render(scene_path, AZURE_THEME, output)
            self.assertEqual(result.returncode, 0, result.stderr)
            html = output.read_text()
            self.assertIn('name: "Azure topology resource blocks"', html)
            self.assertIn('"az-app-service":"\\u003csvg', html)
            self.assertIn('"az-sql-database":"\\u003csvg', html)
            self.assertNotIn('"az-cosmos-db":"\\u003csvg', html)
            self.assertIn("const ICON_SVGS =", html)
            self.assertIn("const iconImages = new Map()", html)
            self.assertIn("function drawNodeIcon", html)
            self.assertIn("ctx.transform(", html)
            self.assertIn("renderedIcons", html)

    def test_renderer_resolves_every_node_to_one_true_cube_mass(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "azure-cubes.html"
            result = self.render(FIXTURE, AZURE_THEME, output)
            self.assertEqual(result.returncode, 0, result.stderr)
            html = output.read_text()
            self.assertIn("function cubeMassFor(node)", html)
            self.assertIn("const cubeEdge = SCENE.canvas.cube_size;", html)
            self.assertNotIn("Math.min(width, depth, node.height)", html)
            self.assertIn("const projectedEdge = Math.hypot(state.tileWidth / 2, state.tileHeight / 2)", html)
            self.assertIn("form: \"cube\"", html)
            self.assertIn("cubeEdge: cube.cubeEdge", html)
            self.assertIn("nodeGeometry", html)
            self.assertNotIn('case "stack":', html)
            self.assertNotIn('case "gateway":', html)

    def test_renderer_draws_vnet_areas_and_exposes_semantic_area_controls(self):
        scene = json.loads(FIXTURE.read_text())
        scene["canvas"]["cube_size"] = 1
        for node in scene["nodes"]:
            node.pop("height", None)
        scene["areas"] = [
            {
                "id": "runtime-vnet",
                "label": "Runtime VNet",
                "kind": "vnet",
                "status": "active",
                "member_ids": ["app", "database", "telemetry"],
                "padding": 0.5,
                "description": "Private runtime attachment area.",
                "evidence": [
                    {
                        "path": "infra/network/main.bicep",
                        "lines": "1-72",
                        "claim": "The runtime resources attach to the application VNet.",
                    }
                ],
            }
        ]

        with tempfile.TemporaryDirectory() as directory:
            scene_path = Path(directory) / "vnet-area.json"
            output = Path(directory) / "vnet-area.html"
            scene_path.write_text(json.dumps(scene))
            result = self.render(scene_path, AZURE_THEME, output)
            self.assertEqual(result.returncode, 0, result.stderr)
            html = output.read_text()
            self.assertIn("function areaRect(area)", html)
            self.assertIn("SCENE.areas.forEach", html)
            self.assertIn("THEME.drawArea", html)
            self.assertIn("areaGeometry", html)
            self.assertEqual(len(re.findall(r'data-target-kind="area"', html)), 1)

    def test_renderer_draws_directional_traffic_layers_and_exposes_semantic_controls(self):
        scene = traffic_scene()
        with tempfile.TemporaryDirectory() as directory:
            scene_path = Path(directory) / "traffic-scene.json"
            output = Path(directory) / "traffic-map.html"
            scene_path.write_text(json.dumps(scene))
            result = self.render(scene_path, AZURE_THEME, output)
            self.assertEqual(result.returncode, 0, result.stderr)
            html = output.read_text()
            traffic_buttons = re.findall(r'<button[^>]+data-target-kind="traffic-layer"', html)
            self.assertEqual(len(traffic_buttons), 4)
            self.assertIn("function trafficLayerRect", html)
            self.assertIn("THEME.drawTrafficLayer", html)
            self.assertIn("THEME.drawTrafficDirection", html)
            self.assertIn("trafficLayerGeometry", html)
            self.assertIn('direction: "bottom-left-to-top-right"', html)
            for theme in (DARK_THEME, PAPER_THEME, AZURE_THEME):
                source = theme.read_text()
                self.assertIn("drawTrafficLayer", source)
                self.assertIn("drawTrafficDirection", source)

    def test_renderer_supports_optional_views_sidecar_without_regressing_scene_only_cli(self):
        views_fixture = Path(__file__).parent / "fixtures" / "directional-views.json"
        scene = json.loads(DIRECTIONAL_FIXTURE.read_text())
        views = json.loads(views_fixture.read_text())
        canonical_scene = json.dumps(scene, sort_keys=True, separators=(",", ":"))
        canonical_views = json.dumps(views, sort_keys=True, separators=(",", ":"))
        scene_hash = hashlib.sha256(canonical_scene.encode()).hexdigest()
        views_hash = hashlib.sha256(canonical_views.encode()).hexdigest()

        with tempfile.TemporaryDirectory() as directory:
            directory_path = Path(directory)
            views_output = directory_path / "directional-with-views.html"
            mismatch_views_path = directory_path / "directional-views-mismatched.json"
            mismatch_output = directory_path / "mismatch.html"
            scene_only_a = directory_path / "scene-only-a.html"
            scene_only_b = directory_path / "scene-only-b.html"

            with self.subTest("views sidecar renders native semantic tabs and static content"):
                result = subprocess.run(
                    [
                        sys.executable,
                        str(RENDERER),
                        str(DIRECTIONAL_FIXTURE),
                        str(AZURE_THEME),
                        str(views_output),
                        "--views",
                        str(views_fixture),
                    ],
                    capture_output=True,
                    text=True,
                    check=False,
                )
                self.assertEqual(result.returncode, 0, result.stderr)
                html = views_output.read_text()
                self.assertIn(f'data-scene-sha256="{scene_hash}"', html)
                self.assertIn(f'data-views-sha256="{views_hash}"', html)
                self.assertIn('role="tablist"', html)
                for view_id, label in (("runtime", "Runtime"), ("network", "Network"), ("ado", "ADO")):
                    self.assertRegex(
                        html,
                        rf'<button[^>]+role="tab"[^>]+aria-controls="{view_id}"[^>]*>\s*{label}\s*</button>',
                    )
                    self.assertRegex(html, rf'<(?:section|div)[^>]+id="{view_id}"[^>]+role="tabpanel"')
                    self.assertRegex(html, rf'<(?:a|section|div)[^>]+(?:id|href)="#{view_id}"|id="{view_id}"')
                self.assertIn("const VIEWS_SIDECAR =", html)
                self.assertIn('"default_view":"network"', html)
                non_canvas_html = re.sub(r"<canvas\b[^>]*>.*?</canvas>", "", html, flags=re.DOTALL)
                for static_text in (
                    "sample-directional-infra subscription",
                    "The runtime resources attach to the application VNet.",
                    "Validate Bicep",
                    "The deployment workflow is sourced from the represented repository.",
                ):
                    self.assertIn(static_text, non_canvas_html)

            with self.subTest("mismatched repository identity fails before writing output"):
                mismatched_views = copy.deepcopy(views)
                mismatched_views["repository"]["commit"] = "ffffffffffffffffffffffffffffffffffffffff"
                mismatch_views_path.write_text(json.dumps(mismatched_views))
                result = subprocess.run(
                    [
                        sys.executable,
                        str(RENDERER),
                        str(DIRECTIONAL_FIXTURE),
                        str(AZURE_THEME),
                        str(mismatch_output),
                        "--views",
                        str(mismatch_views_path),
                    ],
                    capture_output=True,
                    text=True,
                    check=False,
                )
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("repository", result.stderr.lower())
                self.assertIn("commit", result.stderr.lower())
                self.assertFalse(mismatch_output.exists())

            with self.subTest("existing three positional argument scene-only path stays deterministic"):
                first = self.render(DIRECTIONAL_FIXTURE, AZURE_THEME, scene_only_a)
                second = self.render(DIRECTIONAL_FIXTURE, AZURE_THEME, scene_only_b)
                self.assertEqual(first.returncode, 0, first.stderr)
                self.assertEqual(second.returncode, 0, second.stderr)
                first_html = scene_only_a.read_text()
                second_html = scene_only_b.read_text()
                self.assertEqual(first_html, second_html)
                self.assertIn(f'data-scene-sha256="{scene_hash}"', first_html)
                self.assertNotIn('role="tablist"', first_html)
                self.assertNotIn('role="tabpanel"', first_html)
                self.assertNotIn('data-views-sha256=', first_html)

    def test_azure_theme_uses_the_topology_palette_and_semantic_families(self):
        source = AZURE_THEME.read_text()
        self.assertIn('pageBackground: "#ffffff"', source)
        self.assertIn('azureBlue: "#0078d4"', source)
        self.assertIn('compute: { stroke: "#c8460e", fill: "#fde6d4" }', source)
        self.assertIn('data: { stroke: "#107c10", fill: "#d7ebd7" }', source)
        self.assertIn('devops: { stroke: "#a02763", fill: "#f6d8e6" }', source)
        self.assertIn("resource_type", source)

    def test_azure_theme_supports_an_app_configuration_roof_mark(self):
        scene = json.loads(FIXTURE.read_text())
        scene["nodes"][0]["resource_type"] = "Microsoft.AppConfiguration/configurationStores"
        scene["nodes"][0]["icon"] = "az-app-configuration"

        with tempfile.TemporaryDirectory() as directory:
            scene_path = Path(directory) / "app-configuration-scene.json"
            output = Path(directory) / "app-configuration.html"
            scene_path.write_text(json.dumps(scene))
            result = self.render(scene_path, AZURE_THEME, output)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn('"az-app-configuration":"\\u003csvg', output.read_text())

    def test_azure_theme_maps_current_bicep_resource_families(self):
        source = AZURE_THEME.read_text()
        expected_mappings = {
            '"Microsoft.App/managedEnvironments": "compute"',
            '"Microsoft.AppConfiguration/configurationStores": "identity"',
            '"Microsoft.ApiManagement/service/apis": "integration"',
            '"Microsoft.ApiManagement/service/groups": "integration"',
            '"Microsoft.ApiManagement/service/products": "integration"',
            '"Microsoft.Insights/metricAlerts": "monitor"',
            '"Microsoft.Network/virtualNetworks/virtualNetworkPeerings": "network"',
            '"Microsoft.Resources/resourceGroups": "governance"',
            '"Microsoft.Sql/servers/elasticPools": "data"',
        }
        for mapping in expected_mappings:
            self.assertIn(mapping, source)


if __name__ == "__main__":
    unittest.main()
