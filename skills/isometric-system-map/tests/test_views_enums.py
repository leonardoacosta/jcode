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


class NetworkLinkEnumTests(unittest.TestCase):
    def test_network_links_use_canonical_kind_direction_and_evidence_level_enums(self):
        validator = load_module(VALIDATOR, "isometric_views_validator")
        scene = json.loads(SCENE_FIXTURE.read_text())

        canonical_kind_message = (
            "unsupported canonical network link kind '{value}'; "
            "expected peering, private-endpoint, dns, or data"
        )
        cases = [
            (
                "unsupported kind",
                self._with_link_field("kind", "vpn"),
                "network.links[0].kind: " + canonical_kind_message.format(value="vpn"),
            ),
            (
                "unsupported direction",
                self._with_link_field("direction", "sideways"),
                "network.links[0].direction: unsupported canonical network link direction 'sideways'; expected forward, reverse, or both",
            ),
            (
                "unsupported evidence_level",
                self._with_link_field("evidence_level", "guessed"),
                "network.links[0].evidence_level: unsupported canonical network link evidence_level 'guessed'; expected direct, inferred, or held",
            ),
        ]

        for name, views, expected in cases:
            with self.subTest(name=name):
                self.assertIn(expected, validator.validate_views(views, scene))

    def _with_link_field(self, field: str, value: str):
        views = json.loads(VIEWS_FIXTURE.read_text())
        views["network"]["links"][1]["kind"] = "private-endpoint"
        views["network"]["links"][0][field] = value
        return views


if __name__ == "__main__":
    unittest.main()
