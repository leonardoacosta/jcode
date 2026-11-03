#!/usr/bin/env python3
"""Exercise the real dev Cargo wrapper, including isolated build metadata."""

import json
import os
import shutil
from pathlib import Path
import subprocess
import tempfile
import unittest


REPO = Path(__file__).resolve().parent.parent
WRAPPER = REPO / "scripts/dev_cargo.sh"


class CargoWorkingDirectoryTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(dir=os.environ.get("JCODE_SCRATCH_DIR"))
        self.addCleanup(self.tmp.cleanup)
        self.project = Path(self.tmp.name) / "other rust project"
        (self.project / "src").mkdir(parents=True)
        (self.project / "Cargo.toml").write_text(
            '[package]\nname = "cwd_probe"\nversion = "0.1.0"\nedition = "2021"\n[workspace]\n'
        )
        (self.project / "src/lib.rs").write_text("")
        self.env = dict(os.environ, JCODE_RUST_ACTION_LOG="0")
        self.env.pop("CARGO_MANIFEST_DIR", None)

    def metadata(self, cwd, *, child_shell=False):
        args = [str(WRAPPER), "metadata", "--no-deps", "--format-version", "1"]
        if child_shell:
            # Match BashTool's exported shim, including a subsequent cd and
            # inheritance by a child shell.
            args = ["bash", "-c", '''
                cargo() {
                    if [[ "${JCODE_IN_DEV_CARGO:-0}" == "1" ]]; then
                        command cargo "$@"
                    else
                        JCODE_IN_DEV_CARGO=1 "$JCODE_DEV_CARGO_SCRIPT" "$@"
                    fi
                }
                export -f cargo
                cd "$1"
                bash -c 'cargo metadata --no-deps --format-version 1'
            ''', "probe", str(cwd)]
            cwd = REPO
        result = subprocess.run(
            args, cwd=cwd,
            env=dict(self.env, JCODE_DEV_CARGO_SCRIPT=str(WRAPPER)),
            capture_output=True, text=True, timeout=60, check=True,
        )
        return json.loads(result.stdout)

    def test_foreign_project_keeps_its_workspace(self):
        metadata = self.metadata(self.project)
        self.assertEqual(Path(metadata["workspace_root"]), self.project)
        self.assertEqual(metadata["packages"][0]["name"], "cwd_probe")

    def test_exported_shim_after_cd_in_child_shell(self):
        self.assertEqual(Path(self.metadata(self.project, child_shell=True)["workspace_root"]), self.project)

    def test_jcode_root_and_member_keep_wrapper_policy(self):
        for cwd in [REPO, REPO / "crates/jcode-tui"]:
            with self.subTest(cwd=cwd):
                self.assertEqual(Path(self.metadata(cwd)["workspace_root"]), REPO)

    def test_symlink_to_foreign_project_keeps_its_workspace(self):
        link = Path(self.tmp.name) / "linked project"
        link.symlink_to(self.project, target_is_directory=True)
        self.assertEqual(Path(self.metadata(link)["workspace_root"]), self.project)

    def test_foreign_cargo_errors_are_preserved(self):
        result = subprocess.run(
            [str(WRAPPER), "metadata", "--manifest-path", "missing.toml"],
            cwd=self.project, env=self.env, capture_output=True, text=True, timeout=60,
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("missing.toml", result.stderr)


class BuildSourceIdentityTests(unittest.TestCase):
    def test_same_head_edit_and_revert_never_reuse_clean_identity(self):
        with tempfile.TemporaryDirectory(dir=os.environ.get("JCODE_SCRATCH_DIR")) as tmp:
            root = Path(tmp)
            (root / "scripts").mkdir()
            for name in ["dev_cargo.sh", "remote_config.sh", "rustc-parallel-frontend"]:
                shutil.copy2(REPO / "scripts" / name, root / "scripts" / name)
            shutil.copytree(REPO / "crates/jcode-build-meta", root / "crates/jcode-build-meta")
            (root / "src").mkdir()
            source = root / "src/main.rs"
            original = '''fn main() {
    println!("{} {} {}", jcode_build_meta::GIT_HASH,
        jcode_build_meta::git_dirty(), jcode_build_meta::cleanliness_verified());
}
'''
            source.write_text(original)
            (root / "Cargo.toml").write_text('''[package]
name = "identity_probe"
version = "0.1.0"
edition = "2024"
[workspace]
[dependencies]
jcode-build-meta = { path = "crates/jcode-build-meta" }
''')
            (root / ".gitignore").write_text("target/\nprivate/\n")
            (root / ".gitattributes").write_text("src/main.rs filter=identity-test\n")
            (root / ".jcode").mkdir()
            runtime_input = root / ".jcode/policy.md"
            runtime_input.write_text("tracked policy\n")
            env = {k: v for k, v in os.environ.items()
                   if not k.startswith(("JCODE_BUILD_", "CARGO_", "RUST", "GIT_"))}
            env.update(JCODE_RUST_ACTION_LOG="0", JCODE_REMOTE_CONFIG="/dev/null",
                       JCODE_REMOTE_CARGO="0", JCODE_FAST_LINKER="system",
                       JCODE_CARGO_GATE="off", JCODE_DEV_FEATURE_PROFILE="default",
                       JCODE_PARALLEL_FRONTEND="0", SCCACHE_DISABLE="1")

            def run(*args):
                result = subprocess.run(args, cwd=root, env=env, capture_output=True,
                                        text=True, timeout=60)
                self.assertEqual(result.returncode, 0, result.stderr)
                return result.stdout.strip()

            run("cargo", "generate-lockfile", "--offline")
            run("git", "init", "-q")
            run("git", "config", "core.excludesFile", "/dev/null")
            run("git", "add", ".")
            # The isolated fixture uses the configured identity, never an invented one.
            run("git", "commit", "-qm", "Source identity regression fixture")
            head = run("git", "rev-parse", "--short", "HEAD")
            binary = root / "target/debug/identity_probe"
            # These hooks would conceal edits or execute arbitrary code if the
            # wrapper used status/diff/hash-object with filters instead of bytes.
            marker = root / "target/unsafe-hook-ran"
            hook = f"touch '{marker}'; cat"
            run("git", "config", "filter.identity-test.clean", hook)
            run("git", "config", "core.fsmonitor", hook)
            run("git", "update-index", "--assume-unchanged", "src/main.rs")

            def build(wrapped=True):
                run(str(root / "scripts/dev_cargo.sh") if wrapped else "cargo",
                    "build", "--offline", "--quiet")
                return run(str(binary)).split()

            self.assertEqual(build(), [head, "false", "false"])
            metadata_output = next((root / "target/debug/build").glob("jcode-build-meta-*/output"))
            clean_stamp = metadata_output.stat().st_mtime_ns
            self.assertEqual(build(), [head, "false", "false"])
            self.assertEqual(metadata_output.stat().st_mtime_ns, clean_stamp)
            extra = root / "src/untracked.rs"
            extra.write_text("// eligible untracked source\n")
            self.assertEqual(build(), [head, "true", "false"])
            extra.unlink()
            self.assertEqual(build(), [head, "false", "false"])
            runtime_input.write_text("changed tracked policy\n")
            self.assertEqual(build(), [head, "true", "false"])
            runtime_input.write_text("tracked policy\n")
            self.assertEqual(build(), [head, "false", "false"])
            self.assertFalse(marker.exists(), "source identity executed a Git hook/filter")
            clean_binary = root / "target/clean-probe"
            shutil.copy2(binary, clean_binary)
            source.write_text(original.replace("fn main() {", 'fn main() { eprintln!("dirty source executable");'))
            self.assertEqual(build(), [head, "true", "false"])
            self.assertIn("dirty source executable", subprocess.run(
                [str(binary)], capture_output=True, text=True, check=True).stderr)
            dirty_stamp = metadata_output.stat().st_mtime_ns
            # A second dirty edit must refresh even when the dirty bit stays true.
            source.write_text(original.replace("fn main() {", 'fn main() { eprintln!("second dirty executable");'))
            self.assertEqual(build(), [head, "true", "false"])
            self.assertNotEqual(metadata_output.stat().st_mtime_ns, dirty_stamp)
            dirty_binary = root / "target/dirty-probe"
            shutil.copy2(binary, dirty_binary)
            source.write_text(original)
            self.assertEqual(run("git", "rev-parse", "--short", "HEAD"), head)
            self.assertEqual(run(str(dirty_binary)).split(), [head, "true", "false"])
            self.assertIn("second dirty executable", subprocess.run(
                [str(dirty_binary)], capture_output=True, text=True, check=True).stderr)
            self.assertEqual(run(str(clean_binary)).split(), [head, "false", "false"])
            self.assertEqual(build(), [head, "false", "false"])
            run("git", "-c", "core.fsmonitor=false", "update-index", "--no-assume-unchanged", "src/main.rs")
            run("git", "-c", "core.fsmonitor=false", "update-index", "--skip-worktree", "src/main.rs")
            source.write_text(original.replace("fn main() {", 'fn main() { eprintln!("skip-worktree dirty executable");'))
            self.assertEqual(build(), [head, "true", "false"])
            source.write_text(original)
            self.assertEqual(build(), [head, "false", "false"])
            self.assertFalse(marker.exists(), "source identity executed a Git hook/filter")
            # A mutable checkout can change after the wrapper snapshot and revert
            # before Cargo returns. The resulting executable is NOT attested.
            compiler_wrapper = root / "target/edit-during-compile.py"
            compiler_wrapper.write_text('''#!/usr/bin/env python3
import pathlib
import subprocess
import sys
source = pathlib.Path("src/main.rs")
original = source.read_text()
mutate = "--crate-name" in sys.argv and "identity_probe" in sys.argv
try:
    if mutate:
        source.write_text(original.replace("fn main() {", 'fn main() { eprintln!("compiled transient source");'))
    result = subprocess.run(sys.argv[1:])
finally:
    if mutate:
        source.write_text(original)
sys.exit(result.returncode)
''')
            compiler_wrapper.chmod(0o755)
            env["RUSTC_WRAPPER"] = str(compiler_wrapper)
            source.touch()
            transient_identity = build()
            self.assertIn("compiled transient source", subprocess.run(
                [str(binary)], capture_output=True, text=True, check=True).stderr)
            self.assertEqual(source.read_text(), original)
            self.assertEqual(transient_identity, [head, "false", "false"])
            env.pop("RUSTC_WRAPPER")
            # Ignored runtime files do not affect identity or get opened.
            (root / "private").mkdir()
            (root / "private/secret").symlink_to("/nonexistent")
            self.assertEqual(build(), [head, "false", "false"])
            # Raw Cargo cannot make the wrapper snapshot authoritative.
            self.assertEqual(build(wrapped=False), [head, "false", "false"])
            # Nonignored credential paths are dirty, but never opened or logged.
            credential = root / ".env"
            credential.symlink_to("/dev/random")
            self.assertEqual(build(), [head, "true", "false"])
            credential.unlink()
            self.assertEqual(build(), [head, "false", "false"])
            credential.write_text("fixture-private-input\n")
            run("git", "-c", "core.fsmonitor=false", "add", "--", ".env")
            self.assertEqual(build()[2], "false")
            run("git", "-c", "core.fsmonitor=false", "rm", "--cached", "--", ".env")
            credential.unlink()
            # Never follow a source symlink, including one in a parent directory.
            (root / "src/private-link.rs").symlink_to("../private/secret")
            self.assertEqual(build()[2], "false")


if __name__ == "__main__":
    unittest.main()
