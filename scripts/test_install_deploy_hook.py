#!/usr/bin/env python3
"""Regression tests for the deploy-hook installer config update."""

from __future__ import annotations

import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


SOURCE_SCRIPTS = Path(__file__).resolve().parent


class InstallDeployHookTests(unittest.TestCase):
    def invoke_installer(self, config_path: Path) -> subprocess.CompletedProcess[str]:
        repo = config_path.parent.parent / "repo"
        script = repo / "scripts" / "install_deploy_hook.sh"
        env = os.environ.copy()
        env["JCODE_HOME"] = str(config_path.parent)
        return subprocess.run(
            ["bash", str(script), "--config-only"],
            cwd=repo,
            env=env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )

    def run_installer(self, config: bytes) -> tuple[subprocess.CompletedProcess[str], Path]:
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        root = Path(temp.name)
        repo = root / "repo"
        scripts = repo / "scripts"
        runtime = root / "runtime"
        scripts.mkdir(parents=True)
        runtime.mkdir()
        shutil.copy2(SOURCE_SCRIPTS / "install_deploy_hook.sh", scripts)
        shutil.copy2(SOURCE_SCRIPTS / "post_commit_deploy.sh", scripts)
        subprocess.run(["git", "init", "-q", str(repo)], check=True)

        config_path = runtime / "config.toml"
        config_path.write_bytes(config)
        return self.invoke_installer(config_path), config_path

    def test_duplicate_or_malformed_input_is_preserved(self) -> None:
        original = b"[display]\nauto_client_reload = false\nauto_client_reload = true\n"
        result, config_path = self.run_installer(original)

        self.assertNotEqual(result.returncode, 0, result.stdout)
        self.assertIn("config", result.stderr.lower())
        self.assertEqual(config_path.read_bytes(), original)
        self.assertFalse(config_path.with_suffix(".bak").exists())

    def test_existing_false_changes_in_place(self) -> None:
        original = (
            b"# keep this comment\n[display]\ncentered = true\n"
            b"auto_client_reload = false # managed\n\n[auth]\n"
            b"trusted_external_sources = [\"keep\"]\n"
        )
        result, config_path = self.run_installer(original)

        self.assertEqual(result.returncode, 0, result.stderr)
        updated = config_path.read_bytes()
        self.assertIn(b"auto_client_reload = true # managed", updated)
        self.assertIn(b"trusted_external_sources = [\"keep\"]", updated)
        self.assertEqual(config_path.with_suffix(".bak").read_bytes(), original)

        second = self.invoke_installer(config_path)
        self.assertEqual(second.returncode, 0, second.stderr)
        self.assertEqual(config_path.read_bytes(), updated)
        self.assertEqual(config_path.with_suffix(".bak").read_bytes(), original)

    def test_missing_key_is_inserted_once(self) -> None:
        original = b"[display]\ncentered = true\n\n[auth]\ntrusted_external_sources = []\n"
        result, config_path = self.run_installer(original)

        self.assertEqual(result.returncode, 0, result.stderr)
        updated = config_path.read_text()
        self.assertEqual(updated.count("auto_client_reload"), 1)
        self.assertIn("auto_client_reload = true", updated)
        self.assertLess(updated.index("auto_client_reload"), updated.index("[auth]"))

    def test_existing_true_is_byte_identical(self) -> None:
        original = b"[display]\nauto_client_reload = true # already enabled\ncentered = true\n"
        result, config_path = self.run_installer(original)

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(config_path.read_bytes(), original)
        self.assertFalse(config_path.with_suffix(".bak").exists())

    def test_other_malformed_toml_is_preserved(self) -> None:
        original = b"[display\nauto_client_reload = false\n"
        result, config_path = self.run_installer(original)

        self.assertNotEqual(result.returncode, 0, result.stdout)
        self.assertEqual(config_path.read_bytes(), original)
        self.assertFalse(config_path.with_suffix(".bak").exists())


if __name__ == "__main__":
    unittest.main()
