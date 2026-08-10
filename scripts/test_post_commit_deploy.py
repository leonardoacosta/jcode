#!/usr/bin/env python3
"""Regression tests for the post-commit deploy worker."""

from __future__ import annotations

import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import time
import unittest


SOURCE = Path(__file__).resolve().parent


class PostCommitDeployTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.repo = Path(self.temp.name) / "repo"
        (self.repo / "scripts").mkdir(parents=True)
        (self.repo / "crates" / "demo").mkdir(parents=True)
        shutil.copy2(SOURCE / "post_commit_deploy.sh", self.repo / "scripts")
        self.install_log = Path(self.temp.name) / "installed-heads.log"
        (self.repo / ".gitignore").write_text("/target/\n")
        (self.repo / "scripts" / "install_release.sh").write_text(
            "#!/usr/bin/env bash\n"
            "set -euo pipefail\n"
            'printf "%s %s\\n" "$(git rev-parse HEAD)" '
            '"$(git diff --name-only HEAD -- . | wc -l)" >> "$JCODE_DEPLOY_TEST_LOG"\n'
            'sleep "${JCODE_DEPLOY_TEST_SLEEP:-0}"\n'
        )
        os.chmod(self.repo / "scripts" / "install_release.sh", 0o755)
        subprocess.run(["git", "init", "-q", str(self.repo)], check=True)
        subprocess.run(["git", "-C", str(self.repo), "config", "user.email", "test@example.com"], check=True)
        subprocess.run(["git", "-C", str(self.repo), "config", "user.name", "Test"], check=True)

    def commit(self, path: str, content: str) -> str:
        target = self.repo / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(content)
        subprocess.run(["git", "-C", str(self.repo), "add", "."], check=True)
        subprocess.run(["git", "-C", str(self.repo), "commit", "-qm", path], check=True)
        return subprocess.check_output(
            ["git", "-C", str(self.repo), "rev-parse", "HEAD"], text=True
        ).strip()

    def invoke(self, *, sleep: float = 0) -> None:
        env = os.environ.copy()
        env["JCODE_DEPLOY_TEST_LOG"] = str(self.install_log)
        env["JCODE_DEPLOY_TEST_SLEEP"] = str(sleep)
        subprocess.run(
            ["bash", "scripts/post_commit_deploy.sh"],
            cwd=self.repo,
            env=env,
            check=True,
        )

    def wait(self, expected_lines: int, timeout: float = 8) -> list[str]:
        deadline = time.monotonic() + timeout
        state = self.repo / "target" / "commit-deploy"
        while time.monotonic() < deadline:
            lines = self.install_log.read_text().splitlines() if self.install_log.exists() else []
            if (
                len(lines) >= expected_lines
                and not (state / "lock").exists()
                and not (state / "requested-head").exists()
            ):
                return lines
            time.sleep(0.05)
        self.fail("deploy worker did not drain before timeout")

    def test_builds_exact_committed_head_in_clean_detached_worktree(self) -> None:
        head = self.commit("crates/demo/lib.rs", "pub fn one() {}\n")
        # Dirty the main worktree after the commit. The deploy must not include it.
        (self.repo / "crates" / "demo" / "lib.rs").write_text("dirty\n")
        self.invoke()
        self.assertEqual(self.wait(1), [f"{head} 0"])

    def test_docs_only_commit_does_not_deploy(self) -> None:
        self.commit("README.md", "docs only\n")
        self.invoke()
        time.sleep(0.15)
        self.assertFalse(self.install_log.exists())

    def test_commit_during_build_is_coalesced_and_deployed(self) -> None:
        first = self.commit("crates/demo/lib.rs", "pub fn one() {}\n")
        self.invoke(sleep=0.4)
        time.sleep(0.1)
        second = self.commit("crates/demo/lib.rs", "pub fn two() {}\n")
        self.invoke()
        lines = self.wait(2)
        self.assertEqual(lines[0], f"{first} 0")
        self.assertEqual(lines[-1], f"{second} 0")


if __name__ == "__main__":
    unittest.main()
