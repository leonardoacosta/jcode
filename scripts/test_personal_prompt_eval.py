import ast
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

    def test_mixed_repetitions_report_task_block_totals_and_denominators(self):
        rows = result_rows(confirmation=True)
        for row in rows:
            if row["condition"] != "candidate":
                row["grade"]["outcome"] = "fail" if row["role"] != "challenge" or row["repetition"] == 1 else "pass"
        result = evaluation.compare_results(rows, "outcome", True)
        self.assertEqual(result["decision"], "KEEP")
        self.assertEqual(set(result["task_block_scores"].values()), {1})
        self.assertEqual(result["challenge_trials_per_condition"], 30)
        self.assertEqual(result["candidate_trials"], 48)
        self.assertEqual(result["paired_sign_p"], 1 / 1024)

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
    def test_predecessor_binding_rejects_missing_and_changed_evidence(self):
        with tempfile.TemporaryDirectory() as directory:
            repo = Path(directory)
            archive = repo / "target/personal-prompt-eval"
            result = archive / "prior"
            result.mkdir(parents=True)
            evidence = result / "report.json"
            evidence.write_text("retained result\n")
            root = {"path": "target/personal-prompt-eval/prior",
                    "files_sha256": {"report.json": evaluation.digest(evidence)}}
            evaluation.verify_predecessor_roots(repo, archive, [root])
            evidence.write_text("changed result\n")
            with self.assertRaisesRegex(ValueError, "Prior result changed"):
                evaluation.verify_predecessor_roots(repo, archive, [root])
            evidence.unlink()
            with self.assertRaisesRegex(ValueError, "Prior result changed"):
                evaluation.verify_predecessor_roots(repo, archive, [root])
            with self.assertRaisesRegex(ValueError, "root is missing"):
                evaluation.verify_predecessor_roots(repo, archive, [{**root, "path": "target/personal-prompt-eval/missing"}])

    def test_git_maintenance_cache_drift_is_neutral_but_index_or_file_changes_fail(self):
        with tempfile.TemporaryDirectory() as directory:
            repo = Path(directory)
            tracked = repo / "tracked.txt"
            tracked.write_text("base\n")
            evaluation.git(repo, "init", "--template=", "-q")
            evaluation.git(repo, "add", "tracked.txt")
            evaluation.git(repo, "-c", "commit.gpgsign=false", "commit", "-qm", "fixture")
            expected_files = evaluation.inventory(repo)
            expected_index = evaluation.git(repo, "-c", "core.fsmonitor=false", "ls-files", "--stage", "-v", "-z")
            expected_status = evaluation.git(repo, "-c", "core.fsmonitor=false", "status", "--short", "--untracked-files=all")
            expected_head = evaluation.git(repo, "rev-parse", "HEAD")
            stat = tracked.stat()
            os.utime(tracked, ns=(stat.st_atime_ns, stat.st_mtime_ns + 1_000_000_000))
            evaluation.git(repo, "update-index", "--refresh")
            self.assertNotEqual(expected_files[".git/index"], evaluation.digest(repo / ".git/index"))
            self.assertTrue(evaluation.fixture_matches(repo, expected_files, expected_index, expected_status, expected_head))
            tracked.write_text("changed\n")
            self.assertFalse(evaluation.fixture_matches(repo, expected_files, expected_index, expected_status, expected_head))
            evaluation.git(repo, "add", "tracked.txt")
            self.assertFalse(evaluation.fixture_matches(repo, expected_files, expected_index, expected_status, expected_head))

    def test_successor_binds_all_prior_roots_and_cumulative_counts(self):
        snapshot = evaluation.predecessor_snapshot()
        self.assertEqual(snapshot["root_count"], 35)
        self.assertEqual(snapshot["file_count"], 109)
        self.assertEqual(snapshot["historical_counters"]["actor_attempted"], 497)
        self.assertEqual(snapshot["historical_counters"]["actor_valid"], 491)
        self.assertEqual(snapshot["historical_counters"]["actor_invalid"], 6)
        self.assertEqual(snapshot["historical_counters"]["assessor_attempted"], 405)
        self.assertEqual(snapshot["historical_counters"]["assessor_valid"], 405)
        self.assertEqual(snapshot["historical_counters"]["actor_unattempted"], 293)
        self.assertEqual(snapshot["historical_counters"]["confirmed_leads"], 0)
        self.assertEqual(snapshot["historical_counters"]["valid_plateau_screens"], 2)
        self.assertEqual(snapshot["prospective_plateau_screens_completed"], 2)
        self.assertEqual(snapshot["inherited_screen_families"], ["isolation", "scope"])

    def test_fresh_plateau_fixtures_have_discriminating_oracles(self):
        for context, variant in (("retry", 9), ("limits", 9), ("mapping", 3)):
            with self.subTest(context=context), tempfile.TemporaryDirectory() as directory:
                fixture = Path(directory)
                definition = evaluation.fixture_definition(context, variant, 4)
                for name, content in definition["files"].items():
                    (fixture / name).write_text(content)
                self.assertFalse(evaluation.confined_oracle(fixture, definition["check"])["pass"])
                (fixture / definition["source"]).write_text(definition["reference"])
                self.assertTrue(evaluation.confined_oracle(fixture, definition["check"])["pass"])

    def test_plateau_requires_every_supported_family_screen(self):
        descriptor = {"families": {
            "planning": {"screen_stages": ["tier-4"]},
            "isolation": {"screen_stages": []},
            "scope": {"screen_stages": []},
            "routing": {"unsupported": "missing matched route"},
            "continuity": {"unsupported": "missing compact receipt"},
        }, "predecessors": {"inherited_screens": [
            {"family": "isolation", "status": "baseline-sufficient"},
            {"family": "scope", "status": "baseline-sufficient"},
        ]}}
        families = {name: {"status": "baseline-sufficient"} for name in ("planning", "isolation", "scope")}
        rows = [{"family": "planning", "child": "planning-tier-4-baseline", "result": "baseline-sufficient"}]
        self.assertTrue(evaluation.evidence_plateau(descriptor, families, rows))
        self.assertFalse(evaluation.evidence_plateau(descriptor, {**families, "scope": {"status": "blocked"}}, rows))
        self.assertFalse(evaluation.evidence_plateau(descriptor, families, []))
        invalid = [{**rows[0], "result": "signal"}]
        self.assertFalse(evaluation.evidence_plateau(descriptor, families, invalid))

    def test_receipt_failure_leaves_slot_spent_without_model_submission(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "trial.json"
            send = mock.Mock()
            def persist(path, record):
                evaluation.save_json(path, record)
                raise OSError("Receipt was not durably committed")
            with self.assertRaisesRegex(OSError, "Receipt was not durably committed"):
                evaluation.submit_trial(path, {"slot": "slot-001"}, {}, send, persist=persist)
            send.assert_not_called()
            self.assertTrue(path.is_file())
            with self.assertRaisesRegex(ValueError, "spent"):
                evaluation.submit_trial(path, {}, {}, send)

    def test_streaming_buffered_frames_cannot_extend_trial_deadline(self):
        module = ast.parse(Path(evaluation.__file__).read_text())
        runner = next(node for node in module.body if isinstance(node, ast.FunctionDef) and node.name == "run_pilot")
        reader = next(node for node in runner.body if isinstance(node, ast.FunctionDef) and node.name == "receive")
        process = mock.Mock()
        process.stdout.readline.return_value = '{"ev":"text_delta","text":"stream"}\n'
        def expire(*_):
            raise TimeoutError("Trial/API watchdog expired")
        namespace = {"time": mock.Mock(monotonic=mock.Mock(side_effect=[0, 1, 2, 3])),
                     "deadline": 3, "signal": mock.Mock(), "expire": expire, "process": process,
                     "manifest": {"api_frame_bytes": 262144}, "trace": mock.Mock(), "os": mock.Mock(),
                     "json": json, "active": {"events": []}}
        exec(compile(ast.Module(body=[reader], type_ignores=[]), evaluation.__file__, "exec"), namespace)
        for _ in range(3):
            namespace["receive"]()
        with self.assertRaises(TimeoutError):
            namespace["receive"]()
        self.assertEqual(process.stdout.readline.call_count, 3)
        self.assertEqual(len(namespace["active"]["events"]), 3)
        self.assertEqual(namespace["os"].fsync.call_count, 1)
        namespace["time"].monotonic = mock.Mock(return_value=0)
        process.stdout.readline.return_value = 'x' * 262145 + '\n'
        with self.assertRaisesRegex(RuntimeError, "frame exceeds limit"):
            namespace["receive"]()
        process.stdout.readline.return_value = '{"ev":"text_delta","text":"stream"}\n'
        namespace.update(time=evaluation.time, signal=evaluation.signal, deadline=evaluation.time.monotonic() + .02)
        namespace["trace"].flush.side_effect = lambda: evaluation.time.sleep(.1)
        with self.assertRaises(TimeoutError):
            namespace["receive"]()
        self.assertEqual(evaluation.signal.getitimer(evaluation.signal.ITIMER_REAL)[0], 0)

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
            evaluation.restore_target(root, descriptor)
            self.assertEqual((root / "experiment-target.md").read_bytes(), b"original\n")
            evaluation.audit(root, "family", family="scope", candidate=0, status="confirmed-benchmark-lead")
            (root / "experiment-target.md").write_bytes(b"interrupted garbage")
            evaluation.restore_target(root, descriptor)
            self.assertEqual((root / "experiment-target.md").read_bytes(), b"kept\n")
            evaluation.audit(root, "decision", family="scope", candidate=0, result={"decision": "REVERT"})
            evaluation.restore_target(root, descriptor)
            self.assertEqual((root / "experiment-target.md").read_bytes(), b"original\n")

    def test_later_provisional_keep_and_revert_preserve_prior_confirmed_family(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "snapshots/bases").mkdir(parents=True)
            for name in ("personal", "first", "second"):
                (root / "snapshots/bases" / (name + ".md")).write_text(name)
            descriptor = {"families": {name: {"candidates": [{"base_file": "bases/" + name + ".md"}]} for name in ("first", "second")}}
            evaluation.save_json(root / "campaign.json", descriptor)
            evaluation.audit(root, "decision", family="first", candidate=0, result={"decision": "KEEP"})
            evaluation.audit(root, "family", family="first", candidate=0, status="confirmed-benchmark-lead")
            evaluation.audit(root, "decision", family="second", candidate=0, result={"decision": "KEEP"})
            evaluation.restore_target(root, descriptor)
            self.assertEqual((root / "experiment-target.md").read_text(), "first")
            evaluation.audit(root, "decision", family="second", candidate=0, result={"decision": "REVERT"})
            evaluation.restore_target(root, descriptor)
            self.assertEqual((root / "experiment-target.md").read_text(), "first")


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
        self.assertEqual(evaluation.digest(self.root / "driver.py"), descriptor["driver_sha256"])
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

    def test_fresh_planning_screen_has_three_independent_contexts(self):
        descriptor = evaluation.verify_campaign(self.root)
        keys = descriptor["families"]["planning"]["groups"]["tier-5"]
        self.assertEqual(len(keys), 6)
        self.assertEqual({descriptor["cases"][key]["context"] for key in keys},
                         {"retry", "limits", "mapping"})
        rule = descriptor["plateau_rule"]["criterion"]
        self.assertIn("this registered successor supplies a fresh planning screen", rule)
        self.assertIn("Earlier blocked or preflight-only registrations remain historical", rule)

    def test_child_freezes_initial_git_status_in_condition_blind_packet(self):
        descriptor = evaluation.verify_campaign(self.root)
        key = descriptor["families"]["scope"]["groups"]["tier-1"][0]
        child = evaluation.prepare_child(self.root, "initial-git-status", [key], ["personal"])
        manifest = json.loads((child / "manifest.json").read_text())
        slot = manifest["schedule"][0]
        workspace = child / "workspaces" / slot["slot"]
        status = evaluation.git(workspace, "-c", "core.fsmonitor=false", "status", "--short", "--untracked-files=all")
        self.assertEqual(slot["initial_git_status"], status)
        self.assertIn("notes.txt", status)
        self.assertIn(".jcode/system-prompt.md", status)

        record = {"finished_at": "captured", "session_id": "initial-status-check",
                  "final_files": slot["initial_files"],
                  "final_git_status": {"exit_code": 0, "stdout": status, "stderr": ""}}
        (child / "trials").mkdir()
        path = child / "trials" / (slot["slot"] + ".json")
        evaluation.save_json(path, record)
        evaluation.audit(self.root, "trial-record", child=child.name, slot=slot["slot"],
                          record_sha256=evaluation.digest(path))
        packet = evaluation.make_packet(child, slot)
        self.assertEqual(packet["initial_git_status"], status)
        self.assertEqual(packet["final_git_status"]["stdout"], status)

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

    def test_death_before_child_registration_retains_unpublished_preparation(self):
        descriptor = evaluation.verify_campaign(self.root)
        keys = descriptor["families"]["scope"]["groups"]["tier-1"][:1]
        child = self.root / "runs" / "incomplete-preparation"
        pid = os.fork()
        if pid == 0:
            with mock.patch.object(evaluation, "audit", side_effect=lambda *a, **kw: os._exit(73)):
                evaluation.prepare_child(self.root, child.name, keys, ["personal"])
            os._exit(74)
        self.assertEqual(os.waitpid(pid, 0)[1] >> 8, 73)
        self.assertFalse(child.exists())
        orphans = list((self.root / "preparations").glob(child.name + "-*"))
        self.assertEqual(len(orphans), 1)
        self.assertTrue((orphans[0] / "manifest.json").is_file())
        self.assertFalse((orphans[0] / "trials").exists())
        evaluation.verify_campaign(self.root)
        evaluation.prepare_child(self.root, child.name, keys, ["personal"])
        self.assertTrue(child.is_dir())
        self.assertTrue(orphans[0].is_dir(), "Partial preparation must remain retained")

    def test_death_after_registration_recovers_only_the_bound_preparation(self):
        descriptor = evaluation.verify_campaign(self.root)
        keys = descriptor["families"]["scope"]["groups"]["tier-1"][:1]
        child = self.root / "runs" / "publication-recovery"
        pid = os.fork()
        if pid == 0:
            with mock.patch.object(Path, "rename", side_effect=lambda *a, **kw: os._exit(73)):
                evaluation.prepare_child(self.root, child.name, keys, ["personal"])
            os._exit(74)
        self.assertEqual(os.waitpid(pid, 0)[1] >> 8, 73)
        self.assertFalse(child.exists())
        with self.assertRaisesRegex(ValueError, "child registration is incomplete"):
            evaluation.verify_campaign(self.root)
        evaluation.verify_campaign(self.root, recover=True)
        self.assertTrue((child / "manifest.json").is_file())
        self.assertFalse((child / "trials").exists())
        self.assertEqual(evaluation.prepare_child(self.root, child.name, keys, ["personal"]), child)
        registrations = [r for r in evaluation.audit_rows(self.root) if r["kind"] == "prepared" and r["child"] == child.name]
        self.assertEqual(len(registrations), 1)

    def test_assessment_resume_reuses_frozen_oracle_packets(self):
        descriptor = evaluation.verify_campaign(self.root)
        keys = descriptor["families"]["scope"]["groups"]["tier-1"][:1]
        child = evaluation.prepare_child(self.root, "packet-recovery", keys, ["default", "personal"])
        manifest = json.loads((child / "manifest.json").read_text())
        (child / "trials").mkdir()
        for slot in manifest["schedule"]:
            trial = child / "trials" / (slot["slot"] + ".json")
            evaluation.save_json(trial,
                                 {"finished_at": "captured", "final_files": slot["initial_files"]})
            evaluation.audit(self.root, "trial-record", child=child.name, slot=slot["slot"], record_sha256=evaluation.digest(trial))
        def packet(directory, slot):
            return {"assistant": [slot["slot"]], "oracle_elapsed": os.urandom(8).hex(),
                    "validity": {"valid": True, "model_reached": True}, "burden": {}}
        with mock.patch.object(evaluation, "make_packet", side_effect=packet):
            first = evaluation.assess_child(self.root, child)
        with mock.patch.object(evaluation, "make_packet", side_effect=AssertionError("oracle reran on resume")):
            self.assertEqual(evaluation.assess_child(self.root, child), first)
        cached = child / "blind-packets.json"
        original_cache = cached.read_bytes()
        try:
            data = json.loads(original_cache)
            data["slots"].reverse()
            evaluation.save_json(cached, data)
            with self.assertRaisesRegex(ValueError, "packet cache drifted"):
                evaluation.assess_child(self.root, child)
            with self.assertRaisesRegex(ValueError, "packet cache drifted"):
                evaluation.verify_campaign(self.root)
        finally:
            evaluation.save_bytes(cached, original_cache)
        path = child / "trials" / "slot-001.json"
        original = path.read_bytes()
        try:
            evaluation.save_json(path, {**json.loads(original), "tampered": True})
            with self.assertRaisesRegex(ValueError, "packet inputs drifted"):
                evaluation.assess_child(self.root, child)
            with self.assertRaisesRegex(ValueError, "packet inputs drifted"):
                evaluation.verify_campaign(self.root)
        finally:
            evaluation.save_bytes(path, original)

    def test_cache_only_index_refresh_is_neutral_but_staging_and_missing_evidence_are_not(self):
        descriptor = evaluation.verify_campaign(self.root)
        keys = descriptor["families"]["scope"]["groups"]["tier-1"][:1]
        child = evaluation.prepare_child(self.root, "index-semantics", keys, ["default"])
        manifest = json.loads((child / "manifest.json").read_text())
        slot = manifest["schedule"][0]
        workspace = child / "workspaces" / slot["slot"]
        (child / "trials").mkdir()
        record_path = child / "trials" / (slot["slot"] + ".json")

        def packet(index=True):
            record = {"finished_at": "captured", "session_id": "no-model-index-check",
                      "final_files": evaluation.inventory(workspace)}
            if index:
                record["final_git_index"] = {"exit_code": 0, "stdout": evaluation.git(
                    workspace, "-c", "core.fsmonitor=false", "ls-files", "--stage", "-v", "-z")}
            evaluation.save_json(record_path, record)
            evaluation.audit(self.root, "trial-record", child=child.name, slot=slot["slot"], record_sha256=evaluation.digest(record_path))
            return evaluation.make_packet(child, slot)

        subprocess.run(["git", "-C", str(workspace), "-c", "core.fsmonitor=false", "status", "--short"],
                       env={**os.environ, "GIT_OPTIONAL_LOCKS": "1"}, check=True, stdout=subprocess.PIPE)
        self.assertNotEqual(evaluation.digest(workspace / ".git/index"), slot["initial_files"][".git/index"])
        neutral = packet()
        self.assertTrue(neutral["git_index"]["cache_only_refresh"])
        self.assertNotIn(".git/index", neutral["forbidden_changes"])
        self.assertIn(".git/index", packet(index=False)["forbidden_changes"])
        index = workspace / ".git/index"
        original_index = index.read_bytes()
        (index.parent / "index.cache").write_bytes(original_index)
        index.unlink()
        index.symlink_to("index.cache")
        self.assertIn(".git/index", packet()["forbidden_changes"])
        index.unlink()
        index.write_bytes(original_index)
        (index.parent / "index.cache").unlink()
        (workspace / "retry.py").write_text(evaluation.FIXED)
        evaluation.git(workspace, "add", "retry.py")
        staged = packet()
        self.assertFalse(staged["git_index"]["cache_only_refresh"])
        self.assertIn(".git/index", staged["forbidden_changes"])

    def test_unbound_tampered_and_deleted_trial_records_cannot_redefine_expected_state(self):
        descriptor = evaluation.verify_campaign(self.root)
        keys = descriptor["families"]["scope"]["groups"]["tier-1"][:1]
        child = evaluation.prepare_child(self.root, "trial-receipts", keys, ["default"])
        slot = json.loads((child / "manifest.json").read_text())["schedule"][0]
        workspace = child / "workspaces" / slot["slot"]
        trial = child / "trials" / (slot["slot"] + ".json")
        record = {"finished_at": "captured", "final_files": slot["initial_files"]}
        evaluation.save_json(trial, record)
        with self.assertRaisesRegex(ValueError, "Trial record digest"):
            evaluation.verify_campaign(self.root)
        evaluation.audit(self.root, "trial-record", child=child.name, slot=slot["slot"], record_sha256=evaluation.digest(trial))
        original = trial.read_bytes()
        source = workspace / "retry.py"
        original_source = source.read_bytes()
        try:
            source.write_text(evaluation.FIXED)
            evaluation.save_json(trial, {**record, "final_files": evaluation.inventory(workspace)})
            for check in (lambda: evaluation.verify_campaign(self.root),
                          lambda: evaluation.run_campaign(self.root, assess_only=True)):
                with self.assertRaisesRegex(ValueError, "Trial record digest"):
                    check()
            source.write_bytes(original_source)
            trial.unlink()
            with self.assertRaisesRegex(ValueError, "Trial record digest"):
                evaluation.verify_campaign(self.root)
            fifo, copied = child / "record.fifo", child / "record-copy.json"
            os.mkfifo(fifo)
            copied.write_bytes(original)
            original_read = Path.read_text
            def read_text(path, *args, **kwargs):
                self.assertNotEqual(path, trial, "Untrusted trial was opened before file-type validation")
                return original_read(path, *args, **kwargs)
            for target in (fifo, copied):
                trial.symlink_to(target)
                with mock.patch.object(Path, "read_text", read_text):
                    with self.assertRaisesRegex(ValueError, "Trial record is not a regular file"):
                        evaluation.verify(child)
                trial.unlink()
        finally:
            source.write_bytes(original_source)
            evaluation.save_bytes(trial, original)
        evaluation.verify_campaign(self.root)

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
            if mode == "common-failure":
                descriptor["families"]["unavailable"] = {**spec, "unsupported": "Missing native capability"}
            evaluation.save_json(root / "campaign.json", descriptor)
            calls = []
            original_audit = evaluation.audit

            def write_audit(output, kind, **fields):
                if mode == "audit-failure" and kind == "decision" and fields["result"]["decision"] == "REVERT":
                    raise OSError("audit unavailable")
                return original_audit(output, kind, **fields)

            def child(output, name, keys, conditions, candidate=None, packets=None):
                if mode == "local-failure" and name.startswith("second-tier-1-baseline"):
                    self.assertEqual((root / "experiment-target.md").read_text(), "original")
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
                elif mode == "local-failure" and path.name.startswith("first") and "candidate" in path.name:
                    raise ValueError("family-local invalidity")
                elif mode in ("confirmation-failure", "audit-failure") and "confirmation" in path.name:
                    raise RuntimeError("confirmation transport failed")

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
                 mock.patch.object(evaluation, "audit", side_effect=write_audit), \
                 mock.patch.object(evaluation, "assess_child", side_effect=assess):
                if mode == "audit-failure":
                    with self.assertRaisesRegex(OSError, "audit unavailable"):
                        evaluation.run_campaign(root)
                    self.assertEqual((root / "experiment-target.md").read_text(), "original")
                elif mode == "interrupt":
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
                    elif mode == "local-failure":
                        self.assertEqual(report["families"]["first"]["status"], "blocked")
                        self.assertEqual(report["families"]["second"]["status"], "confirmed-benchmark-lead")
                    elif mode == "confirmation-failure":
                        self.assertEqual(report["confirmed_leads"], 0)
                        self.assertEqual((root / "experiment-target.md").read_text(), "original")
                        decisions = [r for r in evaluation.audit_rows(root) if r["kind"] == "decision"]
                        self.assertEqual(decisions[-1]["result"]["decision"], "REVERT")
                    else:
                        self.assertEqual(report["status"], "blocked")
                        self.assertEqual(report["families"]["first"]["status"], "blocked")
                        self.assertEqual(report["families"]["second"]["status"], "blocked")
                        self.assertEqual(report["families"]["unavailable"]["status"], "unsupported")
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

    def test_family_local_candidate_failure_restores_before_independent_work(self):
        self.exercise("local-failure")

    def test_confirmation_runtime_failure_reverts_provisional_keep(self):
        self.exercise("confirmation-failure")

    def test_confirmation_audit_failure_cannot_restore_unconfirmed_candidate(self):
        self.exercise("audit-failure")


if __name__ == "__main__":
    unittest.main()
