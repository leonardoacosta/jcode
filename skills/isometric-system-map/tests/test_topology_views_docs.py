import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC_PATH = ROOT / "references" / "topology-views.md"


class TopologyViewsDocumentationTests(unittest.TestCase):
    def test_topology_views_reference_documents_normative_usage_grammar_metadata_tabs_and_provenance(self):
        self.assertTrue(
            DOC_PATH.exists(),
            "references/topology-views.md must exist and document topology views usage, grammar, metadata rules, semantics, rendering fallbacks, native tabs, and evidence provenance.",
        )

        text = DOC_PATH.read_text(encoding="utf-8")
        checks = {
            "optional --views usage": r"render_canvas\.py[^\n]+--views",
            "scene-only compatibility": r"scene[- ]only|without --views|no --views",
            "version 1 grammar": r"version\s*[:=]?\s*1",
            "repository/default/runtime/network/pipeline grammar": r"repository[\s\S]*default[\s\S]*runtime[\s\S]*network[\s\S]*pipeline",
            "validation command": r"python[^\n]+(jsonschema|validate|topology-views\.schema\.json)",
            "canonical Azure metadata": r"VNet|subnet|CIDR|private endpoint|PaaS|peering|APIM",
            "direct/inferred/held semantics": r"direct[\s\S]*inferred[\s\S]*held",
            "legacy direct default": r"legacy[\s\S]*direct[\s\S]*default",
            "package-owned Azure family fallbacks": r"package-owned|package owned[\s\S]*Azure[\s\S]*(family|fallback)",
            "native tabs and deep links": r"native tabs|tabs[\s\S]*deep links|deep links[\s\S]*tabs",
            "no-JS behavior": r"no-JS|without JavaScript|JavaScript disabled",
            "evidence provenance shape": r"\{\s*path\s*,\s*lines\s*,\s*claim\s*\}",
        }

        missing = [label for label, pattern in checks.items() if not re.search(pattern, text, re.IGNORECASE)]
        if missing:
            self.fail("references/topology-views.md is missing required documentation: " + ", ".join(missing))

    def test_topology_views_reference_uses_real_paths_and_matches_the_normative_contract(self):
        text = DOC_PATH.read_text(encoding="utf-8")

        self.assertIn("skills/isometric-system-map/themes/azure-topology.js", text)
        self.assertNotIn("skills/isometric-system-map/examples/themes/azure-technical.js", text)
        self.assertNotIn('"pipelines": []', text)
        self.assertRegex(text, r"flow_ids[^\n]+optional", "flow_ids must be documented as optional without a validator caveat")
        self.assertNotIn("the current validator expects this key", text)
        self.assertRegex(text, r"views validator[^\n]+Azure ontology", "Azure ontology enforcement belongs to the views validator")


if __name__ == "__main__":
    unittest.main()
