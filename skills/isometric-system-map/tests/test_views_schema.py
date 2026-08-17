import importlib.util
import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
VALIDATOR = ROOT / "scripts" / "validate_views.py"
SCENE_FIXTURE = Path(__file__).parent / "fixtures" / "directional-scene.json"
VIEWS_FIXTURE = Path(__file__).parent / "fixtures" / "directional-views.json"


def load_module(path: Path, name: str):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"Unable to load {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class ViewsSchemaStrictnessTests(unittest.TestCase):
    def test_network_container_unknown_key_reports_stable_nested_field_path(self):
        validator = load_module(VALIDATOR, "isometric_views_validator")
        scene = json.loads(SCENE_FIXTURE.read_text())
        views = json.loads(VIEWS_FIXTURE.read_text())
        views["network"]["containers"][0]["unexpected"] = True

        self.assertIn(
            "network.containers[0].unexpected: unknown key",
            validator.validate_views(views, scene),
        )


if __name__ == "__main__":
    unittest.main()
