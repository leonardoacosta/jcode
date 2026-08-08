#!/usr/bin/env python3
"""Tests for task decomposition eval preparation helpers."""

from __future__ import annotations

import argparse
import importlib.util
import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "eval_task_decomposition.py"
spec = importlib.util.spec_from_file_location("eval_task_decomposition", SCRIPT)
assert spec is not None and spec.loader is not None
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class EvalPreparationTests(unittest.TestCase):
    def test_catalog_validates_intent_contracts(self) -> None:
        result = module.validate_catalog()

        self.assertGreaterEqual(result["fixture_count"], 3)
        self.assertIn("free-design-otaku-staff-console", result["contract_fixture_ids"])
        self.assertEqual(result["failures"], [])

    def test_catalog_rejects_missing_intent_contract(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            catalog_path = Path(tmp) / "catalog.json"
            source = json.loads(module.DEFAULT_CATALOG.read_text())
            source["fixtures"] = [dict(source["fixtures"][0])]
            source["fixtures"][0].pop("intent_contract", None)
            catalog_path.write_text(json.dumps(source))

            with self.assertRaisesRegex(module.EvalError, "intent_contract"):
                module.validate_catalog(catalog_path)

    def test_catalog_rejects_malformed_intent_contract(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            catalog_path = Path(tmp) / "catalog.json"
            source = json.loads(module.DEFAULT_CATALOG.read_text())
            source["fixtures"] = [dict(source["fixtures"][0])]
            source["fixtures"][0]["intent_contract"] = {
                "user_intent": "refresh the console",
                "scope_boundaries": {"in_scope": [], "out_of_scope": []},
                "expected_blast_radius": [],
                "non_goals": [],
                "ambiguity_traps": [],
                "reference_notes": "reference only",
            }
            catalog_path.write_text(json.dumps(source))

            with self.assertRaisesRegex(module.EvalError, "scope_boundaries.in_scope"):
                module.validate_catalog(catalog_path)

    def test_prompt_catalog_validates_known_fixture_ids(self) -> None:
        result = module.validate_prompt_catalog()

        self.assertGreaterEqual(result["prompt_count"], 3)
        self.assertIn("free-design-otaku-staff-console", result["fixture_ids"])
        self.assertEqual(result["failures"], [])

    def test_prepare_run_validates_without_materializing_or_running(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            repo = tmp_path / "otaku"
            repo.mkdir()
            (repo / ".git").mkdir()
            output = tmp_path / "fixture-output"
            args = argparse.Namespace(
                catalog=module.DEFAULT_CATALOG,
                prompts=module.DEFAULT_PROMPTS,
                fixture="free-design-otaku-staff-console",
                repo_root=[f"otaku-odyssey={repo}"],
                output=output,
                baseline_mode="jcode-openspec",
            )

            with patch.object(module, "verify_commit") as verify_commit:
                with patch.object(module, "run", return_value=subprocess.CompletedProcess([], 0, "", "")):
                    result = module.prepare_run(args)

        self.assertEqual(result["fixture"], "free-design-otaku-staff-console")
        self.assertEqual(result["baseline_mode"], "jcode-openspec")
        self.assertFalse(result["will_materialize"])
        self.assertFalse(result["will_run_model"])
        self.assertEqual(result["prompt"]["kind"], "reconstructed")
        self.assertEqual(result["intent_contract"]["user_intent"].startswith("Refresh"), True)
        self.assertEqual(verify_commit.call_count, 2)

    def test_prepare_run_rejects_missing_prompt_metadata(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            prompt_path = tmp_path / "prompts.json"
            prompt_path.write_text(json.dumps({"version": 1, "prompts": []}))
            repo = tmp_path / "otaku"
            repo.mkdir()
            (repo / ".git").mkdir()
            args = argparse.Namespace(
                catalog=module.DEFAULT_CATALOG,
                prompts=prompt_path,
                fixture="free-design-otaku-staff-console",
                repo_root=[f"otaku-odyssey={repo}"],
                output=tmp_path / "out",
                baseline_mode="jcode-openspec",
            )

            with self.assertRaisesRegex(module.EvalError, "missing prompt metadata"):
                with patch.object(module, "verify_commit"):
                    with patch.object(module, "run", return_value=subprocess.CompletedProcess([], 0, "", "")):
                        module.prepare_run(args)

    def test_prepare_run_rejects_contaminated_base_commit(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            repo = tmp_path / "otaku"
            repo.mkdir()
            (repo / ".git").mkdir()
            args = argparse.Namespace(
                catalog=module.DEFAULT_CATALOG,
                prompts=module.DEFAULT_PROMPTS,
                fixture="free-design-otaku-staff-console",
                repo_root=[f"otaku-odyssey={repo}"],
                output=tmp_path / "out",
                baseline_mode="jcode-openspec",
            )

            contaminated = subprocess.CompletedProcess(
                [],
                0,
                "openspec/changes/refresh-staff-operations-console/proposal.md\n",
                "",
            )
            with self.assertRaisesRegex(module.EvalError, "already contains"):
                with patch.object(module, "verify_commit"):
                    with patch.object(module, "run", return_value=contaminated):
                        module.prepare_run(args)

    def test_validate_rubric_score_computes_average(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            score_path = Path(tmp) / "rubric-score.json"
            score_path.write_text(json.dumps({
                "version": 1,
                "fixture_id": "free-design-otaku-staff-console",
                "baseline_mode": "jcode-openspec",
                "reviewer": "test",
                "scores": {
                    "fidelity": 5,
                    "scope_lock": 4,
                    "blast_radius": 3,
                    "risk_dependency_ordering": 4,
                    "verification_executability": 5,
                },
                "notes": {
                    "fidelity": "preserves the requested outcome and domain intent",
                    "scope_lock": "keeps required surfaces without unrelated expansion",
                    "blast_radius": "identifies most affected routes and packages",
                    "risk_dependency_ordering": "orders risks and dependencies plausibly",
                    "verification_executability": "acceptance checks can be run by an engineer",
                },
            }))
            args = argparse.Namespace(
                catalog=module.DEFAULT_CATALOG,
                score=score_path,
            )

            result = module.validate_rubric_score(args)

        self.assertEqual(result["fixture"], "free-design-otaku-staff-console")
        self.assertEqual(result["average"], 4.2)
        self.assertEqual(result["dimensions"], list(module.RUBRIC_DIMENSIONS))
        self.assertEqual(result["failures"], [])

    def test_score_artifacts_labels_overlap_as_support_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            repo = tmp_path / "repo"
            candidate = tmp_path / "candidate"
            candidate.mkdir()
            (candidate / "proposal.md").write_text("proposal")
            fixture = json.loads(module.DEFAULT_CATALOG.read_text())["fixtures"][0]
            args = argparse.Namespace(
                catalog=module.DEFAULT_CATALOG,
                fixture="free-design-otaku-staff-console",
                repo_root=[f"otaku-odyssey={repo}"],
                candidate=candidate,
            )

            with patch.object(module, "require_repo_root", return_value=repo):
                with patch.object(module, "verify_commit"):
                    with patch.object(module, "list_gold_artifacts", return_value=["proposal.md"]):
                        with patch.object(module, "git_show", return_value="proposal"):
                            result = module.score_artifacts(args)

        self.assertEqual(result["score_kind"], "support_evidence")
        self.assertFalse(result["semantic_judge"])
        self.assertIn("not by itself determine planning quality", result["interpretation"])

    def test_extract_evidence_reports_blast_radius_and_non_goal_mentions(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            repo = tmp_path / "repo"
            candidate = tmp_path / "candidate"
            candidate.mkdir()
            (candidate / "proposal.md").write_text(
                "Refresh staff operations routes and RBAC checks. Do not implement billing."
            )
            args = argparse.Namespace(
                catalog=module.DEFAULT_CATALOG,
                fixture="free-design-otaku-staff-console",
                repo_root=[f"otaku-odyssey={repo}"],
                candidate=candidate,
            )

            with patch.object(module, "require_repo_root", return_value=repo):
                with patch.object(module, "verify_commit"):
                    with patch.object(module, "changed_paths", return_value=["apps/web/src/app/staff/operations/page.tsx"]):
                        result = module.extract_evidence(args)

        self.assertEqual(result["fixture"], "free-design-otaku-staff-console")
        self.assertIn("routes", result["expected_blast_radius"]["mentioned"])
        self.assertIn("auth/permissions", result["expected_blast_radius"]["mentioned"])
        self.assertIn("billing/payment changes", result["non_goals"]["mentioned"])
        self.assertEqual(result["reference_surfaces"]["routes"], 1)

    def test_route_proposal_paths_are_documentation_not_tests(self) -> None:
        self.assertEqual(
            module.classify_path_surface(
                "openspec/changes/refresh-staff-operations-console/route-proposals/001-admin.md"
            ),
            "docs/specs",
        )


if __name__ == "__main__":
    unittest.main()
