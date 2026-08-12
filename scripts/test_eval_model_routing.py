#!/usr/bin/env python3
"""Tests for the stdlib-only model routing evaluation CLI."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "eval_model_routing.py"
DESC = ROOT / "evals/model-routing/experiments/local-smoke.json"

class ModelRoutingCliTests(unittest.TestCase):
    def run_cli(self, *args: str, expect: int = 0) -> dict:
        proc = subprocess.run([sys.executable, str(SCRIPT), *args], cwd=ROOT, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        self.assertEqual(proc.returncode, expect, proc.stderr or proc.stdout)
        if expect == 0:
            return json.loads(proc.stdout)
        return {"stderr": proc.stderr, "stdout": proc.stdout}

    def event(self, **kw: object) -> dict:
        base = {
            "attempt_id": "a1", "attempt_index": 0, "role": "semantic-synthesis", "fixture_id": "mr-qualify-synthesis-normal",
            "route_id": "jcode:claude-api:claude-fable-5", "provider": "anthropic", "status": "accepted",
            "confounded": False, "confound_type": None, "cost_usd": 0.42, "normalized_cost_usd": 0.40,
            "latency_ms": 1200, "queue_ms": 10, "ttft_ms": 20, "model_ms": 1000, "tool_ms": 100,
            "judge_ms": 70, "wall_ms": 1200, "timeout": False, "repair_count": 1, "defect_count": 0,
            "request_digest": "sha256:" + "1"*64, "response_digest": "sha256:" + "2"*64,
            "input_tokens": 100, "output_tokens": 20, "reasoning_tokens": 5, "cache_read_tokens": 0,
            "cache_write_tokens": 0, "reasoning_control": "low", "cache_stratum": "cold", "cache_hit": False,
            "tool_calls": [], "shell_activity": [], "artifacts": [], "safety_stops": [], "retry_of": None,
        }
        base.update(kw)
        return base

    def test_validate_catalog_and_dry_run_no_provider_traffic(self) -> None:
        result = self.run_cli("validate")
        self.assertTrue(result["valid"])
        self.assertFalse(result["provider_traffic"])
        self.assertRegex(result["experiment_id"], r"^mr-[0-9a-f]{16}$")
        self.assertIn("partitions", result["checked"])
        estimate = self.run_cli("dry-run-cost")
        self.assertFalse(estimate["will_schedule_trials"])
        self.assertIn("total_conservative_usd", estimate)
        self.assertIn("provider_bounds", estimate)
        self.assertEqual(estimate["normalized_currency"], "USD")

    def test_missing_budget_unavailable_route_partition_overlap_and_leakage_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            base = json.loads(DESC.read_text())
            del base["power_budget"]["total_spending_cap_usd"]
            path = Path(td) / "bad.json"; path.write_text(json.dumps(base))
            self.assertIn("power_budget.total_spending_cap_usd", self.run_cli("validate", "--descriptor", str(path), expect=1)["stderr"])
            base = json.loads(DESC.read_text()); base["routes"][0]["availability"] = "unauthenticated"; path.write_text(json.dumps(base))
            self.assertIn("unavailable route", self.run_cli("smoke-ready", "--descriptor", str(path), expect=1)["stderr"])
            base = json.loads(DESC.read_text()); base["fixtures"] = ["expected model is hidden"] ; path.write_text(json.dumps(base))
            self.assertIn("leakage detected", self.run_cli("validate", "--descriptor", str(path), expect=1)["stderr"])
            bad_partitions = Path(td) / "partitions.json"; bad_partitions.write_text(json.dumps({"version":1,"partitions":{"development":[{"fixture_id":"dup","digest":"sha256:"+"1"*64,"case_type":"normal"}],"holdout":[{"fixture_id":"dup","digest":"sha256:"+"2"*64,"case_type":"boundary"}]},"required_case_types":["normal","boundary"]}))
            self.assertIn("cross-partition reuse", self.run_cli("validate-partitions", "--partitions", str(bad_partitions), expect=1)["stderr"])

    def test_plan_blocks_are_deterministic_isolated_and_have_repetitions_and_strata(self) -> None:
        first = self.run_cli("plan-blocks"); second = self.run_cli("plan-blocks")
        self.assertEqual(first["blocks"], second["blocks"])
        strata = {b["cache_stratum"] for b in first["blocks"]}
        self.assertEqual(strata, {"cold", "warm"})
        attempts = [a for b in first["blocks"] for a in b["attempts"]]
        self.assertEqual(len({a["attempt_id"] for a in attempts}), len(attempts))
        self.assertTrue(all(a["workspace_mode"] == "immutable-input-bundle" for a in attempts))

    def test_events_replay_collisions_partials_retry_lineage_and_bundle_are_deterministic(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            events = Path(td) / "events.jsonl"; bundle = Path(td) / "bundle"
            event_result = self.run_cli("append-event", "--events", str(events), "--event", json.dumps(self.event()))
            self.assertEqual(event_result["event_count"], 1)
            dup = self.run_cli("append-event", "--events", str(events), "--event", json.dumps(self.event()), expect=1)
            self.assertIn("duplicate attempt_id", dup["stderr"])
            retry = self.event(attempt_id="a2", attempt_index=1, status="provider_error", confounded=True, confound_type="provider", retry_of="a1", cost_usd=0.01, normalized_cost_usd=0.01, latency_ms=10, wall_ms=10)
            self.run_cli("append-event", "--events", str(events), "--event", json.dumps(retry))
            events.write_text(events.read_text() + '{"partial":')
            replay = self.run_cli("replay", "--events", str(events))
            self.assertEqual(replay["event_count"], 2)
            self.assertEqual(replay["recovered_partial_lines"], 1)
            self.assertEqual(replay["aggregates"]["defect_escape_rate"], 0.0)
            self.assertEqual(replay["aggregates"]["retry_attempts"], 1)
            bundle_result = self.run_cli("bundle", "--events", str(events), "--output", str(bundle))
            manifest = json.loads(Path(bundle_result["manifest"]).read_text())
            self.assertFalse(manifest["canonical"])
            self.assertIn("events_digest", manifest)

    def test_event_schema_timing_token_price_and_safety_stop_validation(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            events = Path(td) / "events.jsonl"
            bad = self.event(); del bad["request_digest"]
            self.assertIn("event missing keys", self.run_cli("append-event", "--events", str(events), "--event", json.dumps(bad), expect=1)["stderr"])
            bad = self.event(tool_calls=[{"action":"payment"}])
            self.assertIn("safety stop", self.run_cli("append-event", "--events", str(events), "--event", json.dumps(bad), expect=1)["stderr"])
            bad = self.event(normalized_cost_usd=-1)
            self.assertIn("normalized_cost_usd", self.run_cli("append-event", "--events", str(events), "--event", json.dumps(bad), expect=1)["stderr"])

    def test_anonymize_judges_calibration_receipts_adjudication_and_recon_fail_closed(self) -> None:
        candidate = json.dumps({"route_id": "jcode:openai:gpt-5.5", "provider": "openai", "text": "OpenAI route solved it"})
        anon = self.run_cli("anonymize", "--candidate", candidate)
        self.assertNotIn("openai", json.dumps(anon).lower())
        self.assertIn("candidate model cannot be its own sole judge", self.run_cli("validate-judges", "--candidate-route", "jcode:openai:gpt-5.5", "--judges", "jcode:openai:gpt-5.5", expect=1)["stderr"])
        cal = self.run_cli("judge-calibration")
        self.assertIn("false_positive_rate", cal)
        receipt = self.run_cli("judge-receipt", "--candidate-digest", "sha256:" + "a"*64, "--judge-id", "cold-review-openai", "--verdict", "fail")
        self.assertTrue(receipt["immutable"])
        invalid = self.run_cli("invalidate-receipt", "--receipt", json.dumps(receipt), "--candidate-digest", "sha256:" + "b"*64)
        self.assertTrue(invalid["invalidated"])
        adj = self.run_cli("adjudicate", "--candidate-digest", "sha256:" + "a"*64, "--decision", "material-disagreement-human-required")
        self.assertTrue(adj["human_adjudication_recorded"])
        self.assertIn("Recon publication unavailable", self.run_cli("publish-recon", expect=1)["stderr"])

    def test_smoke_selection_spending_stop_qualification_holdout_and_promotion_reports(self) -> None:
        report = self.run_cli("selection-report")
        self.assertFalse(report["mutates_production_routing"])
        self.assertIn("acceptance_blocked", json.dumps(report))
        qual = self.run_cli("qualification-plan")
        self.assertTrue(qual["frozen_repetitions"])
        self.assertFalse(qual["provider_traffic"])
        holdout = self.run_cli("holdout-report")
        self.assertTrue(holdout["holdout_blind"])
        self.assertIn("non_inferiority_margin", holdout)
        promo = self.run_cli("promotion-report")
        self.assertFalse(promo["mutates_production_routing"])
        stop = self.run_cli("spending-stop", "--spent-usd", "10.01")
        self.assertTrue(stop["stop_new_scheduling"])
        self.assertIn("incomplete_cells", stop)

if __name__ == "__main__":
    unittest.main()
