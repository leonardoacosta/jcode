import copy
import importlib.util
import json
import subprocess
import sys
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
VALIDATOR = ROOT / "scripts" / "validate_scene.py"
MATH = ROOT / "scripts" / "scene_math.py"
FIXTURE = Path(__file__).parent / "fixtures" / "valid-scene.json"
DIRECTIONAL_FIXTURE = Path(__file__).parent / "fixtures" / "directional-scene.json"
AZURE_SPRITE = ROOT / "assets" / "azure-icons.svg"
AZURE_TOKENS = ROOT / "assets" / "azure-tokens.json"


def load_module(path: Path, name: str):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"Unable to load {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class SceneContractTests(unittest.TestCase):
    def setUp(self):
        self.document = json.loads(FIXTURE.read_text())

    def traffic_document(self):
        return json.loads(DIRECTIONAL_FIXTURE.read_text())

    def test_valid_scene_has_no_errors(self):
        validator = load_module(VALIDATOR, "isometric_scene_validator")
        self.assertEqual(validator.validate_scene(self.document), [])

    def test_projection_uses_true_two_to_one_isometric_axes(self):
        math = load_module(MATH, "isometric_scene_math")
        self.assertEqual(math.project(0, 0, 0, 64, 32, 100, 50), (100.0, 50.0))
        self.assertEqual(math.project(1, 0, 0, 64, 32, 100, 50), (132.0, 66.0))
        self.assertEqual(math.project(0, 1, 0, 64, 32, 100, 50), (68.0, 66.0))
        self.assertEqual(math.project(1, 1, 2, 64, 32, 100, 50), (100.0, 80.0))

    def test_projection_rejects_non_two_to_one_tiles(self):
        math = load_module(MATH, "isometric_scene_math_ratio")
        with self.assertRaisesRegex(ValueError, "tile_width must equal 2 × tile_height"):
            math.project(1, 1, 0, 64, 40, 100, 50)

    def test_non_finite_numeric_values_return_validation_errors(self):
        validator = load_module(VALIDATOR, "isometric_scene_validator_finite_numbers")
        for value in (float("nan"), float("inf"), float("-inf")):
            traffic = self.traffic_document()
            traffic["traffic"]["layers"][0]["padding"] = value
            self.assertIn(
                "traffic.layers[0].padding: half-grid number from 0 to 2 required",
                validator.validate_scene(traffic),
            )

            cube = copy.deepcopy(self.document)
            cube["canvas"]["cube_size"] = value
            self.assertIn(
                "canvas.cube_size: half-grid number from 0.5 to 2 required",
                validator.validate_scene(cube),
            )

    def test_collision_detection_uses_full_resource_envelopes(self):
        validator = load_module(VALIDATOR, "isometric_scene_validator_collision")
        broken = copy.deepcopy(self.document)
        broken["nodes"][1]["position"] = {"x": 2, "y": 1}
        errors = validator.validate_scene(broken)
        self.assertTrue(any("overlaps nodes[0]" in error for error in errors), errors)

    def test_grid_routes_cannot_cut_through_unrelated_resource_envelopes(self):
        validator = load_module(VALIDATOR, "isometric_scene_validator_routes")
        broken = copy.deepcopy(self.document)
        broken["paths"][0]["route"] = [
            {"x": 3, "y": 1.5},
            {"x": 8.5, "y": 1.5},
            {"x": 8.5, "y": 4.5},
        ]
        errors = validator.validate_scene(broken)
        self.assertTrue(any("route intersects node 'database'" in error for error in errors), errors)

    def test_grid_routes_must_start_on_or_just_outside_the_source_boundary(self):
        validator = load_module(VALIDATOR, "isometric_scene_validator_endpoint")
        broken = copy.deepcopy(self.document)
        broken["paths"][0]["route"][0] = {"x": 2, "y": 1.5}
        errors = validator.validate_scene(broken)
        self.assertIn(
            "paths[0].route[0]: must start on or just outside source node 'pipeline' boundary",
            errors,
        )

    def test_flow_steps_require_direct_structured_evidence(self):
        validator = load_module(VALIDATOR, "isometric_scene_validator_evidence")
        broken = copy.deepcopy(self.document)
        broken["flows"][0]["steps"][0]["evidence"] = []
        errors = validator.validate_scene(broken)
        self.assertIn(
            "flows[0].steps[0].evidence: requires at least one path/lines/claim evidence object",
            errors,
        )

    def test_flow_step_payload_must_travel_on_the_referenced_path(self):
        validator = load_module(VALIDATOR, "isometric_scene_validator_payload_route")
        broken = copy.deepcopy(self.document)
        broken["flows"][0]["steps"][0]["payload"] = "diagnostic"
        errors = validator.validate_scene(broken)
        self.assertIn(
            "flows[0].steps[0].payload: payload 'diagnostic' is not carried by path 'deploy-app'",
            errors,
        )

    def test_static_dependency_scene_can_omit_payloads_and_flows(self):
        validator = load_module(VALIDATOR, "isometric_scene_validator_static")
        static = copy.deepcopy(self.document)
        static["payloads"] = []
        static["flows"] = []
        for path in static["paths"]:
            path["kind"] = "dependency"
            path["payload_ids"] = []
        self.assertEqual(validator.validate_scene(static), [])

    def test_non_request_scene_can_omit_traffic_without_an_entry_node(self):
        validator = load_module(VALIDATOR, "isometric_scene_validator_optional_traffic")
        document = copy.deepcopy(self.document)
        document["nodes"][3]["role"] = "external"
        document["nodes"][3]["status"] = "external"
        self.assertNotIn("traffic", document)
        self.assertEqual(validator.validate_scene(document), [])

    def test_non_dependency_path_requires_a_payload(self):
        validator = load_module(VALIDATOR, "isometric_scene_validator_empty_payload")
        broken = copy.deepcopy(self.document)
        broken["paths"][0]["payload_ids"] = []
        errors = validator.validate_scene(broken)
        self.assertIn(
            "paths[0].payload_ids: may be empty only when kind is 'dependency'",
            errors,
        )

    def test_each_used_path_kind_requires_a_structured_treatment(self):
        validator = load_module(VALIDATOR, "isometric_scene_validator_treatments")
        broken = copy.deepcopy(self.document)
        broken["art_direction"]["path_treatments"] = {
            "delivery": self._treatment(),
            "dependency": self._treatment(),
        }
        errors = validator.validate_scene(broken)
        self.assertIn(
            "art_direction.path_treatments.telemetry: required for used path kind",
            errors,
        )

    def test_path_treatments_require_non_color_channels(self):
        validator = load_module(VALIDATOR, "isometric_scene_validator_treatment_fields")
        broken = copy.deepcopy(self.document)
        broken["art_direction"]["path_treatments"] = {
            "delivery": self._treatment(),
            "dependency": self._treatment(),
            "telemetry": self._treatment(),
        }
        del broken["art_direction"]["path_treatments"]["dependency"]["marker"]
        errors = validator.validate_scene(broken)
        self.assertIn(
            "art_direction.path_treatments.dependency.marker: required non-empty string",
            errors,
        )

    def test_scene_accepts_only_cube_resource_forms(self):
        validator = load_module(VALIDATOR, "isometric_scene_validator_forms")
        document = copy.deepcopy(self.document)
        for node in document["nodes"]:
            node["form"] = "cube"
        self.assertEqual(validator.validate_scene(document), [])

        document["nodes"][0]["form"] = "tower"
        errors = validator.validate_scene(document)
        self.assertIn("nodes[0].form: must be one of ['cube']", errors)

    def test_scene_uses_one_global_cube_size_instead_of_per_node_heights(self):
        validator = load_module(VALIDATOR, "isometric_scene_validator_uniform_cube_size")
        document = copy.deepcopy(self.document)
        document["canvas"]["cube_size"] = 1
        for node in document["nodes"]:
            node.pop("height", None)
        self.assertEqual(validator.validate_scene(document), [])

        del document["canvas"]["cube_size"]
        errors = validator.validate_scene(document)
        self.assertIn("canvas.cube_size: half-grid number from 0.5 to 2 required", errors)

    def test_vnet_areas_require_real_members_and_fit_inside_the_canvas(self):
        validator = load_module(VALIDATOR, "isometric_scene_validator_vnet_areas")
        document = copy.deepcopy(self.document)
        document["canvas"]["cube_size"] = 1
        for node in document["nodes"]:
            node.pop("height", None)
        document["areas"] = [
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
        self.assertEqual(validator.validate_scene(document), [])

        broken_member = copy.deepcopy(document)
        broken_member["areas"][0]["member_ids"].append("missing-resource")
        errors = validator.validate_scene(broken_member)
        self.assertIn(
            "areas[0].member_ids[3]: references unknown node 'missing-resource'",
            errors,
        )

        malformed_member = copy.deepcopy(document)
        malformed_member["areas"][0]["member_ids"] = [["app"]]
        errors = validator.validate_scene(malformed_member)
        self.assertIn(
            "areas[0].member_ids[0]: requires a non-empty node id string",
            errors,
        )

        ambiguous = copy.deepcopy(document)
        ambiguous["areas"][0]["member_ids"] = ["app", "telemetry"]
        errors = validator.validate_scene(ambiguous)
        self.assertIn(
            "areas[0]: derived bounds intersect unrelated node 'database'",
            errors,
        )

        outside = copy.deepcopy(document)
        outside["areas"][0]["member_ids"] = ["pipeline"]
        outside["areas"][0]["padding"] = 2
        errors = validator.validate_scene(outside)
        self.assertIn("areas[0]: padded member bounds extend beyond the canvas", errors)

    def test_traffic_story_requires_four_ordered_layers_and_entry_role(self):
        validator = load_module(VALIDATOR, "isometric_scene_validator_traffic_layers")
        document = self.traffic_document()
        self.assertEqual(validator.validate_scene(document), [])

        missing = copy.deepcopy(document)
        del missing["traffic"]
        errors = validator.validate_scene(missing)
        self.assertIn("traffic: required when a node uses role 'entry'", errors)

        wrong_order = copy.deepcopy(document)
        wrong_order["traffic"]["layers"][1], wrong_order["traffic"]["layers"][2] = (
            wrong_order["traffic"]["layers"][2],
            wrong_order["traffic"]["layers"][1],
        )
        errors = validator.validate_scene(wrong_order)
        self.assertIn(
            "traffic.layers: kinds must be ordered ['ingress', 'projects', 'data-access', 'external-services']",
            errors,
        )

        no_entry = copy.deepcopy(document)
        next(node for node in no_entry["nodes"] if node["id"] == "apim")["role"] = "module"
        errors = validator.validate_scene(no_entry)
        self.assertIn("traffic.layers[0]: ingress must include at least one role 'entry' node", errors)

    def test_traffic_layers_require_unique_members_and_bottom_left_to_top_right_progress(self):
        validator = load_module(VALIDATOR, "isometric_scene_validator_traffic_geometry")
        document = self.traffic_document()

        malformed_member = copy.deepcopy(document)
        malformed_member["traffic"]["layers"][1]["member_ids"] = [["app"]]
        errors = validator.validate_scene(malformed_member)
        self.assertIn(
            "traffic.layers[1].member_ids[0]: requires a non-empty node id string",
            errors,
        )

        duplicate = copy.deepcopy(document)
        duplicate_index = len(duplicate["traffic"]["layers"][2]["member_ids"])
        duplicate["traffic"]["layers"][2]["member_ids"].append("app")
        errors = validator.validate_scene(duplicate)
        self.assertIn(
            f"traffic.layers[2].member_ids[{duplicate_index}]: node 'app' already belongs to traffic layer 'projects'",
            errors,
        )

        reversed_x = copy.deepcopy(document)
        next(node for node in reversed_x["nodes"] if node["id"] == "partner-api")["position"] = {
            "x": 1,
            "y": 1,
        }
        errors = validator.validate_scene(reversed_x)
        self.assertTrue(
            any("must progress toward the top right" in error for error in errors),
            errors,
        )

    def test_nodes_accept_known_azure_resource_metadata(self):
        validator = load_module(VALIDATOR, "isometric_scene_validator_azure_metadata")
        document = copy.deepcopy(self.document)
        document["nodes"][1]["resource_type"] = "Microsoft.Web/sites"
        document["nodes"][1]["icon"] = "az-app-service"
        self.assertEqual(validator.validate_scene(document), [])

    def test_nodes_reject_unknown_azure_icon_ids(self):
        validator = load_module(VALIDATOR, "isometric_scene_validator_azure_icon")
        broken = copy.deepcopy(self.document)
        broken["nodes"][1]["icon"] = "az-made-up-service"
        errors = validator.validate_scene(broken)
        self.assertIn(
            "nodes[1].icon: unsupported Azure topology icon 'az-made-up-service'",
            errors,
        )

    def test_vendored_azure_assets_are_complete_and_in_sync(self):
        validator = load_module(VALIDATOR, "isometric_scene_validator_azure_assets")
        self.assertTrue(AZURE_SPRITE.exists())
        self.assertTrue(AZURE_TOKENS.exists())
        sprite = AZURE_SPRITE.read_text()
        tokens = json.loads(AZURE_TOKENS.read_text())
        self.assertEqual(tokens["canvas"]["azure_blue"], "#0078d4")
        self.assertEqual(tokens["families"]["compute"], {"stroke": "#c8460e", "fill": "#fde6d4"})
        for icon_id in validator.AZURE_ICON_IDS:
            self.assertIn(f'id="{icon_id}"', sprite)

    def test_contract_rejects_surrounding_ui_configuration(self):
        validator = load_module(VALIDATOR, "isometric_scene_validator_ui")
        broken = copy.deepcopy(self.document)
        broken["panels"] = {"metrics": True, "rail": True}
        errors = validator.validate_scene(broken)
        self.assertIn("$.panels: unknown field", errors)

    def test_path_evidence_level_accepts_optional_enum_and_rejects_invalid_values(self):
        validator = load_module(VALIDATOR, "isometric_scene_validator_path_evidence_level")
        document = self.traffic_document()
        self.assertEqual(validator.validate_scene(document), [])

        for level in ("direct", "inferred", "held"):
            with self.subTest(evidence_level=level):
                explicit = copy.deepcopy(document)
                explicit["paths"][0]["evidence_level"] = level
                self.assertEqual(validator.validate_scene(explicit), [])

        invalid = copy.deepcopy(document)
        invalid["paths"][0]["evidence_level"] = "rumored"
        self.assertEqual(
            validator.validate_scene(invalid),
            ["paths[0].evidence_level: must be one of ['direct', 'inferred', 'held']"],
        )

    def test_cli_validates_fixture(self):
        result = subprocess.run(
            [sys.executable, str(VALIDATOR), str(FIXTURE)],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("valid isometric scene", result.stdout.lower())

    @staticmethod
    def _treatment():
        return {
            "stroke_pattern": "solid",
            "weight": "medium",
            "marker": "terminal arrow",
            "texture": "clean ink",
            "motion_cadence": "steady",
            "reduced_motion": "static endpoint marker",
        }


if __name__ == "__main__":
    unittest.main()
