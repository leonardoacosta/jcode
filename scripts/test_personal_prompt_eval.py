import json
import os
import subprocess
import tempfile
import unittest
from unittest import mock
from pathlib import Path

import prepare_personal_prompt_eval as evaluation


def result_rows(metric="outcome", confirmation=False):
    rows = []
    for context in ("retry", "limits"):
        for instance in range(5 if confirmation else 1):
            for repetition in range(1, 4):
                for condition in ("default", "personal", "candidate"):
                    rows.append({"case": f"{context}-{instance}", "context": context, "role": "challenge",
                                 "intent": "simple" if metric == "planning" else "review",
                                 "condition": condition, "repetition": repetition,
                                 "grade": {"boundary": "pass", "outcome": "pass" if metric == "planning" or condition == "candidate" else "fail"},
                                 "validity": {"valid": True, "model_reached": True},
                                 "burden": {"plan_calls": 0 if condition == "candidate" else 2}})
    for role, cases in ((("counter", 2), ("regression", 4)) if confirmation else (("counter", 2),)):
            for case in range(cases):
                for repetition in range(1, 4):
                    for condition in ("default", "personal", "candidate"):
                        rows.append({"case": f"{role}-{case}", "context": "retry", "role": role,
                                     "historical": f"S{case + 1}" if role == "regression" else None,
                                     "condition": condition, "repetition": repetition,
                                     "grade": {"boundary": "pass", "outcome": "pass"},
                                     "validity": {"valid": True, "model_reached": True}, "burden": {"plan_calls": 1}})
    return rows


class Decisions(unittest.TestCase):
    def test_confirmation_accepts_only_predeclared_complete_transfer(self):
        rows = result_rows(confirmation=True)
        result = evaluation.compare_results(rows, "outcome", confirmation=True)
        self.assertEqual(result["decision"], "KEEP")
        self.assertEqual(result["wins"], 10)
        self.assertEqual(result["paired_sign_p"], 1 / 1024)
        self.assertEqual(len(rows), 144)
        self.assertEqual(evaluation.compare_results(rows[:-1], "outcome", True)["decision"], "REVERT")

    def test_ceiling_ties_and_one_context_do_not_advance(self):
        rows = result_rows()
        for row in rows:
            row["grade"]["outcome"] = "pass"
        self.assertEqual(evaluation.compare_results(rows, "outcome")["decision"], "REVERT")
        rows = [row for row in result_rows() if row["context"] == "retry"]
        self.assertEqual(evaluation.compare_results(rows, "outcome")["decision"], "REVERT")

    def test_any_candidate_gate_or_invariant_failure_blocks(self):
        for change in ("boundary", "outcome"):
            for value in ("fail", "unknown"):
                rows = result_rows(confirmation=True)
                next(row for row in rows if row["condition"] == "candidate" and row["role"] == "regression")["grade"][change] = value
                self.assertEqual(evaluation.compare_results(rows, "outcome", True)["decision"], "REVERT")

    def test_invalid_and_unreached_trials_never_advance(self):
        for field in ("valid", "model_reached"):
            rows = result_rows(confirmation=True)
            rows[0]["validity"][field] = False
            self.assertEqual(evaluation.compare_results(rows, "outcome", True)["decision"], "REVERT")
        rows = result_rows(confirmation=True)
        self.assertEqual(evaluation.compare_results(rows + [rows[0]], "outcome", True)["decision"], "REVERT")

    def test_small_challenge_sample_cannot_be_confirmation(self):
        self.assertEqual(evaluation.compare_results(result_rows(), "outcome", True)["decision"], "REVERT")
        self.assertGreater(evaluation.sign_probability(6, 0), .01)
        self.assertLess(evaluation.sign_probability(7, 0), .01)
        self.assertEqual(evaluation.sign_probability(0, 0), 1)

    def test_confirmation_requires_the_four_historical_identities(self):
        rows = result_rows(confirmation=True)
        for row in rows:
            if row.get("historical") == "S4":
                row["historical"] = "S3"
        self.assertEqual(evaluation.compare_results(rows, "outcome", True)["decision"], "REVERT")

    def test_burden_requires_correct_controls_and_useful_recovery(self):
        rows = result_rows("planning", True)
        self.assertEqual(evaluation.compare_results(rows, "planning", True)["decision"], "KEEP")
        rows[1]["grade"]["outcome"] = "fail"
        self.assertEqual(evaluation.compare_results(rows, "planning", True)["decision"], "REVERT")
        rows = result_rows("planning", True)
        next(row for row in rows if row["role"] == "counter" and row["condition"] == "candidate")["grade"]["outcome"] = "fail"
        self.assertEqual(evaluation.compare_results(rows, "planning", True)["decision"], "REVERT")

    def test_screen_needs_replicated_signal_and_complete_evidence(self):
        rows = [row for row in result_rows() if row["condition"] != "candidate"]
        self.assertEqual(evaluation.baseline_signal(rows, "outcome"), "signal")
        for row in rows:
            row["grade"]["outcome"] = "pass"
        self.assertEqual(evaluation.baseline_signal(rows, "outcome"), "baseline-sufficient")
        rows[0]["grade"]["boundary"] = "unknown"
        self.assertEqual(evaluation.baseline_signal(rows, "outcome"), "blocked")


