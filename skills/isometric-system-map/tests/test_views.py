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


class ViewsContractTests(unittest.TestCase):
    def test_valid_version1_companion_fixture_validates_against_public_api(self):
        if not VALIDATOR.exists():
            self.fail(f"missing_import: expected validator module at {VALIDATOR}")

        validator = load_module(VALIDATOR, "isometric_views_validator")
        if not hasattr(validator, "validate_views"):
            self.fail("missing_function: expected validate_views(views, scene) public API")

        scene = json.loads(SCENE_FIXTURE.read_text())
        views = json.loads(VIEWS_FIXTURE.read_text())
        self.assertEqual(validator.validate_views(views, scene), [])

    def test_version1_rejects_unknown_top_level_key_with_stable_diagnostic(self):
        validator = load_module(VALIDATOR, "isometric_views_validator")
        scene = json.loads(SCENE_FIXTURE.read_text())
        views = json.loads(json.dumps(json.loads(VIEWS_FIXTURE.read_text())))
        views["unexpected"] = True

        self.assertIn(
            "$.unexpected: unknown key",
            validator.validate_views(views, scene),
        )

    def test_runtime_projection_references_valid_scene_nodes_and_includes_traffic_layer_members(self):
        validator = load_module(VALIDATOR, "isometric_views_validator")
        scene = json.loads(SCENE_FIXTURE.read_text())

        cases = [
            (
                "unknown runtime node",
                lambda views: views["runtime"]["node_ids"].append("missing-node"),
                "runtime.node_ids[6]: unknown scene node 'missing-node'",
            ),
            (
                "missing traffic-layer member",
                lambda views: views["runtime"]["node_ids"].remove("apim"),
                "runtime.node_ids: missing traffic-layer member 'apim'",
            ),
        ]

        for name, mutate, expected in cases:
            with self.subTest(name=name):
                views = json.loads(json.dumps(json.loads(VIEWS_FIXTURE.read_text())))
                mutate(views)
                self.assertIn(expected, validator.validate_views(views, scene))

    def test_runtime_flow_ids_are_optional_in_version1(self):
        validator = load_module(VALIDATOR, "isometric_views_validator")
        scene = json.loads(SCENE_FIXTURE.read_text())
        views = json.loads(VIEWS_FIXTURE.read_text())
        views["runtime"].pop("flow_ids")

        self.assertEqual([], validator.validate_views(views, scene))

    def test_network_sidecar_rejects_invalid_strict_semantics(self):
        validator = load_module(VALIDATOR, "isometric_views_validator")
        scene = json.loads(SCENE_FIXTURE.read_text())

        cases = [
            (
                "container evidence required",
                lambda views: views["network"]["containers"][0].__setitem__("evidence", []),
                "network.containers[0].evidence: requires at least one path/lines/claim evidence object",
            ),
            (
                "container containment cycle",
                lambda views: views["network"]["containers"][0].__setitem__("parent_id", "subnet-runtime"),
                "network.containers: containment cycle includes 'subscription-main'",
            ),
            (
                "duplicate membership",
                lambda views: views["network"]["memberships"].append(
                    json.loads(json.dumps(views["network"]["memberships"][0]))
                ),
                "network.memberships[5]: duplicate node membership 'app'",
            ),
            (
                "unknown link target",
                lambda views: views["network"]["links"][0].__setitem__("target_id", "missing-target"),
                "network.links[0].target_id: unknown network target 'missing-target'",
            ),
        ]

        for name, mutate, expected in cases:
            with self.subTest(name=name):
                views = json.loads(json.dumps(json.loads(VIEWS_FIXTURE.read_text())))
                mutate(views)
                self.assertIn(expected, validator.validate_views(views, scene))

    def test_network_sidecar_rejects_noncanonical_azure_ontology(self):
        validator = load_module(VALIDATOR, "isometric_views_validator")

        def append_scene_node(scene, node_id, resource_type, icon):
            scene["nodes"].append(
                {
                    "id": node_id,
                    "code": "META",
                    "label": node_id,
                    "role": "network",
                    "form": "cube",
                    "zone": "runtime",
                    "position": {"x": 10, "y": 2},
                    "footprint": {"width": 1, "depth": 1},
                    "status": "active",
                    "resource_type": resource_type,
                    "icon": icon,
                    "description": "Invalid topology node used to prove canonical Azure ontology validation.",
                    "evidence": [
                        {
                            "path": "infra/network/main.bicep",
                            "lines": "1-72",
                            "claim": "This resource must be metadata, an area, or an edge instead of a scene node.",
                        }
                    ],
                }
            )

        cases = [
            (
                "subnet CIDR must be a string container field",
                lambda scene, views: views["network"]["containers"][3].__setitem__("cidr", ["10.42.1.0/24"]),
                "network.containers[3].cidr: subnet CIDR must be a string such as '10.42.1.0/24'",
            ),
            (
                "network links require canonical evidence levels",
                lambda scene, views: views["network"]["links"][0].pop("evidence_level"),
                "network.links[0].evidence_level: required canonical evidence level direct, inferred, or held",
            ),
            (
                "SQL PaaS resource cannot be nested in subnet",
                lambda scene, views: views["network"]["memberships"][3].__setitem__("container_id", "subnet-runtime"),
                "network.memberships[3]: SQL PaaS resource 'database' is resource-group scoped and cannot be directly contained by subnet 'subnet-runtime'",
            ),
            (
                "private endpoint must be nested in subnet",
                lambda scene, views: views["network"]["memberships"][1].__setitem__("container_id", "resource-group-runtime"),
                "network.memberships[1]: private endpoint 'sql-private-endpoint' must be directly contained by subnet container 'subnet-runtime'",
            ),
            (
                "APIM policy is metadata not topology",
                lambda scene, views: (
                    append_scene_node(
                        scene,
                        "apim-policy",
                        "Microsoft.ApiManagement/service/apis/policies",
                        "az-apim",
                    ),
                    views["runtime"]["node_ids"].append("apim-policy"),
                ),
                lambda scene: f"scene.nodes[{len(scene['nodes'])}]: APIM policy/configuration 'apim-policy' is metadata and must not be modeled as a topology node",
            ),
            (
                "VNet boundary is area or edge not topology",
                lambda scene, views: (
                    append_scene_node(
                        scene,
                        "runtime-vnet-node",
                        "Microsoft.Network/virtualNetworks",
                        "az-vnet",
                    ),
                    views["runtime"]["node_ids"].append("runtime-vnet-node"),
                ),
                lambda scene: f"scene.nodes[{len(scene['nodes'])}]: VNet/subnet/peering boundary 'runtime-vnet-node' must be modeled as an area or edge, not as a topology node",
            ),
        ]

        for name, mutate, expected in cases:
            with self.subTest(name=name):
                scene = json.loads(json.dumps(json.loads(SCENE_FIXTURE.read_text())))
                views = json.loads(json.dumps(json.loads(VIEWS_FIXTURE.read_text())))
                expected_message = expected(scene) if callable(expected) else expected
                mutate(scene, views)
                self.assertIn(expected_message, validator.validate_views(views, scene))

    def test_pipeline_rejects_invalid_strict_semantics(self):
        validator = load_module(VALIDATOR, "isometric_views_validator")
        scene = json.loads(SCENE_FIXTURE.read_text())

        def create_cycle(views):
            for edge in views["pipelines"][0]["edges"]:
                if edge["id"] == "edge-gate-deploy-app":
                    edge["target_id"] = "repo"
                    return
            self.fail("missing fixture edge 'edge-gate-deploy-app'")

        cases = [
            (
                "stage evidence required",
                lambda views: views["pipelines"][0]["stages"][0].__setitem__("evidence", []),
                "pipelines[0].stages[0].evidence: requires at least one path/lines/claim evidence object",
            ),
            (
                "stage icon supported",
                lambda views: views["pipelines"][0]["stages"][0].__setitem__("icon", "az-not-real"),
                "pipelines[0].stages[0].icon: unsupported icon 'az-not-real'",
            ),
            (
                "edge target exists",
                lambda views: views["pipelines"][0]["edges"][0].__setitem__("target_id", "missing-stage"),
                "pipelines[0].edges[0].target_id: unknown pipeline stage 'missing-stage'",
            ),
            (
                "edge graph acyclic",
                create_cycle,
                "pipelines[0].edges: cycle includes 'repo'",
            ),
        ]

        for name, mutate, expected in cases:
            with self.subTest(name=name):
                views = json.loads(json.dumps(json.loads(VIEWS_FIXTURE.read_text())))
                mutate(views)
                self.assertIn(expected, validator.validate_views(views, scene))
