"""Regression checks for the benchmark's evidence contract (stdlib only)."""
import importlib.util
import io
import json
import pathlib
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

HERE = pathlib.Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("harness", HERE / "harness.py")
harness = importlib.util.module_from_spec(spec)
spec.loader.exec_module(harness)
A, B, C = "src/a.rs", "src/b.rs", "src/c.rs"


def result(output, code=0):
    return {"argv": ["test"], "cwd": ".", "returncode": code,
            "stdout": output, "stderr": "", "wall_ms": 1}


class HarnessTests(unittest.TestCase):
    def test_ground_truth_requires_real_symbol_anchor_in_each_answer(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            (root / "src").mkdir()
            (root / A).write_text("pub struct Present;\n// Missing\n")
            q = {"id": "q", "answers": [A], "answer_symbols": ["Present"]}
            harness.validate_ground_truth([q], root, {A: "hash"})
            for changes in ({"answer_symbols": ["Missing"]}, {"answer_symbols": []},
                            {"answers": [B]}, {"answers": []}):
                with self.subTest(changes=changes), self.assertRaises(ValueError):
                    harness.validate_ground_truth([dict(q, **changes)], root, {A: "hash"})
            harness.validate_ground_truth([{"id": "q", "answers": [A], "sym": "Present"}], root, {A: "hash"})

    def test_caller_anchor_requires_call_not_declaration_or_comment(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            (root / "src").mkdir()
            q = {"id": "q", "kind": "caller", "answers": [A], "sym": "target"}
            for text in ("fn target() {}\n", "pub async fn target() {}\n", "// target()\n"):
                (root / A).write_text(text)
                with self.subTest(text=text), self.assertRaises(ValueError):
                    harness.validate_ground_truth([q], root, {A: "hash"})
            (root / A).write_text("    self.target().await;\n")
            harness.validate_ground_truth([q], root, {A: "hash"})

    def test_help_does_not_run_benchmark(self):
        r = subprocess.run([sys.executable, str(HERE / "harness.py"), "--help"],
                           capture_output=True, text=True, timeout=5)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("--output", r.stdout)
        self.assertIn("--root", r.stdout)

    def test_existing_output_is_never_overwritten(self):
        r = subprocess.run([sys.executable, str(HERE / "harness.py"), "--output", str(HERE / "queries.json")],
                           capture_output=True, text=True, timeout=5)
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("FileExistsError", r.stderr)

    def test_recall_counts_all_answers_and_exact_cutoff(self):
        self.assertEqual(harness.score([A, B], [A, C]), {"recall_at_1": 0.5, "recall_at_5": 0.5})
        ranked = [f"src/{i}.rs" for i in range(6)]
        self.assertEqual(harness.score(ranked, [ranked[4]])["recall_at_5"], 1)
        self.assertEqual(harness.score(ranked, [ranked[5]])["recall_at_5"], 0)

    def test_graft_only_scores_ranked_pointers(self):
        output = json.dumps({"hits": [{"pointer": A + ":L1-L3", "snippet": C},
                                      {"pointer": A + ":L4-L8"}, {"pointer": B + ":L1-L2"}],
                             "scopes": {"alsoMatched": [{"bestId": C}]}})
        self.assertEqual(harness.ranked_paths("graft", output), [A, B])

    def test_agentgrep_keeps_native_order_not_definition_rerank(self):
        output = json.dumps({"files": [{"path": B, "matches": [{"line_text": A}]}, {"path": A}]})
        self.assertEqual(harness.ranked_paths("agentgrep", output), [B, A])

    def test_astgrep_only_scores_file_fields(self):
        output = json.dumps([{"file": "./" + B, "text": C}, {"file": A}, {"file": B}])
        self.assertEqual(harness.ranked_paths("ast-grep", output), [B, A])

    def test_unsupported_and_missing_arms_have_no_scores(self):
        q = {"id": "q", "kind": "concept", "q": "example", "answers": [A]}
        for strategy, status in (("ast-grep", "unsupported"), ("agentgrep", "not_run"), ("search-subagent", "not_run")):
            row = harness.evaluate_query(strategy, q, HERE)
            self.assertEqual(row["status"], status)
            self.assertNotIn("recall_at_5", row)

    def test_failed_or_malformed_command_is_not_a_scored_miss(self):
        q = {"id": "q", "kind": "concept", "q": "example", "answers": [A]}
        for record in (result("", 7), result("not JSON"), result("{}")):
            with patch.object(harness, "run", return_value=record):
                row = harness.evaluate_query("agentgrep", q, HERE, "test-bin")
            self.assertEqual(row["status"], "error")
            self.assertEqual(row["commands"], [record])
            self.assertNotIn("recall_at_5", row)

    def test_structural_patterns_record_all_calls_and_empty_success(self):
        q = {"id": "q", "kind": "symbol", "q": "example", "answers": [A], "sg": ["first", "second"]}
        with patch.object(harness, "run", return_value=result("[]")) as run:
            row = harness.evaluate_query("ast-grep", q, HERE, "ast-grep")
        self.assertEqual(row["status"], "ok")
        self.assertEqual(row["ranked_paths"], [])
        self.assertEqual(row["recall_at_5"], 0)
        self.assertEqual(row["cli_calls"], 2)
        self.assertEqual(run.call_count, 2)
        self.assertEqual([c.args[0][5] for c in run.call_args_list], ["first", "second"])

    def test_failed_command_keeps_stdout_stderr_and_exit(self):
        with patch.object(harness.subprocess, "run", return_value=subprocess.CompletedProcess(["test"], 3, "out", "err")):
            record = harness.run(["test"], HERE)
        self.assertEqual((record["stdout"], record["stderr"], record["returncode"]), ("out", "err", 3))

    def test_timeout_and_missing_binary_are_evidence_not_crashes(self):
        for exc in (FileNotFoundError("missing"), subprocess.TimeoutExpired(["test"], 1, b"partial", b"error")):
            with patch.object(harness.subprocess, "run", side_effect=exc):
                record = harness.run(["test"], HERE)
            self.assertIsNone(record["returncode"])
            self.assertIn("error", record)
        self.assertEqual((record["stdout"], record["stderr"]), ("partial", "error"))

    def test_malformed_pointer_is_an_error(self):
        with self.assertRaises(ValueError):
            harness.ranked_paths("graft", json.dumps({"hits": [{"pointer": None}]}))
        self.assertIsNone(harness.source_path("src/../../private.rs"))

    def test_stale_index_path_is_not_scored(self):
        q = {"id": "q", "kind": "concept", "q": "example", "answers": [A]}
        with patch.object(harness, "run", return_value=result(json.dumps({"hits": [{"pointer": A}]}))):
            row = harness.evaluate_query("graft", q, HERE, "test-bin")
        self.assertEqual(row["status"], "error")
        self.assertNotIn("recall_at_5", row)

    def test_real_subprocess_failure_and_timeout(self):
        failed = harness.run([sys.executable, "-c", "import sys; print('err', file=sys.stderr); sys.exit(7)"], HERE)
        self.assertEqual(failed["returncode"], 7)
        self.assertIn("err", failed["stderr"])
        timed_out = harness.run([sys.executable, "-c", "import time; time.sleep(5)"], HERE, timeout=0.01)
        self.assertIsNone(timed_out["returncode"])
        self.assertIn("error", timed_out)

    def test_no_matched_scores_when_an_arm_is_missing(self):
        rows = [{"strategy": s, "query": "a", "status": "ok", "recall_at_1": 1, "recall_at_5": 1}
                for s in harness.LOCAL if s != "ast-grep"]
        summary = harness.summarize(rows)
        self.assertEqual(summary["matched_local_query_ids"], [])
        self.assertIsNone(summary["per_strategy"]["agentgrep"]["matched_local_queries"]["recall_at_5"])

    def test_source_drift_invalidates_run_and_exit_status(self):
        qs = json.loads((HERE / "queries.json").read_text())
        before = {"commit": "old", "source_sha256": {a: "sha" for q in qs for a in q["answers"]}, "graft_graph_sha256": "graph"}
        after = dict(before, commit="changed")
        def fake_row(s, q, *args):
            return {"strategy": s, "query": q["id"], "status": "ok", "recall_at_1": 1, "recall_at_5": 1}
        output = io.StringIO()
        with patch.object(sys, "argv", [str(HERE / "harness.py")]), patch.object(sys, "stdout", output), \
             patch.object(harness, "provenance", side_effect=[before, after]), \
             patch.object(harness, "run", return_value=result("version")), \
             patch.object(harness, "evaluate_query", side_effect=fake_row):
            status = harness.main()
        data = json.loads(output.getvalue())
        self.assertEqual(status, 1)
        self.assertFalse(data["source_stable"])
        self.assertEqual(data["status"], "invalid_source_changed")

    def test_matched_summary_uses_identical_query_ids(self):
        rows = [{"strategy": s, "query": q, "status": "ok", "recall_at_1": 1, "recall_at_5": 1}
                for s in harness.LOCAL for q in (["a"] if s == "ast-grep" else ["a", "b"])]
        summary = harness.summarize(rows)
        self.assertEqual(summary["matched_local_query_ids"], ["a"])
        self.assertTrue(all(v["matched_local_queries"]["n"] == 1 for v in summary["per_strategy"].values()))
        self.assertEqual(summary["per_strategy"]["agentgrep"]["available_queries"]["n"], 2)


if __name__ == "__main__":
    unittest.main()
