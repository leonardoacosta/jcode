#!/usr/bin/env python3
"""Tests for task decomposition eval preparation helpers."""

from __future__ import annotations

import argparse
import importlib.util
import json
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
                result = module.prepare_run(args)

        self.assertEqual(result["fixture"], "free-design-otaku-staff-console")
        self.assertEqual(result["baseline_mode"], "jcode-openspec")
        self.assertFalse(result["will_materialize"])
        self.assertFalse(result["will_run_model"])
        self.assertEqual(result["prompt"]["kind"], "reconstructed")
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
                    "requirement_coverage": 5,
                    "decomposition_quality": 4,
                    "risk_handling": 3,
                    "scope_control": 4,
                    "executability": 5,
                },
                "notes": {
                    "requirement_coverage": "covers the main requested outcomes",
                    "decomposition_quality": "tasks are ordered and bounded",
                    "risk_handling": "risks are present but not exhaustive",
                    "scope_control": "avoids extra systems",
                    "executability": "engineer can run the checklist",
                },
            }))
            args = argparse.Namespace(
                catalog=module.DEFAULT_CATALOG,
                score=score_path,
            )

            result = module.validate_rubric_score(args)

        self.assertEqual(result["fixture"], "free-design-otaku-staff-console")
        self.assertEqual(result["average"], 4.2)
        self.assertEqual(result["failures"], [])


if __name__ == "__main__":
    unittest.main()
