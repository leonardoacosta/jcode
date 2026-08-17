import json
import unittest
from pathlib import Path


SCENE_FIXTURE = Path(__file__).parent / "fixtures" / "directional-scene.json"
VIEWS_FIXTURE = Path(__file__).parent / "fixtures" / "directional-views.json"


class DirectionalViewsFixtureTests(unittest.TestCase):
    def test_runtime_and_delivery_fixtures_are_separated(self):
        scene = json.loads(SCENE_FIXTURE.read_text())
        views = json.loads(VIEWS_FIXTURE.read_text())

        nodes = {node["id"]: node for node in scene["nodes"]}
        paths = {path["id"]: path for path in scene["paths"]}
        runtime_node_ids = set(views["runtime"]["node_ids"])
        runtime_path_ids = set(views["runtime"].get("path_ids", []))
        pipeline_collections = views.get("pipelines", [])
        delivery_stage_targets = [
            stage.get("target_node_id")
            for pipeline in pipeline_collections
            for stage in pipeline.get("stages", [])
            if stage.get("stage_type") == "deployment"
        ]

        violations = []
        if any(node.get("role") == "pipeline" for node in scene["nodes"]):
            violations.append("core scene includes role 'pipeline'")
        if any(node.get("resource_type") == "Azure DevOps pipeline" for node in scene["nodes"]):
            violations.append("core scene includes Azure DevOps pipeline resource_type")
        if any(path.get("kind") == "delivery" for path in scene["paths"]):
            violations.append("core scene includes delivery-only path")
        if any("deployment" in path.get("payload_ids", []) for path in scene["paths"]):
            violations.append("core scene includes deployment payload")
        delivery_primitive_ids = {
            node_id
            for node_id, node in nodes.items()
            if node.get("role") == "pipeline" or node.get("resource_type") == "Azure DevOps pipeline"
        } | {
            path_id
            for path_id, path in paths.items()
            if path.get("kind") == "delivery" or "deployment" in path.get("payload_ids", [])
        }
        runtime_delivery_ids = sorted((runtime_node_ids | runtime_path_ids) & delivery_primitive_ids)
        if runtime_delivery_ids:
            violations.append(f"runtime projection includes delivery primitive IDs: {runtime_delivery_ids}")
        if not pipeline_collections:
            violations.append("companion pipelines collection is empty")
        missing_stage_targets = sorted(target for target in delivery_stage_targets if target not in nodes)
        if not delivery_stage_targets:
            violations.append("companion pipelines collection has no deployment stages targeting scene resources")
        elif missing_stage_targets:
            violations.append(f"deployment stages target unknown scene resources: {missing_stage_targets}")

        self.assertEqual([], violations)


if __name__ == "__main__":
    unittest.main()
