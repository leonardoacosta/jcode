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


class ViewsOntologyTests(unittest.TestCase):
    def test_runtime_view_rejects_delivery_and_resource_group_topology_nodes(self):
        validator = load_module(VALIDATOR, "isometric_views_validator_ontology")
        scene = json.loads(SCENE_FIXTURE.read_text())
        views = json.loads(VIEWS_FIXTURE.read_text())

        delivery_node_ids = {
            node["id"]
            for node in scene["nodes"]
            if node.get("role") == "pipeline" or node.get("zone") == "delivery"
        }
        scene["nodes"] = [node for node in scene["nodes"] if node["id"] not in delivery_node_ids]
        views["runtime"]["node_ids"] = [
            node_id for node_id in views["runtime"]["node_ids"] if node_id not in delivery_node_ids
        ]

        synthetic_pipeline_index = len(scene["nodes"])
        scene["nodes"].append(
            {
                "id": "synthetic-pipeline",
                "code": "CI",
                "label": "Synthetic deploy pipeline",
                "role": "pipeline",
                "form": "cube",
                "zone": "delivery",
                "position": {"x": 10, "y": 1},
                "footprint": {"width": 1, "depth": 1},
                "status": "active",
                "resource_type": "Azure DevOps pipeline",
                "icon": "az-release-pipeline",
                "description": "Synthetic invalid delivery primitive used to prove ADO sidecar ontology validation.",
                "evidence": [
                    {
                        "path": ".azuredevops/deploy.yml",
                        "lines": "12-44",
                        "claim": "Delivery primitives belong in the ADO sidecar, not runtime topology nodes.",
                    }
                ],
            }
        )
        views["runtime"]["node_ids"].append("synthetic-pipeline")

        synthetic_resource_group_index = len(scene["nodes"])
        scene["nodes"].append(
            {
                "id": "rg-runtime-node",
                "code": "RG",
                "label": "runtime resource group",
                "role": "network",
                "form": "cube",
                "zone": "runtime",
                "position": {"x": 10, "y": 2},
                "footprint": {"width": 1, "depth": 1},
                "status": "active",
                "resource_type": "Microsoft.Resources/resourceGroups",
                "icon": "az-resource-group",
                "description": "Synthetic invalid topology node for resource group containment ontology.",
                "evidence": [
                    {
                        "path": "infra/main.bicep",
                        "lines": "19-40",
                        "claim": "Resource groups should be represented by Network containment, not runtime topology nodes.",
                    }
                ],
            }
        )
        views["runtime"]["node_ids"].append("rg-runtime-node")

        errors = validator.validate_views(views, scene)

        expected_errors = [
            f"scene.nodes[{synthetic_pipeline_index}]: delivery primitive 'synthetic-pipeline' belongs in the ADO sidecar, not as a views-enabled Azure topology node",
            f"scene.nodes[{synthetic_resource_group_index}]: resource group 'rg-runtime-node' belongs in Network containment, not as a topology node",
        ]
        for expected in expected_errors:
            with self.subTest(expected=expected):
                self.assertIn(expected, errors)


if __name__ == "__main__":
    unittest.main()
