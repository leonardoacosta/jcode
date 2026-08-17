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

    def test_valid_scene_has_no_errors(self):
        validator = load_module(VALIDATOR, "isometric_scene_validator")
        self.assertEqual(validator.validate_scene(self.document), [])

    def test_projection_uses_true_two_to_one_isometric_axes(self):
        math = load_module(MATH, "isometric_scene_math")
        self.assertEqual(math.project(0, 0, 0, 64, 32, 100, 50), (100.0, 50.0))
        self.assertEqual(math.project(1, 0, 0, 64, 32, 100, 50), (132.0, 66.0))
        self.assertEqual(math.project(0, 1, 0, 64, 32, 100, 50), (68.0, 66.0))
        self.assertEqual(math.project(1, 1, 2, 64, 32, 100, 50), (100.0, 80.0))

    def test_collision_detection_uses_full_building_footprints(self):
        validator = load_module(VALIDATOR, "isometric_scene_validator_collision")
        broken = copy.deepcopy(self.document)
        broken["nodes"][1]["position"] = {"x": 2, "y": 1}
        errors = validator.validate_scene(broken)
        self.assertTrue(any("overlaps nodes[0]" in error for error in errors), errors)

    def test_grid_routes_cannot_cut_through_unrelated_buildings(self):
        validator = load_module(VALIDATOR, "isometric_scene_validator_routes")
        broken = copy.deepcopy(self.document)
        broken["paths"][0]["route"] = [
            {"x": 3, "y": 1.5},
            {"x": 8.5, "y": 1.5},
            {"x": 8.5, "y": 4.5},
        ]
        errors = validator.validate_scene(broken)
        self.assertTrue(any("route intersects node 'database'" in error for error in errors), errors)

    def test_flow_steps_require_direct_structured_evidence(self):
        validator = load_module(VALIDATOR, "isometric_scene_validator_evidence")
        broken = copy.deepcopy(self.document)
        broken["flows"][0]["steps"][0]["evidence"] = []
        errors = validator.validate_scene(broken)
        self.assertIn(
            "flows[0].steps[0].evidence: requires at least one path/lines/claim evidence object",
            errors,
        )

    def test_scene_requires_varied_building_forms(self):
        validator = load_module(VALIDATOR, "isometric_scene_validator_forms")
        broken = copy.deepcopy(self.document)
        for node in broken["nodes"]:
            node["form"] = "tower"
        errors = validator.validate_scene(broken)
        self.assertTrue(any("at least 3 distinct building forms" in error for error in errors), errors)

    def test_contract_rejects_surrounding_ui_configuration(self):
        validator = load_module(VALIDATOR, "isometric_scene_validator_ui")
        broken = copy.deepcopy(self.document)
        broken["panels"] = {"metrics": True, "rail": True}
        errors = validator.validate_scene(broken)
        self.assertIn("$.panels: unknown field", errors)

    def test_cli_validates_fixture(self):
        result = subprocess.run(
            [sys.executable, str(VALIDATOR), str(FIXTURE)],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("valid isometric scene", result.stdout.lower())


if __name__ == "__main__":
    unittest.main()
