import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SCHEMA_PATH = ROOT / "references" / "topology-views.schema.json"


class TopologyViewsSchemaContractTests(unittest.TestCase):
    def test_topology_views_schema_is_normative_2020_12_v1_grammar(self):
        schema = json.loads(SCHEMA_PATH.read_text())
        defs = schema.get("$defs", {})

        failures = []

        def at(path):
            node = schema
            for part in path:
                if not isinstance(node, dict) or part not in node:
                    failures.append(f"missing schema path: {'/'.join(path)}")
                    return {}
                node = node[part]
            return node

        def require_enum(path, values):
            node = at(path)
            enum = node.get("enum")
            if enum != values:
                failures.append(
                    f"{'/'.join(path)} enum must be {values!r}, got {enum!r}"
                )

        def require_required(def_name, fields):
            node = defs.get(def_name)
            if not isinstance(node, dict):
                failures.append(f"missing object definition: {def_name}")
                return
            required = node.get("required")
            if required != fields:
                failures.append(
                    f"$defs/{def_name} required must be {fields!r}, got {required!r}"
                )

        self.assertEqual(
            schema.get("$schema"),
            "https://json-schema.org/draft/2020-12/schema",
            "schema must declare JSON Schema 2020-12",
        )
        self.assertEqual(schema.get("properties", {}).get("version"), {"const": 1})

        require_enum(["properties", "default_view"], ["runtime", "network", "ado"])
        require_enum(["$defs", "networkContainer", "properties", "kind"], ["subscription", "resource-group", "vnet", "subnet"])
        require_enum(["$defs", "networkLink", "properties", "kind"], ["peering", "private-endpoint", "dns", "data"])
        require_enum(["$defs", "networkLink", "properties", "direction"], ["forward", "reverse", "both"])
        require_enum(["$defs", "networkLink", "properties", "evidence_level"], ["direct", "inferred", "held"])
        require_enum(["$defs", "pipelineStage", "properties", "stage_type"], ["repository", "validation", "build", "artifact", "gate", "deployment", "held"])
        require_enum(["$defs", "pipelineEdge", "properties", "kind"], ["automatic", "dependency", "approval", "manual", "held"])

        require_required("evidence", ["path", "lines", "claim"])
        require_required("networkContainer", ["id", "kind", "label", "status", "evidence"])
        require_required("networkMembership", ["container_id", "node_id", "evidence"])
        require_required("networkLink", ["id", "kind", "source_id", "target_id", "direction", "evidence_level", "label", "evidence"])
        require_required("pipeline", ["id", "label", "stages", "edges"])
        require_required("pipelineStage", ["id", "label", "stage_type", "icon", "status", "evidence"])
        require_required("pipelineEdge", ["id", "source_id", "target_id", "label", "kind", "evidence"])

        for array_name in ("node_ids", "path_ids", "flow_ids"):
            node = at(["properties", "runtime", "properties", array_name])
            if node.get("uniqueItems") is not True:
                failures.append(f"runtime.{array_name} must set uniqueItems: true")

        cidr = at(["$defs", "networkContainer", "properties", "cidr"])
        if cidr.get("type") != "string":
            failures.append("$defs/networkContainer/properties/cidr must be a string property")

        def walk_objects(node, path="$"):
            if isinstance(node, dict):
                if node.get("type") == "object" and node.get("additionalProperties") is not False:
                    failures.append(f"{path} object must set additionalProperties: false")
                for key, value in node.items():
                    walk_objects(value, f"{path}/{key}")
            elif isinstance(node, list):
                for index, value in enumerate(node):
                    walk_objects(value, f"{path}/{index}")

        walk_objects(schema)

        if failures:
            self.fail("Topology views schema is still shallow:\n" + "\n".join(failures))


if __name__ == "__main__":
    unittest.main()