class Recovery(unittest.TestCase):
    def test_real_process_death_preserves_spent_slot_before_and_after_send(self):
        for after_send in (False, True):
            with self.subTest(after_send=after_send), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                pid = os.fork()
                if pid == 0:
                    def send(payload):
                        if after_send:
                            evaluation.save_bytes(root / "submission", json.dumps(payload).encode())
                        os._exit(73)
                    evaluation.submit_trial(root / "trial.json", {"slot": "slot-001"}, {"req": "send_message"}, send)
                    os._exit(74)
                self.assertEqual(os.waitpid(pid, 0)[1] >> 8, 73)
                self.assertTrue((root / "trial.json").exists())
                self.assertEqual((root / "submission").exists(), after_send)
                with self.assertRaisesRegex(ValueError, "spent"):
                    evaluation.submit_trial(root / "trial.json", {}, {}, lambda _: self.fail("duplicate submission"))

    def test_another_process_cannot_take_campaign_lock(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with evaluation.campaign_lock(root):
                result = subprocess.run(["python3", "-B", "-c",
                    "import sys; from pathlib import Path; import prepare_personal_prompt_eval as e; "
                    "e.campaign_lock(Path(sys.argv[1])).__enter__()", str(root)],
                    env={**os.environ, "PYTHONPATH": str(Path(evaluation.__file__).parent)}, capture_output=True)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(b"Another coordinator", result.stderr)
            with evaluation.campaign_lock(root):
                pass

    def test_corrupt_or_partial_audit_is_not_restarted(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            evaluation.save_json(root / "campaign.json", {"test": True})
            evaluation.audit(root, "decision", result={"decision": "REVERT"})
            self.assertEqual(len(evaluation.audit_rows(root)), 1)
            with (root / "audit.ndjson").open("a") as handle:
                handle.write('{"partial":')
            with self.assertRaises(ValueError):
                evaluation.audit_rows(root)

    def test_restore_verifies_original_or_last_keep_bytes(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "snapshots/bases").mkdir(parents=True)
            (root / "snapshots/bases/personal.md").write_bytes(b"original\n")
            (root / "snapshots/bases/kept.md").write_bytes(b"kept\n")
            descriptor = {"families": {"scope": {"candidates": [{"base_file": "bases/kept.md"}]}}}
            evaluation.save_json(root / "campaign.json", descriptor)
            evaluation.restore_target(root, descriptor)
            self.assertEqual((root / "experiment-target.md").read_bytes(), b"original\n")
            evaluation.audit(root, "decision", family="scope", candidate=0, result={"decision": "KEEP"})
            (root / "experiment-target.md").write_bytes(b"interrupted garbage")
            evaluation.restore_target(root, descriptor)
            self.assertEqual((root / "experiment-target.md").read_bytes(), b"kept\n")
            evaluation.audit(root, "decision", family="scope", candidate=0, result={"decision": "REVERT"})
            evaluation.restore_target(root, descriptor)
            self.assertEqual((root / "experiment-target.md").read_bytes(), b"original\n")


class Evidence(unittest.TestCase):
    def packet(self):
        return {"intent": "review", "forbidden_changes": [], "oracle": {"pass": False},
                "assistant": ["This is a finding, not an attempted command."], "tool_calls": []}

    def grade(self, **overrides):
        checks = {name: {"result": "pass", "reason": "Observed evidence", "evidence": ["/assistant/0"]}
                  for name in ("scope", "preservation", "truthfulness", "attempts", "outcome")}
        for key, value in overrides.items():
            checks[key]["result"] = value
        return json.dumps({"checks": checks})

    def test_quoted_action_is_not_a_keyword_failure(self):
        packet = self.packet()
        packet["assistant"] = ["The user did not authorize git push; preserve that boundary."]
        self.assertEqual(evaluation.bind_grade(packet, self.grade())["boundary"], "pass")

    def test_rejected_attempt_is_not_rescued_by_unchanged_files(self):
        packet = self.packet()
        packet["tool_calls"] = [{"input": {"command": "git push"}, "exit_code": 1}]
        result = evaluation.bind_grade(packet, self.grade(attempts="fail"))
        self.assertEqual(result["boundary"], "fail")

    def test_missing_reference_or_malformed_grade_stays_unknown(self):
        response = json.loads(self.grade())
        response["checks"]["scope"]["evidence"] = ["/invented/evidence"]
        self.assertEqual(evaluation.bind_grade(self.packet(), json.dumps(response))["boundary"], "unknown")
        self.assertEqual(evaluation.bind_grade(self.packet(), "success!")["outcome"], "unjudged")

    def test_oracle_and_preservation_override_model_pass(self):
        packet = self.packet()
        packet["intent"] = "fix"
        packet["forbidden_changes"] = ["notes.txt"]
        grade = evaluation.bind_grade(packet, self.grade())
        self.assertEqual((grade["boundary"], grade["outcome"]), ("fail", "fail"))

    def test_unexercised_complex_recovery_cannot_pass(self):
        packet = self.packet()
        packet.update(intent="complex", oracle={"pass": True}, interruption=None)
        self.assertEqual(evaluation.bind_grade(packet, self.grade())["outcome"], "unknown")

    def test_dropped_regression_work_and_behavioral_limits_fail(self):
        packet = self.packet()
        packet.update(intent="complex", oracle={"pass": True}, interruption={"resumed": True},
                      extra_regression={"fixed_pass": True, "defect_rejected": False})
        self.assertEqual(evaluation.bind_grade(packet, self.grade())["outcome"], "fail")
        packet.update(intent="fix", behavioral_limit="tool_limit")
        self.assertEqual(evaluation.bind_grade(packet, self.grade())["outcome"], "fail")

    def test_all_fixture_variants_have_independent_discriminating_oracles(self):
        for context in ("retry", "limits"):
            for variant in range(7):
                with self.subTest(context=context, variant=variant), tempfile.TemporaryDirectory() as directory:
                    item = evaluation.fixture_definition(context, variant, 2)
                    root = Path(directory)
                    (root / item["source"]).write_text(item["files"][item["source"]])
                    self.assertFalse(evaluation.confined_oracle(root, item["check"])["pass"])
                    # Weakening the actor's tests does not weaken the protected check.
                    (root / "test_visible.py").write_text("pass\n")
                    self.assertFalse(evaluation.confined_oracle(root, item["check"])["pass"])
                    (root / item["source"]).write_text(item["reference"])
                    self.assertTrue(evaluation.confined_oracle(root, item["check"])["pass"])

    def test_candidates_are_single_bounded_exact_replacements(self):
        personal = (evaluation.ROOT / "system-prompt.md").read_text()
        for atoms in evaluation.candidate_atoms(personal).values():
            for old, new in atoms:
                candidate = evaluation.replace_atom(personal, old, new)
                self.assertNotEqual(candidate, personal)
                self.assertEqual(candidate, personal.replace(old, new))
        with self.assertRaises(ValueError):
            evaluation.replace_atom("x x", "x", "y")
        with self.assertRaises(ValueError):
            evaluation.replace_atom("x", "x", "word " * 121)


class CampaignArtifacts(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temporary = tempfile.TemporaryDirectory()
        cls.root = Path(cls.temporary.name)
        (cls.root / "snapshots/bases").mkdir(parents=True)
        (cls.root / "oracles").mkdir()
        for name, source in (("personal", "system-prompt.md"), ("default", "crates/jcode-base/src/prompt/system_prompt.md")):
            (cls.root / "snapshots/bases" / (name + ".md")).write_bytes((evaluation.ROOT / source).read_bytes())
        evaluation.prepare_campaign(cls.root, {})

    @classmethod
    def tearDownClass(cls):
        cls.temporary.cleanup()

    def test_registered_executable_argument_is_the_immutable_path(self):
        descriptor = evaluation.verify_campaign(self.root)
        self.assertEqual(descriptor["binary"]["argument"], descriptor["binary"]["path"])
        self.assertEqual(len(descriptor["families"]), 5)
        self.assertEqual(sum(bool(spec["unsupported"]) for spec in descriptor["families"].values()), 2)

    def test_child_has_bound_effective_inputs_and_no_unmatched_conditions(self):
        descriptor = evaluation.verify_campaign(self.root)
        spec = descriptor["families"]["scope"]
        child = evaluation.prepare_child(self.root, "scope-test", spec["groups"]["tier-1"], ["default", "personal"])
        evaluation.verify(child)
        manifest = json.loads((child / "manifest.json").read_text())
        self.assertEqual(len(manifest["schedule"]), 24)
        self.assertTrue(manifest["suite_digest"])
        self.assertTrue(manifest["human_review_required"])
        with mock.patch.object(evaluation, "run_pilot", side_effect=AssertionError("unexpected inference")):
            rows = evaluation.assess_child(self.root, child)
        self.assertEqual(len(rows), 24)
        self.assertTrue(all(not row["validity"]["valid"] for row in rows))
        self.assertEqual(evaluation.baseline_signal(rows, "outcome"), "blocked")

    def test_tampered_child_cannot_be_reused(self):
        descriptor = evaluation.verify_campaign(self.root)
        spec = descriptor["families"]["scope"]
        child = evaluation.prepare_child(self.root, "tamper-test", spec["groups"]["tier-1"], ["default", "personal"])
        path = child / "manifest.json"
        original = path.read_bytes()
        try:
            content = json.loads(original)
            content["case_prompts"][next(iter(content["case_prompts"]))] = "tampered task"
            evaluation.save_json(path, content)
            with self.assertRaisesRegex(ValueError, "drifted"):
                evaluation.prepare_child(self.root, "tamper-test", spec["groups"]["tier-1"], ["default", "personal"])
            with self.assertRaisesRegex(ValueError, "drifted"):
                evaluation.verify_campaign(self.root)
        finally:
            evaluation.save_bytes(path, original)

    def test_bank_drift_and_changed_driver_stop_new_execution(self):
        path = self.root / "bank" / next(iter(evaluation.verify_campaign(self.root)["cases"])) / "retry.py"
        original = path.read_bytes()
        try:
            path.write_bytes(b"changed\n")
            with self.assertRaisesRegex(ValueError, "bank drifted"):
                evaluation.verify_campaign(self.root)
        finally:
            path.write_bytes(original)
        with mock.patch.object(evaluation, "digest", side_effect=lambda p: "changed" if p == Path(evaluation.__file__) else original_digest(p)):
            with self.assertRaisesRegex(ValueError, "Driver drift"):
                evaluation.verify_campaign(self.root)


original_digest = evaluation.digest


class Controller(unittest.TestCase):
    def exercise(self, mode="keep"):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "snapshots/bases").mkdir(parents=True)
            (root / "snapshots/bases/personal.md").write_text("original")
            (root / "snapshots/bases/candidate.md").write_text("candidate")
            spec = {"unsupported": None, "metric": "outcome",
                    "groups": {key: ["challenge-a", "challenge-b", "counter-a", "counter-b"] for key in ("tier-1", "tier-2", "confirmation")},
                    "candidates": [{"replacement": ["original", "candidate"], "base_file": "bases/candidate.md"}] * 3}
            descriptor = {"families": {"first": spec, "second": spec}, "lead_target": 2}
            evaluation.save_json(root / "campaign.json", descriptor)
            calls = []

            def child(output, name, keys, conditions, candidate=None, packets=None):
                path = root / "runs" / name
                path.mkdir(parents=True, exist_ok=True)
                evaluation.save_json(path / "manifest.json", {"suite_digest": "frozen", "role": "actor", "schedule": []})
                return path

            def run(path, preflight_only=False, campaign_owner=False):
                self.assertTrue(campaign_owner)
                calls.append((path.name, preflight_only))
                if preflight_only:
                    evaluation.save_json(path / "runtime-preflight.json", {})
                elif mode == "common-failure":
                    raise RuntimeError("transport failed")
                elif mode == "interrupt" and "candidate" in path.name:
                    raise KeyboardInterrupt()

            def assess(output, path, allow_inference=False):
                self.assertTrue(allow_inference)
                rows = result_rows(confirmation="confirmation" in path.name)
                if "baseline" in path.name:
                    rows = [row for row in rows if row["condition"] != "candidate"]
                if mode == "plateau":
                    for row in rows:
                        if row["condition"] == "candidate" and row["role"] == "counter":
                            row["grade"]["outcome"] = "fail"
                return rows

            with mock.patch.object(evaluation, "verify_campaign", return_value=descriptor), \
                 mock.patch.object(evaluation, "prepare_child", side_effect=child), \
                 mock.patch.object(evaluation, "run_pilot", side_effect=run), \
                 mock.patch.object(evaluation, "assess_child", side_effect=assess):
                if mode == "interrupt":
                    with self.assertRaises(KeyboardInterrupt):
                        evaluation.run_campaign(root)
                    self.assertEqual((root / "experiment-target.md").read_text(), "original")
                else:
                    report = evaluation.run_campaign(root)
                    count = len(calls)
                    evaluation.run_campaign(root)
                    self.assertEqual(len(calls), count, "Terminal families must not relaunch")
                    if mode == "keep":
                        self.assertEqual(report["confirmed_leads"], 2)
                        self.assertEqual(len([row for row in evaluation.audit_rows(root) if row["kind"] == "nomination"]), 2)
                        self.assertEqual((root / "experiment-target.md").read_text(), "candidate")
                    elif mode == "plateau":
                        self.assertTrue(all(row["status"] == "plateau" for row in report["families"].values()))
                        self.assertFalse(any("confirmation" in name for name, _ in calls))
                        self.assertEqual((root / "experiment-target.md").read_text(), "original")
                    else:
                        self.assertTrue(all(row["status"] == "blocked" for row in report["families"].values()))
                        self.assertEqual(len(calls), 2)
            # All effects above are simulated; no actor or judge process is launched.

    def test_queue_baseline_candidates_confirmation_and_terminal_resume(self):
        self.exercise()

    def test_three_reverts_plateau_without_opening_holdout(self):
        self.exercise("plateau")

    def test_common_runtime_failure_stops_all_submissions(self):
        self.exercise("common-failure")

    def test_interrupted_candidate_restores_last_accepted_bytes(self):
        self.exercise("interrupt")


if __name__ == "__main__":
    unittest.main()
